use super::conf::*;
use embedded_graphics::text::TextStyle;
use grow::ops::display::DisplayStatus;
use grow::ops::display::Indicator;
use grow::zone::ZoneDisplay;
use grow::zone::ZoneStatusRx;

extern crate alloc;
use alloc::collections::BTreeMap;
use core::error::Error;
// use parking_lot::Mutex;
use async_trait::async_trait;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Baseline, LineHeight, Text, TextStyleBuilder},
};
use parking_lot::RwLock;
use rppal::gpio::{Gpio, OutputPin, Trigger};
use rppal::hal::Timer;
use rppal::i2c::I2c;
use ssd1306::command::Command;
use ssd1306::{command, mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use super::conf::*;
use grow::ops::{OpsChannelsTx, SysLog, SysLogTx, TextDisplay};
type OledDisplay = Ssd1306<
    I2CInterface<I2c>,
    DisplaySize128x64,
    ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
>;
type DisplayMutex = Arc<Mutex<OledDisplay>>;
use grow::zone::ZoneKind;

pub struct Oled {
    cancel: CancellationToken,
    // ops_tx: OpsChannelsTx,
    // display: Arc<RwLock<OledDisplay>>,
    // char_style_large: MonoTextStyle<'a, BinaryColor>,
    // char_style_small: MonoTextStyle<'a, BinaryColor>,
    // text_style_1: MonoTextStyle<'a, BinaryColor>,
    // text_style_2: TextStyle,
}
#[async_trait]
impl TextDisplay for Oled {
    fn init(
        &self,
        mut from_zones: ZoneStatusRx,
        to_syslog: SysLogTx,
        // mut from_syslog: ,
        // mutex: DisplayMutex,
        // display: OledDisplay,
    ) -> Result<(), Box<dyn Error>> {
        // let mut d = self.display.lock().unwrap();
        // match self.display.init() {
        //     Ok(_) => {
        //         self.ops_tx
        //             .syslog
        //             .send(SysLog::new(format!("Oled panel initialized")));
        //     }
        //     Err(e) => {
        //         self.ops_tx
        //             .syslog
        //             .send(SysLog::new(format!("Display init error: {:?}", e)));
        //         // eprintln!("Display init error: {:?}", e);
        //     }
        // }

        let control = self.display_control(from_zones, self.cancel.clone(), to_syslog);

        Ok(())
    }
    fn set(
        &mut self,
        // rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
        rx: tokio::sync::broadcast::Receiver<ZoneDisplay>,
    ) -> Result<(), Box<dyn Error>> {
        // let lock = self.display.lock().unwrap();
        // Oled::write_test(self.display.clone());
        // Oled::write_test();

        // match self.write_test() {
        //     Ok(_) => {
        //         self.ops_tx
        //             .syslog
        //             .send(SysLog::new(format!("Write test ok")));
        //     }
        //     Err(e) => {
        //         self.ops_tx
        //             .syslog
        //             .send(SysLog::new(format!("Write test error: {:?}", e)));
        //     }
        // }

        Ok(())
    }
}
impl Oled {
    pub fn new(cancel: CancellationToken) -> Self {
        // let mut i2c = I2c::with_bus(DISPLAY_BUS).expect("I2C bus not found");
        // i2c.set_slave_address(DISPLAY_ADDR);
        // println!("i2c bus: {:#?}", i2c.bus());
        // println!("i2c speed: {:#?}", i2c.clock_speed());

        // let mut interface = I2CDisplayInterface::new(i2c);
        // let d = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        //     .into_buffered_graphics_mode();

        // let d_rw = Arc::new(RwLock::new(d));
        // let d_clone = d_rw.clone();
        // let cancel_clone = cancel.clone();
        // let shutdown_task = tokio::spawn(async move {
        //     tokio::select! {
        //         _ = cancel_clone.cancelled() => {
        //             d_clone.write().disable_output();
        //             println!("Oled disabled");
        //         }
        //     }
        // });

        Self {
            cancel,
            // ops_tx,
            // display: Arc::new(Mutex::new(d)),
            // display: d_rw,
            // char_style_small: MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
            // char_style_large: MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
            // text_style_1: MonoTextStyleBuilder::new()
            //     .font(&FONT_10X20)
            //     .text_color(BinaryColor::On)
            //     .build(),
            // text_style_2: TextStyleBuilder::new()
            //     .alignment(Alignment::Left)
            //     .line_height(LineHeight::Percent(100))
            //     .build(),
        }
    }

    fn get_display(&self) -> OledDisplay {
        let mut i2c = I2c::with_bus(DISPLAY_BUS).expect("I2C bus not found");
        i2c.set_slave_address(DISPLAY_ADDR);
        println!("i2c bus: {:#?}", i2c.bus());
        println!("i2c speed: {:#?}", i2c.clock_speed());

        let mut interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().expect("Display init error");

        display
    }

    fn write_test(
        &mut self,
        // display: OledDisplay,
        // display_m: DisplayMutex,
    ) -> Result<(), Box<dyn Error>> {
        // let display = display_m.lock().unwrap();

        // // Draw
        // Text::with_baseline("Hello world!", Point::zero(), self.text_style_1, Baseline::Top)
        //     .draw(&mut self.display)
        //     .unwrap();

        // Text::with_baseline("Hello Rust!", Point::new(0, 16), self.text_style_1, Baseline::Top)
        //     .draw(&mut self.display)
        //     .unwrap();

        // self.display.flush().expect("Flush error");     // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: BusWriteError', src/hardware/ssd1306.rs:127:30
        // thread::sleep(Duration::from_millis(3000));
        // self.display.clear_buffer();
        // self.display.flush().expect("Flush error");

        // Text::with_text_style(
        //     "First line\nSecond line",
        //     Point::new(20, 30),
        //     self.char_style_small,
        //     self.text_style_2,
        // )
        // .draw(&mut self.display);
        // self.display.flush().expect("Flush error");

        // self.display.clear_buffer();
        // self.display.flush().expect("Flush error");
        // // Draw the first text at (20, 30) using the small character style.
        // let next = Text::new("small ", Point::new(20, 30), self.char_style_small)
        //     .draw(&mut self.display)
        //     .unwrap();
        // // Draw the second text after the first text using the large character style.
        // let next = Text::new("large", next, self.char_style_large)
        //     .draw(&mut self.display)
        //     .unwrap();
        // self.display.flush().expect("Flush error");

        Ok(())
    }

    fn display_control(
        &self,
        mut from_zones: ZoneStatusRx,
        cancel: CancellationToken,
        to_syslog: SysLogTx,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let mut display = self.get_display();
        let character_style_small = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let character_style_large = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
        let text_style_big = MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(BinaryColor::On)
            .build();
        let text_style2 = TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .line_height(LineHeight::Percent(100))
            .build();
        let text_style_big_rightalign = TextStyleBuilder::new()
            // .font(&FONT_10X20)
            // .text_color(BinaryColor::On)
            .alignment(Alignment::Right)
            .build();
        let mut interval = interval(Duration::from_millis(3000));

        Ok(tokio::spawn(async move {
            let mut pagemap: BTreeMap<(ZoneKind, u8), (String, String, String)> = BTreeMap::new();
            // let pages: Vec<String> = Vec::new();
            let mut next_page: usize = 0;
            let mut text = (
                String::from("No zone"),
                String::from("No lvl"),
                String::from("No msg"),
            );
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        // display.clear();
                        display.set_display_on(false);

                        println!("Oled disabled");
                        break;
                    }
                    _ = interval.tick()  => {
                        let mut pages: Vec<( String, String, String )> = pagemap.values().map(|x|x.clone()).collect();
                        dbg!(&pages); dbg!(&next_page);
                        if pages.len() > 0 {
                            if pages.len() > next_page {
                                text = pages.remove(next_page);
                                next_page += 1;
                            }
                            else {
                                text = pages.remove(0);
                                next_page = 1;
                            }
                        }
                        display.clear_buffer();
                        let next_text = Text::new(&text.0, Point::new(0,20), text_style_big).draw(&mut display).unwrap();
                        Text::with_alignment(&text.1, Point::new(128,20), text_style_big, Alignment::Right).draw(&mut display).unwrap();
                        Text::new(&text.2, Point::new(0, 34), character_style_small).draw(&mut display).unwrap();
                        display.flush().unwrap();
                    }
                    Ok(data) = from_zones.recv() => {
                        match data {
                            ZoneDisplay::Air {id, info} => {
                                // let text = format!("Air {} {}", id, Self::format_displaystatus(&info));
                                let text = ( format!("Air {} ", id),  Self::format_indicator(&info.indicator), Self::format_msg(info.msg) );
                                pagemap.insert(( ZoneKind::Air, id), text.clone() );
                            }
                            ZoneDisplay::Light {id, info} => {
                                let text = ( format!("Light {} ", id),  Self::format_indicator(&info.indicator), Self::format_msg(info.msg) );
                                pagemap.insert(( ZoneKind::Light, id), text.clone() );
                            }
                            ZoneDisplay::Tank {id, info} => {
                                let text = ( format!("Tank {} ", id),  Self::format_indicator(&info.indicator), Self::format_msg(info.msg) );
                                pagemap.insert(( ZoneKind::Tank, id), text.clone() );
                            }
                            ZoneDisplay::Water {id, info} if id == 1 => {
                                let text = ( format!("Plant {} ", id),  Self::format_indicator(&info.indicator), Self::format_msg(info.msg) );
                                pagemap.insert(( ZoneKind::Water, id), text.clone() );
                            }
                            ZoneDisplay::Tank {id, info} if id == 2 => {
                                let text = ( format!("Plant {} ", id),  Self::format_indicator(&info.indicator), Self::format_msg(info.msg) );
                                pagemap.insert(( ZoneKind::Water, id), text.clone() );
                            }
                            ZoneDisplay::Aux {id, info} => {
                                let text = ( format!("Lego",    ),  Self::format_indicator(&info.indicator), Self::format_msg(info.msg) );
                                pagemap.insert(( ZoneKind::Aux, id), text.clone() );
                            }
                            _ => {}
                        }
                    }
                    else => { break }
                };
            }
        }))
    }

    fn format_zonedisplay(zd: &ZoneDisplay) -> String {
        match zd {
            ZoneDisplay::Air { id, info } => format!(
                "Air {} {} {}",
                id,
                Self::format_indicator(&info.indicator),
                info.msg.as_ref().unwrap_or(&String::from("No msg"))
            ),
            _ => String::from("Unkowns zone"),
        }
    }

    fn format_displaystatus(ds: DisplayStatus) -> String {
        format!(
            "{} {}",
            Self::format_indicator(&ds.indicator),
            ds.msg.as_ref().unwrap_or(&String::from("No msg"))
        )
    }

    fn format_indicator(i: &Indicator) -> String {
        match i {
            Indicator::Blue => String::from("Blue"),
            Indicator::Red => String::from("Red"),
            Indicator::Yellow => String::from("Yellow"),
            Indicator::Green => String::from("Green"),
        }
    }
    fn format_msg(msg: Option<String>) -> String {
        match msg {
            Some(msg) => msg,
            None => String::from("No msg"),
        }
    }
}

