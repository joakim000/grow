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
use lego_powered_up::notifications::HubAction;
use lego_powered_up::ConnectedHub;
use lego_powered_up::IoDevice;

pub struct LpuRemote {
    hub: Option<ConnectedHub>,
    pu: Arc<TokioMutex<PoweredUp>>,
    feedback_task: Option<JoinHandle<()>>,
}
#[async_trait]
impl RemoteControl for LpuRemote {
    async fn init(
        &mut self,
        tx_rc: tokio::sync::mpsc::Sender<RcInput>,
        cancel: CancellationToken,
    ) -> Result<(), Box<dyn Error + '_>> {
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

        // println!("RC hub setup, setting up device...");

        // Set up RC input
        let rc: IoDevice;
        {
            let lock = self
                .hub
                .as_mut()
                .expect("ConnectedHub not found")
                .mutex
                .lock()
                .await;
            rc = lock.io_from_port(named_port::A)?;
            // println!("RC device: {:?}", rc);
        }
        println!("Setting up device and channel");
        let (mut rx_rc, _rc_task) = rc.remote_connect_with_green()?;
        println!("Starting feedback task");

        let hub_clone = self.hub.as_ref().expect("No connected hub").mutex.clone();
        let feedback_task = self
            .rc_feedback(tx_rc, rx_rc, cancel, hub_clone)
            // .await
            .expect("Error initializing feedback task");
      
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
        cancel: CancellationToken,
        hub: HubMutex,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let mut red_down = (false, false);
        Ok(tokio::spawn(async move {
            println!("Spawned RC feedback");
            loop {
                tokio::select! {
                    _ = tx.closed() => {
                        // Managers' RC receiver dropped, shutdown RC hub and exit RC feedback task
                        hub
                        .lock()
                        .await
                        // Note: bluez gets confused if we shutdown the peripheral (lpu hub) rather than disconnect from central side 
                        // .hub_action(HubAction::Shutdown)
                        // .expect("Error on hub shutdown action");
                        .disconnect().await
                        .expect("Error on hub disconnect");;
                        println!("RC hub disconnected");
                        break;
                    }

                    Ok(data) = rx.recv() => {
                        // println!("RC input: {:?} ", data,);
                        match data {
                            RcButtonState::Aup => {
                                red_down.0 = false;
                                let _ = tx.send(RcInput::LeftUp).await;
                                let _ = tx.send(RcInput::RightUp).await;
                                let _ = tx.send(RcInput::BackUp).await;
                            }
                            RcButtonState::Aplus => {
                                let _ = tx.send(RcInput::Right).await;
                            }
                            RcButtonState::Ared => {
                                red_down.0 = true;
                                let _ =  tx.send(RcInput::Back).await;
                            }
                            RcButtonState::Aminus => {
                                let _ = tx.send(RcInput::Left).await;
                            }
                            RcButtonState::Bup => {
                                red_down.1 = false;
                                let _ = tx.send(RcInput::DownUp).await;
                                let _ = tx.send(RcInput::UpUp).await;
                                let _ = tx.send(RcInput::ConfirmUp).await;
                            }
                            RcButtonState::Bplus => {
                                let _ = tx.send(RcInput::Up).await;
                            }
                            RcButtonState::Bred => {
                                red_down.1 = true;
                                let _ = tx.send(RcInput::Confirm).await;
                            }
                            RcButtonState::Bminus => {
                                let _ = tx.send(RcInput::Down).await;
                            }
                            RcButtonState::Green => {
                                let _ = tx.send(RcInput::Mode).await;
                            }
                            RcButtonState::GreenUp => {}
                        }
                        if red_down == (true, true) {
                            let _ = tx.send(RcInput::Exit).await;
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
