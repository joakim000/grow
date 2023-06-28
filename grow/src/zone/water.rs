use super::Zone;
use super::ZoneUpdate;
use async_trait::async_trait;
use core::error::Error;
use core::fmt::Debug;
use core::time::Duration;
use parking_lot::RwLock;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
// use cond_utils::Between;

use crate::ops::display::{DisplayStatus, Indicator};
use crate::ops::OpsChannelsTx;
use crate::ops::SysLog;
use crate::zone::*;
use crate::TIME_OFFSET;
pub mod arm;
pub mod pump;
pub mod tank;

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status {
        moisture_level: None,
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
            changed: OffsetDateTime::UNIX_EPOCH,
        },
        kind: None,
    };
    let status_mutex = Arc::new(RwLock::new(status));
    Zone::Water {
        id,
        settings,
        runner: Runner::new(id, status_mutex.clone()),
        status: status_mutex,
        interface: Interface { moist: None },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub moisture_limit_water: f32,
    pub moisture_high_yellow_warning: f32,
    pub moisture_high_red_alert: f32,
    pub moisture_low_yellow_warning: f32,
    pub moisture_low_red_alert: f32,
    pub pump_id: u8,
    pub pump_time: Duration,
    pub position: super::arm::Position,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub moisture_level: Option<f32>,
    pub disp: DisplayStatus,
    kind: Option<WaterStatusKind>,
}

#[async_trait]
pub trait MoistureSensor: Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_moist: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<(), Box<dyn Error>>;
    fn read(&self) -> Result<(f32), Box<dyn Error + '_>>;
}

#[derive(Debug)]
pub struct Interface {
    pub moist: Option<Box<dyn MoistureSensor>>,
}

impl Debug for dyn MoistureSensor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MoistureSensor {{{}}}", self.id())
    }
}

#[derive(Debug)]
pub struct Runner {
    id: u8,
    tx_moisture: broadcast::Sender<(u8, Option<f32>)>,

    pub task: tokio::task::JoinHandle<()>,
    status: Arc<RwLock<Status>>,
}
impl Runner {
    pub fn new(id: u8, status: Arc<RwLock<Status>>) -> Self {
        Self {
            id,
            status,
            tx_moisture: broadcast::channel(168).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn moisture_feedback_sender(&self) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx_moisture.clone()
    }

    pub fn run(
        &mut self,
        settings: Settings,
        zone_channels: ZoneChannelsTx,
        ops_channels: OpsChannelsTx,
    ) {
        let id = self.id;
        let to_manager = zone_channels.zoneupdate;
        let to_status_subscribers = zone_channels.zonestatus;
        let to_logger = zone_channels.zonelog;
        let to_syslog = ops_channels.syslog;
        let mut rx = self.tx_moisture.subscribe();
        let status = self.status.clone();

        self.task = tokio::spawn(async move {
            to_syslog
                .send(SysLog::new(format!("Spawned water runner id {}", &id)))
                .await;
            let set_and_send = |ds: DisplayStatus| {
                *&mut status.write().disp = ds.clone();
                &to_status_subscribers.send(ZoneDisplay::Water { id, info: ds });
            };
            set_and_send(DisplayStatus::new(Indicator::Green, Some( format!("Water running") )) );
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        let mut o_ds: Option<DisplayStatus> = None;
                        match data {
                            (id, None) if status.read().kind.as_ref().is_some_and(|k| k != &WaterStatusKind::NoData) => {
                                o_ds = Some(DisplayStatus::new(Indicator::Red, Some( format!("No data from moisture sensor") )) );
                            },
                            (id, Some(moisture)) => {
                                // Watering needed
                                if moisture < settings.moisture_limit_water {
                                    to_manager.send(ZoneUpdate::Water{id, moisture}).await;
                                }

                                // Status update
                                if (moisture < settings.moisture_low_red_alert) { //& (status.read().kind.as_ref().is_some_and(|k| k != &WaterStatusKind::AlertLow)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Red, Some( format!("Alert: Moisture LOW {}", moisture) )) );
                                    status.write().kind == Some(WaterStatusKind::AlertLow);
                                }
                                else if (moisture > settings.moisture_high_red_alert) { //& !(status.read().kind.as_ref().is_some_and(|k| k == &WaterStatusKind::AlertHigh)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Red, Some( format!("Alert: Moisture HIGH {}", moisture) )) );
                                    status.write().kind == Some(WaterStatusKind::AlertHigh);
                                }
                                else if (moisture < settings.moisture_low_yellow_warning)  { //& (status.read().kind.as_ref().is_some_and(|k| k != &WaterStatusKind::WarningLow)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Yellow, Some( format!("Warning: Moisture LOW {}", moisture) )) );
                                    status.write().kind == Some(WaterStatusKind::WarningLow);
                                }
                                else if (moisture > settings.moisture_high_yellow_warning) { //& (status.read().kind.as_ref().is_some_and(|k| k != &WaterStatusKind::WarningLow)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Yellow, Some( format!("Warning: Moisture HIGH {}", moisture) )) );
                                    status.write().kind == Some(WaterStatusKind::WarningLow);
                                }
                                else { // if (status.read().kind.as_ref().is_some_and(|k| k != &WaterStatusKind::Ok)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Green, Some( format!("Ok: Moisture {}", moisture) )) );
                                    status.write().kind == Some(WaterStatusKind::Ok);
                                }
                            },
                            _ => ()
                        }
                        to_logger.send(ZoneLog::Water{id: data.0, moisture: data.1, changed_status: o_ds.clone() }).await;
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

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
enum WaterStatusKind {
    AlertLow,
    AlertHigh,
    WarningLow,
    WarningHigh,
    Ok,
    NoData,
}