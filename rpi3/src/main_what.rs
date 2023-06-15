#![feature(error_in_core)]
#![allow(unused)]
use grow::ops;
use grow::zone::*;
// use grow::Result;
mod hardware;

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use lego_powered_up::iodevice::hubled::HubLed;
use lego_powered_up::iodevice::motor::EncoderMotor;
// use anyhow;
use chrono::{Local, NaiveTime, Timelike}; // deprecated; use time
use core::error::Error;
use core::time::Duration;
use time;

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use simple_signal::{self, Signal};

use std::time::Instant;

use rppal::gpio::{Gpio, Trigger};
// use rppal::i2c::I2c;
use pcf8591::{Pin, PCF8591};
use rppal::pwm::{Channel, Polarity, Pwm};
// use ssd1306;

use lego_powered_up::{notifications::Power, PoweredUp};
use lego_powered_up::{HubMutex, IoDevice, IoTypeId};

use crate::hardware::conf::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let keepalive = signal_handler();
    let keepalive = Arc::new(AtomicBool::new(true));
    // When a SIGINT (Ctrl-C) or SIGTERM signal is caught, atomically set running to false.
    simple_signal::set_handler(&[Signal::Int, Signal::Term], {
        let running = keepalive.clone();
        move |_| {
            running.store(false, Ordering::SeqCst);
        }
    });
    // let mut house = init::house_init();

    // let run_task = grow::ops::running::Runner::run().await;

    // let mut adc = PCF8591::new(ADC_1_BUS, ADC_1_ADDR, ADC_1_VREF).unwrap();
    let mut adc = PCF8591::new("/dev/i2c-1", 0x48, 5.0)?;
    // let mutex = Arc::new(Mutex::new(control));
    // let adc = mutex.clone();
    let adctask = tokio::spawn(async move {
        loop {
            // let reading: f32;
            // {
                 // Get sensor values
            let v0 = adc.analog_read_byte(Pin::AIN0).unwrap(); // photoresistor
            let v1 = adc.analog_read_byte(Pin::AIN1).unwrap(); // thermistor
            let v2 = adc.analog_read_byte(Pin::AIN2).unwrap(); // capacitive soil moisture 1
            let v3 = adc.analog_read_byte(Pin::AIN3).unwrap(); // capacitive soil moisture 2

            // Convert sensor values
            let light_raw_1: u8 = v0 as u8; 
            let light_1: u8 = light_from_byte(&light_raw_1);
            let temperature_raw_1: f64 = v1 as f64;
            let temperature_1: f64 = celcius_from_byte(&temperature_raw_1);
            let moisture_raw_1: i16 = v2 as i16;
            let moisture_1: i16 = moist_from_byte(&moisture_raw_1);
            let moisture_raw_2: i16 = v3 as i16;
            let moisture_2: i16 = moist_from_byte(&moisture_raw_2);
            println!("Light {:?}  Temp {:?}  Moist_1 {:?}   Moist_2   {:?}",
                    light_1, temperature_1, moisture_1, moisture_2);
            
            // thread::sleep(Duration::from_millis(1000));
            // tokio::time::sleep(Duration::from_millis(10)).await;

                // println!("ADC lock req for ADC ");
                // let mut lock = adc.lock().await;
                // let mut lock = adc.lock().unwrap();
                // // println!("ADC lock aquired for ADC");
                // let v0 = lock.analog_read_byte(Pin::AIN0); // photoresistor
                // let v1 = lock.analog_read_byte(Pin::AIN1); // thermistor
                // let v2 = lock.analog_read_byte(Pin::AIN2); // capacitive soil moisture 1
                // let v3 = lock.analog_read_byte(Pin::AIN3); // capacitive soil moisture 2

                // let v0 = adc.analog_read_byte(Pin::AIN0); // photoresistor
                // let v1 = adc.analog_read_byte(Pin::AIN1); // thermistor
                // let v2 = adc.analog_read_byte(Pin::AIN2); // capacitive soil moisture 1
                // let v3 = adc.analog_read_byte(Pin::AIN3); // capacitive soil moisture 2
                
                // println!("Light {:?}  Temp {:?}    Moist 1 {:?}     Moist 2 {:?} ",&v0, &v1, &v2, &v3);
                // let c0 = light_from_byte(v0.unwrap().into());
                // let c1 = celcius_from_byte(v1.unwrap().into());
                // let c2 = moist_from_byte(v2.unwrap().into());
                // let c3 = moist_from_byte(v3.unwrap().into());
                // println!("Light {:?}  Temp {:?}    Moist 1 {:?}     Moist 2 {:?} ",c0, c1, c2, c3);
            // }
            println!("Brought to you by main");


            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });


    while keepalive.load(Ordering::SeqCst) {
        let now = Local::now(); // Time when this loop starts
                                // activity_led.set_high(); // Act led on when loop running

        // activity_led.set_low();
        tokio::time::sleep(Duration::from_millis(1000)); // main loop interval
    }

    // let shutdown_task = grow::ops::running::Runner::shutdown().await;

    // activity_led.set_low();
    println!("\nCleanup success");
    Ok(()) // main() return
}

fn signal_handler() -> Arc<AtomicBool> {
    let keepalive = Arc::new(AtomicBool::new(true));
    // When a SIGINT (Ctrl-C) or SIGTERM signal is caught, atomically set running to false.
    simple_signal::set_handler(&[Signal::Int, Signal::Term], {
        let running = keepalive.clone();
        move |_| {
            running.store(false, Ordering::SeqCst);
        }
    });

    keepalive
}


mod init;



/// Conversions
// fn celcius_from_byte(value: f32) -> f32 {
//     let coeff_b = 3950f32; // thermistor coefficient
//     let res_r0 = 10000f32; // resistance @ room temperature
//     let res_r1 = 1000f32; // resistance of R1
//     let room_temperature_in_kelvin = 297.15f32;

//     let res_r6: f32 = (res_r1 * value) / (256.0 - value);
//     let kelvin: f32 =
//         1.0 / ((1.0 / room_temperature_in_kelvin) + (1.0 / coeff_b) * (res_r6 / res_r0).ln());
    
//     kelvin - 273.15
// }
// fn moist_from_byte(value: u8) -> f32 {
//     // 115 = 100% moist, 215 = 0% moist
//     (0f32 - value as f32 + 215f32) as f32
// }
// fn light_from_byte(value: u8) -> f32 {
//     // 15(240) = dark, 40 = 5v LED up close, 208(47) = very light,
//     (255f32 - value as f32) as f32
// }


fn celcius_from_byte(value: &f64) -> f64 {
    let coeff_b: f64 = 3950.0;  // thermistor coefficient
    let res_r0: f64 = 10000.0;  // resistance @ room temperature
    let res_r1: f64 = 1000.0;   // resistance of R1
    let room_temperature_in_kelvin: f64 = 297.15; 
    
    let res_r6: f64 = (res_r1*value) / (256.0-value);
    let kelvin: f64 = 1.0 / ( (1.0/room_temperature_in_kelvin) + (1.0/coeff_b) * ( res_r6/res_r0 ).ln() );
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
