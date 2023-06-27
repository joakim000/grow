#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use core::error::Error;
use core::result::Result;
use tokio::task::JoinHandle;
use tokio::sync::{broadcast, mpsc};
use std::sync::Arc;
use core::fmt;
// use tokio::sync::Mutex;
use std::sync::Mutex;
use light::LampState;
use parking_lot::RwLock;
use crate::ops::display::DisplayStatus;

pub type ZoneUpdateRx = tokio::sync::mpsc::Receiver<ZoneUpdate>;
pub type ZoneUpdateTx = tokio::sync::mpsc::Sender<ZoneUpdate>;
pub type ZoneStatusRx = tokio::sync::broadcast::Receiver<ZoneDisplay>;
pub type ZoneStatusTx = tokio::sync::broadcast::Sender<ZoneDisplay>;
pub type ZoneLogRx = tokio::sync::mpsc::Receiver<ZoneLog>;
pub type ZoneLogTx = tokio::sync::mpsc::Sender<ZoneLog>;

pub mod air;
pub mod auxiliary;
pub mod light;
pub mod irrigation;
pub use irrigation::{pump, tank, arm};


#[derive(Debug, )]
pub enum Zone {
    Air {
        id: u8,
        settings: air::Settings,
        status: Arc<RwLock<air::Status>>,
        interface: air::Interface,
        runner: air::Runner,
    },
    Aux {
        id: u8,
        settings: auxiliary::Settings,
        status: Arc<RwLock<auxiliary::Status>>,
        interface: auxiliary::Interface,
        runner: auxiliary::Runner,
    },
    Light {
        id: u8,
        settings: light::Settings,
        status: Arc<RwLock<light::Status>>,
        interface: light::Interface,
        runner: light::Runner,
    },
    Irrigation {
        id: u8,
        settings: irrigation::Settings,
        status: Arc<RwLock<irrigation::Status>>,
        interface: irrigation::Interface,
        runner: irrigation::Runner,
    },
    Arm {
        id: u8,
        settings: arm::Settings,
        status: Arc<RwLock<arm::Status>>,
        interface: arm::Interface,
        runner: arm::Runner,
    },
    Pump {
        id: u8,
        settings: pump::Settings,
        status: Arc<RwLock<pump::Status>>,
        interface: pump::Interface,
        runner: pump::Runner,
    },
    Tank {
        id: u8,
        settings: tank::Settings,
        status: Arc<RwLock<tank::Status>>,
        interface: tank::Interface,
        runner: tank::Runner,
    },
  
}


#[derive(Debug, Clone)]
pub enum ZoneUpdate {
    Irrigation{id: u8, moisture: f32},
}


#[derive(Debug, Clone)]
pub enum ZoneLog {
    Air{id: u8, temp: Option<f64>, fan_rpm: Option<f32>, changed_status: Option<DisplayStatus>},
    Aux{id: u8, changed_status: Option<DisplayStatus>},
    Light{id: u8, lamp_on: Option<LampState>, light_level: Option<f32>, changed_status: Option<DisplayStatus>},
    Irrigation{id: u8, moisture: Option<f32>, changed_status: Option<DisplayStatus>},
    Arm{id: u8, x: i32, y: i32, z: i32, changed_status: Option<DisplayStatus>},
    Pump{id: u8, changed_status: Option<DisplayStatus>},
    Tank{id: u8, changed_status: Option<DisplayStatus>},
}

#[derive(Clone, Debug, PartialEq)]
pub enum ZoneDisplay {
    Air {id: u8, info: DisplayStatus},
    Aux {id: u8, info: DisplayStatus},
    Light {id: u8, info: DisplayStatus},
    Irrigation {id: u8, info: DisplayStatus},
    Arm {id: u8, info: DisplayStatus},
    Pump {id: u8, info: DisplayStatus},
    Tank {id: u8, info: DisplayStatus},
}   

pub enum ZoneKind {
    Air,
    Aux,
    Light,
    Irrigation,
    Arm,
    Pump,
    Tank,
}


