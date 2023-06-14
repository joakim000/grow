use alloc::collections::BTreeMap;
use async_trait::async_trait;
use core::error::Error;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
pub type FanMutex = Arc<Mutex<Box<dyn Fan>>>;
use super::Zone;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub temp_fan_low: f64,
    pub temp_fan_high: f64,
    pub temp_warning: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Status {
    pub temp: Option<f32>,
    pub fan_rpm: Option<f32>,
    pub indicator: Option<crate::Indicator>,
}

// #[derive(Clone, Debug, PartialEq)]
pub struct Interface {
    pub fan: Option<Box<dyn Fan>>,
    pub thermo: Option<Box<dyn Thermometer>>,
    // pub fan_control: FanMutex,
}
impl Interface {
    pub fn set_fan(&mut self, fan: Box<dyn Fan>) -> () {
        self.fan = Some(fan);
    }
    pub fn set_thermo(&mut self, thermo: Box<dyn Thermometer>) -> () {
        self.thermo = Some(thermo);
    }
}

#[async_trait]
pub trait Fan {
    fn init(
        &mut self,
        tx_rpm: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
        rx_control: tokio::sync::broadcast::Receiver<FanSetting>,
    ) -> FanMutex;
    fn to_high(&self) -> Result<(), Box<dyn Error>>;
    fn to_low(&self) -> Result<(), Box<dyn Error>>;
}
pub trait Thermometer {
    fn read_temp(&self) -> Result<(i32), Box<dyn Error>>;
    fn init(
        &mut self,
        tx_temp: tokio::sync::broadcast::Sender<(u8, Option<f64>)>
    ) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum FanSetting {
    Off,
    Low,
    High,
}

pub struct Runner {
    pub fan_control: (
        broadcast::Sender<FanSetting>,
        broadcast::Receiver<FanSetting>,
    ),
    pub fan_rpm: (
        broadcast::Sender<(u8, Option<f32>)>,
        broadcast::Receiver<(u8, Option<f32>)>,
    ),
    pub temp: (
        broadcast::Sender<(u8, Option<f64>)>,
        broadcast::Receiver<(u8, Option<f64>)>,
    ),
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            fan_control: broadcast::channel(1),
            fan_rpm: broadcast::channel(1),
            temp: broadcast::channel(1),
            task: tokio::spawn(async move {}),
        }
    }

    pub fn channels_for_fan(
        &self,
    ) -> (
        broadcast::Sender<(u8, Option<f32>)>,
        broadcast::Receiver<FanSetting>,
    ) {
        (self.fan_rpm.0.clone(), self.fan_control.0.subscribe())
    }
    pub fn channel_for_thermo(
        &self,
    ) -> broadcast::Sender<(u8, Option<f64>)> {
        self.temp.0.clone()
    }

    pub fn run(&mut self, fan: FanMutex) {
        let mut rx_rpm = self.fan_rpm.0.subscribe();
        let mut rx_temp = self.temp.0.subscribe();
        self.task = tokio::spawn(async move {
            // while let Ok(data) = rx_rpm.recv().await {
            //     println!("Runner says: Fan rpm: {:?}", data);
            // }

            loop {
                tokio::select! {
                    Ok(data) = rx_rpm.recv() => {
                        println!("Runner says: Fan rpm: {:?}", data);
                    }
                    Ok(data) = rx_temp.recv() => {
                        println!("Runner says: Temp: {:?}", data);
                        match data.1 {
                            Some(temp) => {
                                if temp > 25f64 {
                                    
                                }
                            }
                            _ => ()
                        }
                    }
                    else => { break }
                };
            }
        });
    }
}

pub fn new(id: u8, settings: Settings) -> super::Zone {
    Zone::Air {
        id: 0,
        settings,
        status: Status {
            temp: None,
            fan_rpm: None,
            indicator: None,
        },
        interface: Interface {
            fan: None,
            thermo: None,
        },
        runner: Runner::new(),
    }
}
