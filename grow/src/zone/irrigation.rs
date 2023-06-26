use super::Zone;
use super::ZoneUpdate;
use core::error::Error;
use core::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use core::fmt::Debug;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;
use crate::ops::OpsChannelsTx;
use crate::ops::SysLog;

// use tokio::sync::Mutex;
use std::sync::Mutex;
use crate::ops::display::{Indicator, DisplayStatus};
use cond_utils::Between;

use crate::zone::*;

pub mod arm;
pub mod pump;
pub mod tank;

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
        moisture_level: None,
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
        }
       };
    let status_mutex = Arc::new(RwLock::new(status));
    Zone::Irrigation  {
        id,
        settings,
        runner: Runner::new(id, status_mutex.clone()),
        status: status_mutex,
        interface: Interface {
            moist: None,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub moisture_limit_water: f32,
    pub moisture_high_yellow_warning: f32,
    pub moisture_high_red_warning: f32,
    pub moisture_low_yellow_warning: f32,
    pub moisture_low_red_warning: f32,
    pub pump_id: u8,
    pub pump_time: Duration,
    pub position: Option<super::arm::Move>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub moisture_level: Option<f32>,
    pub disp: DisplayStatus,
}

#[async_trait]
pub trait MoistureSensor : Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_moist: tokio::sync::broadcast::Sender<(u8, Option<f32>)>
    ) -> Result<(), Box<dyn Error>>;
    fn read(&self) -> Result<(f32), Box<dyn Error  + '_>>;
}

#[derive(Debug,  )] 
pub struct Interface {
    pub moist: Option<Box<dyn MoistureSensor>>,
}

impl Debug for dyn MoistureSensor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MoistureSensor {{{}}}", self.id())
    }
}

#[derive(Debug, )]
pub struct Runner {
    id: u8,
    tx_moisture: broadcast::Sender<(u8, Option<f32>)>,
    
    pub task: tokio::task::JoinHandle<()>,
    status: Arc<RwLock<Status>>,
}
impl Runner {
    pub fn new(id: u8, status: Arc<RwLock<Status>>) -> Self {
        Self {
            id,
            status,
            tx_moisture: broadcast::channel(168).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn moisture_feedback_sender(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx_moisture.clone()
    }

    pub fn run(&mut self, settings: Settings, zone_channels: ZoneChannelsTx, ops_channels: OpsChannelsTx) {
        let id = self.id;
        let to_manager = zone_channels.zoneupdate;
        let to_status_subscribers = zone_channels.zonestatus; 
        let to_logger = zone_channels.zonelog;
        let to_syslog = ops_channels.syslog;
        let mut rx = self.tx_moisture.subscribe();
        let status = self.status.clone();
        self.task = tokio::spawn(async move {
            // println!("Spawned irrigation runner");
            to_syslog.send(SysLog::new(format!("Spawned irrigation runner id {}", &id))).await;
            let set_and_send = |ds: DisplayStatus | {
                *&mut status.write().disp = ds.clone(); 
                &to_status_subscribers.send(ZoneDisplay::Irrigation { id, info: ds });        
            };
            set_and_send( DisplayStatus {indicator: Indicator::Green, msg: Some(format!("Irrigation running"))} );
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        // println!("\tMoisture: {:?}", data);
                        let mut o_ds: Option<DisplayStatus> = None;
                        match data {
                            (id, None) => {
                                o_ds = Some(DisplayStatus {indicator: Indicator::Red, msg: Some(format!("No data from moisture sensor"))} );
                            },
                            (id, Some(moisture)) => {
                                if moisture < settings.moisture_limit_water {
                                    to_manager.send(ZoneUpdate::Irrigation{id, moisture}).await;
                                }
                                if (moisture < settings.moisture_low_red_warning) {   
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Red, msg: Some(format!("Alert: Moisture LOW {}", moisture))} );
                                } 
                                else if (moisture > settings.moisture_high_red_warning) {
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Red, msg: Some(format!("Alert: Moisture HIGH {}", moisture))} );
                                }
                                else if (moisture < settings.moisture_low_yellow_warning)  {  
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Yellow, msg: Some(format!("Warning: Moisture LOW {}", moisture))} );
                                }
                                else if (moisture > settings.moisture_high_yellow_warning) {  
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Yellow, msg: Some(format!("Warning: Moisture LOW{}", moisture))} );
                                }
                                else { 
                                    o_ds = Some(DisplayStatus {indicator: Indicator::Green, msg: Some(format!("Moisture {}", moisture))} );
                                }
                            },
                            _ => () 
                        }
                        to_logger.send(ZoneLog::Irrigation{id: data.0, moisture: data.1, changed_status: o_ds.clone() }).await;
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



