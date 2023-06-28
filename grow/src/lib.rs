#![feature(error_in_core)]

extern crate alloc;
use core::error::Error;
pub type BoxResult<T> = core::result::Result<T, Box<dyn Error>>;
use std::sync::Arc;
use ops::SysLog;
use ops::remote::RcInput;
use tokio::sync::Mutex;
use zone::ZoneDisplay;
use zone::arm::Arm;
use zone::{arm::ArmCmd, pump::PumpCmd, Zone};
// use std::sync::Mutex;
use crate::ops::OpsChannelsTx;
pub use tokio::sync::broadcast;
pub use tokio::sync::mpsc;

mod error;
pub use error::ZoneError;
pub mod ops;
pub mod zone;
use zone::light::LampState;
use zone::tank::TankLevel;
use zone::*;
use ops::remote::RcModeExit;

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
        Self { zones: Vec::new(), 
            zone_tx, 
            ops_tx,
        }
    }
    pub fn zones(&mut self) -> &mut Vec<Zone> {
        &mut self.zones
    }
    pub async fn init(&mut self,) -> () {
        let zone_channels = self.zone_tx.clone();
        let ops_channels = self.ops_tx.clone();
        for zone in self.zones() {
            match zone {
                Zone::Air {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let channels = runner.fan_channels();
                    let _ = interface.fan.as_mut().unwrap().init(channels.0, channels.1);
                    let _ = interface
                        .thermo
                        .as_mut()
                        .unwrap()
                        .init(runner.thermo_feedback_sender());
                    runner.run(
                        settings.clone(),
                        zone_channels.clone(),
                        ops_channels.clone(),
                    );
                }
                Zone::Aux {
                    settings,
                    interface,
                    runner,
                    ..
                } => {
                    let _ = interface
                        .aux_device
                        .as_mut()
                        .unwrap()
                        .init(runner.aux_feedback_sender())
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
                            runner.feedback_sender().0,
                            runner.feedback_sender().1,
                            runner.feedback_sender().2,
                            runner.cmd_receiver(),
                        )
                        .await;
                    runner.run(settings.clone());
                    // runner.run(settings.clone(), zone_channels.clone(), ops_channels.clone() );
                } // _ => ()
            }
        }
    }

    // Macro candidate
    pub fn collect_display_status(&mut self) -> Vec<ZoneDisplay> {
        let mut r: Vec<ZoneDisplay> = Vec::new();
        for zone in self.zones() {
            // May be a use for settings later
            match zone {
                Zone::Air {
                    id,
                    settings: _,
                    status,
                    ..
                } => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Air {
                        id: *id,
                        info: lock.disp.clone(),
                    })
                }
                Zone::Aux {
                    id,
                    settings: _,
                    status,
                    ..
                } => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Aux {
                        id: *id,
                        info: lock.disp.clone(),
                    })
                }
                Zone::Light {
                    id,
                    settings: _,
                    status,
                    ..
                } => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Light {
                        id: *id,
                        info: lock.disp.clone(),
                    })
                }
                Zone::Water {
                    id,
                    settings: _,
                    status,
                    ..
                } => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Water {
                        id: *id,
                        info: lock.disp.clone(),
                    })
                }
                Zone::Arm {
                    id,
                    settings: _,
                    status,
                    ..
                } => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Arm {
                        id: *id,
                        info: lock.disp.clone(),
                    })
                }
                Zone::Pump {
                    id,
                    settings: _,
                    status,
                    ..
                } => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Pump {
                        id: *id,
                        info: lock.disp.clone(),
                    })
                }
                Zone::Tank {
                    id,
                    settings: _,
                    status,
                    ..
                } => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Tank {
                        id: *id,
                        info: lock.disp.clone(),
                    })
                }
            }
        }

        // dbg!(&r);
        r
    }

    pub fn get_water_settings(&mut self, zid: u8) -> Option<zone::water::Settings> {
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

        // dbg!(&r);
        r
    }
    // pub fn set_water_position_from_rc(&mut self, zid: u8,  mut rc_rx: mpsc::Receiver<RcInput> ) -> (i32, i32, i32) {
    //     let mut arm_o: Option<Arc<&(dyn Arm + '_)>> = None;
    //     let arm: Arc<&(dyn Arm + '_)>;
    //     for z in self.zones() {
    //         match z {
    //             Zone::Arm {
    //                 id,
    //                 settings: _,
    //                 status: _,
    //                 interface,
    //                 runner: _,
    //             } if id == &zid => {
    //                 // let arm = Arc::new(interface.arm.as_deref().expect("Interface not found"));
    //                 // arm_o = Some(interface.arm.as_deref().expect("Interface not found"));
    //                 arm_o = Some(Arc::new(interface.arm.as_deref().expect("Interface not found")));
    //             }
    //             _ => continue,
    //         }
    //     }
    //     arm = arm_o.expect("Zone not found");
    //     let to_log = self.ops_tx.syslog.clone();
    //     let position_finder = tokio::spawn(async move {
    //         let _ = to_log
    //             .send(SysLog::new(
    //                 format!("Spawned position finder"),
    //             ))
    //             .await;
    //         loop {
    //             tokio::select! {
    //                 Some(data) = rc_rx.recv() => {
    //                     match data {
    //                         RcInput::LeftUp | RcInput::RightUp => {
    //                             arm.stop_x();
    //                         }
    //                         RcInput::DownUp | RcInput::UpUp => {
    //                             arm.stop_y();
    //                         }
    //                         RcInput::Left => {
    //                             arm.start_x(-20);
    //                         }
    //                         RcInput::Right => {
    //                             arm.start_x(20);
    //                         }
    //                         RcInput::Up => {
    //                             arm.start_y(80);
    //                         }
    //                         RcInput::Down => {
    //                             arm.start_y(-80);
    //                         }
    //                         RcInput::Confirm => {
    //                             break RcModeExit::Confirm;
    //                         }
    //                         RcInput::Back => {
    //                             break RcModeExit::Cancel;
    //                         }
    //                         RcInput::Mode => {
    //                         }
    //                         RcInput::Exit => {
    //                             break RcModeExit::Cancel;
    //                         }


    //                         RcInput::ConfirmUp => {
    //                         }
    //                         RcInput::DownUp | RcInput::UpUp => {
    //                         }
    //                         RcInput::BackUp => {
    //                         }
    //                         RcInput::ModeUp => {
    //                         }
    //                     }
    //                 }
    //                 else => { break RcModeExit::ElseExit; }
    //             };
    //         }
          
    //     });


        
    //    (0, 0, 0)
    // }
    pub fn set_water_position(&mut self, zid: u8, pos: (i32, i32, i32)) -> () {
        for zone in self.zones() {
            match zone {
                Zone::Water {
                    id,
                    settings,
                    status: _,
                    ..
                } if id == &zid => {
                    // let lock = status.read();
                    settings.position = zone::arm::Position {
                        arm_id: zid,
                        x: pos.0,
                        y: pos.1,
                        z: pos.2,
                    }
                }
                _ => continue,
            }
        }
    }


    /// Sensor commands
    pub fn read_moisture_value(&mut self, zid: u8) -> Result<f32, Box<dyn Error + '_>> {
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
    pub fn read_light_value(&mut self, zid: u8) -> Result<f32, Box<dyn Error + '_>> {
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
    pub fn read_temperature_value(&mut self, zid: u8) -> Result<f64, Box<dyn Error + '_>> {
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
    pub fn read_tank_level(&mut self, zid: u8) -> Result<TankLevel, Box<dyn Error + '_>> {
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
    pub fn read_fan_speed(&mut self, zid: u8) -> Result<Option<f32>, Box<dyn Error + '_>> {
        for z in self.zones() {
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
    pub fn set_lamp_state(&mut self, zid: u8, state: LampState) -> Result<(), Box<dyn Error + '_>> {
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
    pub fn pump_run(&mut self, zid: u8) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Pump { id, interface, .. } if id == &zid => {
                    return interface.pump.as_ref().expect("Interface not found").run()
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn pump_stop(&mut self, zid: u8) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Pump { id, interface, .. } if id == &zid => {
                    return interface.pump.as_ref().expect("Interface not found").stop()
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
    pub fn arm_goto(&mut self, zid: u8, x: i32, y: i32, z: i32) -> Result<(), Box<dyn Error + '_>> {
        for zone in self.zones() {
            match zone {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .goto(x, y, z);
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn arm_goto_x(&mut self, zid: u8, x: i32) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .goto_x(x)
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub fn arm_goto_y(&mut self, zid: u8, y: i32) -> Result<(), Box<dyn Error + '_>> {
        for z in self.zones() {
            match z {
                Zone::Arm { id, interface, .. } if id == &zid => {
                    return interface
                        .arm
                        .as_ref()
                        .expect("Interface not found")
                        .goto_y(y)
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub async fn arm_update(&mut self, zid: u8) -> Result<(), Box<dyn Error + '_>> {
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
    pub fn arm_position(&mut self, zid: u8) -> Result<(i32, i32, i32), Box<dyn Error + '_>> {
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
    pub async fn arm_calibrate_x(
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
                        .calibrate_x()
                        .await;
                }
                _ => continue,
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")));
    }
    pub async fn arm_calibrate_y(
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
                        .calibrate_y()
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

// impl Default for House {
//     fn default() -> Self {
//         Self::new()
//     }
// }

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
