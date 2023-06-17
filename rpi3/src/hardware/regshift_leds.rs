use super::conf::*;

use rppal::gpio::{Gpio, OutputPin, Trigger};
use drive_74hc595::ShiftRegister;
use std::time::{Duration, Instant};
use tokio::time::sleep as sleep;
use grow::Indicator;


// pub struct Shiftreg {
//     sr: ShiftRegister<OE, SER, SRCLR, SRCLK, RCLK>, 

// }

pub struct Shiftreg (
    ShiftRegister<OE, SER, SRCLR, SRCLK, RCLK>, 
);

impl Shiftreg {
    // pub fn new() -> Self {
    //     let sr_data = Gpio::new()?.get(INDICATORS_SR_DATA)?.into_output();
    //     let sr_enable = Gpio::new()?.get(INDICATORS_SR_ENABLE)?.into_output();
    //     let sr_clk = Gpio::new()?.get(INDICATORS_SR_CLK)?.into_output();
    //     let sr_latch = Gpio::new()?.get(INDICATORS_SR_LATCH)?.into_output();
    //     let sr_reset = Gpio::new()?.get(INDICATORS_SR_RESET)?.into_output();
    //     // let mut sr = ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch);
    //     Self {
    //         sr: ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch),
    //     }
    // }
    pub fn new() -> Self {
        let sr_data = Gpio::new()?.get(INDICATORS_SR_DATA)?.into_output();
        let sr_enable = Gpio::new()?.get(INDICATORS_SR_ENABLE)?.into_output();
        let sr_clk = Gpio::new()?.get(INDICATORS_SR_CLK)?.into_output();
        let sr_latch = Gpio::new()?.get(INDICATORS_SR_LATCH)?.into_output();
        let sr_reset = Gpio::new()?.get(INDICATORS_SR_RESET)?.into_output();
        // let mut sr = ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch);
        Self (
            ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch),
        )
    }

    pub fn blink(&self) -> () {
        self.0.begin();
        self.0.enable_output();
        self.0.output_clear();
        let mut led_byte: u8 = 0;

        // Blink all
        loop {
            led_byte = 0b11111111;
            // println!("loading: {:?}", led_byte);
            sr.load(led_byte);
            tokio::time::sleep(Duration::from_millis(1000)).await;
            led_byte = 0b00000000;
            // println!("loading: {:?}", led_byte);
            sr.load(led_byte);
        }
    }
}

    
    