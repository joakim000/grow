#![feature(error_in_core)]
// #![feature(async_closure)]
// #![feature(file_create_new)]

extern crate alloc;
use core::error::Error;
pub type BoxResult<T> = core::result::Result<T, Box<dyn Error>>;
use std::sync::Arc;
use tokio::sync::Mutex;
// use std::sync::Mutex;
pub use tokio::sync::broadcast;
pub use tokio::sync::mpsc;
// use parking_lot::RwLock;
use std::fs::File;
use std::io::Write;

mod error;
pub use error::ZoneError;
pub mod ops;
pub mod zone;
use ops::display::DisplayStatus;
use ops::OpsChannelsTx;
use ops::SysLog;
use zone::light::LampState;
use zone::tank::TankLevel;
use zone::ZoneDisplay;
use zone::*;
use zone::{arm::ArmCmd, pump::PumpCmd, Zone};
pub type HouseMutex = Arc<Mutex<House>>;
pub type ManagerMutex = Arc<Mutex<ops::manager::Manager>>;

// pub const TIME_OFFSET: time::UtcOffset = time::macros::offset!(+1); // CET
pub const TIME_OFFSET: time::UtcOffset = time::macros::offset!(+2); // CEST

#[derive(Debug)]
pub struct House {
    zones: Vec<Zone>,
    ops_tx: OpsChannelsTx,
    zone_tx: ZoneChannelsTx,
}

