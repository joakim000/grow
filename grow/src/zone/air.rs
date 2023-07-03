
use async_trait::async_trait;
use core::error::Error;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::broadcast;
// use tokio::sync::Mutex;
use core::fmt::Debug;
use std::cmp;
use std::sync::Mutex;
use time::OffsetDateTime;
use serde::{Serialize, Deserialize};

pub type FanMutex = Arc<Mutex<Box<dyn Fan>>>;
use super::Zone;
use super::*;
use crate::ops::display::{DisplayStatus, Indicator};
use crate::ops::OpsChannelsTx;
use crate::ops::SysLog;
// use crate::TIME_OFFSET;

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status {
        temp: None,
        fan_rpm: None,
        fan_mode: None,
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
            changed: OffsetDateTime::UNIX_EPOCH,
        },
    };
    let status_mutex = Arc::new(RwLock::new(status));
    Zone::Air {
        id,
        settings,
        runner: Runner::new(id, status_mutex.clone()),
        status: status_mutex,
        interface: Interface {
            fan: None,
            thermo: None,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub temp_high_yellow_warning: f64,
    pub temp_high_red_alert: f64,
    pub temp_fan_low: f32,
    pub temp_fan_high: f32,
    pub fan_rpm_low_red_alert: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub temp: Option<f64>,
    pub fan_rpm: Option<f32>,
    pub fan_mode: Option<FanSetting>,
    pub disp: DisplayStatus,
}

#[derive(Debug)]
pub struct Interface {
    pub fan: Option<Box<dyn Fan>>,
    pub thermo: Option<Box<dyn Thermometer>>,
}
impl Interface {}

#[async_trait]
pub trait Fan: Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_rpm: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
        rx_control: tokio::sync::broadcast::Receiver<FanSetting>,
    ) -> Result<(), Box<dyn Error>>;
    fn read(&mut self) -> Result<Option<f32>, Box<dyn Error + '_>>;
    fn to_high(&self) -> Result<(), Box<dyn Error + '_>>;
    fn to_low(&self) -> Result<(), Box<dyn Error + '_>>;
    fn set_duty_cycle(
        &self,
        duty_cycle: f64,
    ) -> Result<(), Box<dyn Error + '_>>;
}
impl Debug for dyn Fan {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Fan: {{{}}}", self.id())
    }
}
pub trait Thermometer: Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_temp: tokio::sync::broadcast::Sender<(u8, Option<f64>)>,
    ) -> Result<(), Box<dyn Error>>;
    fn read(&self) -> Result<f64, Box<dyn Error + '_>>;
}
impl Debug for dyn Thermometer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thermometer: {{{}}}", self.id())
    }
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub enum FanSetting {
    Off,
    Low,
    Medium,
    High,
    // Low(f32),
    // High(f32),
}

