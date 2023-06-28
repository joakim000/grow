use super::conf::*;
use embedded_graphics::text::TextStyle;
use grow::zone::ZoneDisplay;

use core::error::Error;
// use parking_lot::Mutex;
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use async_trait::async_trait;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Baseline, LineHeight, Text, TextStyleBuilder},
};
use rppal::gpio::{Gpio, OutputPin, Trigger};
use rppal::hal::Timer;
use rppal::i2c::I2c;
use ssd1306::command::Command;
use ssd1306::{command, mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};

use super::conf::*;
use grow::ops::{OpsChannelsTx, SysLog, TextDisplay};
type OledDisplay = Ssd1306<
    I2CInterface<I2c>,
    DisplaySize128x64,
    ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
>;
type DisplayMutex = Arc<Mutex<OledDisplay>>;

pub struct Oled<'a> {
    cancel: CancellationToken,
    ops_tx: OpsChannelsTx,
    // display: DisplayMutex,
    display: OledDisplay,
    char_style_large: MonoTextStyle<'a, BinaryColor>,
    char_style_small: MonoTextStyle<'a, BinaryColor>,
    text_style_1: MonoTextStyle<'a, BinaryColor>,
    text_style_2: TextStyle,
}
#[async_trait]
impl TextDisplay for Oled<'_> {
    fn init(
        &mut self,
        // rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>> {
        // let mut d = self.display.lock().unwrap();
        match self.display.init() {
            Ok(_) => {
                self.ops_tx
                    .syslog
                    .send(SysLog::new(format!("Oled panel initialized")));
            }
            Err(e) => {
                self.ops_tx
                    .syslog
                    .send(SysLog::new(format!("Display init error: {:?}", e)));
                // eprintln!("Display init error: {:?}", e);
            }
        }

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
        
        match self.write_test() {
            Ok(_) => { 
                self.ops_tx
                    .syslog
                    .send(SysLog::new(format!("Write test ok")));
            }
            Err(e) => {
                self.ops_tx
                    .syslog
                    .send(SysLog::new(format!("Write test error: {:?}", e)));
            }
        }

        Ok(())
    }
}
impl Oled<'_> {
    pub fn new(cancel: CancellationToken, ops_tx: OpsChannelsTx) -> Self {
        let mut i2c = I2c::with_bus(DISPLAY_BUS).expect("I2C bus not found");
        i2c.set_slave_address(DISPLAY_ADDR);
        println!("i2c bus: {:#?}", i2c.bus());
        println!("i2c speed: {:#?}", i2c.clock_speed());

        let mut interface = I2CDisplayInterface::new(i2c);
        let d = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        Self {
            cancel,
            ops_tx,
            // display: Arc::new(Mutex::new(d)),
            display: d,
            char_style_small: MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
            char_style_large: MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
            text_style_1: MonoTextStyleBuilder::new()
                .font(&FONT_10X20)
                .text_color(BinaryColor::On)
                .build(),
            text_style_2: TextStyleBuilder::new()
                .alignment(Alignment::Left)
                .line_height(LineHeight::Percent(100))
                .build(),
        }
    }

    fn write_test(&mut self,
        // display: OledDisplay,
        // display_m: DisplayMutex,
    ) -> Result<(), Box<dyn Error>> {
        // let display = display_m.lock().unwrap();
        
        // Draw
        Text::with_baseline("Hello world!", Point::zero(), self.text_style_1, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        Text::with_baseline("Hello Rust!", Point::new(0, 16), self.text_style_1, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush().expect("Flush error");     // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: BusWriteError', src/hardware/ssd1306.rs:127:30
        thread::sleep(Duration::from_millis(3000));
        self.display.clear_buffer();
        self.display.flush().expect("Flush error");

        Text::with_text_style(
            "First line\nSecond line",
            Point::new(20, 30),
            self.char_style_small,
            self.text_style_2,
        )
        .draw(&mut self.display);
        self.display.flush().expect("Flush error");

        self.display.clear_buffer();
        self.display.flush().expect("Flush error");
        // Draw the first text at (20, 30) using the small character style.
        let next = Text::new("small ", Point::new(20, 30), self.char_style_small)
            .draw(&mut self.display)
            .unwrap();
        // Draw the second text after the first text using the large character style.
        let next = Text::new("large", next, self.char_style_large)
            .draw(&mut self.display)
            .unwrap();
        self.display.flush().expect("Flush error");

        Ok(())
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