/// Enable any auxiliary equipment to provide status (ex. UPS, servo controller)
/// Example rpi3 uses this for alerts and updates from Lego hub  

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
    let status_mutex = Arc::new(Mutex::new(status));
    Zone::Aux {
        id,
        settings,
        runner: Runner::new(status_mutex.clone()),
        status: status_mutex,
        interface: Interface {
            aux_device: None,
        },
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
    pub aux_device: Option<Box<dyn AuxDevice>>,
}

#[async_trait]
pub trait AuxDevice : Send + Sync {
    fn id(&self) -> u8;
    async fn init(
        &mut self,
        tx: tokio::sync::broadcast::Sender<(u8, DisplayStatus)>
    ) -> Result<(), Box<dyn Error>>;
    fn read(&self) -> Result<String, Box<dyn Error  + '_>>;
}
impl Debug for dyn AuxDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Aux device {{{}}}", self.id())
    }
}


#[derive(Debug, )]
pub struct Runner {
    tx: broadcast::Sender<(u8, DisplayStatus)>,
    task: tokio::task::JoinHandle<()>,
    status: Arc<Mutex<Status>>,
}
impl Runner {
    pub fn new(status: Arc<Mutex<Status>>) -> Self {
        Self {
            tx: broadcast::channel(8).0,
            task: tokio::spawn(async move {}),
            status,
        }
    }

    pub fn channel(
        &self,
    ) -> broadcast::Sender<(u8, DisplayStatus)> {
        self.tx.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx = self.tx.subscribe();
        let status = self.status.clone();
        self.task = tokio::spawn(async move {
            println!("Spawned aux runner");
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        println!("\tAux: {:?}", data);
                        match data {
                            (_, display_status) => {
                                let mut lock = status.lock().unwrap();
                                lock.disp = display_status;
                            }
                        }



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
    // Blue,
    Green,
    Yellow,
    Red,
}