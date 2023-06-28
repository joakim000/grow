#![allow(unused)]

extern crate alloc;
use super::House;
// use super::HouseMapped;
use crate::zone;
use crate::zone::Zone;
use crate::ZoneDisplay;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use async_trait::async_trait;
use core::error::Error;
use core::time::Duration;
use tokio::sync::{broadcast, mpsc};
pub mod conf;
pub mod input;
pub mod manager;
pub mod remote;
// mod warning;
use core::fmt::Debug;
use tokio::task::JoinHandle;

pub mod display;

#[async_trait]
pub trait Board: Send + Sync {
    fn init(
        &mut self,
        rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>>;
    async fn set(&mut self, zones: Vec<ZoneDisplay>) -> Result<(), Box<dyn Error>>;
    fn blink_all(&mut self, on: Duration, off: Duration) -> ();
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Indicator board: {{{}}}", 0)
    }
}
#[async_trait]
pub trait TextDisplay: Send  {
    fn init(
        &mut self,
        // rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>>;
    fn set(
        &mut self,
        // rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
        rx: tokio::sync::broadcast::Receiver<ZoneDisplay>,
    ) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn TextDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Text display: {{{}}}", 0)
    }
}

// }
#[derive(Clone, Debug)]
pub struct SysLog {
    msg: String,
}
impl SysLog {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

pub type SysLogRx = tokio::sync::mpsc::Receiver<SysLog>;
pub type SysLogTx = tokio::sync::mpsc::Sender<SysLog>;

pub fn ops_channels() -> (OpsChannelsTx, OpsChannelsRx) {
    let (syslog_tx, syslog_rx) = mpsc::channel::<SysLog>(128);
    let rx = OpsChannelsRx { syslog: syslog_rx };
    let tx = OpsChannelsTx { syslog: syslog_tx };

    (tx, rx)
}

#[derive(Debug)]
pub struct OpsChannelsRx {
    pub syslog: SysLogRx,
}
#[derive(Clone, Debug)]
pub struct OpsChannelsTx {
    pub syslog: SysLogTx,
}
