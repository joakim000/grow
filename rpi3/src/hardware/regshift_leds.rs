use super::conf::*;
use core::error::Error;
use rppal::gpio::{Gpio, OutputPin, Trigger};
use drive_74hc595::ShiftRegister;
use std::time::{Duration, Instant};
use tokio::time::sleep as sleep;
use grow::ops::display::Indicator;
use grow::ops::display::ZoneDisplay;
use grow::ops::display::DisplayStatus;
use embedded_hal::digital::v2::OutputPin as HalOutputPin;


pub struct Shiftreg {
    // reg: ShiftRegister<OE: HalOutputPin, SER: HalOutputPin, SRCLR: HalOutputPin, SRCLK: HalOutputPin, RCLK: HalOutputPin>, 
    reg: ShiftRegister<OutputPin, OutputPin, OutputPin, OutputPin, OutputPin>, 
    // reg: ShiftRegister,
    current: u8,
}

impl Shiftreg {
    pub fn new() -> Self {
        // let sr_data = Gpio::new()?.get(INDICATORS_SR_DATA)?.into_output();
        // let sr_enable = Gpio::new()?.get(INDICATORS_SR_ENABLE)?.into_output();
        // let sr_clk = Gpio::new()?.get(INDICATORS_SR_CLK)?.into_output();
        // let sr_latch = Gpio::new()?.get(INDICATORS_SR_LATCH)?.into_output();
        // let sr_reset = Gpio::new()?.get(INDICATORS_SR_RESET)?.into_output();
        let sr_data = Gpio::new().expect("New gpio error").get(INDICATORS_SR_DATA).expect("Get pin error").into_output();
        let sr_enable = Gpio::new().expect("New gpio error").get(INDICATORS_SR_ENABLE).expect("Get pin error").into_output();
        let sr_clk = Gpio::new().expect("New gpio error").get(INDICATORS_SR_CLK).expect("Get pin error").into_output();
        let sr_latch = Gpio::new().expect("New gpio error").get(INDICATORS_SR_LATCH).expect("Get pin error").into_output();
        let sr_reset = Gpio::new().expect("New gpio error").get(INDICATORS_SR_RESET).expect("Get pin error").into_output();

        // let mut sr = ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch);
        Self {
            reg: ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch),
            current: 0b00000000,
        }
    }

    pub async fn blink_all(&mut self, on: Duration, off: Duration) -> () {
        self.reg.begin();
        self.reg.enable_output();
        self.reg.output_clear();
        let mut led_byte: u8 = 0;

        // Blink all
        loop {
            led_byte = 0b11111111;
            // println!("loading: {:?}", led_byte);
            self.reg.load(led_byte);
            tokio::time::sleep(on).await;

            led_byte = 0b00000000;
            // println!("loading: {:?}", led_byte);
            self.reg.load(led_byte);
            tokio::time::sleep(off).await;
        }
    }

    pub fn set(&mut self, zones: Vec<ZoneDisplay>) -> Result<(), Box<dyn Error>> {

        Ok(())
    } 

}
    
enum Leds {
    Blue =              0b00000001,
    TankGreen =         0b00000010,
    TankYellow =        0b00000100,
    TankRed =           0b00001000,
    AirRed =            0b00010000,
    LightRed =          0b00100000,
    IrrigationRed =     0b01000000,
    LpuRed =            0b10000000,
}

