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

    pub fn read_moisture_value(&mut self, zid: &u8) -> Result<f32, ZoneError> { 
        println!("Get moist value. zones.len = {}", self.zones().len() );
        for z in self.zones() {
            dbg!(&z);
            match z {
                Zone::Irrigation {id, settings:_, status:_, interface, runner: _} if id == zid => {
                    return Ok(interface.moist.as_ref().expect("Interface not found").read().expect("Couldn't read value"))
                }
                _ => continue
            }
        }
        return Err(ZoneError::new("Zone not found"));
    }

    // pub fn read_moisture_value(&mut self, id: &u8) -> Result<f32, Box<dyn Error>> { 
    //     println!("Get moist value. zones.len = {}", self.zones().len() );
    //     let mut moist_value = |id: u8| -> Result<f32, Box<dyn Error>> {
    //         for z in self.zones() {
    //             dbg!(&z);
    //             // let id = 1;
    //             match z {
    //                 Zone::Irrigation {id, settings:_, status:_, interface, runner: _} => {
    //                     return Ok(interface.moist.as_ref().unwrap().read().unwrap())
    //                 }
    //                 _ => continue
    //             }
    //         }
    //         return Err(Box::new(ZoneError::new("Zone not found")))
    //     };
    //     moist_value(&id)
    // }

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
                    interface:_,
                    runner,
                } => {
                    // let _ = interface.lightmeter.as_mut().unwrap().init(runner.lightmeter_channel());
                    // let _ = interface.lamp.as_mut().unwrap().init(runner.lamp_channel());
                    runner.run(settings.clone());
                }
                // _ => ()
            }
        }
    }

    // pub fn fan_settings(&self, find_id: &u8) -> Option<zone::air::Settings> {
    //     for z in &self.zones {
    //         match z {
    //             Zone::Air {
    //                 id,
    //                 settings: set,
    //                 status: _,
    //                 interface: _,
    //                 runner: _,
    //             } => {
    //                 if id == find_id {
    //                     return Some(set.clone());
    //                 } else {
    //                     continue;
    //                 }
    //             }
    //             _ => {
    //                 continue;
    //             }
    //         }
    //     }
    //     None
    // }
}

impl Default for House {
    fn default() -> Self {
        Self::new()
    }
}

// use std::ops::Deref;
// impl Deref for House {
//     type Target = Vec<Zone>;
//     fn deref(&self) -> &Vec<Zone> {
//         &self.zones
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Indicator {
    Green,
    Yellow,
    Red,
    Blue,
}

// pub struct HouseMapped {
//     air: BTreeMap<u8, Zone>,
//     arm: BTreeMap<u8, Zone>,
//     irrigation: BTreeMap<u8, Zone>,
//     light: BTreeMap<u8, Zone>,
//     pump: BTreeMap<u8, Zone>,
//     tank: BTreeMap<u8, Zone>,
// }
// impl HouseMapped {
//     fn new() -> Self {
//         Self {
//             air: BTreeMap::new(),
//             arm: BTreeMap::new(),
//             irrigation: BTreeMap::new(),
//             light: BTreeMap::new(),
//             pump: BTreeMap::new(),
//             tank: BTreeMap::new(),
//         }
//     }

//     pub fn fan_settings(&self, find_id: &u8) -> Option<zone::air::Settings> {
//         match self.air.get(find_id) {
//             Some(zone) => match zone {
//                 Zone::Air {
//                     id: _,
//                     set,
//                     status: _,
//                 } => Some(set.clone()),
//                 _ => None,
//             },
//             None => None,
//         }
//     }
// }
