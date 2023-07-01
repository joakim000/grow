use alloc::collections::BTreeMap;
use async_trait::async_trait;
use core::error::Error;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::broadcast;
// use tokio::sync::Mutex;
use core::fmt::Debug;
use std::sync::Mutex;
use time::OffsetDateTime;
use time::Time;
use core::time::Duration;

use super::Zone;
use super::*;
use crate::ops::display::{DisplayStatus, Indicator};
use crate::ops::OpsChannelsTx;
use crate::ops::SysLog;
// use crate::TIME_OFFSET;

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status {
        lamp_state: Some(LampState::Off),
        light_level: None,
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
            changed: OffsetDateTime::UNIX_EPOCH,
        },
        kind: None,
    };
    let status_mutex = Arc::new(RwLock::new(status));
    Zone::Light {
        id,
        settings,
        runner: Runner::new(id, status_mutex.clone()),
        status: status_mutex,
        interface: Interface {
            lamp: None,
            lightmeter: None,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub lightlevel_low_yellow_warning: f32,
    pub lightlevel_low_red_alert: f32,
    pub lamp_on: Time,
    pub lamp_off: Time,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub lamp_state: Option<LampState>,
    pub light_level: Option<f32>,
    pub disp: DisplayStatus,
    kind: Option<LightStatusKind>,
}

#[derive(Debug)]
pub struct Interface {
    pub lamp: Option<Box<dyn Lamp>>,
    pub lightmeter: Option<Box<dyn Lightmeter>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LampState {
    On,
    Off,
}

pub trait Lamp: Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        rx_lamp: tokio::sync::broadcast::Receiver<(u8, bool)>,
    ) -> Result<(), Box<dyn Error>>;
    fn set_state(&self, state: LampState) -> Result<(), Box<dyn Error + '_>>;
    fn state(&self) -> Result<LampState, Box<dyn Error>>;
}
impl Debug for dyn Lamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Lamp: {{{}}}", 0)
    }
}

pub trait Lightmeter: Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_light: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<(), Box<dyn Error>>;
    fn read(&self) -> Result<(f32), Box<dyn Error + '_>>;
}
impl Debug for dyn Lightmeter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LightMeter: {{{}}}", 0)
    }
}

#[derive(Debug)]
pub struct Runner {
    id: u8,
    status: Arc<RwLock<Status>>,
    tx_lightmeter: broadcast::Sender<(u8, Option<f32>)>,
    tx_lamp: broadcast::Sender<(u8, bool)>,
    task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new(id: u8, status: Arc<RwLock<Status>>) -> Self {
        Self {
            tx_lightmeter: broadcast::channel(1).0,
            tx_lamp: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
            id,
            status,
        }
    }