#[derive(Debug)]
pub struct Runner {
    id: u8,
    tx_fan_control: broadcast::Sender<FanSetting>,
    tx_fan_rpm: broadcast::Sender<(u8, Option<f32>)>,
    temp: broadcast::Sender<(u8, Option<f64>)>,
    task: tokio::task::JoinHandle<()>,
    status: Arc<RwLock<Status>>,
}
impl Runner {
    pub fn new(id: u8, status: Arc<RwLock<Status>>) -> Self {
        Self {
            id,
            status,
            tx_fan_control: broadcast::channel(1).0,
            tx_fan_rpm: broadcast::channel(8).0,
            temp: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn fan_channels(
        &self,
    ) -> (
        broadcast::Sender<(u8, Option<f32>)>,
        broadcast::Receiver<FanSetting>,
    ) {
        (self.tx_fan_rpm.clone(), self.tx_fan_control.subscribe())
    }
    pub fn thermo_feedback_sender(
        &self,
    ) -> broadcast::Sender<(u8, Option<f64>)> {
        self.temp.clone()
    }

    pub fn run(
        &mut self,
        settings: Settings,
        zone_channels: ZoneChannelsTx,
        ops_channels: OpsChannelsTx,
        have_fan: bool,
    ) {
        let id = self.id;
        let _to_manager = zone_channels.zoneupdate;
        let to_status_subscribers = zone_channels.zonestatus;
        let to_logger = zone_channels.zonelog;
        let to_syslog = ops_channels.syslog;
        let status = self.status.clone();
        let mut rx_rpm = self.tx_fan_rpm.subscribe();
        let mut rx_temp = self.temp.subscribe();
        let tx_fan = self.tx_fan_control.clone();
        let mut requested_fan_mode: FanSetting = FanSetting::Off;

        self.task = tokio::spawn(async move {
            let _ = to_syslog
                .send(SysLog::new(format!("Spawned air runner id {}", &id)))
                .await;
            let set_and_send = |ds: DisplayStatus| {
                *&mut status.write().disp = ds.clone();
                let _ = &to_status_subscribers.send(ZoneDisplay::Air { id, info: ds });
            };
            set_and_send(DisplayStatus::new(
                Indicator::Green,
                Some(format!("Air running")),
            ));

            // Workaround for temp and fan statuses overlappning in this zone. Not sure if zone should be refactored or DisplayStatus system?
            // This solutions displays comma-separated temp and fan msgs and the most severe indicator of the two.
            let mut buf_temp = String::from("No data");
            let mut buf_fan = String::from("No fan"); 
            if have_fan { buf_fan = String::from("No data"); }
            let mut buf_temp_ind = Indicator::Blue;
            let mut buf_fan_ind = Indicator::Blue;
            loop {
                tokio::select! {
                    Ok(data) = rx_rpm.recv() => {
                        // println!("\tFan rpm: {:?}", data);
                        let o_ds: Option<DisplayStatus>; // = None;
                        status.write().fan_rpm = data.1;
                        match data {
                            (_id, None) => {
                                // if { (status.read().fan_rpm.is_some()) {
                                    buf_fan = format!("No rpm data");
                                    buf_fan_ind = Indicator::Red;
                                    o_ds = Some(DisplayStatus::new(cmp::max(buf_fan_ind, buf_temp_ind), Some( format!("{},  {}", buf_temp, buf_fan) )) );
                                // }
                            }
                            (_id, Some(rpm)) if rpm < settings.fan_rpm_low_red_alert => {
                                buf_fan = format!("Fan LOW: {:.0} rpm", rpm);
                                buf_fan_ind = Indicator::Yellow;
                                o_ds = Some(DisplayStatus::new(cmp::max(buf_fan_ind, buf_temp_ind), Some( format!("{},  {}", buf_temp, buf_fan) )) );
                            }
                            (_id, Some(rpm)) => { // if (status.read().disp.indicator != Indicator::Green) => {
                                buf_fan = format!("Fan: {:.0} rpm", rpm);
                                buf_fan_ind = Indicator::Green;
                                o_ds = Some(DisplayStatus::new(cmp::max(buf_fan_ind, buf_temp_ind), Some( format!("{},  {}", buf_temp, buf_fan) )) );
                            }
                            // _ => {}
                        }
                        let temp = status.read().temp;
                        let _ = to_logger.send(ZoneLog::Air {id: data.0, temp: temp, fan_rpm: data.1, changed_status: o_ds.clone() }).await;
                        match o_ds {
                            Some(ds) => { set_and_send(ds); }
                            None => {}
                        }
                    }
                    Ok(data) = rx_temp.recv() => {
                        // println!("\tTemp: {:?}", data);
                        let o_ds: Option<DisplayStatus>; // = None;
                        status.write().temp = data.1;
                        match data {
                            (_id, None) => {
                                buf_temp = format!("No temp data");
                                buf_temp_ind = Indicator::Red;
                                o_ds = Some(DisplayStatus::new(cmp::max(buf_fan_ind, buf_temp_ind), Some( format!("{},  {}", buf_temp, buf_fan) )) );
                            }
                            (_id, Some(temp)) => {
                                // Fan control
                                if temp > settings.temp_fan_high.into() { requested_fan_mode = FanSetting::High }
                                else if temp > settings.temp_fan_low.into() { requested_fan_mode = FanSetting::Low }
                                else { requested_fan_mode = FanSetting::Off; }

                                // Status from temperature
                                if temp > settings.temp_high_red_alert {
                                    buf_temp = format!("Temp HIGH: {:.1}°C", &temp);
                                    buf_temp_ind = Indicator::Red;
                                    o_ds = Some(DisplayStatus::new(cmp::max(buf_fan_ind, buf_temp_ind), Some( format!("{},  {}", buf_temp, buf_fan) )) );
                                }
                                else if temp > settings.temp_high_yellow_warning {
                                    buf_temp = format!("Temp HIGH: {:.1}°C", &temp);
                                    buf_temp_ind = Indicator::Yellow;
                                    o_ds = Some(DisplayStatus::new(cmp::max(buf_fan_ind, buf_temp_ind), Some( format!("{},  {}", buf_temp, buf_fan) )) );
                                }
                                else {
                                    buf_temp = format!("Temp: {:.1}°C", &temp);
                                    buf_temp_ind = Indicator::Green;
                                    o_ds = Some(DisplayStatus::new(cmp::max(buf_fan_ind, buf_temp_ind), Some( format!("{},  {}", buf_temp, buf_fan) )) );
                                }
                            }
                        }

                        if have_fan {
                            let current_mode = status.read().fan_mode;
                            if !(current_mode.is_some_and(|x|x == requested_fan_mode))  {
                                match tx_fan.send(requested_fan_mode) {
                                    Ok(_) => {
                                        let _ = to_syslog.send(SysLog::new(format!("Air {} fan set to {:?}", &id, &requested_fan_mode))).await;
                                        status.write().fan_mode = Some(requested_fan_mode);
                                    },
                                    Err(e) => {
                                        let _ = to_syslog.send(SysLog::new(format!("Air {} fan error: {:?}", &id, e))).await;
                                    }
                                }
                            }
                        }
                        let fan_rpm = status.read().fan_rpm;
                        let _ = to_logger.send(ZoneLog::Air {id: data.0, temp: data.1, fan_rpm, changed_status: o_ds.clone() }).await;
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
