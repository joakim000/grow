#![allow(unused)]

extern crate alloc;
use super::House;
// use super::HouseMapped;

use crate::zone;
use crate::zone::Zone;
use crate::ZoneDisplay;
use async_trait::async_trait;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use core::time::Duration;
use core::error::Error;
pub mod conf;
pub mod running;
pub mod remote;
pub mod input;
// mod warning;
use tokio::task::JoinHandle;
use core::fmt::Debug;

pub mod display;

pub mod warning {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {}

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}
#[async_trait]
pub trait Board : Send + Sync {
    fn init(
        &mut self,
        rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>>;
    fn set(&mut self, zones: Vec<ZoneDisplay>) -> Result<(), Box<dyn Error>>;
    fn blink_all(&mut self, on: Duration, off: Duration) -> ();
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>>;

}
impl Debug for dyn Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Indicator board: {{{}}}", 0)
    }
}
pub trait TextDisplay : Send {
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


// pub struct Manager {

// }

// pub fn Manager(house: House) -> Result<JoinHandle<()>, Box<dyn Error>> {
//     Ok(tokio::spawn(async move {
//         println!("Spawned manager");
//         loop {
//             tokio::select! {
//                 Ok(data) = rx_rpm.recv() => {
//                 }
//                 Ok(data) = rx_temp.recv() => {
//                 }
//                 else => { break }
//             };
//         }

//     }))
// }