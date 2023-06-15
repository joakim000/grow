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
       
       };
    Zone::Pump  {
        id,
        settings,
        status: Arc::new(Mutex::new(status)),
        interface: Interface {
        },
        runner: Runner::new(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    pub run_for_secs: u16,
    pub rest_secs: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status {}

#[derive(Debug, )]
pub struct Interface {
    // arm: Option<Box<dyn Arm>>,
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