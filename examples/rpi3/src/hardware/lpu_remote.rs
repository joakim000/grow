use core::error::Error;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use lego_powered_up::consts::{named_port, HubType};
use lego_powered_up::iodevice::remote::{RcButtonState, RcDevice};
use lego_powered_up::HubMutex;
use lego_powered_up::{HubFilter};

use lego_powered_up::{ConnectedHub, IoDevice, PoweredUp};

use grow::ops::remote::RcInput;
use grow::ops::remote::RemoteControl;

pub struct LpuRemote {
    hub: Option<ConnectedHub>,
    pu: Arc<TokioMutex<PoweredUp>>,
    _feedback_task: Option<JoinHandle<()>>,
    position_finder_cancel: CancellationToken,
}
#[async_trait]
impl RemoteControl for LpuRemote {
    async fn init(
        &mut self,
        tx_rc: tokio::sync::mpsc::Sender<RcInput>,
        position_finder_cancel: CancellationToken,
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
        let (rx_rc, _rc_task) = rc.remote_connect_with_green().await?;
        println!("Starting feedback task");

        let hub_clone =
            self.hub.as_ref().expect("No connected hub").mutex.clone();
        let _feedback_task = self
            .rc_feedback(tx_rc, rx_rc, position_finder_cancel, hub_clone)
            // .await
            .expect("Error initializing feedback task");

        Ok(())
    }
}

// #[async_trait]
// impl zone::auxiliary::AuxDevice for LpuRemote {
//     fn id(&self) -> u8 {
//         self.id
//     }
//     async fn init(
//         &mut self,
//         tx_status: tokio::sync::broadcast::Sender<(u8, DisplayStatus)>,
//     ) -> Result<(), Box<dyn Error>> {
//         self.feedback_task = Some(
//             self.hub_feedback(tx_status)
//                 .await
//                 .expect("Error initializing feedback task"),
//         );
//         Ok(())
//     }
//     fn read(&self) -> Result<String, Box<dyn Error>> {
//         Ok(String::from("Placeholder"))
//     }
// }

