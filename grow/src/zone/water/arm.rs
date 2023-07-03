
use async_trait::async_trait;
use core::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast;

// use tokio::sync::Mutex;
use core::fmt::Debug;
use parking_lot::RwLock;

use time::OffsetDateTime;
use serde::{Serialize, Deserialize};

use super::Zone;
use super::*;
use crate::ops::display::{DisplayStatus};


// use crate::TIME_OFFSET;
pub type ControlFeedbackRx = broadcast::Receiver<ArmState>;
pub type ControlFeedbackTx = broadcast::Sender<ArmState>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArmCmd {
    Confirm, // Not implemented
    Stop,
    StopX,
    StopY,
    StartX { speed: i8 },
    StartY { speed: i8 },
    Goto { x: i32, y: i32 },
    GotoX { x: i32 },
    GotoY { y: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxisState {
    Idle,
    BusyQueueEmpty,
    BusyQueuedCmds(u16),
    BusyQueueFull,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmState {
    Idle,
    Busy,
}

pub fn new(id: u8, settings: Settings) -> super::Zone {
    let status = Status {
        pos_x: 0,
        pos_y: 0,
        pos_z: 0,
        disp: DisplayStatus {
            indicator: Default::default(),
            msg: None,
            changed: OffsetDateTime::UNIX_EPOCH,
        },
    };
    let status_mutex = Arc::new(RwLock::new(status));
    Zone::Arm {
        id,
        settings,
        runner: Runner::new(id, status_mutex.clone()),
        status: status_mutex,
        interface: Interface { arm: None },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub arm_id: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
#[derive(Debug)]
pub struct Interface {
    pub arm: Option<Box<dyn Arm>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    // position: (i32, i32, i32),
    pub pos_x: i32,
    pub pos_y: i32,
    pub pos_z: i32,
    pub disp: DisplayStatus,
}

#[async_trait]
pub trait Arm: Send + Sync {
    fn id(&self) -> u8;
    async fn init(
        &mut self,
        tx_axis_x: tokio::sync::broadcast::Sender<(i8, i32)>,
        tx_axis_y: tokio::sync::broadcast::Sender<(i8, i32)>,
        tx_axis_z: tokio::sync::broadcast::Sender<(i8, i32)>,
        tx_control: ControlFeedbackTx,
        rx_cmd: tokio::sync::broadcast::Receiver<ArmCmd>,
    ) -> Result<(), Box<dyn Error>>;
    fn goto(&self, x: i32, y: i32, z: i32) -> Result<(), Box<dyn Error>>;
    fn goto_x(&self, x: i32) -> Result<(), Box<dyn Error>>;
    fn goto_y(&self, y: i32) -> Result<(), Box<dyn Error>>;
    // async fn confirm(&self, x: i32, y: i32, z: i32, delta: u32) -> Result<bool, Box<dyn Error>>;
    // fn confirm(&self, x: i32, y: i32, z:i32, acceptable_delta: u32) -> Result<( bool, ( i32, i32, i32 ) ), Box<dyn Error>>;
    fn stop(&self) -> Result<(), Box<dyn Error>>;
    fn start_x(&self, speed: i8) -> Result<(), Box<dyn Error>>;
    fn stop_x(&self) -> Result<(), Box<dyn Error>>;
    fn start_y(&self, speed: i8) -> Result<(), Box<dyn Error>>;
    fn stop_y(&self) -> Result<(), Box<dyn Error>>;
    async fn update_pos(&self) -> Result<(), Box<dyn Error>>;
    fn position(&self) -> Result<(i32, i32, i32), Box<dyn Error>>;
    async fn calibrate(&self) -> Result<(i32, i32, i32), Box<dyn Error>>;
    async fn calibrate_with_range(&self) -> Result<(), Box<dyn Error>>;
}

impl Debug for dyn Arm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Arm: {{{}}}", self.id())
    }
}

#[derive(Debug)]
pub struct Runner {
    _id: u8,
    pub tx_axis_x: broadcast::Sender<(i8, i32)>,
    // axis_x: (watch::Sender<(i8, i32)>, watch::Receiver<(i8, i32)> ),
    pub tx_axis_y: broadcast::Sender<(i8, i32)>,
    pub tx_axis_z: broadcast::Sender<(i8, i32)>,
    pub tx_cmd: broadcast::Sender<ArmCmd>,
    pub tx_control: ControlFeedbackTx,
    pub task: tokio::task::JoinHandle<()>,
    status: Arc<RwLock<Status>>,
}
impl Runner {
    pub fn new(_id: u8, status: Arc<RwLock<Status>>) -> Self {
        Self {
            _id,
            status,
            tx_axis_x: broadcast::channel(64).0,
            // axis_x: tokio::sync::watch::channel((0, 0)),
            tx_axis_y: broadcast::channel(64).0,
            tx_axis_z: broadcast::channel(64).0,
            tx_control: broadcast::channel(64).0,
            tx_cmd: broadcast::channel(8).0,
            task: tokio::spawn(async move {}),
        }
    }
    pub fn cmd_sender(&self) -> broadcast::Sender<ArmCmd> {
        self.tx_cmd.clone()
    }
    pub fn cmd_receiver(&self) -> broadcast::Receiver<ArmCmd> {
        self.tx_cmd.subscribe()
    }
    pub fn control_feedback_sender(&self) -> ControlFeedbackTx {
        self.tx_control.clone()
    }
    pub fn pos_feedback_sender(
        &self,
    ) -> (
        broadcast::Sender<(i8, i32)>,
        broadcast::Sender<(i8, i32)>,
        broadcast::Sender<(i8, i32)>,
    ) {
        (
            self.tx_axis_x.clone(),
            self.tx_axis_y.clone(),
            self.tx_axis_z.clone(),
        )
    }

    pub fn run(&mut self, _settings: Settings) {
        let mut rx_axis_x = self.tx_axis_x.subscribe();
        let mut rx_axis_y = self.tx_axis_y.subscribe();
        let mut rx_axis_z = self.tx_axis_z.subscribe();
        let mut rx_control = self.tx_control.subscribe();
        let status = self.status.clone();
        self.task = tokio::spawn(async move {
            println!("Spawned arm runner");
            loop {
                tokio::select! {
                    Ok(data) = rx_axis_x.recv() => {
                        status.write().pos_x = data.1;
                        // println!("\tX:{:?} ", data);
                    }
                    Ok(data) = rx_axis_y.recv() => {
                        status.write().pos_y = data.1;
                        // println!("\tY:{:?} ", data);
                    }
                    Ok(data) = rx_axis_z.recv() => {
                        status.write().pos_z = data.1;
                    }
                    Ok(_data) = rx_control.recv() => {
                        // println!("\tControl:{:?} ", data);
                    }
                    else => { break }
                };
            }
        });
    }
}
