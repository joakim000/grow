/// Enable any auxiliary equipment to provide status (ex. UPS, servo controller)
/// Example rpi3 uses this for alerts and updates from Lego hub  

use core::error::Error;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::ops::Deref;
use tokio::sync::broadcast;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
use core::fmt::Debug;
use super::Zone;
use super::*;
use crate::ops::display::{Indicator, DisplayStatus};
use crate::ops::OpsChannelsTx;
use crate::ops::SysLog;


pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
        }
    };
    let status_mutex = Arc::new(RwLock::new(status));
    Zone::Aux {
        id,
        settings,
        runner: Runner::new(id, status_mutex.clone()),
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
    id: u8,
    tx_aux: broadcast::Sender<(u8, DisplayStatus)>,
    task: tokio::task::JoinHandle<()>,
    status: Arc<RwLock<Status>>,
}
impl Runner {
    pub fn new(id: u8, status: Arc<RwLock<Status>>) -> Self {
        Self {
            id,
            status,
            tx_aux: broadcast::channel(16).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn aux_feedback_sender(
        &self,
    ) -> broadcast::Sender<(u8, DisplayStatus)> {
        self.tx_aux.clone()
    }

    pub fn run(&mut self, settings: Settings, zone_channels: ZoneChannelsTx, ops_channels: OpsChannelsTx) {
        let id = self.id;
        let to_manager = zone_channels.zoneupdate;
        let to_status_subscribers = zone_channels.zonestatus; 
        let to_logger = zone_channels.zonelog;
        let to_syslog = ops_channels.syslog;
        let mut rx = self.tx_aux.subscribe();
        let status = self.status.clone();
        self.task = tokio::spawn(async move {
            to_syslog.send(SysLog::new(format!("Spawned irrigation runner id {}", &id))).await;
            let set_and_send = |ds: DisplayStatus | {
                *&mut status.write().disp = ds.clone(); 
                &to_status_subscribers.send(ZoneDisplay::Aux { id, info: ds });        
            };
            set_and_send( DisplayStatus {indicator: Indicator::Green, msg: Some(format!("Aux running"))} );
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        match data {
                            (_, display_status) => {
                                set_and_send(display_status.clone());
                                to_logger.send(ZoneLog::Aux{id: data.0, changed_status: Some(display_status) }).await;
                            },
                            _ => () 
                        }
                    }
                    else => { break }
                };
            }
        });
    }
}