// fn init(
//     &mut self,
//     rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
// ) -> Result<(), Box<dyn Error>> {
//     let mut i2c = I2c::new()?;
//     i2c.set_slave_address(0xc3);
//     println!("i2c bus: {:#?}", i2c.bus());
//     println!("i2c speed: {:#?}", i2c.clock_speed());

//     let mut interface = I2CDisplayInterface::new(i2c);
//     // Commands not working
//     // Command::Contrast(0x20).send(&mut interface);
//     // Command::EnableScroll(true).send(&mut interface);
//     // Command::Invert(true).send(&mut interface);

//     let mut display: Ssd1306<
//         I2CInterface<I2c>,
//         DisplaySize128x64,
//         ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
//     > = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
//         .into_buffered_graphics_mode();
//     display.init().unwrap();

//     Ok(())
// }

// fn write_test(
//     // display: OledDisplay,
//     display_m: DisplayMutex,

//     // character_style_small: MonoTextStyle<'_, BinaryColor>,
//     // character_style_large: MonoTextStyle<'_, BinaryColor>,
//     // text_style: embedded_graphics::text::TextStyle,
//     // text_style2: embedded_graphics::text::TextStyle,
// ) {
//     let display = display_m.lock().unwrap();

//     // Styles
//     let character_style_small = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
//     let character_style_large = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