impl LpuRemote {
    pub fn new(
        pu: Arc<TokioMutex<PoweredUp>>,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            pu,
            hub: None,
            _feedback_task: None,
            position_finder_cancel: cancel,
        }
    }
    fn rc_feedback(
        &self,
        tx: mpsc::Sender<RcInput>,
        mut rx: broadcast::Receiver<RcButtonState>,
        _position_finder_cancel: CancellationToken,
        hub: HubMutex,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let mut red_down = (false, false);
        let main_cancel = self.position_finder_cancel.clone();
        Ok(tokio::spawn(async move {
            // println!("Spawned RC feedback");
            loop {
                tokio::select! {
                    _ = main_cancel.cancelled() => {
                        match hub.lock().await.shutdown().await {
                            Ok(_) => { println!("LPU remote off"); }
                            Err(e) => { println!("LPU remote shutdown error: {:?}", e); }
                        }
                        match hub.lock().await.disconnect().await {
                            Ok(_) => { println!("LPU remote disconnected"); }
                            Err(e) => { println!("LPU remote disconnect error: {:?}", e); }
                        }
                        break;
                    }
                    _ = tx.closed() => {
                        // Managers' RC receiver dropped, shutdown RC hub and exit RC feedback task
                        // Note: bluez gets confused if we shutdown the peripheral (lpu hub) rather than disconnect from central side
                        match hub.lock().await.shutdown().await {
                            Ok(_) => { println!("LPU remote off"); }
                            Err(e) => { println!("LPU remote shutdown error: {:?}", e); }
                        }
                        match hub.lock().await.disconnect().await {
                            Ok(_) => { println!("LPU remote disconnected"); }
                            Err(e) => { println!("LPU remote disconnect error: {:?}", e); }
                        }
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

    //  async fn hub_feedback(
    //         &self,
    //         tx: broadcast::Sender<(u8, DisplayStatus)>,
    //     ) -> Result<JoinHandle<()>, Box<dyn Error>> {
    //         let id = self.id;
    //         let hub = self.hub.clone();
    //         let mut rx_hub: broadcast::Receiver<HubNotification>;
    //         {
    //             //     let mut lock = tokio::task::block_in_place(move || {
    //             //         hub.blocking_lock_owned()
    //             //     });
    //             let mut lock = hub.lock().await;
    //             rx_hub = lock
    //                 .channels()
    //                 .hubnotification_sender
    //                 .as_ref()
    //                 .unwrap()
    //                 .subscribe();
    //             // These will send current status when enabling updates
    //             let _ = lock.hub_props(
    //                 HubPropertyRef::Button,
    //                 HubPropertyOperation::EnableUpdatesDownstream,
    //             )?;
    //             let _ = lock.hub_props(
    //                 HubPropertyRef::BatteryType,
    //                 HubPropertyOperation::EnableUpdatesDownstream,
    //             )?;
    //             let _ = lock.hub_props(
    //                 HubPropertyRef::Rssi,
    //                 HubPropertyOperation::EnableUpdatesDownstream,
    //             )?;
    //             let _ = lock.hub_props(
    //                 HubPropertyRef::BatteryVoltage,
    //                 HubPropertyOperation::EnableUpdatesDownstream,
    //             )?;

    //             // These will not send current status when enabling updates; request single update first
    //             let _ = lock.hub_alerts(AlertType::LowVoltage, AlertOperation::RequestUpdate)?;
    //             let _ = lock.hub_alerts(AlertType::LowVoltage, AlertOperation::EnableUpdates)?;

    //             let _ = lock.hub_alerts(AlertType::HighCurrent, AlertOperation::RequestUpdate)?;
    //             let _ = lock.hub_alerts(AlertType::HighCurrent, AlertOperation::EnableUpdates)?;

    //             let _ = lock.hub_alerts(AlertType::LowSignalStrength, AlertOperation::RequestUpdate)?;
    //             let _ = lock.hub_alerts(AlertType::LowSignalStrength, AlertOperation::EnableUpdates)?;

    //             let _ =
    //                 lock.hub_alerts(AlertType::OverPowerCondition, AlertOperation::RequestUpdate)?;
    //             let _ =
    //                 lock.hub_alerts(AlertType::OverPowerCondition, AlertOperation::EnableUpdates)?;
    //         }

    //         let cancel_clone = self.main_cancel.clone();
    //         Ok(tokio::spawn(async move {
    //             println!("Spawned hub feedback");
    //             loop {
    //                 tokio::select! {
    //                     Ok(data) = rx_hub.recv() => {
    //                         // println!("Hub {:?} sent: {:?}", id, data,);
    //                         match data {
    //                             HubNotification {
    //                                 hub_alert:
    //                                     Some(HubAlert {
    //                                         alert_type,
    //                                         payload,
    //                                         ..
    //                                     }),
    //                                     ..
    //                             } if payload == AlertPayload::Alert => {
    //                                 tx.send(( id, DisplayStatus::new(Indicator::Red, Some(alert_type.to_string())) ));
    //                             },

    //                             HubNotification {
    //                                 hub_property:
    //                                     Some(HubProperty {
    //                                         property,
    //                                         operation,
    //                                         ..
    //                                     }),
    //                                     ..
    //                             } if operation == HubPropertyOperation::UpdateUpstream => {
    //                                 match property {
    //                                     HubPropertyValue::Button(state) =>  {

    //                                     }
    //                                     HubPropertyValue::BatteryType(t) => {

    //                                     }
    //                                     HubPropertyValue::BatteryVoltage(v) => {
    //                                         if v < 15 {
    //                                             tx.send(( id, DisplayStatus::new(Indicator::Red, Some( format!("Battery: {}%", v) )) ));
    //                                         }
    //                                         else if v < 30 {
    //                                             tx.send(( id, DisplayStatus::new(Indicator::Yellow, Some( format!("Battery: {}%", v) )) ));
    //                                         }
    //                                         else {
    //                                             tx.send(( id, DisplayStatus::new(Indicator::Green, Some( format!("Battery: {}%", v) )) ));
    //                                         }

    //                                     }
    //                                     HubPropertyValue::Rssi(signal) => {

    //                                     }
    //                                     _ => {}
    //                                 }

    //                             }
    //                             _ => {}
    //                         }
    //                     }
    //                     _ = cancel_clone.cancelled() => {
    //                         match hub.lock().await.shutdown().await {
    //                             Ok(_) => { println!("LPU hub off"); }
    //                             Err(e) => { println!("LPU hub shutdown error: {:?}", e); }
    //                         }
    //                         match hub.lock().await.disconnect().await {
    //                             Ok(_) => { println!("LPU hub disconnected"); }
    //                             Err(e) => { println!("LPU hub disconnect error: {:?}", e); }
    //                         }
    //                         break;
    //                     }
    //                     else => { break }
    //                 };
    //             }
    //         }))
    //     }
}
