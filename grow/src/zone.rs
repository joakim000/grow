// #![allow(unused)]

extern crate alloc;
use core::result::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

// use tokio::sync::Mutex;
use crate::ops::display::DisplayStatus;
use light::LampState;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

pub mod air;
pub mod auxiliary;
pub mod light;
pub mod water;
pub use water::{arm, pump, tank};

pub type ZoneUpdateRx = tokio::sync::mpsc::Receiver<ZoneUpdate>;
pub type ZoneUpdateTx = tokio::sync::mpsc::Sender<ZoneUpdate>;
pub type ZoneStatusRx = tokio::sync::broadcast::Receiver<ZoneDisplay>;
pub type ZoneStatusTx = tokio::sync::broadcast::Sender<ZoneDisplay>;
pub type ZoneLogRx = tokio::sync::mpsc::Receiver<ZoneLog>;
pub type ZoneLogTx = tokio::sync::mpsc::Sender<ZoneLog>;

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


#[derive(Debug)]
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
impl Zone {
    pub fn display_status(&self) -> DisplayStatus {
        match self {
            Zone::Air { status, .. } => status.read().disp.clone(),
            Zone::Light { status, .. } => status.read().disp.clone(),
            Zone::Water { status, .. } => status.read().disp.clone(),
            Zone::Arm { status, .. } => status.read().disp.clone(),
            Zone::Tank { status, .. } => status.read().disp.clone(),
            Zone::Pump { status, .. } => status.read().disp.clone(),
            Zone::Aux { status, .. } => status.read().disp.clone(),
        }
    }
    pub fn zone_display(&self) -> ZoneDisplay {
        match self {
            Zone::Air { id, status, .. } => ZoneDisplay::Air {
                id: *id,
                info: status.read().disp.clone(),
            },
            Zone::Light { id, status, .. } => ZoneDisplay::Light {
                id: *id,
                info: status.read().disp.clone(),
            },
            Zone::Water { id, status, .. } => ZoneDisplay::Water {
                id: *id,
                info: status.read().disp.clone(),
            },
            Zone::Arm { id, status, .. } => ZoneDisplay::Arm {
                id: *id,
                info: status.read().disp.clone(),
            },
            Zone::Tank { id, status, .. } => ZoneDisplay::Tank {
                id: *id,
                info: status.read().disp.clone(),
            },
            Zone::Pump { id, status, .. } => ZoneDisplay::Pump {
                id: *id,
                info: status.read().disp.clone(),
            },
            Zone::Aux { id, status, .. } => ZoneDisplay::Aux {
                id: *id,
                info: status.read().disp.clone(),
            },
        }
    }
    pub fn get_ref(self) -> Arc<Zone> {
        Arc::new(self)
    }
}

#[derive(Clone, Debug)]
pub enum ZoneUpdate {
    Water {
        id: u8,
        settings: water::Settings,
        status: Arc<RwLock<water::Status>>,
    },
    Tank {
        id: u8,
        moisture: f32,
    },
    Arm {
        id: u8,
        state: arm::ArmState,
        x: i32,
        y: i32,
        z: i32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ZoneLog {
    Air {
        id: u8,
        temp: Option<f64>,
        fan_rpm: Option<f32>,
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
    Tank {
        id: u8,
        changed_status: Option<DisplayStatus>,
    },
    Pump {
        id: u8,
        changed_status: Option<DisplayStatus>,
    },
    Arm {
        id: u8,
        x: i32,
        y: i32,
        z: i32,
        changed_status: Option<DisplayStatus>,
    },
    Aux {
        id: u8,
        changed_status: Option<DisplayStatus>,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum ZoneDisplay {
    Air { id: u8, info: DisplayStatus },
    Light { id: u8, info: DisplayStatus },
    Water { id: u8, info: DisplayStatus },
    Tank { id: u8, info: DisplayStatus },
    Pump { id: u8, info: DisplayStatus },
    Arm { id: u8, info: DisplayStatus },
    Aux { id: u8, info: DisplayStatus },
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


#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ZoneSave {
    Air {
        id: u8,
        settings: air::Settings,
    },
    Aux {
        id: u8,
        settings: auxiliary::Settings,
    },
    Light {
        id: u8,
        settings: light::Settings,
    },
    Water {
        id: u8,
        settings: water::Settings,
    },
    Arm {
        id: u8,
        settings: arm::Settings,
    },
    Pump {
        id: u8,
        settings: pump::Settings,
    },
    Tank {
        id: u8,
        settings: tank::Settings,
    },
}

// #[derive(Clone, Debug)]
// pub enum ZoneCmd {
//     // Air { id: u8, sender: broadcast::Sender<AirCmd>, },
//     // Aux { id: u8, sender: broadcast::Sender<AuxCmd>, },
//     Light {
//         id: u8,
//         sender: broadcast::Sender<(u8, bool)>,
//     },
//     // Water { id: u8, sender: broadcast::Sender<WaterCmd>, },
//     Arm {
//         id: u8,
//         sender: broadcast::Sender<ArmCmd>,
//     },
//     Pump {
//         id: u8,
//         sender: broadcast::Sender<(u8, PumpCmd)>,
//     },
//     // Tank { id: u8, sender: broadcast::Sender<(u8, TankCmd)>, },
// }