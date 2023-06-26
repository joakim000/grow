#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
// use anyhow;
use core::error::Error;
use core::result::Result;
use tokio::task::JoinHandle;
use tokio::sync::{broadcast, mpsc};
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
// #[derive(Clone, Debug, PartialEq)]
pub use irrigation::{pump, tank, arm};
use parking_lot::RwLock;

pub type ZoneUpdateRx = tokio::sync::mpsc::Receiver<ZoneUpdate>;
pub type ZoneUpdateTx = tokio::sync::mpsc::Sender<ZoneUpdate>;
pub type ZoneStatusRx = tokio::sync::broadcast::Receiver<ZoneDisplay>;
pub type ZoneStatusTx = tokio::sync::broadcast::Sender<ZoneDisplay>;
pub type ZoneLogRx = tokio::sync::mpsc::Receiver<ZoneLog>;
pub type ZoneLogTx = tokio::sync::mpsc::Sender<ZoneLog>;

#[derive(Debug, )]
pub enum Zone {
    Air {
        id: u8,
        settings: air::Settings,
        // status: air::Status,
        status: Arc<RwLock<air::Status>>,
        interface: air::Interface,
        runner: air::Runner,
    },
    Aux {
        id: u8,
        settings: auxiliary::Settings,
        // status: air::Status,
        status: Arc<RwLock<auxiliary::Status>>,
        interface: auxiliary::Interface,
        runner: auxiliary::Runner,
    },
    Light {
        id: u8,
        settings: light::Settings,
        // status: light::Status,
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
        // status: pump::Status,
        status: Arc<RwLock<pump::Status>>,
        interface: pump::Interface,
        runner: pump::Runner,
    },
    Tank {
        id: u8,
        settings: tank::Settings,
        // status: tank::Status,
        status: Arc<RwLock<tank::Status>>,
        interface: tank::Interface,
        runner: tank::Runner,
    },
  
}
impl Zone {
    // pub fn status(&self) {
    //     match self {
    //         Zone::Air { status, ..} => *status,
    //         Zone::Arm { status, ..} => *status,
    //         Zone::Aux { status, ..} => *status,
    //         Zone::Light { status, ..} => *status,
    //         Zone::Irrigation { status, ..} => *status,
    //         Zone::Pump { status, ..} => *status,
    //         Zone::Tank { status, ..} => *status,
    //     }
    // }
}

// impl TransactionsEnum {
//     pub fn id(&self) -> i64 {
//         match self {
//             TransactionsEnum::TransactionOrderA(value) => value.id,
//             TransactionsEnum::TransactionOrderB(value) => value.id,
//         }
//     }
// }


pub mod air;
pub mod auxiliary;
// pub mod arm;
pub mod irrigation;
pub mod light;
// pub mod pump;
// pub mod tank;

#[derive(Debug)]
pub struct Handles {
    control_task: JoinHandle<()>,
    feedback_task: JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub enum ZoneUpdate {
    Irrigation{id: u8, moisture: f32},
}

#[derive(Debug, Clone)]
pub enum ZoneLog {
    Air{id: u8, temp: Option<f32>, fan_rpm: Option<f32>, changed_status: Option<DisplayStatus>},
    Aux{id: u8, changed_status: Option<DisplayStatus>},
    Light{id: u8, lamp_on: Option<bool>, light_level: Option<f32>, changed_status: Option<DisplayStatus>},
    Irrigation{id: u8, moisture: Option<f32>, changed_status: Option<DisplayStatus>},
    Arm{id: u8, x: i32, y: i32, z: i32, changed_status: Option<DisplayStatus>},
    Pump{id: u8, changed_status: Option<DisplayStatus>},
    Tank{id: u8, changed_status: Option<DisplayStatus>},
}

use crate::ops::display::DisplayStatus;
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
