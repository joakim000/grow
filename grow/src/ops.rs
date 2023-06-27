#![allow(unused)]

extern crate alloc;
use super::House;
// use super::HouseMapped;
use tokio::sync::{broadcast, mpsc};
use crate::zone;
use crate::zone::Zone;
use crate::ZoneDisplay;
use async_trait::async_trait;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use core::time::Duration;
use core::error::Error;
pub mod conf;
pub mod manager;
pub mod remote;
pub mod input;
// mod warning;
use tokio::task::JoinHandle;
use core::fmt::Debug;

pub mod display;

#[async_trait]
pub trait Board : Send + Sync {
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
pub trait TextDisplay : Send + Sync {
    fn init(
        &mut self,
        rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn TextDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Text display: {{{}}}", 0)
    }
}

// }
#[derive(Clone, Debug,)]
pub struct SysLog {
    msg: String,
}
impl SysLog {
    pub fn new(msg: String) -> Self {
        Self {
            msg,
        }
    }
}

pub type SysLogRx = tokio::sync::mpsc::Receiver<SysLog>;
pub type SysLogTx = tokio::sync::mpsc::Sender<SysLog>;

pub fn ops_channels() -> (OpsChannelsTx, OpsChannelsRx) {
    let (syslog_tx, syslog_rx) = mpsc::channel::<SysLog>(128);
    let rx = OpsChannelsRx {
        syslog: syslog_rx,
    };
    let tx = OpsChannelsTx {
        syslog: syslog_tx,
    };

    (tx, rx)
}

#[derive(Debug, )]
pub struct OpsChannelsRx {
    pub syslog: SysLogRx,
}
#[derive(Clone, Debug,)]
pub struct OpsChannelsTx {
    pub syslog: SysLogTx,
}
