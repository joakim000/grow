#![feature(error_in_core)]

extern crate alloc;
use core::error::Error;
pub type BoxResult<T> = core::result::Result<T, Box<dyn Error>>;
use ops::display::{ZoneDisplay, };
use text_io::read;
// use alloc::collections::BTreeMap;
use zone::{Zone, pump::PumpCmd, arm::ArmCmd};
use std::sync::Arc;
use tokio::{sync::Mutex, };
// use std::sync::Mutex;
pub use tokio::sync::broadcast;

mod error;
pub use error::ZoneError;
pub mod ops;
pub mod zone;
use zone::light::LampState;

pub type HouseMutex = Arc<Mutex<House>>;
pub type ManagerMutex = Arc<Mutex<ops::running::Manager>>;


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

    pub fn read_moisture_value(&mut self, zid: u8) -> Result<f32, Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Irrigation {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return Ok(interface.moist.as_ref().expect("Interface not found").read()?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn read_light_value(&mut self, zid: u8) -> Result<f32, Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Light {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return Ok(interface.lightmeter.as_ref().expect("Interface not found").read()?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn read_temperature_value(&mut self, zid: u8) -> Result<f64, Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Air {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return Ok(interface.thermo.as_ref().expect("Interface not found").read()?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn set_lamp_state(&mut self, zid: u8, state:LampState) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Light {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return Ok(interface.lamp.as_ref().expect("Interface not found").set_state(state)?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn read_fan_speed(&mut self, zid: u8) -> Result<Option<f32>, Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Air {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return Ok(interface.fan.as_mut().expect("Interface not found").read()?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn set_duty_cycle(&mut self, zid: u8, duty_cycle: f64) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Air {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return Ok(interface.fan.as_ref().expect("Interface not found").set_duty_cycle(duty_cycle)?)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub async fn run_pump_for_secs(&mut self, zid: u8, secs:u16) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Pump {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return interface.pump.as_ref().expect("Interface not found").run_for_secs(secs).await
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn run_pump(&mut self, zid: u8) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Pump {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return interface.pump.as_ref().expect("Interface not found").run()
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn stop_pump(&mut self, zid: u8,) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Pump {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return interface.pump.as_ref().expect("Interface not found").stop()
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn arm_goto_x(&mut self, zid: u8, x: i32) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Arm {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return interface.arm.as_ref().expect("Interface not found").goto_x(x)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn arm_goto_y(&mut self, zid: u8, y: i32) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Arm {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return interface.arm.as_ref().expect("Interface not found").goto_y(y)
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub async fn arm_update(&mut self, zid: u8) -> Result<(), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Arm {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return interface.arm.as_ref().expect("Interface not found").update_pos().await;
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }
    pub fn arm_position(&mut self, zid: u8) -> Result<(i32, i32), Box<dyn Error + '_>> { 
        for z in self.zones() {
            match z {
                Zone::Arm {id, settings:_, status:_, interface, runner: _} if id == &zid => {
                    return interface.arm.as_ref().expect("Interface not found").position();
                }
                _ => continue
            }
        }
        return Err(Box::new(ZoneError::new("Zone not found")))
    }

    pub fn collect_display_status(&mut self) -> Vec<ZoneDisplay> {
        let mut r: Vec<ZoneDisplay> = Vec::new();
        for zone in self.zones() {
            // May be a use for settings later    
            match zone {
                Zone::Air{id, settings:_, status, ..} => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Air { id: *id, info: lock.disp.clone() })
                }
                Zone::Aux{id, settings:_, status, ..} => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Aux { id: *id, info: lock.disp.clone() })
                }
                Zone::Light{id, settings:_, status, ..} => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Light { id: *id, info: lock.disp.clone() })
                }
                Zone::Irrigation{id, settings:_, status, ..} => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Irrigation { id: *id, info: lock.disp.clone() })
                }
                Zone::Arm{id, settings:_, status, ..} => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Arm { id: *id, info: lock.disp.clone() })
                }
                Zone::Pump{id, settings:_, status, ..} => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Pump { id: *id, info: lock.disp.clone() })
                }
                Zone::Tank{id, settings:_, status, ..} => {
                    let lock = status.read();
                    r.push(ZoneDisplay::Tank { id: *id, info: lock.disp.clone() })
                }
            }    
        }

        dbg!(&r);
        r
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
                    let _ = interface.fan.as_mut().unwrap()
                        .init(channels.0, channels.1);
                    let _ = interface.thermo.as_mut().unwrap()
                        .init(runner.thermo_channel());
                    runner.run(settings.clone());
                },
                Zone::Aux {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.aux_device.as_mut().unwrap()
                        .init(runner.channel()).await;
                    runner.run(settings.clone());
                },
                Zone::Light {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.lightmeter.as_mut().unwrap()
                        .init(runner.lightmeter_feedback_sender());
                    let _ = interface.lamp.as_mut().unwrap()
                        .init(runner.lamp_cmd_receiver());
                    runner.run(settings.clone());
                }
                Zone::Irrigation {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.moist.as_mut().unwrap()
                        .init(runner.channel());
                    runner.run(settings.clone());
                }
                Zone::Tank {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.tank_sensor.as_mut().unwrap()
                        .init(runner.channel()).await;
                    runner.run(settings.clone());
                }
                Zone::Pump {
                    id: _,
                    settings,
                    status: _,
                    interface,
                    runner,
                } => {
                    let _ = interface.pump.as_mut().unwrap()
                        .init(runner.cmd_receiver(), runner.feedback_sender()).await;
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
                        .init(runner.feedback_sender().0, 
                              runner.feedback_sender().1, 
                              runner.feedback_sender().2,
                              runner.cmd_receiver(),
                        ).await;
                    runner.run(settings.clone());
                }
                // _ => ()
            }
        }

        
        // let _ = self.house_cmds();
    }

    pub fn collect_cmd_senders(&mut self) -> Vec<ZoneCmd> {
        let mut r: Vec<ZoneCmd> = Vec::new();
        // let mut lock = house.lock().await;
        // let mut lock = tokio::task::block_in_place(move || {
            // house.blocking_lock_owned()
        // });
        for zone in self.zones() {
            match zone {
                Zone::Arm{id, runner, ..} => {
                    r.push(ZoneCmd::Arm { id: *id, sender: runner.cmd_sender() })
                }
                Zone::Pump{id, runner, ..} => {
                    r.push(ZoneCmd::Pump { id: *id, sender: runner.cmd_sender() })
                }
                Zone::Light{id, runner, ..} => {
                    r.push(ZoneCmd::Light { id: *id, sender: runner.lamp_cmd_sender() })
                }
                _ => {}
            }    
        }
        dbg!(&r);
        r
    }
  