    pub fn lightmeter_feedback_sender(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx_lightmeter.clone()
    }
    pub fn lamp_cmd_receiver(&self) -> broadcast::Receiver<(u8, bool)> {
        self.tx_lamp.subscribe()
    }
    pub fn lamp_cmd_sender(&self) -> broadcast::Sender<(u8, bool)> {
        self.tx_lamp.clone()
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
        let mut rx = self.tx_lightmeter.subscribe();
        let status = self.status.clone();
        let to_lamp = self.lamp_cmd_sender();
        let mut each_minute = tokio::time::interval(Duration::from_secs(60));
        self.task = tokio::spawn(async move {
            to_syslog
                .send(SysLog::new(format!("Spawned light runner id {}", &id)))
                .await;
            let set_and_send = |ds: DisplayStatus| {
                *&mut status.write().disp = ds.clone();
                &to_status_subscribers
                    .send(ZoneDisplay::Light { id, info: ds });
            };
            set_and_send(DisplayStatus::new(
                Indicator::Green,
                Some(format!("Light running")),
            ));
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        // println!("Light: {:?}", data);
                        let mut o_ds: Option<DisplayStatus> = None;
                        let state = status.read().lamp_state.expect("Lamp status error");
                        match data {
                            (id, None) => {
                                o_ds = Some(DisplayStatus::new(Indicator::Red, Some( format!("No data from lightmeter") )));
                            },
                            (id, Some(lightlevel)) => {
                                if (&state == &LampState::Off) { // & (status.read().kind.as_ref().is_some_and(|k| k != &LightStatusKind::OffOk)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Green, Some( format!("Lamp OFF, Ambient: {}", lightlevel) )) );
                                    status.write().kind == Some(LightStatusKind::OffOk);
                                }
                                else if (lightlevel < settings.lightlevel_low_red_alert) { // & (status.read().kind.as_ref().is_some_and(|k| k != &LightStatusKind::OnAlert)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Red, Some( format!("Lamp ON, Alert: {}", lightlevel) )) );
                                    status.write().kind == Some(LightStatusKind::OnAlert);
                                }
                                else if (lightlevel < settings.lightlevel_low_yellow_warning) { //& (status.read().kind.as_ref().is_some_and(|k| k != &LightStatusKind::OnWarning)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Yellow, Some( format!("Lamp ON, Warning: {}", lightlevel) )) );
                                    status.write().kind == Some(LightStatusKind::OnWarning);
                                }
                                else { // if (status.read().kind.as_ref().is_some_and(|k| k != &LightStatusKind::OnOk)) {
                                    o_ds = Some(DisplayStatus::new(Indicator::Green, Some( format!("Lamp ON, Ok: {}", lightlevel) )) );
                                    status.write().kind == Some(LightStatusKind::OnOk);
                                }
                            },
                            _ => ()
                        }
                        to_logger.send(ZoneLog::Light {id: data.0, lamp_on: Some(state), light_level: data.1, changed_status: o_ds.clone() }).await;
                        match o_ds {
                            Some(ds) => { set_and_send(ds); }
                            None => {}
                        }
                    }
                    _ = each_minute.tick() => {
                        let now = OffsetDateTime::now_utc().to_offset(crate::TIME_OFFSET);
                        let state = status.read().lamp_state;
                        if now.time() > settings.lamp_off { 
                            match state {
                                Some(LampState::On) | None => {
                                    to_lamp.send((id, false));
                                    status.write().lamp_state = Some(LampState::Off);
                                    to_syslog.send(SysLog::new(format!("Lamp OFF @ {} (Set: {}", crate::ops::display::format_time(now), settings.lamp_off))).await;
                                }
                                _ => {}
                            }
                        }
                        else if now.time() > settings.lamp_on {
                            match state {
                                Some(LampState::Off) | None => {
                                    to_lamp.send((id, true));
                                    status.write().lamp_state = Some(LampState::On);
                                    to_syslog.send(SysLog::new(format!("Lamp ON @ {} (Set: {}", crate::ops::display::format_time(now), settings.lamp_on))).await;
                                }
                                _ => {}
                            }
                        }    
                        // match state {
                        //     Some(LampState::On) | None => {
                        //         if now.time() > settings.lamp_off {
                        //             to_lamp.send((id, false));
                        //             status.write().lamp_state = Some(LampState::Off);
                        //             to_syslog.send(SysLog::new(format!("Lamp OFF @ {} (Set: {}", crate::ops::display::format_time(now), settings.lamp_off))).await;
                        //         }
                        //     }
                        //     Some(LampState::Off) | None => {
                        //         if now.time() > settings.lamp_on {
                        //             to_lamp.send((id, true));
                        //             status.write().lamp_state = Some(LampState::On);
                        //             to_syslog.send(SysLog::new(format!("Lamp ON @ {} (Set: {}", crate::ops::display::format_time(now), settings.lamp_on))).await;
                        //         }
                        //     }
                        // }
                    }
                    else => { break }
                };
            }
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
enum LightStatusKind {
    OffOk,
    OnOk,
    OnWarning,
    OnAlert,
}

// struct Timer{}
