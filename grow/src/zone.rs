#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
// use anyhow;
use core::error::Error;
use core::result::Result;
use tokio::task::JoinHandle;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
// #[derive(Clone, Debug, PartialEq)]
pub use irrigation::{pump, tank, arm};
use parking_lot::RwLock;

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
