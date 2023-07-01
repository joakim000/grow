use alloc::collections::BTreeMap;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use core::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast;
// use tokio::sync::Mutex;
use super::Zone;
use crate::ops::display::{DisplayStatus, Indicator};
use core::fmt::Debug;
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
pub enum ButtonInput {
    One,
    Two,
}

#[derive(Debug)]
pub struct Buttons {
    interface: Option<Box<dyn ButtonPanel>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    // lamp_on: Option<bool>,
    // light_level: Option<f32>,
    pub disp: DisplayStatus,
}
// #[async_trait]
pub trait ButtonPanel: Send {
    // fn id(&self) -> u8;
    fn init(
        &mut self,
        tx_rc: tokio::sync::broadcast::Sender<ButtonInput>,
    ) -> Result<(), Box<dyn Error>>;
    // fn read(&self) -> Result<(f32), Box<dyn Error  + '_>>;
}
impl Debug for dyn ButtonPanel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Buttons: {{{}}}", 0)
    }
}

#[derive(Debug)]
pub struct Runner {
    pub tx_from_rc: broadcast::Sender<ButtonInput>,
    pub tx_to_manager: broadcast::Sender<ButtonInput>,
    pub task: tokio::task::JoinHandle<()>,
}
impl Runner {
    pub fn new() -> Self {
        Self {
            tx_from_rc: broadcast::channel(1).0,
            tx_to_manager: broadcast::channel(1).0,
            task: tokio::spawn(async move {}),
        }
    }

    pub fn channel(&self) -> broadcast::Sender<ButtonInput> {
        self.tx_from_rc.clone()
    }

    pub fn run(&mut self) {
        let mut rx = self.tx_from_rc.subscribe();
        let tx = self.tx_to_manager.clone(); // Få från manager i init istället
        self.task = tokio::spawn(async move {
            println!("Spawned remote runner");
            loop {
                tokio::select! {
                    Ok(data) = rx.recv() => {
                        println!("\tRC input: {:?}", data);
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
