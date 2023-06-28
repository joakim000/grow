use alloc::collections::BTreeMap;
use async_trait::async_trait;
use core::error::Error;
use parking_lot::RwLock;
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio::time::sleep;
// use tokio::sync::Mutex;
use core::fmt::Debug;
use std::sync::Mutex;
use time::OffsetDateTime;

use super::Zone;
use super::*;
use crate::ops::display::{DisplayStatus, Indicator};
use crate::ops::OpsChannelsTx;
use crate::ops::SysLog;
use crate::TIME_OFFSET;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PumpCmd {
    RunForSec(u16),
    Stop,
}

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status {
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
            changed: OffsetDateTime::UNIX_EPOCH,
        },
    };
    let status_mutex = Arc::new(RwLock::new(status));
    Zone::Pump {
        id,
        settings,
        status: status_mutex,
        interface: Interface { pump: None },
        runner: Runner::new(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    pub run_for_secs: u16,
    pub rest_secs: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub disp: DisplayStatus,
}

#[derive(Debug)]
pub struct Interface {
    pub pump: Option<Box<dyn Pump>>,
}

#[async_trait]
pub trait Pump: Send + Sync {
    fn id(&self) -> u8;
    async fn init(
        &mut self,
        rx_pump: tokio::sync::broadcast::Receiver<(u8, PumpCmd)>,
        tx_pump: tokio::sync::broadcast::Sender<(u8, (i8, i32))>,
    ) -> Result<(), Box<dyn Error>>;
    async fn run_for_secs(&self, secs: u16) -> Result<(), Box<dyn Error>>;
    fn run(&self) -> Result<(), Box<dyn Error>>;
    fn stop(&self) -> Result<(), Box<dyn Error>>;
    fn float(&self) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn Pump {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Pump {{{}}}", self.id())
    }
}

#[derive(Debug)]
pub struct Runner {
    tx_feedback: broadcast::Sender<(u8, (i8, i32))>,
    tx_pumpcmd: broadcast::Sender<(u8, PumpCmd)>,
    task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx_feedback: broadcast::channel(8).0,
            tx_pumpcmd: broadcast::channel(8).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn cmd_sender(&self) -> broadcast::Sender<(u8, PumpCmd)> {
        self.tx_pumpcmd.clone()
    }
    pub fn cmd_receiver(&self) -> broadcast::Receiver<(u8, PumpCmd)> {
        self.tx_pumpcmd.subscribe()
    }
    pub fn feedback_sender(&self) -> broadcast::Sender<(u8, (i8, i32))> {
        self.tx_feedback.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx_feedback = self.tx_feedback.subscribe();
        let tx_pumpcmd = self.tx_pumpcmd.clone();
        self.task = tokio::spawn(async move {
            println!("Spawned pump runner");
            loop {
                tokio::select! {
                    Ok(data) = rx_feedback.recv() => {
                        // println!("\tPump speed: {:?}", data);
                    }
                    else => { break }
                };
            }
        });
    }
}