impl House {
    pub fn new(zone_tx: ZoneChannelsTx, ops_tx: OpsChannelsTx) -> Self {
        Self {
            zones: Vec::new(),
            zone_tx,
            ops_tx,
        }
    }
    pub fn new2(zones: Vec<Zone>, zone_tx: ZoneChannelsTx, ops_tx: OpsChannelsTx) -> Self {
        Self {
            zones,
            zone_tx,
            ops_tx,
        }
    }
    // pub fn zones(&self) -> &Vec<Zone> {
    //     &self.zones
    // }
    pub fn zones(&mut self) -> &mut Vec<Zone> {
        &mut self.zones
    }
    pub fn zones_mut(&mut self) -> &mut Vec<Zone> {
        &mut self.zones
    }
    // Macro candidate
    pub async fn init(&mut self) -> () {
        let zone_channels = self.zone_tx.clone();
        let ops_channels = self.ops_tx.clone();
        for zone in self.zones_mut() {
            match zone {
                Zone::Air {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let mut have_fan = false;
                    let channels = runner.fan_channels();
                    let i = interface
                        .fan
                        .as_mut();
                        if i.is_some() {
                            let _ = i.unwrap().init(channels.0, channels.1);
                            have_fan = true;
                        } 
                    let _ = interface
                        .thermo
                        .as_mut()
                        .unwrap()
                        .init(runner.thermo_feedback_sender()).await;
                    runner.run(
                        settings.clone(),
                        zone_channels.clone(),
                        ops_channels.clone(),
                        have_fan,
                    );
                }
                Zone::Aux {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let _ = interface
                        .auxiliary_device
                        .as_mut()
                        .unwrap()
                        .init(runner.auxiliary_feedback_sender())
                        .await;
                    runner.run(
                        settings.clone(),
                        zone_channels.clone(),
                        ops_channels.clone(),
                    );
                }
                Zone::Light {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let _ = interface
                        .lightmeter
                        .as_mut()
                        .unwrap()
                        .init(runner.lightmeter_feedback_sender());
                    let _ = interface
                        .lamp
                        .as_mut()
                        .unwrap()
                        .init(runner.lamp_cmd_receiver());
                    runner.run(
                        settings.clone(),
                        zone_channels.clone(),
                        ops_channels.clone(),
                    );
                }
                Zone::Water {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let _ = interface
                        .moist
                        .as_mut()
                        .unwrap()
                        .init(runner.moisture_feedback_sender());
                    runner.run(
                        settings.clone(),
                        zone_channels.clone(),
                        ops_channels.clone(),
                    );
                }
                Zone::Tank {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let _ = interface
                        .tank_sensor
                        .as_mut()
                        .unwrap()
                        .init(runner.tank_feedback_sender())
                        .await;
                    runner.run(
                        settings.clone(),
                        zone_channels.clone(),
                        ops_channels.clone(),
                    );
                }
                Zone::Pump {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let _ = interface
                        .pump
                        .as_mut()
                        .unwrap()
                        .init(runner.cmd_receiver(), runner.feedback_sender())
                        .await;
                    runner.run(settings.clone());
                    // runner.run(settings.clone(), zone_channels.clone(), ops_channels.clone() );
                }
                Zone::Arm {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let _ = interface
                        .arm
                        .as_mut()
                        .unwrap()
                        .init(
                            runner.pos_feedback_sender().0,
                            runner.pos_feedback_sender().1,
                            runner.pos_feedback_sender().2,
                            runner.control_feedback_sender(),
                            runner.cmd_receiver(),
                        )
                        .await;
                    runner.run(settings.clone());
                    // let _ = interface.arm.as_ref().unwrap().calibrate();
                    // runner.run(settings.clone(), zone_channels.clone(), ops_channels.clone() );
                } // _ => ()
            }
        }
        let _ = self
            .ops_tx
            .syslog
            .send(SysLog::new(format!("House zones initiated")))
            .await;
    }
    pub fn collect_display_status(&mut self) -> Vec<ZoneDisplay> {
        let mut r: Vec<ZoneDisplay> = Vec::new();
        for zone in self.zones() {
            r.push(zone.zone_display());
        }

        r
    }
    pub fn load_settings(&mut self) -> Result<(), Box<dyn Error>> {
        let readdata = std::fs::read_to_string("grow-conf.js")?;
        println!("File data:{:?}", &readdata);
        let loaddata: Vec<ZoneSave> = serde_json::from_str(&readdata)?;
        println!("Deser:{:?}", &loaddata);
        
        
        Ok(())
    }
    pub fn save_settings(&self) -> Result<(), Box<dyn Error>> {
        let mut savedata: Vec<ZoneSave> = Vec::new();
        for zone in &self.zones {
            match zone {
                Zone::Water {id, settings, ..} => {
                    savedata.push(ZoneSave::Water { id:*id, settings:*settings });
                }
                Zone::Air {id, settings, ..} => {
                    savedata.push(ZoneSave::Air { id:*id, settings:*settings });
                }
                Zone::Light {id, settings, ..} => {
                    savedata.push(ZoneSave::Light { id:*id, settings:*settings });
                }
                Zone::Aux {id, settings, ..} => {
                    savedata.push(ZoneSave::Aux { id:*id, settings:*settings });
                }
                Zone::Tank {id, settings, ..} => {
                    savedata.push(ZoneSave::Tank { id:*id, settings:*settings });
                }
                Zone::Pump {id, settings, ..} => {
                    savedata.push(ZoneSave::Pump { id:*id, settings:*settings });
                }
                Zone::Arm {id, settings, ..} => {
                    savedata.push(ZoneSave::Arm { id:*id, settings:*settings });
                }
            }
        }
        let writestring = serde_json::to_string_pretty(&savedata)?;
        let mut f = File::create("grow-conf.js")?;
        f.write_all(writestring.as_bytes())?;


        Ok(())
    }


    pub fn get_water_settings(
        &mut self,
        zid: u8,
    ) -> Option<zone::water::Settings> {
        let mut r: Option<zone::water::Settings> = None;
        for zone in self.zones() {
            match zone {
                Zone::Water {
                    id,
                    settings,
                    status: _,
                    ..
                } if id == &zid => {
                    // let lock = status.read();
                    r = Some(settings.clone());
                }
                _ => continue,
            }
        }

        r
    }

    pub fn get_displaystatus(
        &mut self,
        kind: ZoneKind,
        zid: u8,
    ) -> Option<DisplayStatus> {
        let mut r: Option<DisplayStatus> = None;
        for zone in self.zones() {
            match zone {
                Zone::Tank { id, status, .. }
                    if (kind == ZoneKind::Tank) & (id == &zid) =>
                {
                    r = Some(status.read().disp.clone());
                }
                _ => continue,
            }
        }

        r
    }

