use core::error::Error;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
use core::fmt::Debug;
use super::Zone;
use crate::ops::display::{Indicator, DisplayStatus};


pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
        lamp_on: None,
        light_level: None,
        disp: DisplayStatus {
                indicator: Default::default(),
                msg: None,
            }
       };
       let status_mutex = Arc::new(RwLock::new(status));
    Zone::Light {
        id,
        settings,
        status: status_mutex,
        interface: Interface {
            lamp: None,
            lightmeter: None,
        },
        runner: Runner::new(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    lamp_on: Option<bool>,
    light_level: Option<f32>,
    pub disp: DisplayStatus,
}

#[derive( Debug, )]
pub struct Interface {
    pub lamp: Option<Box<dyn Lamp>>,
    pub lightmeter: Option<Box<dyn Lightmeter>>,
}


pub enum LampState {
    On,
    Off,
}

// pub struct Foo {
//     stuff: Vec<u8>
// }

pub trait Lamp : Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        rx_lamp: tokio::sync::broadcast::Receiver<(u8, bool)>
    ) -> Result<(), Box<dyn Error>>;
    fn set_state(&self, state: LampState) -> Result<(), Box<dyn Error + '_>>;

}
impl Debug for dyn Lamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Lamp: {{{}}}", 0)
    }
}


pub trait Lightmeter : Send {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_light: tokio::sync::broadcast::Sender<(u8, Option<f32>)>
    ) -> Result<(), Box<dyn Error>>;   
    fn read(&self) -> Result<(f32), Box<dyn Error  + '_>>;
    
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

    pub fn lightmeter_feedback_sender(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx_lightmeter.clone()
    }
    pub fn lamp_cmd_receiver(
        &self,
    ) -> broadcast::Receiver<(u8, bool)> {
        self.tx_lamp.subscribe()
    }
    pub fn lamp_cmd_sender(
        &self,
    ) -> broadcast::Sender<(u8, bool)> {
        self.tx_lamp.clone()
    }

    // This could handle scheudlineg and to that timed lightchecks, only waking manager if warning
    // Keep lamp channel for runner lamp control
    pub fn run(&mut self, settings: Settings) {
        let mut rx = self.tx_lightmeter.subscribe();
        let tx = self.tx_lamp.clone();
        self.task = tokio::spawn(async move {
            println!("Spawned light runner");
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        println!("\tLight level: {:?}", data);
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
