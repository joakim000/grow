extern crate alloc;
use alloc::collections::BTreeMap;
use core::error::Error;
// use parking_lot::Mutex;

use async_trait::async_trait;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use rppal::i2c::I2c;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
type OledDisplay = Ssd1306<
    I2CInterface<I2c>,
    DisplaySize128x64,
    ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
>;

use super::conf::*;
use grow::ops::display::DisplayStatus;
use grow::ops::display::Indicator;
use grow::ops::SysLogTx;
use grow::ops::io::TextDisplay;

use grow::zone::ZoneDisplay;
use grow::zone::ZoneKind;
use grow::zone::ZoneStatusRx;

pub struct Oled {
    cancel: CancellationToken,
}
#[async_trait]
impl TextDisplay for Oled {
    fn init(
        &self,
        from_zones: ZoneStatusRx,
        to_syslog: SysLogTx,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        self.display_control(from_zones, self.cancel.clone(), to_syslog)
    }
    fn set(
        &mut self,
        _status_all: Vec<ZoneDisplay>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
impl Oled {
    pub fn new(cancel: CancellationToken) -> Self {
        Self { cancel }
    }

    fn get_display(&self) -> OledDisplay {
        let mut i2c = I2c::with_bus(SSD1306_BUS).expect("I2C bus not found");
        let _ = i2c.set_slave_address(SSD1306_ADDR);
        println!("i2c bus: {:#?}", i2c.bus());
        println!("i2c speed: {:#?}", i2c.clock_speed());

        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(
            interface,
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();
        display.init().expect("Display init error");

        display
    }

    fn display_control(
        &self,
        mut from_zones: ZoneStatusRx,
        cancel: CancellationToken,
        _to_syslog: SysLogTx,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let mut display = self.get_display();
        let style_heading = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
        let style_msg = MonoTextStyle::new(&FONT_7X14_BOLD, BinaryColor::On);
        let style_dt = MonoTextStyle::new(&FONT_6X13_ITALIC, BinaryColor::On);
        let mut interval = interval(Duration::from_millis(3000));

        Ok(tokio::spawn(async move {
            let mut pagemap: BTreeMap<
                (ZoneKind, u8),
                (String, String, String, String),
            > = BTreeMap::new();
            // let pages: Vec<String> = Vec::new();
            let mut next_page: usize = 0;
            let mut text = (
                String::from("No zone"),
                String::from("No lvl"),
                String::from("No msg"),
                String::from("No time"),
            );
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        let _ = display.set_display_on(false);
                        println!("Oled disabled");
                        break;
                    }
                    _ = interval.tick()  => {
                        let mut pages: Vec<( String, String, String, String )> = pagemap.values().map(|x|x.clone()).collect();
                        // dbg!(&pages); dbg!(&next_page);
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
                        let _next_text = Text::new(&text.0, Point::new(0,14), style_heading).draw(&mut display).unwrap();
                        Text::with_alignment(&text.1, Point::new(128,14), style_heading, Alignment::Right).draw(&mut display).unwrap();
                        Text::new(&text.2, Point::new(0, 30), style_msg).draw(&mut display).unwrap();
                        Text::with_alignment(&text.3, Point::new(128, 60), style_dt, Alignment::Right).draw(&mut display).unwrap(); //thread 'tokio-runtime-worker' panicked at 'called `Result::unwrap()` on an `Err` value: BusWriteError', src/hardware/ssd1306.rs:140:41
                        display.flush().unwrap();
                    }
                    Ok(data) = from_zones.recv() => {
                        match data {
                            ZoneDisplay::Air {id, info} => {
                                let text = Self::format_zonedisplay(id, info, "Luft");
                                pagemap.insert(( ZoneKind::Air, id), text );
                            }
                            ZoneDisplay::Light {id, info} => {
                                let text = Self::format_zonedisplay(id, info, "Ljus");
                                pagemap.insert(( ZoneKind::Light, id), text.clone() );
                            }
                            ZoneDisplay::Tank {id, info} => {
                                let text = Self::format_zonedisplay(id, info, "Tunna");
                                pagemap.insert(( ZoneKind::Tank, id), text.clone() );
                            }
                            ZoneDisplay::Water {id, info} if id == 1 => {
                                let text = Self::format_zonedisplay(id, info, "Planta");
                                pagemap.insert(( ZoneKind::Water, id), text.clone() );
                            }
                            ZoneDisplay::Water {id, info} if id == 2 => {
                                let text = Self::format_zonedisplay(id, info, "Planta");
                                pagemap.insert(( ZoneKind::Water, id), text.clone() );
                            }
                            ZoneDisplay::Aux {id, info} => {
                                let text = ( format!("Lego"),  Self::format_indicator(&info.indicator), Self::format_msg(info.msg), Self::format_time(info.changed) );
                                pagemap.insert(( ZoneKind::Aux, id), text.clone() );
                            }
                            ZoneDisplay::Arm {id, info} => {
                                let text = Self::format_zonedisplay(id, info, "Arm");
                                pagemap.insert(( ZoneKind::Arm, id), text.clone() );
                            }
                            ZoneDisplay::Pump {id, info} => {
                                let text = Self::format_zonedisplay(id, info, "Pump");
                                pagemap.insert(( ZoneKind::Pump, id), text.clone() );
                            }
                            _ => {}
                        }
                    }
                    else => { break }
                };
            }
        }))
    }

    /// Format for display on small screen
    fn format_zonedisplay(
        id: u8,
        info: DisplayStatus,
        show: &str,
    ) -> (String, String, String, String) {
        (
            format!("{} {} ", String::from(show), id),
            Self::format_indicator(&info.indicator),
            Self::format_msg(info.msg),
            Self::format_time(info.changed),
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
            Some(msg) => msg.replace(", ", "\n"),
            None => String::from("No message"),
        }
    }
    fn format_time(dt: OffsetDateTime) -> String {
        // format!("{}", dt.format(&Rfc2822).expect("Time formatting error"))
        let hms = dt.to_hms();
        format!("{} {}:{:02}:{:02}", dt.date(), hms.0, hms.1, hms.2)
    }
}

// macro_rules! format_zonedisplay {
//     ($zone_variant:tt, $show:expr) =>
//     {ZoneDisplay::$zone_variant {id, info} => {
//         let text = ( format!("{} {} ", $show, id),  Self::format_indicator(&info.indicator), Self::format_msg(info.msg), Self::format_time(info.changed) );
//         pagemap.insert(( ZoneKind::$zone_variant, id), text.clone() );
//     }}
// }
