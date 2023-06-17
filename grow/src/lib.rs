#![feature(error_in_core)]

extern crate alloc;
use core::error::Error;
pub type BoxResult<T> = core::result::Result<T, Box<dyn Error>>;
// use alloc::collections::BTreeMap;
use zone::Zone;
// use std::sync::Arc;
// use tokio::sync::Mutex;
pub use tokio::sync::broadcast;

mod error;
pub use error::ZoneError;
pub mod ops;
pub mod zone;
use zone::light::LampState;

#[derive( Debug, )]
pub struct House {
    zones: Vec<Zone>,
}

impl House {
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    pub fn zones(&mut self) -> &mut Vec<Zone> {
        &mut self.zones
    }

    pub fn read_moisture_value(&mut self, zid: &u8) -> Result<f32, Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Irrigation {id, settings:_, status:_, interface, runner: _} if id == zid => {
                    return Ok(interface.moist.as_ref().expect("Interface not found").read()?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn read_light_value(&mut self, zid: &u8) -> Result<f32, Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Light {id, settings:_, status:_, interface, runner: _} if id == zid => {
                    return Ok(interface.lightmeter.as_ref().expect("Interface not found").read()?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn read_temperature_value(&mut self, zid: &u8) -> Result<f32, Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Air {id, settings:_, status:_, interface, runner: _} if id == zid => {
                    return Ok(interface.thermo.as_ref().expect("Interface not found").read()?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn set_lamp_state(&mut self, zid: &u8, state:LampState) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Light {id, settings:_, status:_, interface, runner: _} if id == zid => {
                    return Ok(interface.lamp.as_ref().expect("Interface not found").set_state(state)?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub async fn run_pump(&mut self, zid: &u8, secs:u16) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Pump {id, settings:_, status:_, interface, runner: _} if id == zid => {
                    // interface.pump.as_ref().expect("Interface not found").run_for_secs(secs).await?;
                    // return Ok(())
                    return interface.pump.as_ref().expect("Interface not found").run_for_secs(secs).await
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub async fn arm_goto_x(&mut self, zid: &u8, x: i32) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Arm {id, settings:_, status:_, interface, runner: _} if id == zid => {
                    return interface.arm.as_ref().expect("Interface not found").goto_x(x).await
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub async fn arm_goto_y(&mut self, zid: &u8, y: i32) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Arm {id, settings:_, status:_, interface, runner: _} if id == zid => {
                    return interface.arm.as_ref().expect("Interface not found").goto_y(y).await
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }



    pub async fn init(&mut self) -> () {
        for zone in self.zones() {
            match zone {
                Zone::Air {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let channels = runner.fan_channels();
                    let _ = interface.fan.as_mut().unwrap().init(channels.0, channels.1);
                    let _ = interface.thermo.as_mut().unwrap().init(runner.thermo_channel());
                    runner.run(settings.clone());
                },
                Zone::Light {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.lightmeter.as_mut().unwrap().init(runner.lightmeter_channel());
                    let _ = interface.lamp.as_mut().unwrap().init(runner.lamp_channel());
                    runner.run(settings.clone());
                }
                Zone::Irrigation {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.moist.as_mut().unwrap().init(runner.channel());
                    runner.run(settings.clone());
                }
                Zone::Tank {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.tank_sensor.as_mut().unwrap().init(runner.channel()).await;
                    runner.run(settings.clone());
                }
                Zone::Pump {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.pump.as_mut().unwrap().init(runner.cmd_channel());
                    // let _ = interface.lamp.as_mut().unwrap().init(runner.lamp_channel());
                    runner.run(settings.clone());
                }
                Zone::Arm {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.arm.as_mut().unwrap()
                        .init(runner.channel().0, runner.channel().1, runner.channel().2);
                    runner.run(settings.clone());
                }
                // _ => ()
            }
        }
    }

  
}

impl Default for House {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Indicator {
    Green,
    Yellow,
    Red,
    Blue,
}

