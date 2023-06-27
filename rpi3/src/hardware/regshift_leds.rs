use super::conf::*;
use core::error::Error;
use drive_74hc595::ShiftRegister;
use grow::ops::display::DisplayStatus;
use grow::ops::display::Indicator;
use grow::zone::ZoneDisplay;
use rppal::gpio::{Gpio, OutputPin, Trigger};
use std::time::{Duration, Instant};
use tokio::time::sleep;
// use grow::ops::display::DisplayStatus::*;
use embedded_hal::digital::v2::OutputPin as HalOutputPin;
use grow::ops::Board;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use async_trait::async_trait;

#[repr(u8)]
#[derive(Debug)]
// enum Leds {
//     Blue =          0b00000001,
//     TankGreen =     0b00000010,
//     TankYellow =    0b00000100,
//     TankRed =       0b00001000,
//     IrrigationRed = 0b00010000,
//     AirRed =        0b00100000,
//     LightRed =      0b01000000,
//     AuxRed =        0b10000000,
// }
enum Leds {
    Blue =          0b10000000,
    TankGreen =     0b01000000,
    TankYellow =    0b00100000,
    TankRed =       0b00010000,
    IrrigationRed = 0b00001000,
    AirRed =        0b00000100,
    LightRed =      0b00000010,
    AuxRed =        0b00000001,
}

// #[derive(Debug, )]
pub struct Shiftreg {
    reg: Arc<RwLock<ShiftRegister<OutputPin, OutputPin, OutputPin, OutputPin, OutputPin>>>,
    current: u8,
    blink: bool,
    cancel: CancellationToken,
}
#[async_trait]
impl Board for Shiftreg {
    fn init(
        &mut self,
        rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>> {
        // self.blink_all(Duration::from_millis(1000), Duration::from_millis(1000));

        println!("********** Shiftreg init ***********");

        // let cancel = self.cancel.clone();
        // let reg = self.reg.clone();
        // let shutdown_task = tokio::spawn(async move {
        //     tokio::select! {
        //         _ = cancel.cancelled() => {
        //             println!("Cancel token: shut down shiftreg");
        //             reg.write().output_clear();
        //             reg.write().disable_output();
        //         }
        //     }
        // });
        Ok(())
    }

    async fn set(&mut self, zones: Vec<ZoneDisplay>) -> Result<(), Box<dyn Error>> {
        let mut led_byte = 0;

        let mut irrigation_lit = false;
        let mut blue_lit = false;
        for z in zones {
            match z {
                ZoneDisplay::Air {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => led_byte += Leds::AirRed as u8,
                    Indicator::Blue => {
                        if !blue_lit {
                            led_byte += Leds::Blue as u8;
                            blue_lit = true;
                        }
                    },
                    _ => {}
                },
                ZoneDisplay::Aux {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => led_byte += Leds::AuxRed as u8,
                    Indicator::Blue => {
                        if !blue_lit {
                            led_byte += Leds::Blue as u8;
                            blue_lit = true;
                        }
                    },
                    _ => {}
                },
                ZoneDisplay::Light {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => led_byte += Leds::LightRed as u8,
                    Indicator::Blue => {
                        if !blue_lit {
                            led_byte += Leds::Blue as u8;
                            blue_lit = true;
                        }
                    },
                    _ => {}
                },
                ZoneDisplay::Irrigation {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => {
                        if !irrigation_lit {
                            led_byte += Leds::IrrigationRed as u8;
                            irrigation_lit = true;
                        }
                    },
                    Indicator::Blue => {
                        if !blue_lit {
                            led_byte += Leds::Blue as u8;
                            blue_lit = true;
                        }
                    },
                    _ => {}
                },
                ZoneDisplay::Irrigation {
                    id: 2,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => {
                        if !irrigation_lit {
                            led_byte += Leds::IrrigationRed as u8;
                            irrigation_lit = true;
                        }
                    },
                    Indicator::Blue => {
                        if !blue_lit {
                            led_byte += Leds::Blue as u8;
                            blue_lit = true;
                        }
                    },
                    _ => {}
                },
                ZoneDisplay::Pump {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    // Indicator::Red => {} led_byte += Leds::PumpRed as u8,
                    Indicator::Blue => {
                        if !blue_lit {
                            led_byte += Leds::Blue as u8;
                            blue_lit = true;
                        }
                    },
                    _ => {}
                },
                ZoneDisplay::Tank {
                    id: 1,
                    info: DisplayStatus { indicator, msg: _ },
                } => match indicator {
                    Indicator::Red => led_byte += Leds::TankRed as u8,
                    Indicator::Yellow => led_byte += Leds::TankYellow as u8,
                    Indicator::Green => led_byte += Leds::TankGreen as u8,
                    Indicator::Blue => {
                        if !blue_lit {
                            led_byte += Leds::Blue as u8;
                            blue_lit = true;
                        }
                    },
                },
                _ => continue,
            }
        }
        // println!("\tLoading board byte: {:b}", &led_byte);
        self.reg.write().load(led_byte);
        sleep(Duration::from_millis(200));
        self.reg.write().load(led_byte);
        sleep(Duration::from_millis(200));
        self.reg.write().load(led_byte);
        Ok(())
    }

    fn blink_all(&mut self, on: Duration, off: Duration) -> () {
        let mut led_byte: u8 = 0;
        self.blink = true;
        // while self.blink == true {

        let reg = self.reg.clone();
        tokio::spawn(async move {
            loop {
                led_byte = 0b11111111;
                reg.write().load(led_byte);
                tokio::time::sleep(on).await;

                led_byte = 0b00000000;
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
    pub fn new(cancel: CancellationToken) -> Self {
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

        let reg_rw = Arc::new(RwLock::new(reg));

        let reg_clone = reg_rw.clone();
        let cancel_clone = cancel.clone();
        let shutdown_task = tokio::spawn(async move {
            tokio::select! {
                _ = cancel_clone.cancelled() => {
                    println!("Cancel token: shut down shiftreg");
                    reg_clone.write().output_clear();
                    reg_clone.write().disable_output();
                }
            }
        });

        // let mut s = Self {
        //     reg: reg_rw,
        //     current: 0b00000000,
        //     blink: false,
        //     cancel,
        // };
        // s

        Self {
            reg: reg_rw,
            current: 0b00000000,
            blink: false,
            cancel,
        }

    }
}

