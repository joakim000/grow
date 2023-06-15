#![feature(error_in_core)]
#![allow(unused)]
use grow::ops;
use grow::zone::*;
// use grow::Result;
mod hardware;

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{Vec, IntoIter};
use lego_powered_up::iodevice::hubled::HubLed;
use lego_powered_up::iodevice::motor::EncoderMotor;
// use anyhow;
use time;
use chrono::{NaiveTime, Timelike, Local};   // deprecated; use time
use core::error::Error;
use core::time::{Duration, };

use simple_signal::{self, Signal};
use core::sync::atomic::{AtomicBool, Ordering};
use alloc::sync::Arc;

use std::time::Instant;

use rppal::gpio::{Gpio, Trigger};
// use rppal::i2c::I2c;
use rppal::pwm::{Channel, Polarity, Pwm};
use pcf8591::{PCF8591, Pin};
// use ssd1306;

use lego_powered_up::{notifications::Power, PoweredUp};
use lego_powered_up::{IoDevice, IoTypeId, HubMutex};

use crate::hardware::conf::*;

const PUMP_INTERVAL_SECS: u64 = 20;
const MOISTURE_1_RUN_PUMP: i16 = 50; // Run pump if moisture below this level
// const MOISTURE_1_WARNING = 30;  // Warn if moisture below this level  


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let keepalive = signal_handler();
    
    
    let house = ops::Conf::read_test_into_house();
    let conf = HwConf::test_config();


    let mut activity_led = Gpio::new()?.get(conf.activity_led_pin)?.into_output();
    let mut adc_control = PCF8591::new(ADC_1_BUS, ADC_1_ADDR, ADC_1_VREF)?;

    // Init fan control
    let fan1 = hardware::Fan::new();
    let fan1_settings = house.fan_settings(&0).expect("Air zone not found");
    

    // Init lamp control
    let mut lamp_1_is_on: bool = false;
    let lamp_1_ontime = NaiveTime::from_hms_opt(15, 48, 00);
    let lamp_1_offtime = NaiveTime::from_hms_opt(16, 41, 00);
 
     // Init LPU
    let hub = lego_powered_up::setup::single_hub().await?;
    let hubled: IoDevice;
    let motor_a: IoDevice;
    {
        let lock = hub.mutex.lock().await;
        hubled = lock.io_from_kind(IoTypeId::HubLed).await?;
        motor_a = lock.io_from_port(0x0).await?;
        
    }
    hubled.set_hubled_mode(lego_powered_up::iodevice::hubled::HubLedMode::Colour).await?;
    hubled.set_hubled_color(lego_powered_up::consts::Color::Green).await?;
 

    // Init pump control
    // let mut time_since_pump_run_check = Duration::from_secs(PUMP_INTERVAL_SECS);
    let mut last_pump_run_check = Instant::now();

    while keepalive.load(Ordering::SeqCst) {
        let now = Local::now();  // Time when this loop starts
        activity_led.set_high(); // Act led on when loop running
    
        // Get sensor values
        let light_1: u8 = light_from_byte(adc_control.analog_read_byte(LIGHT_SENSOR_1)? as &u8);
        let temperature_1: f32 = celcius_from_byte(adc_control.analog_read_byte(TEMP_SENSOR_1)? as &f32);
        let moisture_1: i16 = moist_from_byte(adc_control.analog_read_byte(MOIST_SENSOR_1)? as &i16);
        let moisture_2: i16 = moist_from_byte(adc_control.analog_read_byte(MOIST_SENSOR_2)? as &i16);

        // Control fan


        // Control lamps
        if !lamp_1_is_on && 
        now.hour() >= lamp_1_ontime.expect("REASON").hour() && 
        now.minute() >= lamp_1_ontime.expect("REASON").minute() {
            println!("{} Lamp_1_ontime: {} Lamp turns ON", now.to_rfc2822(), lamp_1_ontime.unwrap());
            adc_control.analog_write_byte(255)?;
            lamp_1_is_on = true;
        } 
        if lamp_1_is_on &&
        now.hour() >= lamp_1_offtime.expect("REASON").hour() &&
        now.minute() >= lamp_1_offtime.expect("REASON").minute() {
            println!("{} Lamp_1_offtime: {} Lamp turns OFF", now.to_rfc2822(), lamp_1_offtime.unwrap());
            adc_control.analog_write_byte(0)?;
            lamp_1_is_on = false;
        }

        // Print status 
        println!("{}" , now.to_rfc2822());
        if lamp_1_is_on {
            print!("\tLamp_1 is ON")
        } else {
            print!("\tLamp_1 is OFF")
        }
        println!("\t(on: {}  off: {})" , lamp_1_ontime.unwrap(), lamp_1_offtime.unwrap());
        // println!("\tLight: {}\tTemp: {}\tMoist_1: {}\tMoist_2: {}\tFan_1: {}\tRes_1: {}\tPump_1: {}", 
        // light_1, temperature_1.round(), moisture_1, v3, fan_1_rpm, 0, 0); 

        // Possible watering
        if last_pump_run_check.elapsed().as_secs() > PUMP_INTERVAL_SECS {
            if moisture_1 < MOISTURE_1_RUN_PUMP {
                last_pump_run_check = Instant::now();
                print!("\tMoisture sensor 1 @ {} is below watering level ({}): ", moisture_1, MOISTURE_1_RUN_PUMP);    
                println!("Run motor A (pump)");
                motor_a.start_speed(50, Power::Cw(50)).await?;
                tokio::time::sleep(Duration::from_secs(6)).await;
                println!("Stop motor A");
                motor_a.start_power(Power::Float).await?;  
            }
        } else {
            println!("\tPump is resting ({} / {})", last_pump_run_check.elapsed().as_secs(), PUMP_INTERVAL_SECS);
        }




        activity_led.set_low();
        tokio::time::sleep(Duration::from_millis(1000)); // main loop interval

    }

    // Cleanup after main loop end
    // Reset pins
    
    // pwm.set_duty_cycle(0.0)?;           // Turn off fan
    // fan_1_rpm_in.clear_interrupt()?;    // Clear fan rpm interrupt
    adc_control.analog_write_byte(0)?;  // Turn off lamp

    // Clean LPU
    print!("\nDisconnect from hub `{}` ...", hub.name);
    {
        let lock = hub.mutex.lock().await;
        lock.disconnect().await?;
    }
    println!("   Success!");

    activity_led.set_low();
    println!("\nCleanup success");
    Ok(())  // main() return
}


