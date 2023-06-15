#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
// use anyhow;
use core::error::Error;
use core::result::Result;
use tokio::task::JoinHandle;
use std::sync::Arc;
use tokio::sync::Mutex;
// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
// #[derive(Clone, Debug, PartialEq)]
use irrigation::{pump, tank, arm};
pub enum Zone {
    Light {
        id: u8,
        settings: light::Settings,
        status: light::Status,
    },
    Tank {
        id: u8,
        set: tank::Settings,
        status: tank::Status,
    },
    Irrigation {
        id: u8,
        set: irrigation::Settings,
        status: irrigation::Status,
    },
    Pump {
        id: u8,
        set: pump::Settings,
        status: pump::Status,
    },
    Arm {
        id: u8,
        set: arm::Settings,
        status: arm::Status,
    },
    Air {
        id: u8,
        settings: air::Settings,
        // status: air::Status,
        status: Arc<Mutex<air::Status>>,
        interface: air::Interface,
        runner: air::Runner,
    },
}
pub mod air;
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
