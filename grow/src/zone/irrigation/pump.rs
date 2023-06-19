use core::error::Error;
use tokio::time::sleep as sleep;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use core::fmt::Debug;
use super::Zone;
use crate::ops::display::{Indicator, DisplayStatus};

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
       
       };
    Zone::Pump  {
        id,
        settings,
        status: Arc::new(Mutex::new(status)),
        interface: Interface {
            pump: None,
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
    pub pump: Option<Box<dyn Pump>>,
}

#[async_trait]
pub trait Pump : Send + Sync {
    fn id(&self) -> u8;
    async fn init(
        &mut self,
        rx_pump: tokio::sync::broadcast::Receiver<(u8, PumpCmd)>,
        tx_pump: tokio::sync::broadcast::Sender<(u8, (i8, i32) )>
    ) -> Result<(), Box<dyn Error>>;
    async fn run_for_secs(&self, secs: u16) -> Result<(), Box<dyn Error>>;
    async fn stop(&self) -> Result<(), Box<dyn Error>>;
   
}
impl Debug for dyn Pump {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Pump {{{}}}", self.id())
    }
}


#[derive(Debug, )]
pub struct Runner {
    pub tx_feedback: broadcast::Sender<(u8, (i8, i32) )>,
    pub tx_pumpcmd: broadcast::Sender<(u8, PumpCmd)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx_feedback: broadcast::channel(8).0,
            tx_pumpcmd: broadcast::channel(8).0,
            task: tokio::spawn(async move {}),
        }
    }
    pub fn cmd_channel(
        &self,
    ) -> broadcast::Receiver<(u8, PumpCmd)> {
        self.tx_pumpcmd.subscribe()
    }
    pub fn channel(
        &self,
    ) -> broadcast::Sender<(u8, (i8, i32) )> {
        self.tx_feedback.clone()
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx_feedback = self.tx_feedback.subscribe();
        let tx_pumpcmd = self.tx_pumpcmd.clone();
        self.task = tokio::spawn(async move {
            println!("Spawned pump runner");
            loop {
                tokio::select! {
                    Ok(data) = rx_feedback.recv() => {
                        println!("\tPump speed: {:?}", data);
                    }
                    // Ok(data) = rx_2.recv() => {
                    //     println!("Secondary:"" {:?}", data);
                    // }
                    else => { break }
                };
            }
        });
        // Cmd test
        // sleep(Duration::from_secs(15)).await;
        // println!("Pump run from runner");
        // tx_pumpcmd.send( (1, PumpCmd::RunForSec(2)) );

    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Ord, PartialOrd, Hash)]
pub enum PumpCmd {
    RunForSec(u16),
    Stop
}