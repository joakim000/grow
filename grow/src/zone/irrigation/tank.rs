use core::error::Error;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
use core::fmt::Debug;
use super::Zone;
use crate::ops::display::{Indicator, DisplayStatus};

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
        }
    };
    Zone::Tank {
        id,
        settings,
        status: Arc::new(Mutex::new(status)),
        interface: Interface {
            tank_sensor: None,
        },
        runner: Runner::new(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub disp: DisplayStatus,
}

#[derive(Debug, )]
pub struct Interface {
    pub tank_sensor: Option<Box<dyn TankSensor>>,
}

#[async_trait]
pub trait TankSensor : Send + Sync {
    fn id(&self) -> u8;
    async fn init(
        &mut self,
        tx_tank: tokio::sync::broadcast::Sender<(u8, Option<TankLevel>)>
    ) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn TankSensor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Tanksensor {{{}}}", self.id())
    }
}


#[derive(Debug, )]
pub struct Runner {
    pub tx: broadcast::Sender<(u8, Option<TankLevel>)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx: broadcast::channel(8).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn channel(
        &self,
    ) -> broadcast::Sender<(u8, Option<TankLevel>)> {
        self.tx.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx = self.tx.subscribe();
        self.task = tokio::spawn(async move {
            println!("Spawned tank runner");
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        println!("\tTank: {:?}", data);
                    }
                    // Ok(data) = rx_2.recv() => {
                    //     println!("Secondary:"" {:?}", data);
                    // }
                    else => { break }
                };
            }
        });
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum TankLevel {
    Blue,
    Green,
    Yellow,
    Red,
}