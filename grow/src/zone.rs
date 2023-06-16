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
pub use irrigation::{pump, tank, arm};

#[derive(Debug, )]
pub enum Zone {
    Light {
        id: u8,
        settings: light::Settings,
        // status: light::Status,
        status: Arc<Mutex<light::Status>>,
        interface: light::Interface,
        runner: light::Runner,
    },
    Tank {
        id: u8,
        settings: tank::Settings,
        // status: tank::Status,
        status: Arc<Mutex<tank::Status>>,
        interface: tank::Interface,
        runner: tank::Runner,
    },
    Irrigation {
        id: u8,
        settings: irrigation::Settings,
        status: Arc<Mutex<irrigation::Status>>,
        interface: irrigation::Interface,
        runner: irrigation::Runner,
    },
    Pump {
        id: u8,
        settings: pump::Settings,
        // status: pump::Status,
        status: Arc<Mutex<pump::Status>>,
        interface: pump::Interface,
        runner: pump::Runner,
    },
    Arm {
        id: u8,
        settings: arm::Settings,
        status: Arc<Mutex<arm::Status>>,
        interface: arm::Interface,
        runner: arm::Runner,
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
