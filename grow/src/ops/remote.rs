use alloc::collections::BTreeMap;
use async_trait::async_trait;
use core::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast;
// use tokio::sync::Mutex;
use super::Zone;
use crate::ops::display::{DisplayStatus, Indicator};
use core::fmt::Debug;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, PartialEq)]
pub enum RcInput {
    Down,
    DownUp,
    Left,
    LeftUp,
    Right,
    RightUp,
    Up,
    UpUp,

    Back, // Back / Cancel
    BackUp,
    Confirm, // Forward / Confirm
    ConfirmUp,
    Mode, // Mode / Menu
    ModeUp,

    Exit,
}

#[derive(Debug)]
pub enum RcModeExit {
    Confirm,
    Cancel,
    SwitchFromOpsMode,
    SwitchFromPositionMode,
    ElseExit,
}

#[derive(Debug)]
pub struct Rc {
    interface: Option<Box<dyn RemoteControl>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    // lamp_on: Option<bool>,
    // light_level: Option<f32>,
    pub disp: DisplayStatus,
}

#[async_trait]
pub trait RemoteControl: Send {
    async fn init(
        &mut self,
        // tx_rc: tokio::sync::broadcast::Sender<RcInput>,
        tx_rc: tokio::sync::mpsc::Sender<RcInput>,
        cancel: CancellationToken,
    ) -> Result<(), Box<dyn Error + '_>>;
    // fn read(&self) -> Result<(f32), Box<dyn Error  + '_>>;
}
impl Debug for dyn RemoteControl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Remote control: {{{}}}", 0)
    }
}

#[derive(Debug)]
pub struct Runner {
    pub tx_from_rc: broadcast::Sender<RcInput>,
    pub tx_to_manager: broadcast::Sender<RcInput>,
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

    pub fn channel(&self) -> broadcast::Sender<RcInput> {
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
