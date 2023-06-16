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
    arm: Option<Box<dyn Arm>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status {}

trait Arm {
    fn goto(&self, x: i32, y: i32) -> Result<(), Box<dyn Error>>;
    fn confirm(&self, x: i32, y: i32) -> Result<bool, Box<dyn Error>>;
}
impl Debug for dyn Arm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Arm: {{{}}}", 0)
    }
}

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
                        println!("Arm: {:?}", data);
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