// struct HwConf {
//     activity_led_pin: u8,
//     fan_1_rpm_pin: u8,
//     fan_1_pwm_hz: f32,
// }
// impl HwConf {
//     pub fn test_config() -> Self {
//         Self {
//             activity_led_pin: 17, //p11
//             fan_1_rpm_pin: 23, //? 26p16
//             fan_1_pwm_hz: 25000.0,
//         }
//     }
// }

fn signal_handler() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    // When a SIGINT (Ctrl-C) or SIGTERM signal is caught, atomically set running to false.
    simple_signal::set_handler(&[Signal::Int, Signal::Term], {
        let running = running.clone();
        move |_| {
            running.store(false, Ordering::SeqCst);
        }
    });
    running
}

fn celcius_from_byte(value: &f32) -> f32 {
    let coeff_b: f32 = 3950.0;  // thermistor coefficient
    let res_r0: f32 = 10000.0;  // resistance @ room temperature
    let res_r1: f32 = 1000.0;   // resistance of R1
    let room_temperature_in_kelvin: f32 = 297.15; 
    
    let res_r6: f32 = (res_r1*value) / (256.0-value);
    let kelvin: f32 = 1.0 / ( (1.0/room_temperature_in_kelvin) + (1.0/coeff_b) * ( res_r6/res_r0 ).ln() );
    kelvin - 273.15
}


fn moist_from_byte(value: &i16) -> i16 {
    // 115 = 100% moist, 215 = 0% moist
    0 - value + 215
}

fn light_from_byte(value: &u8) -> u8 {
    // 15(240) = dark, 40 = 5v LED up close, 208(47) = very light,  
    255 - value
}



#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum FanSetting {
    Off,
    Low,
    High,
}
