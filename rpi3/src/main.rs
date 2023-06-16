#![feature(error_in_core)]
#![allow(unused)]
mod hardware;
mod init;
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
    
    sleep(Duration::from_secs(15));
    // let mut moist_value = |id: u8|->f32 {
    //         println!("Get moist value. zones.len = {}", house.zones().len() );
    //         for z in house.zones() {
    //             dbg!(&z);
    //             match z {
    //                 Zone::Irrigation {id, settings:_, status:_, interface, runner: _} => {
    //                     return interface.moist.as_ref().expect("Interface not found").read().expect("Couldn't read value")
    //                 }
    //                 _ => continue
    //             }
    //         }
    //         return 666f32
    // };
    // println!("READ MOIST ONETIME: {}", moist_value(1));

    println!("READ MOIST ONETIME: {}", house.read_moisture_value(&1u8).unwrap());

    // Activity
    let mut activity_led = Gpio::new()?.get(ACTIVITY_LED_PIN)?.into_output();
    println!("LED pin initialized");

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

    while keepalive.load(Ordering::SeqCst) {
        activity_led.set_high();

        // Blink all
        led_byte = 0b11111111;
        // println!("loading: {:?}", led_byte);
        sr.load(led_byte);
        tokio::time::sleep(Duration::from_millis(1000)).await;
        led_byte = 0b00000000;
        // println!("loading: {:?}", led_byte);
        sr.load(led_byte);

        tokio::time::sleep(Duration::from_millis(1000)).await;
        activity_led.set_low();
    }

    // Reset pins

    lpu_hub.lock().await.disconnect().await;

    led_byte = 0b00000000;
    sr.load(led_byte);
    sr.output_clear();
    sr.disable_output();

    activity_led.set_low();

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