pub fn house_cmds(&mut self) //manager: ManagerMutex) 
    // -> Result<JoinHandle<()>, Box<dyn Error>> {
        -> () {
    // let senders = Arc::new(self.collect_cmd_senders());
    let senders = self.collect_cmd_senders();
    // let foo = senders.clone();
    // let house = Arc::new(Mutex::new(self));
    // Ok(
        tokio::spawn(async move {
        loop {
            print!("(l)ist cmds, or (q)uit > ");
            let line: String = read!("{}\n");
            if (line.len() == 0) | line.starts_with("\r") {
                continue;
            }
            match line {
                // Sync commands
                // line if line.contains("moist") => {
                //     print!("Read moisture from Irrigation zone > ");
                //     let line: String = read!("{}\n");
                //     let zid = line.trim().parse::<u8>().unwrap();
                //     let mut lock = house.lock().await;
                //     let response = lock.read_moisture_value(zid);
                //     println!("Irrigation zone {} moisture: {:?}", &zid, &response);
                // }
                // line if line.contains("light1") => {
                //     let mut lock = house.lock().await;
                //     let response = lock.read_light_value(1u8);
                //     println!("Light zone {} brightness: {:?}", 1, &response);
                // }
                // line if line.contains("temp1") => {
                //     let mut lock = house.lock().await;
                //     let response = lock.read_temperature_value(1u8);
                //     println!("Air zone {} temperature: {:?}", 1, &response);
                // }
              
                line if line.contains("lamp1onc") => {
                    let fake_input = 1u8;
                    // let mut cmd: Option<broadcast::Sender<(u8,LampState)>> = None;
                    // tokio::spawn(async move {
                    for z in &senders {
                        match z {
                            ZoneCmd::Light { id, sender } if id == &fake_input => {
                                println!("Light zone {} found, sending cmd.", &fake_input);
                                let result = sender.send((fake_input, true));
                                println!("Lamp cmd result: {:?}", &result);
                                tokio::task::yield_now().await;
                                // println!("Pump zone {} found, getting sender.", &fake_input);
                                // pump_cmd = Some(sender.clone());
                            }
                            _ => {
                            }
                        }
                    }
                // });
                }
                // line if line.contains("lamp1on") => {
                //     let _ = house.lock().await.set_lamp_state(1u8, LampState::On);
                // }

      
                line if line.contains("lamp1offc") => {
                    let fake_input = 1u8;
                    // let mut cmd: Option<broadcast::Sender<(u8,LampState)>> = None;
                    // let s = senders.clone();
                    for z in &senders {
                        match z {
                            ZoneCmd::Light { id, sender } if id == &fake_input => {
                                println!("Light zone {} found, sending cmd.", &fake_input);
                                let result = sender.send((fake_input, false));
                                println!("Lamp cmd result: {:?}", &result);

                                // println!("Pump zone {} found, getting sender.", &fake_input);
                                // pump_cmd = Some(sender.clone());
                            }
                            _ => {
                            }
                        }
                    }
                }
                // line if line.contains("lamp1off") => {
                //     let _ = house.lock().await.set_lamp_state(1u8, LampState::Off);
                // }
                // line if line.contains("fan1dc") => {
                //     print!("Fan 1 duty cycle > ");
                //     let line: String = read!("{}\n");
                //     let input = line.trim().parse::<f64>().unwrap();
                //     let _ = house.lock().await.set_duty_cycle(1, input);
                // }
                // line if line.contains("fan1") => {
                //     let mut lock = house.lock().await;
                //     let response = lock.read_fan_speed(1u8);
                //     println!("Air zone {} fan speed: {:?}", 1, &response);
                // }
                // line if line.contains("board") => {
                //     let _ = house.lock().await.collect_display_status();
                // }
          
                // Async commands use command channels
                line if line.contains("pump1c") => {
                    let fake_input = 1u8;
                    // let mut pump_cmd: Option<broadcast::Sender<(u8,PumpCmd)>> = None;

                    for z in &senders {
                        match z {
                            ZoneCmd::Pump { id, sender } if id == &fake_input => {
                                println!("Pump zone {} found, sending cmd.", &fake_input);
                                let result = sender.send((1, PumpCmd::RunForSec(2)));
                                println!("Pump cmd result: {:?}", &result);
                            }
                            _ => {
                            }
                        }
                    }
                }
                // line if line.contains("pump1") => {
                //     // let m = house.clone();
                //     // tokio::spawn(async move {
                //         let mut lock = house.lock().await;
                //         let _ = lock.run_pump(1u8, 2).await;
                //     // });
                // }
                // line if line.contains("ps") => {
                //     let mut lock = house.lock().await;
                //     let _ = lock.stop_pump(1u8).await;
                // }
                // line if line.contains("arm1x") => {
                //     print!("Arm 1 goto X > ");
                //     let line: String = read!("{}\n");
                //     let pos = line.trim().parse::<i32>().unwrap();
                //     let mut lock = house.lock().await;
                //     let _ = lock.arm_goto_x(1u8, pos).await;
                // }
                // line if line.contains("arm1y") => {
                //     print!("Arm 1 goto Y > ");
                //     let line: String = read!("{}\n");
                //     let pos = line.trim().parse::<i32>().unwrap();
                //     let mut lock = house.lock().await;
                //     let _ = lock.arm_goto_y(1u8, pos).await;
                // }
               
                // line if line.contains("remote") => {    // Connect to remote
                //     println!("Calling pos from rc");
                //     let _ = manager.lock().await.position_from_rc(1).await;
                // }
              
                line if line.contains("q") => {
                    break; // Ok(())
                }
                String { .. } => (),
            }
            println!("Cmd loop end");
        }
        println!("Cmd loop exit");
    // }))
    });
}

}

impl Default for House {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub enum ZoneCmd {
    // Air {id: u8, info: DisplayStatus},
    // Aux {id: u8, info: DisplayStatus},
    Light {id: u8, sender: broadcast::Sender<(u8, bool)>},
    // Irrigation {id: u8, info: DisplayStatus},
    Arm {id: u8, sender: broadcast::Sender<ArmCmd>},
    Pump {id: u8, sender: broadcast::Sender<(u8, PumpCmd)>},
    // Tank {id: u8, info: DisplayStatus},
}
