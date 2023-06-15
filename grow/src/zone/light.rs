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
    fn id(&self) -> u8;
    fn on(&self) -> Result<(), Box<dyn Error>>;
    fn off(&self) -> Result<(), Box<dyn Error>>;
    // fn init(&self) -> Result<(), Box<dyn Error>>;
    fn init(
        &mut self,
        rx_lamp: tokio::sync::broadcast::Receiver<(u8, bool)>
    ) -> Result<(), Box<dyn Error>>;
        
}
impl Debug for dyn Lamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Lamp: {{{}}}", 0)
    }
}


pub trait Lightmeter {
    fn id(&self) -> u8;
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
    pub tx_lightmeter: broadcast::Sender<(u8, Option<f32>)>,
    pub tx_lamp: broadcast::Sender<(u8, bool)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx_lightmeter: broadcast::channel(1).0,
            tx_lamp: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn lightmeter_channel(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx_lightmeter.clone()
    }
    pub fn lamp_channel(
        &self,
    ) -> broadcast::Receiver<(u8, bool)> {
        self.tx_lamp.subscribe()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx = self.tx_lightmeter.subscribe();
        let tx = self.tx_lamp.clone();
        self.task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        println!("Light level: {:?}", data);
                        match data {
                            (id, Some(lvl)) => {
                                if lvl < 20f32 {
                                    tx.send( (1, true) );
                                } else {
                                    tx.send( (1, false) );
                                }
                            }
                            (_, None) => ()
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

// struct Lamp {}
// struct Sensor{}
// struct Timer{}
