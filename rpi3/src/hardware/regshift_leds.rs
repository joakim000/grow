use super::conf::*;
use core::error::Error;
use drive_74hc595::ShiftRegister;
use grow::ops::display::DisplayStatus;
use grow::ops::display::Indicator;
use grow::ops::display::ZoneDisplay;
use rppal::gpio::{Gpio, OutputPin, Trigger};
use std::time::{Duration, Instant};
use tokio::time::sleep;
// use grow::ops::display::DisplayStatus::*;
use embedded_hal::digital::v2::OutputPin as HalOutputPin;
use grow::ops::Board;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Shiftreg {
    // reg: ShiftRegister<OutputPin, OutputPin, OutputPin, OutputPin, OutputPin>,
    reg: Arc<RwLock<ShiftRegister<OutputPin, OutputPin, OutputPin, OutputPin, OutputPin>>>,
    current: u8,
    blink: bool,
}
impl Board for Shiftreg {
    fn init(
        &mut self,
        rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>> {

        // self.blink_all(Duration::from_millis(1000), Duration::from_millis(1000));

        Ok(())
    }

    fn set(&mut self, zones: Vec<ZoneDisplay>) -> Result<(), Box<dyn Error>> {
        let mut led_byte = 0;

        for z in zones {
            match z {
                ZoneDisplay::Air {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                        Indicator::Red => led_byte += Leds::AirRed as u8,
                    _ => {}
                },
                ZoneDisplay::Aux {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                        Indicator::Red => led_byte += Leds::AirRed as u8,
                        _ => {}
                    },
                ZoneDisplay::Light {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => led_byte += Leds::LightRed as u8,
                    _ => {}
                },
                ZoneDisplay::Irrigation {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => led_byte += Leds::IrrigationRed as u8,
                    _ => {}
                },
                ZoneDisplay::Irrigation {
                    id: 2,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => led_byte += Leds::IrrigationRed as u8,
                    _ => {}
                },
                ZoneDisplay::Pump {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                }  => 
                    match indicator {
                        Indicator::Red => {} //led_byte += Leds::PumpRed as u8,
                    _ => {}
                },
                ZoneDisplay::Tank {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                }  => 
                    match indicator {
                        Indicator::Red => led_byte += Leds::TankRed as u8,
                        Indicator::Yellow => led_byte += Leds::TankYellow as u8,
                        Indicator::Green => led_byte += Leds::TankGreen as u8,
                        Indicator::Blue => led_byte += Leds::Blue as u8,
                    },
                _ => continue,
            }
        }

        Ok(())
    }

    fn blink_all(&mut self, on: Duration, off: Duration) -> () {
        // self.reg.begin();
        // self.reg.enable_output();
        // self.reg.output_clear();
        let mut led_byte: u8 = 0;
        self.blink = true;
        // Blink all
        // while self.blink == true {
        
        let reg = self.reg.clone();
        tokio::spawn(async move {
            
            loop {
                led_byte = 0b11111111;
                // println!("loading: {:?}", led_byte);
                reg.write().load(led_byte);
                tokio::time::sleep(on).await;

                led_byte = 0b00000000;
                // println!("loading: {:?}", led_byte);
                reg.write().load(led_byte);
                tokio::time::sleep(off).await;
            }
        // }
            });
    }

    

    fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        self.reg.write().output_clear();
        self.reg.write().disable_output();

        Ok(())
    }

}
impl Shiftreg {
    pub fn new() -> Self {
        // let sr_data = Gpio::new()?.get(INDICATORS_SR_DATA)?.into_output();
        // let sr_enable = Gpio::new()?.get(INDICATORS_SR_ENABLE)?.into_output();
        // let sr_clk = Gpio::new()?.get(INDICATORS_SR_CLK)?.into_output();
        // let sr_latch = Gpio::new()?.get(INDICATORS_SR_LATCH)?.into_output();
        // let sr_reset = Gpio::new()?.get(INDICATORS_SR_RESET)?.into_output();
        let sr_data = Gpio::new()
            .expect("New gpio error")
            .get(INDICATORS_SR_DATA)
            .expect("Get pin error")
            .into_output();
        let sr_enable = Gpio::new()
            .expect("New gpio error")
            .get(INDICATORS_SR_ENABLE)
            .expect("Get pin error")
            .into_output();
        let sr_clk = Gpio::new()
            .expect("New gpio error")
            .get(INDICATORS_SR_CLK)
            .expect("Get pin error")
            .into_output();
        let sr_latch = Gpio::new()
            .expect("New gpio error")
            .get(INDICATORS_SR_LATCH)
            .expect("Get pin error")
            .into_output();
        let sr_reset = Gpio::new()
            .expect("New gpio error")
            .get(INDICATORS_SR_RESET)
            .expect("Get pin error")
            .into_output();

        let mut reg = ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch);
        reg.begin();
        reg.enable_output();
        reg.output_clear();

        // let mut sr = ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch);
        let mut s = Self {
            // let reg: ShiftRegister::new(sr_enable, sr_data, sr_reset, sr_clk, sr_latch),
            reg: Arc::new(RwLock::new(reg)),
            current: 0b00000000,
            blink: false,
        };
        // s.reg.begin();
        // s.reg.enable_output();
        // s.reg.output_clear();

        s
    }

   

}

#[repr(u8)]
enum Leds {
    Blue = 0b00000001,
    TankGreen = 0b00000010,
    TankYellow = 0b00000100,
    TankRed = 0b00001000,
    IrrigationRed = 0b00010000,
    AirRed = 0b00100000,
    LightRed = 0b01000000,
    AuxRed = 0b10000000,
}
