use alloc::collections::BTreeMap;
use async_trait::async_trait;
use core::error::Error;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
use core::fmt::Debug;
pub type FanMutex = Arc<Mutex<Box<dyn Fan>>>;
use super::Zone;

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status {
        temp: None,
        fan_rpm: None,
        indicator: None,
        msg: None,
    };
    Zone::Air {
        id,
        settings,
        status: Arc::new(Mutex::new(status)),
        // status: Status {
        //     temp: None,
        //     fan_rpm: None,
        //     indicator: None,
        //     msg: None,
        // },
        interface: Interface {
            fan: None,
            thermo: None,
        },
        runner: Runner::new(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub temp_fan_low: f32,
    pub temp_fan_high: f32,
    pub temp_warning: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub temp: Option<f32>,
    pub fan_rpm: Option<f32>,
    pub indicator: Option<crate::Indicator>,
    pub msg: Option<String>,
}

#[derive( Debug, )]
pub struct Interface {
    pub fan: Option<Box<dyn Fan>>,
    pub thermo: Option<Box<dyn Thermometer>>,
}
impl Interface {
    // pub fn set_fan(&mut self, fan: Box<dyn Fan>) -> () {
    //     self.fan = Some(fan);
    // }
    // pub fn set_thermo(&mut self, thermo: Box<dyn Thermometer>) -> () {
    //     self.thermo = Some(thermo);
    // }
}




#[async_trait]
pub trait Fan {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_rpm: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
        rx_control: tokio::sync::broadcast::Receiver<FanSetting>,
    ) -> Result<(), Box<dyn Error>>;
    fn to_high(&self) -> Result<(), Box<dyn Error + '_>>;
    fn to_low(&self) -> Result<(), Box<dyn Error + '_>>;
}
impl Debug for dyn Fan {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Fan: {{{}}}", self.id())
    }
}
pub trait Thermometer {
    fn id(&self) -> u8;
    fn read_temp(&self) -> Result<(i32), Box<dyn Error>>;
    fn init(
        &mut self,
        tx_temp: tokio::sync::broadcast::Sender<(u8, Option<f32>)>
    ) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn Thermometer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thermometer: {{{}}}", self.id())
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum FanSetting {
    Off,
    Low,
    High,
}

#[derive(Debug, )]
pub struct Runner {
    pub fan_control: broadcast::Sender<FanSetting>,
    pub fan_rpm: broadcast::Sender<(u8, Option<f32>)>,
    pub temp: broadcast::Sender<(u8, Option<f32>)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            fan_control: broadcast::channel(1).0,
            fan_rpm: broadcast::channel(1).0,
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
        (self.fan_rpm.clone(), self.fan_control.subscribe())
    }
    pub fn thermo_channel(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.temp.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx_rpm = self.fan_rpm.subscribe();
        let mut rx_temp = self.temp.subscribe();
        let tx_fan = self.fan_control.clone();
        let mut current_setting = FanSetting::Off;
        self.task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(data) = rx_rpm.recv() => {
                        println!("Fan rpm: {:?}", data);
                    }
                    Ok(data) = rx_temp.recv() => {
                        println!("Temp: {:?}", data);
                        match data.1 {
                            Some(temp) => {
                                if temp > 25f32 {
                                    println!("temp > 25 deg C");
                                    if current_setting != FanSetting::Low {
                                        match tx_fan.send(FanSetting::Low) {
                                            Ok(_) => {
                                                current_setting = FanSetting::Low;
                                            },
                                            Err(e) => {
                                                eprintln!("Fan control error: {:?}", e);
                                            }
                                        }
                                    }
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

