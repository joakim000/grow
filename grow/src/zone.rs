#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use core::error::Error;
use core::fmt;
use core::result::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
// use tokio::sync::Mutex;
use crate::ops::display::DisplayStatus;
use light::LampState;
use parking_lot::RwLock;
use std::sync::Mutex;

pub type ZoneUpdateRx = tokio::sync::mpsc::Receiver<ZoneUpdate>;
pub type ZoneUpdateTx = tokio::sync::mpsc::Sender<ZoneUpdate>;
pub type ZoneStatusRx = tokio::sync::broadcast::Receiver<ZoneDisplay>;
pub type ZoneStatusTx = tokio::sync::broadcast::Sender<ZoneDisplay>;
pub type ZoneLogRx = tokio::sync::mpsc::Receiver<ZoneLog>;
pub type ZoneLogTx = tokio::sync::mpsc::Sender<ZoneLog>;

pub mod air;
pub mod auxiliary;
pub mod water;
pub mod light;
pub use water::{arm, pump, tank};

#[derive( Debug, )]
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
    Water {
        id: u8,
        settings: water::Settings,
        status: Arc<RwLock<water::Status>>,
        interface: water::Interface,
        runner: water::Runner,
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

#[derive(Clone, Debug, PartialEq, PartialOrd, )]
pub enum ZoneUpdate {
    Water { id: u8, moisture: f32 },
}

#[derive(Clone, Debug, PartialEq,  )]
pub enum ZoneLog {
    Air {
        id: u8,
        temp: Option<f64>,
        fan_rpm: Option<f32>,
        changed_status: Option<DisplayStatus>,
    },
    Aux {
        id: u8,
        changed_status: Option<DisplayStatus>,
    },
    Light {
        id: u8,
        lamp_on: Option<LampState>,
        light_level: Option<f32>,
        changed_status: Option<DisplayStatus>,
    },
    Water {
        id: u8,
        moisture: Option<f32>,
        changed_status: Option<DisplayStatus>,
    },
    Arm {
        id: u8,
        x: i32,
        y: i32,
        z: i32,
        changed_status: Option<DisplayStatus>,
    },
    Pump {
        id: u8,
        changed_status: Option<DisplayStatus>,
    },
    Tank {
        id: u8,
        changed_status: Option<DisplayStatus>,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum ZoneDisplay {
    Air { id: u8, info: DisplayStatus },
    Aux { id: u8, info: DisplayStatus },
    Light { id: u8, info: DisplayStatus },
    Water { id: u8, info: DisplayStatus },
    Arm { id: u8, info: DisplayStatus },
    Pump { id: u8, info: DisplayStatus },
    Tank { id: u8, info: DisplayStatus },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum ZoneKind {
    Air,
    Aux,
    Light,
    Water,
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

#[derive(Debug)]
pub struct ZoneChannelsRx {
    pub zoneupdate: ZoneUpdateRx,
    pub zonestatus: ZoneStatusRx,
    pub zonelog: ZoneLogRx,
}
#[derive(Clone, Debug)]
pub struct ZoneChannelsTx {
    pub zoneupdate: ZoneUpdateTx,
    pub zonestatus: ZoneStatusTx,
    pub zonelog: ZoneLogTx,
}

