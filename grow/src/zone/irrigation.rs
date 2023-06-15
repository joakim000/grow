use super::Zone;
use core::error::Error;
use tokio::sync::broadcast;
use core::fmt::Debug;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
        moisture_level: None,
        indicator: None,
        msg: None,
       };
    Zone::Irrigation  {
        id,
        settings,
        status: Arc::new(Mutex::new(status)),
        interface: Interface {
            moist: None,
        },
        runner: Runner::new(),
    }
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    pub moisture_limit_water: i16,
    pub moisture_limit_low_warning: i16,
    pub moisture_limit_high_warning: i16,
    pub pump_id: u8,
    pub position: Option<super::arm::Move>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub moisture_level: Option<f32>,
    pub indicator: Option<crate::Indicator>,
    pub msg: Option<String>,
}

#[async_trait]
pub trait MoistureSensor {
    fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_moist: tokio::sync::broadcast::Sender<(u8, Option<f32>)>
    ) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug,  )] 
pub struct Interface {
    pub moist: Option<Box<dyn MoistureSensor>>,
}
impl Interface {
    // pub fn set_moist(&mut self, moist: Box<dyn MoistureSensor>) -> () {
    //     self.moist = Some(moist);
    // }
}
// impl Debug for Interface {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         write!(f, "Interface: ")
//     }
// }
impl Debug for dyn MoistureSensor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MoistureSensor {{{}}}", self.id())
    }
}


pub mod pump;
pub mod tank;
pub mod arm;

#[derive(Debug, )]
pub struct Runner {
    pub tx: broadcast::Sender<(u8, Option<f32>)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn channel(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx = self.tx.subscribe();
        self.task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        println!("Moisture: {:?}", data);
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
