use core::error::Error;

use grow::ops::remote::RcInput;
use grow::ops::remote::RemoteControl;

use crate::hardware::conf::*;
use async_trait::async_trait;
use lego_powered_up::HubMutex;
use lego_powered_up::PoweredUp;
use lego_powered_up::{Hub, HubFilter};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use lego_powered_up::consts::{named_port, HubType};
use lego_powered_up::iodevice::remote::{RcButtonState, RcDevice};
use lego_powered_up::ConnectedHub;
use lego_powered_up::IoDevice;
use lego_powered_up::notifications::HubAction;

pub struct LpuRemote {
    hub: Option<ConnectedHub>,
    pu: Arc<TokioMutex<PoweredUp>>,
    feedback_task: Option<JoinHandle<()>>,
}
#[async_trait]
impl RemoteControl for LpuRemote {
    async fn init(
        &mut self,
        // tx_rc: tokio::sync::broadcast::Sender<RcInput>,
        tx_rc: tokio::sync::mpsc::Sender<RcInput>,
    ) -> Result<(), Box<dyn Error + '_>> {              // TODO Return shutdown-task?
        let mut lock = self.pu.lock().await;
        println!("Waiting for hub...");
        // let hub = lock.wait_for_hub().await?;
        let discovered_hub = lock
            .wait_for_hub_filter(HubFilter::Kind(HubType::RemoteControl))
            .await?;
        println!("Connecting to hub...");
        let created_hub = lock.create_hub(&discovered_hub).await?;
        self.hub = Some(ConnectedHub::setup_hub(created_hub).await?);

        // self.hub = Some(
        //     ConnectedHub::setup_hub(lock.create_hub(&discovered_hub).await.expect("Error creating hub")) // thread 'tokio-runtime-worker' panicked at 'Error creating hub: BluetoothError(Other(DbusError(D-Bus error: Software caused connection abort (org.bluez.Error.Failed))))', src/hardware/lpu_remote.rs:38:41
        //         .await
        //         .expect("Error setting up hub"),
        // );

        println!("RC hub setup, setting up device...");

        // Set up RC input
        let rc: IoDevice;
        {
            println!("Req rchub lock");
            let lock = self.hub.as_mut().expect("ConnectedHub not found").mutex.lock().await;
            println!("Got rchub lock");
            rc = lock.io_from_port(named_port::A)?;
            // println!("RC device: {:?}", rc);
        }
        println!("Setting up device and channel");
        let (mut rx_rc, _rc_task) = rc.remote_connect_with_green()?;
        println!("Starting feedback task");
        // self.feedback_task = Some(
        let feedback_task =
            self.rc_feedback(tx_rc, rx_rc)
                // .await
                .expect("Error initializing feedback task");
        
        let hub_clone = self.hub.as_ref().expect("No connected hub").mutex.clone();
        let shutdown_task = tokio::spawn(async move {
            feedback_task.await;
            println!("RC feedback task exited, shutting down RC hub");
            hub_clone.lock().await.hub_action(HubAction::Shutdown).expect("Error on hub shutdown action");
            println!("RC hub shutdown action sent");
        });

        println!("Returning from remote.init()");
        Ok(())
    }
}

impl LpuRemote {
    pub fn new(pu: Arc<TokioMutex<PoweredUp>>, cancel: CancellationToken) -> Self {
        Self {
            pu,
            hub: None,
            feedback_task: None,
        }
    }
    fn rc_feedback(
        &self,
        tx: mpsc::Sender<RcInput>,
        mut rx: broadcast::Receiver<RcButtonState>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        // let id = self.id;
        let mut red_down = (false, false);
        Ok(tokio::spawn(async move {
            println!("Spawned RC feedback");
            loop {
                tokio::select! {
                    _ = tx.closed() => { 
                        println!("Managers' RC receiver dropped, exit RC feedback task");
                        break; 
                    }

                    Ok(data) = rx.recv() => {
                        // println!("RC input: {:?} ", data,);
                        match data {
                            RcButtonState::Aup => {
                                red_down.0 = false;
                                tx.send(RcInput::LeftUp).await;
                                let res = tx.send(RcInput::RightUp).await;
                                tx.send(RcInput::BackUp).await;
                                // println!("Send RcInput::Aup: {:?}", res);
                            }
                            RcButtonState::Aplus => {
                                tx.send(RcInput::Right).await;
                            }
                            RcButtonState::Ared => {
                                red_down.0 = true;
                                let res = tx.send(RcInput::Back).await;
                                // println!("Send RcInput::Back: {:?}", res);
                            }
                            RcButtonState::Aminus => {
                                tx.send(RcInput::Left).await;
                            }
                            RcButtonState::Bup => {
                                red_down.1 = false;
                                tx.send(RcInput::DownUp).await;
                                tx.send(RcInput::UpUp).await;
                                tx.send(RcInput::ConfirmUp).await;
                            }
                            RcButtonState::Bplus => {
                                tx.send(RcInput::Up).await;
                            }
                            RcButtonState::Bred => {
                                red_down.1 = true;
                                tx.send(RcInput::Confirm).await;
                            }
                            RcButtonState::Bminus => {
                                tx.send(RcInput::Down).await;
                            }
                            RcButtonState::Green => {
                                tx.send(RcInput::Mode).await;
                            }
                            RcButtonState::GreenUp => {}
                        }
                        if red_down == (true, true) {
                            tx.send(RcInput::Exit).await;
                        }
                    }
                    else => { break; }
                };
            }
        }))
    }

}




// loop {
//     tokio::select! {
//         _ = tx.closed() => { 
//             println!("Managers' RC receiver dropped, exit RC feedback task");
//             break; 
//         }

//         Ok(data) = rx.recv() => {
         
//         }
//         else => { break; }
//     };
// }