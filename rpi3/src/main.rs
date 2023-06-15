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

// const PUMP_INTERVAL_SECS: u64 = 20;
// const MOISTURE_1_RUN_PUMP: i16 = 50; // Run pump if moisture below this level
//                                      // const MOISTURE_1_WARNING = 30;  // Warn if moisture below this level

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let keepalive = signal_handler();

    let mut house = init::house_init();

    // let run_task = grow::ops::running::Runner::run().await;

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