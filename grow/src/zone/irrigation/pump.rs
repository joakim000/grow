use core::error::Error;
use tokio::time::sleep as sleep;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::{sync::Arc, time::Duration};
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

#[async_trait]
pub trait Pump {
    fn id(&self) -> u8;
    fn run_for_secs(&self, secs: u16) -> Result<(), Box<dyn Error>>;
    fn stop(&self) -> Result<(), Box<dyn Error>>;
    async fn init(
        &mut self,
        rx_pump: tokio::sync::broadcast::Receiver<(u8, PumpCmd)>
    ) -> Result<(), Box<dyn Error>>;
}


#[derive(Debug, )]
pub struct Runner {
    pub tx_speed: broadcast::Sender<(u8, Option<f32>)>,
    pub tx_pumpcmd: broadcast::Sender<(u8, PumpCmd)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx_speed: broadcast::channel(1).0,
            tx_pumpcmd: broadcast::channel(2).0,
            task: tokio::spawn(async move {}),
        }
    }
    pub fn cmd_channel(
        &self,
    ) -> broadcast::Receiver<(u8, PumpCmd)> {
        self.tx_pumpcmd.subscribe()
    }
    pub fn feedback_channel(
        &self,
    ) -> broadcast::Sender<(u8, Option<f32>)> {
        self.tx_speed.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx_speed = self.tx_speed.subscribe();
        let tx_pumpcmd = self.tx_pumpcmd.clone();
        self.task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(data) = rx_speed.recv() => {
                        println!("Pump speed: {:?}", data);
                    }
                    // Ok(data) = rx_2.recv() => {
                    //     println!("Secondary:"" {:?}", data);
                    // }
                    else => { break }
                };
            }
        });
        // Cmd test
        sleep(Duration::from_secs(10));
        tx_pumpcmd.send( (1, PumpCmd::RunForSec(5)) );

    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum PumpCmd {
    RunForSec(u16),
    Stop
}