//     let text_style = MonoTextStyleBuilder::new()
//         .font(&FONT_10X20)
//         .text_color(BinaryColor::On)
//         .build();

//     let text_style2 = TextStyleBuilder::new()
//         .alignment(Alignment::Left)
//         .line_height(LineHeight::Percent(100))
//         .build();

//     // Draw
//     Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
//         .draw(&mut display)
//         .unwrap();

//     Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
//         .draw(&mut display)
//         .unwrap();

//     display.flush().unwrap();
//     thread::sleep(Duration::from_millis(3000));
//     display.clear_buffer();
//     display.flush().unwrap();

//     Text::with_text_style(
//         "First line\nSecond line",
//         Point::new(20, 30),
//         character_style_small,
//         text_style2,
//     )
//     .draw(&mut display);
//     display.flush().unwrap();

//     display.clear_buffer();
//     display.flush().unwrap();
//     // Draw the first text at (20, 30) using the small character style.
//     let next = Text::new("small ", Point::new(20, 30), character_style_small)
//         .draw(&mut display)
//         .unwrap();
//     // Draw the second text after the first text using the large character style.
//     let next = Text::new("large", next, character_style_large)
//         .draw(&mut display)
//         .unwrap();
//     display.flush().unwrap();
// }

// fn styles() {
//     let character_style_small = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
//     let character_style_large = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
//     let text_style = MonoTextStyleBuilder::new()
//         .font(&FONT_10X20)
//         .text_color(BinaryColor::On)
//         .build();
//     let text_style2 = TextStyleBuilder::new()
//         .alignment(Alignment::Left)
//         .line_height(LineHeight::Percent(100))
//         .build();
// }

// match display.init() {
//     Ok(_) => {
//         self.ops_tx
//             .syslog
//             .send(SysLog::new(format!("Oled panel initialized")));
//     }
//     Err(e) => {
//         self.ops_tx
//             .syslog
//             .send(SysLog::new(format!("Display init error: {:?}", e)));
//         // eprintln!("Display init error: {:?}", e);
//     }
// }

// struct Styles {
//     character_style_small = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
//     character_style_large = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
//     text_style = MonoTextStyleBuilder::new()
//     .font(&FONT_10X20)
//     .text_color(BinaryColor::On)
//     .build();
//     text_style2 = TextStyleBuilder::new()
//     .alignment(Alignment::Left)
//     .line_height(LineHeight::Percent(100))
//     .build();
// }
// impl Styles {
//     fn new() {
//         let character_style_small = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
//         let character_style_large = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
//         let text_style = MonoTextStyleBuilder::new()
//             .font(&FONT_10X20)
//             .text_color(BinaryColor::On)
//             .build();
//         let text_style2 = TextStyleBuilder::new()
//             .alignment(Alignment::Left)
//             .line_height(LineHeight::Percent(100))
//             .build();
//     }
// }
