#![feature(error_in_core)]
#![allow(unused)]
mod cmd;
mod hardware;
mod init;
use crate::hardware::conf::*;
use grow::ops;
use grow::zone::*;

use std::error::Error;
// use std::thread;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
pub type HouseMutex = Arc<TokioMutex<grow::House>>;
use tokio::signal;
use tokio::sync::mpsc;

use rppal::gpio::{Gpio, OutputPin, Trigger};
use rppal::pwm::{Channel, Polarity, Pwm};

use drive_74hc595::ShiftRegister;
// use dummy_pin::DummyPin;
use lego_powered_up::PoweredUp;
use pcf8591::{Pin, PCF8591};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // construct a subscriber that prints formatted traces to stdout
    // let subscriber = tracing_subscriber::FmtSubscriber::new();
    // use that subscriber to process traces emitted after this point
    // tracing::subscriber::set_global_default(subscriber)?;
    // tracing_subscriber::fmt::try_init()?;

    // console_subscriber::init();
    // console_subscriber::ConsoleLayer::builder()
    // .retention(Duration::from_secs(60))
    // .server_addr(([127, 0, 0, 1], 5555))
    // .server_addr(([192, 168, 0, 106], 9090))
    // .init();

    let (shutdown_send, mut shutdown_recv) = mpsc::unbounded_channel::<bool>();
    let cancel_token = CancellationToken::new();

    let (house, manager) = init::hardware_init(cancel_token.clone()).await;
    let cmd_task = cmd::manual_cmds(house.clone(), manager.clone(), shutdown_send);

    // Buttons
    let mut btn_1 = Gpio::new()?.get(BUTTON_1_PIN)?.into_input_pulldown();
    let mut btn_2 = Gpio::new()?.get(BUTTON_2_PIN)?.into_input_pulldown();
    println!("Button pins initialized");
    btn_1.set_async_interrupt(Trigger::Both, |l| println!("Btn 111: {:?}", l));
    btn_2.set_async_interrupt(Trigger::Both, |l| println!("Btn 2 2 2: {:?}", l));

    // OLED
    // oled::test_oled();

    
    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = shutdown_recv.recv() => {},
    }
    // Cleanup
    cancel_token.cancel();
    cmd_task.unwrap().abort();
    sleep(Duration::from_millis(500)).await;

    // lpu_hub.lock().await.disconnect().await;

    // led_task.abort();
    // let led_byte = 0b00000000;
    // {
    //     let mut lock =
    //         sr_mutex_2.lock().unwrap();
    //     lock.load(led_byte);
    //     lock.output_clear();
    //     lock.disable_output();
    // }
    // sr.load(led_byte);
    // sr.output_clear();
    // sr.disable_output();

    println!("Cleanup successful");
    Ok(())
}