    pub fn set_water_position(&mut self, zid: u8, pos: (i32, i32, i32)) -> () {
        for zone in self.zones_mut() {
            match zone {
                Zone::Water {
                    id,
                    settings,
                    status: _,
                    ..
                } if id == &zid => {
                    settings.position = zone::arm::Position {
                        arm_id: settings.position.arm_id,
                        x: pos.0,
                        y: pos.1,
                        z: pos.2,
                    }
                }
                _ => continue,
            }
        }
    }

    pub fn confirm_arm_position(
        &mut self,
        zid: u8,
        acceptable_delta: u32,
    ) -> Result<(bool, (i32, i32, i32)), Box<dyn Error + '_>> {
        if let Some(ws) = self.get_water_settings(zid) {
            if let Ok(ap) = self.arm_position(ws.position.arm_id) {
                let mut r = true;
                let diff = (
                    (ws.position.x - ap.0),
                    (ws.position.y - ap.1),
                    (ws.position.z - ap.2),
                );
                if (diff.0.abs() as u32 > acceptable_delta)
                    | (diff.0.abs() as u32 > acceptable_delta)
                    | (diff.0.abs() as u32 > acceptable_delta)
                {
                    r = false;
                }
                return Ok((r, diff));
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }

    /// Sensor commands
    pub fn read_moisture_value(
        &mut self,
        zid: u8,
    ) -> Result<f32, Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Water {
                    id,
                    settings: _,
                    status: _,
                    interface,
                    runner: _,
                } if id == &zid => {
                    return Ok(interface
                        .moist
                        .as_ref()
                        .expect("Interface not found")
                        .read()?)
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn read_light_value(
        &mut self,
        zid: u8,
    ) -> Result<f32, Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Light {
                    id,
                    settings: _,
                    status: _,
                    interface,
                    runner: _,
                } if id == &zid => {
                    return Ok(interface
                        .lightmeter
                        .as_ref()
                        .expect("Interface not found")
                        .read()?)
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn read_temperature_value(
        &mut self,
        zid: u8,
    ) -> Result<f64, Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Air { id, interface, .. } if id == &zid => {
                    return Ok(interface
                        .thermo
                        .as_ref()
                        .expect("Interface not found")
                        .read()?)
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn read_tank_level(
        &mut self,
        zid: u8,
    ) -> Result<TankLevel, Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Tank { id, interface, .. } if id == &zid => {
                    return Ok(interface
                        .tank_sensor
                        .as_ref()
                        .expect("Interface not found")
                        .read()?)
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn read_fan_speed(
        &mut self,
        zid: u8,
    ) -> Result<Option<f32>, Box<dyn Error + '_>> {
        for z in self.zones_mut() {
            match z {
                Zone::Air { id, interface, .. } if id == &zid => {
                    return Ok(interface
                        .fan
                        .as_mut()
                        .expect("Interface not found")
                        .read()?)
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }

    /// General action commands
    pub fn set_lamp_state(
        &mut self,
        zid: u8,
        state: LampState,
    ) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Light {
                    id,
                    interface,
                    status,
                    ..
                } if id == &zid => {
                    status.write().lamp_state = Some(state);
                    return Ok(interface
                        .lamp
                        .as_ref()
                        .expect("Interface not found")
                        .set_state(state)?);
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn set_fan_duty_cycle(
        &mut self,
        zid: u8,
        duty_cycle: f64,
    ) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Air { id, interface, .. } if id == &zid => {
                    return Ok(interface
                        .fan
                        .as_ref()
                        .expect("Interface not found")
                        .set_duty_cycle(duty_cycle)?)
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }

    /// Pump commands
    pub async fn pump_run(&mut self, zid: u8) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Pump { id, interface, .. } if id == &zid => {
                    return interface
                        .pump
                        .as_ref()
                        .expect("Interface not found")
                        .run().await
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub async fn pump_stop(&mut self, zid: u8) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Pump { id, interface, .. } if id == &zid => {
                    return interface
                        .pump
                        .as_ref()
                        .expect("Interface not found")
                        .stop().await
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub async fn pump_run_for_secs(
        &mut self,
        zid: u8,
        secs: u16,
    ) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Pump { id, interface, .. } if id == &zid => {
                    return interface
                        .pump
                        .as_ref()
                        .expect("Interface not found")
                        .run_for_secs(secs)
                        .await
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }

    /// Arm commands
    pub async fn arm_goto(
        &mut self,
        zid: u8,
        x: i32,
        y: i32,
        z: i32,
    ) -> Result<(), Box<dyn Error + '_>> {
        for zone in self.zones() {
            match zone {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .goto(x, y, z).await;
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub async fn arm_goto_x(
        &mut self,
        zid: u8,
        x: i32,
    ) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .goto_x(x).await
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub async fn arm_goto_y(
        &mut self,
        zid: u8,
        y: i32,
    ) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .goto_y(y).await
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub async fn arm_update(
        &mut self,
        zid: u8,
    ) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .update_pos()
                        .await;
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn arm_position(
        &mut self,
        zid: u8,
    ) -> Result<(i32, i32, i32), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .position();
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub async fn arm_calibrate(
        &mut self,
        zid: u8,
    ) -> Result<(i32, i32, i32), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .calibrate()
                        .await;
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }

    /// Alternative command model
    pub fn collect_cmd_senders(&mut self) -> Vec<ZoneCmd> {
        let mut r: Vec<ZoneCmd> = Vec::new();
        for zone in self.zones() {
            match zone {
                Zone::Arm { id, runner, .. } => r.push(ZoneCmd::Arm {
                    id: *id,
                    sender: runner.cmd_sender(),
                }),
                Zone::Pump { id, runner, .. } => r.push(ZoneCmd::Pump {
                    id: *id,
                    sender: runner.cmd_sender(),
                }),
                Zone::Light { id, runner, .. } => r.push(ZoneCmd::Light {
                    id: *id,
                    sender: runner.lamp_cmd_sender(),
                }),
                _ => {}
            }
        }
        dbg!(&r);
        r
    }
}

/// Alternative command model
#[derive(Clone, Debug)]
pub enum ZoneCmd {
    // Air {id: u8, info: DisplayStatus},
    // Aux {id: u8, info: DisplayStatus},
    Light {
        id: u8,
        sender: broadcast::Sender<(u8, bool)>,
    },
    // Water {id: u8, info: DisplayStatus},
    Arm {
        id: u8,
        sender: broadcast::Sender<ArmCmd>,
    },
    Pump {
        id: u8,
        sender: broadcast::Sender<(u8, PumpCmd)>,
    },
    // Tank {id: u8, info: DisplayStatus},
}

// pub fn get_tank_status(&self, zid: u8) -> Option<Arc<RwLock<zone::tank::Status>>> {
//     let mut r: Option<Arc<RwLock<zone::tank::Status>>> = None;
//     for zone in self.zones() {
//         match zone {
//             Zone::Tank {
//                 id,
//                 status,
//                 ..
//             } if id == &zid => {
//                 r = Some(status.clone());
//             }
//             _ => continue,
//         }
//     }

//     r
// }

// pub fn get_pump_status(&mut self, zid: u8) -> Option<Arc<RwLock<zone::pump::Status>>> {
//     let mut r: Option<Arc<RwLock<zone::pump::Status>>> = None;
//     for zone in self.zones() {
//         match zone {
//             Zone::Pump {
//                 id,
//                 status,
//                 ..
//             } if id == &zid => {
//                 r = Some(status.clone());
//             }
//             _ => continue,
//         }
//     }

//     r
// }
// pub fn get_arm_status(&mut self, zid: u8) -> Option<Arc<RwLock<zone::arm::Status>>> {
//     let mut r: Option<Arc<RwLock<zone::arm::Status>>> = None;
//     for zone in self.zones() {
//         match zone {
//             Zone::Arm {
//                 id,
//                 status,
//                 ..
//             } if id == &zid => {
//                 r = Some(status.clone());
//             }
//             _ => continue,
//         }
//     }

//     r
// }
