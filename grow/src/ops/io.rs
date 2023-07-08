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
use core::fmt::Debug;
use tokio::task::JoinHandle;
use zone::ZoneStatusRx;
use super::SysLogTx;

/// Indicator lights
#[async_trait]
pub trait Board: Send + Sync {
    // fn init(
    //     &mut self,
    //     rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    // ) -> Result<(), Box<dyn Error>>;
    async fn set(
        &mut self,
        zones: Vec<ZoneDisplay>,
    ) -> Result<(), Box<dyn Error>>;
    fn blink_all(&mut self, on: Duration, off: Duration) -> ();
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Indicator board")
    }
}

/// Text display panel
#[async_trait]
pub trait TextDisplay: Send {
    fn init(
        &self,
        from_zones: ZoneStatusRx,
        to_syslog: SysLogTx,
    ) -> Result<(JoinHandle<()>), Box<dyn Error>>;
    fn set(
        &mut self,
        status_all: Vec<ZoneDisplay>,
    ) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn TextDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Text display")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ButtonInput {
    OneDown,
    OneUp,
    TwoDown,
    TwoUp,
}


// #[async_trait]
pub trait ButtonPanel: Send {
    // fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_rc: tokio::sync::broadcast::Sender<ButtonInput>,
    ) -> Result<(), Box<dyn Error>>;
    // fn read(&self) -> Result<(f32), Box<dyn Error  + '_>>;
}
impl Debug for dyn ButtonPanel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Buttons: {{{}}}", 0)
    }
}