// let set_and_send = |indicator:Indicator, msg:Option<String> | {
//     let ds = DisplayStatus { indicator, msg };
//     *&mut status.write().disp = ds.clone(); 
//     &to_status_subscribers.send(ZoneDisplay::Irrigation { id, info: ds });        
// };
// Ok(data) = rx.recv() => {
//     println!("\tMoisture: {:?}", data);
//     match data {
//         (id, None) => {
//             set_and_send(Indicator::Red, Some(format!("No data from moisture sensor")) );
//         }
//         (id, Some(moisture)) if !(moisture.between(settings.moisture_high_red_warning, settings.moisture_low_red_warning)) => {   
//             to_manager.send(ZoneUpdate::Irrigation{id, moisture}).await;
//             set_and_send(Indicator::Red, Some(format!("Moisture: {}", moisture)));
//         },
//         (id, Some(moisture)) if !(moisture.between(settings.moisture_high_yellow_warning, settings.moisture_low_yellow_warning)) => {   
//             to_manager.send(ZoneUpdate::Irrigation{id, moisture}).await;
//             set_and_send(Indicator::Yellow, Some(format!("Moisture: {}", moisture)));
//         },
//         (id, Some(moisture)) if moisture < settings.moisture_limit_water => {
//             to_manager.send(ZoneUpdate::Irrigation{id, moisture}).await;
//         },
//         _ => () 
//     }
//     to_logger.send(ZoneLog::Irrigation{id: data.0, moisture: data.1, status: None }).await;
// }



// match data {
//     (id, None) => {
//         o_ds = Some(DisplayStatus {indicator: Indicator::Red, msg: Some(format!("No data from moisture sensor"))} );
//     },
//     (id, Some(moisture)) => {
//         if moisture < settings.moisture_limit_water {
//             to_manager.send(ZoneUpdate::Irrigation{id, moisture}).await;
//         }
//         if (moisture < settings.moisture_low_red_warning) &
//             (status.read().disp.indicator != Indicator::Red) {   
//             o_ds = Some(DisplayStatus {indicator: Indicator::Red, msg: Some(format!("Alert: Moisture HIGH {}", moisture))} );
//         } 
//         else if (moisture > settings.moisture_high_red_warning) &
//            (status.read().disp.indicator != Indicator::Red) {   
//             o_ds = Some(DisplayStatus {indicator: Indicator::Red, msg: Some(format!("Alert: Moisture LOW {}", moisture))} );
//         }
//         else if (moisture < settings.moisture_low_yellow_warning) &
//             (status.read().disp.indicator != Indicator::Red) & 
//             (status.read().disp.indicator != Indicator::Yellow) {  
//                  o_ds = Some(DisplayStatus {indicator: Indicator::Yellow, msg: Some(format!("Warning: Moisture {}", moisture))} );
//             }
//         else if !(moisture.between(settings.moisture_high_yellow_warning, settings.moisture_low_yellow_warning)) &
//                 (status.read().disp.indicator != Indicator::Red) & 
//                 (status.read().disp.indicator != Indicator::Yellow) {  
//             o_ds = Some(DisplayStatus {indicator: Indicator::Yellow, msg: Some(format!("Warning: Moisture {}", moisture))} );
//         }
//         else if (status.read().disp.indicator != Indicator::Red) &
//                 (status.read().disp.indicator != Indicator::Yellow) &
//                 (status.read().disp.indicator != Indicator::Green)  { 
//             o_ds = Some(DisplayStatus {indicator: Indicator::Green, msg: Some(format!("Moisture {}", moisture))} );
//         }
//     },
//     _ => () 
// }