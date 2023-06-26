use core::error::Error;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
use core::fmt::Debug;
use super::Zone;
use crate::ops::display::{Indicator, DisplayStatus};
use super::*;
use crate::ops::OpsChannelsTx;
use crate::ops::SysLog;


pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
        lamp_state: Some(LampState::Off),
        light_level: None,
        disp: DisplayStatus {
                indicator: Default::default(),
                msg: None,
            }
       };
       let status_mutex = Arc::new(RwLock::new(status));
    Zone::Light {
        id,
        settings,
        runner: Runner::new(id, status_mutex.clone()),
        status: status_mutex,
        interface: Interface {
            lamp: None,
            lightmeter: None,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub lightlevel_low_yellow_warning: f32,
    pub lightlevel_low_red_alert: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub lamp_state: Option<LampState>,
    pub light_level: Option<f32>,
    pub disp: DisplayStatus,
}

#[derive( Debug, )]
pub struct Interface {
    pub lamp: Option<Box<dyn Lamp>>,
    pub lightmeter: Option<Box<dyn Lightmeter>>,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LampState {
    On,
    Off,
}


pub trait Lamp : Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        rx_lamp: tokio::sync::broadcast::Receiver<(u8, bool)>
    ) -> Result<(), Box<dyn Error>>;
    fn set_state(&self, state: LampState) -> Result<(), Box<dyn Error + '_>>;
    fn state(&self) -> Result<LampState, Box<dyn Error>>;

}
impl Debug for dyn Lamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Lamp: {{{}}}", 0)
    }
}


pub trait Lightmeter : Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_light: tokio::sync::broadcast::Sender<(u8, Option<f32>)>
    ) -> Result<(), Box<dyn Error>>;   
    fn read(&self) -> Result<(f32), Box<dyn Error  + '_>>;
    
}
impl Debug for dyn Lightmeter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LightMeter: {{{}}}", 0)
    }
}


#[derive(Debug, )]
pub struct Runner {
    id: u8,
    status: Arc<RwLock<Status>>,
    tx_lightmeter: broadcast::Sender<(u8, Option<f32>)>,
    tx_lamp: broadcast::Sender<(u8, bool)>,
    task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new(id: u8, status: Arc<RwLock<Status>>) -> Self {
        Self {
            tx_lightmeter: broadcast::channel(1).0,
            tx_lamp: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
            id,
            status,
        }
    }

    pub fn lightmeter_feedback_sender(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx_lightmeter.clone()
    }
    pub fn lamp_cmd_receiver(
        &self,
    ) -> broadcast::Receiver<(u8, bool)> {
        self.tx_lamp.subscribe()
    }
    pub fn lamp_cmd_sender(
        &self,
    ) -> broadcast::Sender<(u8, bool)> {
        self.tx_lamp.clone()
    }

    pub fn run(&mut self, settings: Settings, zone_channels: ZoneChannelsTx, ops_channels: OpsChannelsTx) {
        let id = self.id;
        let to_manager = zone_channels.zoneupdate;
        let to_status_subscribers = zone_channels.zonestatus; 
        let to_logger = zone_channels.zonelog;
        let to_syslog = ops_channels.syslog;
        let mut rx = self.tx_lightmeter.subscribe();
        let status = self.status.clone();
        self.task = tokio::spawn(async move {
            to_syslog.send(SysLog::new(format!("Spawned light runner id {}", &id))).await;
            let set_and_send = |ds: DisplayStatus | {
                *&mut status.write().disp = ds.clone(); 
                &to_status_subscribers.send(ZoneDisplay::Light { id, info: ds });        
            };
            set_and_send( DisplayStatus {indicator: Indicator::Green, msg: Some(format!("Light running"))} );
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        // println!("Light: {:?}", data);
                        let mut o_ds: Option<DisplayStatus> = None;
                        match data {
                            (id, None) => {
                                o_ds = Some(DisplayStatus {indicator: Indicator::Red, msg: Some(format!("No data from lightmeter"))} );
                            },
                            (id, Some(lightlevel)) => { 
                                if (status.read().lamp_state.expect("Lamp status error") == LampState::Off)  {
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Green, 
                                        msg: Some(format!("Light {} (lamp OFF)", lightlevel))} );
                                }
                                else if lightlevel < settings.lightlevel_low_red_alert {
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Red, 
                                        msg: Some(format!("Alert: {} (lamp ON)", lightlevel))} );
                                }
                                else if lightlevel < settings.lightlevel_low_yellow_warning {
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Yellow, 
                                        msg: Some(format!("Warning: {} (lamp ON)", lightlevel))} );
                                }
                                else {
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Yellow, 
                                        msg: Some(format!("Ok: {} (lamp ON)", lightlevel))} );
                                }
                            },
                            _ => () 
                        }
                        to_logger.send(ZoneLog::Tank {id: data.0, changed_status: o_ds.clone() }).await;
                        match o_ds {
                            Some(ds) => { set_and_send(ds); }
                            None => {}
                        }
                    }
                    else => { break }
                };
            }
        });
    }
}

// #[derive(Debug, )]
// pub struct Runner {
//     pub tx_lightmeter: broadcast::Sender<(u8, Option<f32>)>,
//     pub tx_lamp: broadcast::Sender<(u8, bool)>,
//     pub task: tokio::task::JoinHandle<()>,
// }
// impl Runner {
//     pub fn new() -> Self {
//         Self {
//             tx_lightmeter: broadcast::channel(1).0,
//             tx_lamp: broadcast::channel(1).0,
//             task: tokio::spawn(async move {}),
//         }
//     }

//     pub fn lightmeter_feedback_sender(
//         &self,
//     ) -> broadcast::Sender<(u8, Option<f32>)> {
//         self.tx_lightmeter.clone()
//     }
//     pub fn lamp_cmd_receiver(
//         &self,
//     ) -> broadcast::Receiver<(u8, bool)> {
//         self.tx_lamp.subscribe()
//     }
//     pub fn lamp_cmd_sender(
//         &self,
//     ) -> broadcast::Sender<(u8, bool)> {
//         self.tx_lamp.clone()
//     }

//     // This could handle scheudlineg and to that timed lightchecks, only waking manager if warning
//     // Keep lamp channel for runner lamp control
//     pub fn run(&mut self, settings: Settings) {
//         let mut rx = self.tx_lightmeter.subscribe();
//         let tx = self.tx_lamp.clone();
//         self.task = tokio::spawn(async move {
//             println!("Spawned light runner");
//             loop {
//                 tokio::select! {
//                     Ok(data) = rx.recv() => {
//                         println!("\tLight level: {:?}", data);
//                     }
//                     // Ok(data) = rx_2.recv() => {
//                     //     println!("Secondary:"" {:?}", data);
//                     // }
//                     else => { break }
//                 };
//             }
//         });
//     }
// }

// // struct Lamp {}
// // struct Sensor{}
// // struct Timer{}
