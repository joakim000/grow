use core::error::Error;

use grow::ops::remote::RemoteControl;
use grow::ops::remote::RcInput;

use crate::hardware::conf::*;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use lego_powered_up::PoweredUp;
use lego_powered_up::HubMutex;
use lego_powered_up::{Hub, HubFilter};
use tokio::task::JoinHandle;
use tokio::sync::broadcast;
use async_trait::async_trait;

use lego_powered_up::consts::{named_port, HubType};
use lego_powered_up::IoDevice;
use lego_powered_up::iodevice::remote::{RcButtonState, RcDevice};
use lego_powered_up::ConnectedHub;

pub struct LpuRemote {
    hub: Option<ConnectedHub>,
    pu: Arc<TokioMutex<PoweredUp>>,
    feedback_task: Option<JoinHandle<()>>,
}
#[async_trait]
impl RemoteControl for LpuRemote {
    async fn init(
        &mut self,
        tx_rc: tokio::sync::broadcast::Sender<RcInput>,
    ) -> Result<(), Box<dyn Error + '_>> {
        let mut lock = self.pu.lock().await;
        println!("Waiting for hub...");
        // let hub = lock.wait_for_hub().await?;
        let hub = lock.wait_for_hub_filter(HubFilter::Kind(HubType::RemoteControl)).await?;
        println!("Connecting to hub...");
        self.hub = Some(ConnectedHub::setup_hub(
            lock.create_hub(&hub).await.expect("Error creating hub"))
        .await
        .expect("Error setting up hub"));

        // Set up RC input
        let rc: IoDevice;
        {
            let lock = self.hub.as_mut().unwrap().mutex.lock().await;
            rc = lock.io_from_port(named_port::A)?;
        }
        let (mut rx_rc, _rc_task) = rc.remote_connect_with_green().await?;
        self.feedback_task = Some(
            self.rc_feedback(tx_rc, rx_rc)
                .await
                .expect("Error initializing feedback task"),
        );

        Ok(())
    }   
}

impl LpuRemote {
    pub fn new(pu:Arc<TokioMutex<PoweredUp>>) -> Self {
        Self {
            pu,
            hub: None,
            feedback_task: None,
        }
    }
    async fn rc_feedback(
        &self,
        tx: broadcast::Sender<RcInput>,
        mut rx: broadcast::Receiver<RcButtonState>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        // let id = self.id;
        let mut red_down = (false, false);
        Ok(tokio::spawn(async move {
            println!("Spawned RC feedback");
            while let Ok(data) = rx.recv().await {
                println!("RC input: {:?} ", data,);
                match data {
                    RcButtonState::Aup => {
                        red_down.0 = false;
                        tx.send(RcInput::LeftUp);
                        tx.send(RcInput::RightUp);
                        tx.send(RcInput::BackUp);
                    }
                    RcButtonState::Aplus => {
                        tx.send(RcInput::Right);
                    }
                    RcButtonState::Ared => {
                        red_down.0 = true;
                        tx.send(RcInput::Back);
                    }
                    RcButtonState::Aminus => {
                        tx.send(RcInput::Left);
                    }
                    RcButtonState::Bup => {
                        red_down.1 = false;
                        tx.send(RcInput::DownUp);
                        tx.send(RcInput::UpUp);
                        tx.send(RcInput::ConfirmUp);
                    }
                    RcButtonState::Bplus => {
                        tx.send(RcInput::Up);
                    }
                    RcButtonState::Bred => {
                        red_down.1 = true;
                        tx.send(RcInput::Confirm);
                    }
                    RcButtonState::Bminus => {
                        tx.send(RcInput::Down);
                    }
                    RcButtonState::Green => {
                        tx.send(RcInput::Mode);
                    }
                    RcButtonState::GreenUp => {
                    }
                }
                if red_down == (true, true) {
                    tx.send(RcInput::Exit);
                }
            }
        }))
    }
}
