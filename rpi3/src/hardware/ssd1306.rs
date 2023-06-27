use super::conf::*;
use grow::zone::ZoneDisplay;

use core::error::Error;
use std::thread;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use rppal::gpio::{Gpio, OutputPin, Trigger};
use rppal::hal::Timer;
use async_trait::async_trait;
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Baseline, LineHeight, Text, TextStyleBuilder},
};
use rppal::i2c::I2c;
use ssd1306::command::Command;
use ssd1306::{command, mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};

use grow::ops::TextDisplay;

pub struct Oled {}
#[async_trait]
impl TextDisplay for Oled {
    fn init(
        &mut self,
        rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(0xc3);
        println!("i2c bus: {:#?}", i2c.bus());
        println!("i2c speed: {:#?}", i2c.clock_speed());

        let mut interface = I2CDisplayInterface::new(i2c);
        // Commands not working
        // Command::Contrast(0x20).send(&mut interface);
        // Command::EnableScroll(true).send(&mut interface);
        // Command::Invert(true).send(&mut interface);

        let mut display: Ssd1306<
            I2CInterface<I2c>,
            DisplaySize128x64,
            ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
        > = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().unwrap();

        Ok(())
    }
}
impl Oled {
    pub fn new(cancel: CancellationToken) -> Self {
        Self {}
    }

    fn write_test(
        mut display: Ssd1306<
            I2CInterface<I2c>,
            DisplaySize128x64,
            ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
        >,
        // character_style_small: MonoTextStyle<'_, BinaryColor>,
        // character_style_large: MonoTextStyle<'_, BinaryColor>,
        // text_style: embedded_graphics::text::TextStyle,
        // text_style2: embedded_graphics::text::TextStyle,
    ) {
        // Styles
        let character_style_small = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let character_style_large = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(BinaryColor::On)
            .build();

        let text_style2 = TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .line_height(LineHeight::Percent(100))
            .build();

        // Draw
        Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();
        thread::sleep(Duration::from_millis(3000));
        display.clear_buffer();
        display.flush().unwrap();

        Text::with_text_style(
            "First line\nSecond line",
            Point::new(20, 30),
            character_style_small,
            text_style2,
        )
        .draw(&mut display);
        display.flush().unwrap();

        display.clear_buffer();
        display.flush().unwrap();
        // Draw the first text at (20, 30) using the small character style.
        let next = Text::new("small ", Point::new(20, 30), character_style_small)
            .draw(&mut display)
            .unwrap();
        // Draw the second text after the first text using the large character style.
        let next = Text::new("large", next, character_style_large)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
    }

    fn styles() {
        let character_style_small = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let character_style_large = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(BinaryColor::On)
            .build();
        let text_style2 = TextStyleBuilder::new()
            .alignment(Alignment::Left)
            .line_height(LineHeight::Percent(100))
            .build();
    }
}
