#![feature(error_in_core)]
#![allow(unused)]
mod hardware;
mod init;
mod cmd;
use crate::hardware::conf::*;
use grow::ops;
use grow::zone::*;

use std::error::Error;
// use std::thread;
use std::time::{Duration, Instant};
use tokio::time::sleep as sleep;

use simple_signal::{self, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
// use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
pub type HouseMutex = Arc<TokioMutex<grow::House>>;


use rppal::gpio::{Gpio, OutputPin, Trigger};
use rppal::pwm::{Channel, Polarity, Pwm};

use drive_74hc595::ShiftRegister;
use dummy_pin::DummyPin;
use pcf8591::{Pin, PCF8591};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let keepalive = signal_handler();
    let lpu_hub = crate::hardware::lpu::init().await.unwrap();
    let mut house = init::house_init(lpu_hub.clone()).await;
    

    // sleep(Duration::from_secs(5)).await;
    // println!("READ LIGHT ONETIME: {:?}", house.read_light_value(&1u8));
    // println!("READ TEMP ONETIME: {:?}", house.read_temperature_value(&1u8));
    // println!("READ MOIST 2 ONETIME: {:?}", house.read_moisture_value(&1u8));
    // println!("READ MOIST 2 ONETIME: {:?}", house.read_moisture_value(&2u8));
    // println!("LAMP ON: {:?}", house.set_lamp_state(&1u8, grow::zone::light::LampState::Off));
    
    // println!("PUMP RUN ON: {:?}", house.run_pump(&1u8, 2u16).await);
    // sleep(Duration::from_secs(5)).await;
    // println!("AXIS X: {:?}", house.arm_goto_x(&1u8, -25i32).await);
    // sleep(Duration::from_secs(5)).await;
    // println!("AXIS Y: {:?}", house.arm_goto_y(&1u8, 235i32).await);

    // Activity
    // let mut activity_led = Gpio::new()?.get(ACTIVITY_LED_PIN)?.into_output();
    // println!("LED pin initialized");

    // Shiftreg leds
    let sr_data = Gpio::new()?.get(INDICATORS_SR_DATA)?.into_output();
    let sr_enable = Gpio::new()?.get(INDICATORS_SR_ENABLE)?.into_output();
    let sr_clk = Gpio::new()?.get(INDICATORS_SR_CLK)?.into_output();
    let sr_latch = Gpio::new()?.get(INDICATORS_SR_LATCH)?.into_output();
    let sr_reset = Gpio::new()?.get(INDICATORS_SR_RESET)?.into_output();
    let mut sr = ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch);
    sr.begin();
    println!("SR initizalied");
    sr.enable_output();
    sr.output_clear();
    let mut led_byte: u8 = 0;

    // Buttons
    let mut btn_1 = Gpio::new()?.get(BUTTON_1_PIN)?.into_input_pulldown();
    let mut btn_2 = Gpio::new()?.get(BUTTON_2_PIN)?.into_input_pulldown();
    println!("Button pins initialized");
    btn_1.set_async_interrupt(Trigger::Both, |l| println!("Btn 111: {:?}", l));
    btn_2.set_async_interrupt(Trigger::Both, |l| println!("Btn 222: {:?}", l));

    // OLED
    // oled::test_oled();

    // let sr_mutex = Arc::new(TokioMutex::new(sr));
    // let sr_mutex_2 = sr_mutex.clone();
    // let led_task = tokio::spawn(async move {
    //     loop { 
    //         // Blink all
    //         led_byte = 0b11111111;        // println!("loading: {:?}", led_byte);
    //         {
    //             sr_mutex.lock().await.load(led_byte);
    //         }
    //         tokio::time::sleep(Duration::from_millis(1000)).await;
    //         led_byte = 0b00000000;
    //         {
    //             sr_mutex.lock().await.load(led_byte);
    //         }
    //         //  sr.load(led_byte);
    //         tokio::time::sleep(Duration::from_millis(1000)).await;
    //     }
    // });

    // let house_mutex = Arc::new(TokioMutex::new(house));

    // let cmd_task = tokio::spawn(async move {
        // cmd::house_cmds(house_mutex).await;
    // });
    // let running = keepalive.clone();
    // cmd::house_cmds(house_mutex);

    while keepalive.load(Ordering::SeqCst) {
        // activity_led.set_high();


        // Blink all
        led_byte = 0b11111111;        // println!("loading: {:?}", led_byte);
        sr.load(led_byte);
        tokio::time::sleep(Duration::from_millis(1000)).await;
        led_byte = 0b00000000;
        sr.load(led_byte);
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // activity_led.set_low();
    }

    // Cleanup
    lpu_hub.lock().await.disconnect().await;
    // led_byte = 0b00000000;
    // { 
    //     let mut lock = sr_mutex_2.lock().await;
    //     lock.load(led_byte);
    //     lock.output_clear();
    //     lock.disable_output();
    // }
    sr.load(led_byte);
    sr.output_clear();
    sr.disable_output();
    // activity_led.set_low();
    println!("Cleanup successful");
    Ok(())
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
