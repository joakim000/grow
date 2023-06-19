use core::error::Error;
use alloc::collections::BTreeMap;
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
use core::fmt::Debug;
use super::Zone;
use crate::ops::display::{Indicator, DisplayStatus};

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status { 
       
       };
    Zone::Arm  {
        id,
        settings,
        status: Arc::new(Mutex::new(status)),
        interface: Interface {
            arm: None,
        },
        runner: Runner::new(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move {
    pub arm_id: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
#[derive(Debug, )]
pub struct Interface {
    pub arm: Option<Box<dyn Arm>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status {}

#[async_trait]
pub trait Arm : Send {
    fn id(&self) -> u8;
    async fn init(
        &mut self,
        tx_axis_x: tokio::sync::broadcast::Sender<((i8, i32))>,
        tx_axis_y: tokio::sync::broadcast::Sender<((i8, i32))>,
        tx_axis_z: tokio::sync::broadcast::Sender<((i8, i32))>,
    ) -> Result<(), Box<dyn Error>>;
    async fn goto(&self, x: i32, y: i32) -> Result<(), Box<dyn Error>>;
    async fn goto_x(&self, x: i32) -> Result<(), Box<dyn Error>>;
    async fn goto_y(&self, y: i32) -> Result<(), Box<dyn Error>>;
    async fn confirm(&self, x: i32, y: i32) -> Result<bool, Box<dyn Error>>;
    async fn stop(&self) -> Result<(), Box<dyn Error>>; 
}
impl Debug for dyn Arm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Arm: {{{}}}", self.id())
    }
}

#[derive(Debug, )]
pub struct Runner {
    pub tx_axis_x: broadcast::Sender<(i8, i32)>,
    pub tx_axis_y: broadcast::Sender<(i8, i32)>,
    pub tx_axis_z: broadcast::Sender<(i8, i32)>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx_axis_x: broadcast::channel(1).0,
            tx_axis_y: broadcast::channel(1).0,
            tx_axis_z: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn channel(
        &self,
    ) -> ( broadcast::Sender<(i8, i32)>,
            broadcast::Sender<(i8, i32)>,
            broadcast::Sender<(i8, i32)> ) {
        ( self.tx_axis_x.clone(), self.tx_axis_y.clone(), self.tx_axis_z.clone() )
    }

    pub fn run(&mut self, settings: Settings) {
        let mut rx_axis_x = self.tx_axis_x.subscribe();
        let mut rx_axis_y = self.tx_axis_y.subscribe();
        let mut rx_axis_z = self.tx_axis_z.subscribe();
        self.task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(data) = rx_axis_x.recv() => {
                        println!("\tArm X: {:?}", data);
                    }
                    Ok(data) = rx_axis_y.recv() => {
                        println!("\tArm Y: {:?}", data);
                    }
                    Ok(data) = rx_axis_z.recv() => {
                        println!("\tArm Z: {:?}", data);
                    }
                    else => { break }
                };
            }
        });
    }
}