pub fn zone_channels() -> (ZoneChannelsTx, ZoneChannelsRx) {
    let (zoneupdate_tx, zoneupdate_rx) = mpsc::channel::<ZoneUpdate>(128);
    let (zonestatus_tx, zonestatus_rx) = broadcast::channel::<ZoneDisplay>(128);
    let (zonelog_tx, zonelog_rx) = mpsc::channel::<ZoneLog>(128);
    let rx = ZoneChannelsRx {
        zoneupdate: zoneupdate_rx,
        zonestatus: zonestatus_rx,
        zonelog: zonelog_rx,
    };
    let tx = ZoneChannelsTx {
        zoneupdate: zoneupdate_tx,
        zonestatus: zonestatus_tx,
        zonelog: zonelog_tx,
    };

    (tx, rx)
}

#[derive(Debug, )]
pub struct ZoneChannelsRx {
    pub zoneupdate: ZoneUpdateRx,
    pub zonestatus: ZoneStatusRx,
    pub zonelog: ZoneLogRx,
}
#[derive(Clone, Debug,)]
pub struct ZoneChannelsTx {
    pub zoneupdate: ZoneUpdateTx,
    pub zonestatus: ZoneStatusTx,
    pub zonelog: ZoneLogTx,
}

impl fmt::Display for ZoneDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneDisplay::Air { id, info } => { write!(f, "ZoneStatus Air {} {{ {} }}", id, info ) },
            ZoneDisplay::Light { id, info } => { write!(f, "ZoneStatus Light {} {{ {} }}", id, info ) },
            ZoneDisplay::Irrigation { id, info } => { write!(f, "ZoneStatus Irrigation {} {{ {} }}", id, info ) },
            ZoneDisplay::Aux { id, info } => { write!(f, "ZoneStatus Aux {} {{ {} }}", id, info ) },
            ZoneDisplay::Arm { id, info } => { write!(f, "ZoneStatus Arm {} {{ {} }}", id, info ) },
            ZoneDisplay::Pump { id, info } => { write!(f, "ZoneStatus Pump {} {{ {} }}", id, info ) },
            ZoneDisplay::Tank { id, info } => { write!(f, "ZoneStatus Tank {} {{ {} }}", id, info ) },
        }
    }
}
impl fmt::Display for ZoneLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneLog::Air { id, temp, fan_rpm, changed_status } => {
                let temp_text = match temp {
                    None => {format!("None" )},
                    Some(temp) => {
                        format!("{:.1}", temp)
                    },
                };  
                let fan_text = match fan_rpm {
                    None => {format!("None" )},
                    Some(rpm) => {
                        format!("{:.0}", rpm)
                    },
                };      
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Air {} {{Temp {}°C, Fan {} rpm, Status change: {} }}",
                        id,temp_text,fan_text,status_text
                )    
            },
            ZoneLog::Light { id, lamp_on, light_level, changed_status } => {
                let lamp_text = match lamp_on {
                    None => {format!("None" )},
                    Some(x) => {
                        format!("{:?}", x)
                    },
                };  
                let light_text = match light_level {
                    None => {format!("None" )},
                    Some(x) => {
                        format!("{:.0}", x)
                    },
                };      
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Light {} {{Lamp {}, Light level {}, Status change: {} }}",
                        id,lamp_text,light_text,status_text
                )    
            },
            ZoneLog::Irrigation { id, moisture, changed_status } => {
                let moist_text = match moisture {
                    None => {format!("None" )},
                    Some(x) => {
                        format!("{:?}", x)
                    },
                };  
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Irrigation {} {{Moisture {}, Status change: {} }}",
                        id,moist_text,status_text
                )    
            },
            ZoneLog::Aux { id,  changed_status } => {
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Aux {} {{Status change: {} }}",
                        id, status_text
                )    
            }
            ZoneLog::Arm { id, x:_, y:_, z:_, changed_status } => {
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Arm {} {{Status change: {} }}",
                        id, status_text
                )    
            }
            ZoneLog::Pump { id,  changed_status } => {
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Pump {} {{Status change: {} }}",
                        id, status_text
                )    
            }
            ZoneLog::Tank { id,  changed_status } => {
                let status_text = match changed_status {
                    None => {format!("None" )},
                    Some(ds) => {
                        format!("{}", ds)
                    },
                };    
                write!(
                        f,
                        "ZoneLog Tank {} {{Status change: {} }}",
                        id, status_text
                )    
            }
        }
    }
}