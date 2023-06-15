use core::error::Error;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
use core::fmt::Debug;
use super::Zone;


pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
        lamp_on: None,
        light_level: None,
       };
    Zone::Light  {
        id,
        settings,
        status: Arc::new(Mutex::new(status)),
        interface: Interface {
            lamp: None,
            lightmeter: None,
        },
        runner: Runner::new(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Status {
    lamp_on: Option<bool>,
    light_level: Option<f32>
}

#[derive( Debug, )]
pub struct Interface {
    pub lamp: Option<Box<dyn Lamp>>,
    pub lightmeter: Option<Box<dyn Lightmeter>>,
}
impl Interface {
    // pub fn set_lamp(&mut self, lamp: Box<dyn Lamp>) -> () {
    //     self.lamp = Some(lamp);
    // }
    // pub fn set_lightmeter(&mut self, lightmeter: Box<dyn Lightmeter>) -> () {
    //     self.lightmeter = Some(lightmeter);
    // }
}


pub trait Lamp {
    fn on(&self) -> Result<(), Box<dyn Error>>;
    fn off(&self) -> Result<(), Box<dyn Error>>;
    fn init(&self) -> Result<(), Box<dyn Error>>;
    // fn init(
    //     &mut self,
    //     tx_light: tokio::sync::broadcast::Sender<(u8, Option<f32>)>
    // ) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn Lamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Lamp: {{{}}}", 0)
    }
}


pub trait Lightmeter {
    // fn init(&self) -> Result<(), Box<dyn Error>>;
    fn init(
        &mut self,
        tx_light: tokio::sync::broadcast::Sender<(u8, Option<f32>)>
    ) -> Result<(), Box<dyn Error>>;   
    
}
impl Debug for dyn Lightmeter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LightMeter: {{{}}}", 0)
    }
}


#[derive(Debug, )]
pub struct Runner {
    pub moist: broadcast::Sender<(u8, Option<f32>)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            moist: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn channel_for_moist(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.moist.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx_moist = self.moist.subscribe();
        // let mut current_setting = FanSetting::Off;
        self.task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(data) = rx_moist.recv() => {
                        println!("Moisture: {:?}", data);
                    }
                    // Ok(data) = rx_moist.recv() => {
                    //     println!("Temp: {:?}", data);
                    // }
                    else => { break }
                };
            }
        });
    }
}


// struct Lamp {}
// struct Sensor{}
// struct Timer{}
