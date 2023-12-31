
use async_trait::async_trait;
use core::error::Error;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::broadcast;
// use tokio::sync::Mutex;
use core::fmt::Debug;

use time::OffsetDateTime;
use serde::{Serialize, Deserialize};

use super::Zone;
use super::*;
use crate::ops::display::{DisplayStatus, Indicator};
use crate::ops::OpsChannelsTx;
use crate::ops::SysLog;
// use crate::TIME_OFFSET;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum TankLevel {
    Ok,
    Low,
    Empty,
    Overfill,
    NoData
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
    Zone::Tank {
        id,
        settings,
        runner: Runner::new(id, status_mutex.clone()),
        status: status_mutex,
        interface: Interface { tank_sensor: None },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub disp: DisplayStatus,
}

#[derive(Debug)]
pub struct Interface {
    pub tank_sensor: Option<Box<dyn TankSensor>>,
}

#[async_trait]
pub trait TankSensor: Send + Sync {
    fn id(&self) -> u8;
    async fn init(
        &mut self,
        tx_tank: tokio::sync::broadcast::Sender<(u8, Option<TankLevel>)>,
    ) -> Result<(), Box<dyn Error>>;
    fn read(&self) -> Result<TankLevel, Box<dyn Error>>;
}
impl Debug for dyn TankSensor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Tanksensor {{{}}}", self.id())
    }
}

#[derive(Debug)]
pub struct Runner {
    id: u8,
    tx_tank: broadcast::Sender<(u8, Option<TankLevel>)>,
    task: tokio::task::JoinHandle<()>,
    status: Arc<RwLock<Status>>,
}
impl Runner {
    pub fn new(id: u8, status: Arc<RwLock<Status>>) -> Self {
        Self {
            id,
            status,
            tx_tank: broadcast::channel(64).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn tank_feedback_sender(
        &self,
    ) -> broadcast::Sender<(u8, Option<TankLevel>)> {
        self.tx_tank.clone()
    }

    pub fn run(
        &mut self,
        _settings: Settings,
        zone_channels: ZoneChannelsTx,
        ops_channels: OpsChannelsTx,
    ) {
        let id = self.id;
        let _to_manager = zone_channels.zoneupdate;
        let to_status_subscribers = zone_channels.zonestatus;
        let to_logger = zone_channels.zonelog;
        let to_syslog = ops_channels.syslog;
        let mut rx = self.tx_tank.subscribe();
        let status = self.status.clone();

        self.task = tokio::spawn(async move {
            let _ = to_syslog
                .send(SysLog::new(format!("Spawned tank runner id {}", &id)))
                .await;
            let set_and_send = |ds: DisplayStatus| {
                *&mut status.write().disp = ds.clone();
                let _ = &to_status_subscribers.send(ZoneDisplay::Tank { id, info: ds });
            };
            set_and_send(DisplayStatus::new(
                Indicator::Blue,
                Some(format!("Tank running")),
            ));
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        let mut o_ds: Option<DisplayStatus> = None;
                        match data {
                            (_id, None) => {
                                o_ds = Some(DisplayStatus::new(Indicator::Red, Some( format!("No data from tank sensor") )) );
                            },
                            (_id, Some(TankLevel::Ok)) => {
                                o_ds = Some(DisplayStatus::new(Indicator::Green, Some( format!("Tank ok") )) );
                            },
                            (_id, Some(TankLevel::Low)) => {
                                o_ds = Some(DisplayStatus::new(Indicator::Yellow, Some( format!("Tank low") )) );
                            },
                            (_id, Some(TankLevel::Empty)) => {
                                o_ds = Some(DisplayStatus::new(Indicator::Red, Some( format!("Tank empty") )) );
                            },
                            (_id, Some(TankLevel::Overfill)) => {
                                o_ds = Some(DisplayStatus::new(Indicator::Red, Some( format!("Tank overfill") )) );
                            },
                            _ => ()
                        }
                        let _ = to_logger.send(ZoneLog::Tank {id: data.0, changed_status: o_ds.clone() }).await;
                        match o_ds {
                            Some(ds) => { set_and_send(ds); }
                            None => {}
                        }
                    }
                    else => { break }
                };
            }
        });
    }
}
