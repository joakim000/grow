use super::Zone;
use core::error::Error;
use tokio::sync::broadcast;
use core::fmt::Debug;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
use crate::ops::display::{Indicator, DisplayStatus};

pub mod arm;
pub mod pump;
pub mod tank;

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
        moisture_level: None,
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
        }
       };
    let status_mutex = Arc::new(RwLock::new(status));
    Zone::Irrigation  {
        id,
        settings,
        status: status_mutex,
        interface: Interface {
            moist: None,
        },
        runner: Runner::new(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    pub moisture_limit_water: i16,
    pub moisture_limit_low_warning: i16,
    pub moisture_limit_high_warning: i16,
    pub pump_id: u8,
    pub position: Option<super::arm::Move>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub moisture_level: Option<f32>,
    pub disp: DisplayStatus,
}

#[async_trait]
pub trait MoistureSensor : Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_moist: tokio::sync::broadcast::Sender<(u8, Option<f32>)>
    ) -> Result<(), Box<dyn Error>>;
    fn read(&self) -> Result<(f32), Box<dyn Error  + '_>>;
}

#[derive(Debug,  )] 
pub struct Interface {
    pub moist: Option<Box<dyn MoistureSensor>>,
}

impl Debug for dyn MoistureSensor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MoistureSensor {{{}}}", self.id())
    }
}




#[derive(Debug, )]
pub struct Runner {
    pub tx: broadcast::Sender<(u8, Option<f32>)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn channel(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx = self.tx.subscribe();
        self.task = tokio::spawn(async move {
            println!("Spawned irrigation runner");
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        println!("\tMoisture: {:?}", data);
                    }
                    else => { break }
                };
            }
        });
    }
}
