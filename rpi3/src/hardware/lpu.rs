use super::conf::*;

use lego_powered_up::iodevice::visionsensor::VisionSensor;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use async_trait::async_trait;

use core::error::Error;
use core::time::Duration;
use tokio::time::sleep as sleep;

use lego_powered_up::consts::named_port;
use lego_powered_up::consts::LEGO_COLORS;
use lego_powered_up::error::{Error as LpuError, OptionContext, Result as LpuResult};
use lego_powered_up::iodevice::modes;
use lego_powered_up::iodevice::remote::{RcButtonState, RcDevice};
use lego_powered_up::iodevice::{hubled::*, motor::*, sensor::*};
use lego_powered_up::notifications::Power;
use lego_powered_up::{ConnectedHub, IoDevice, IoTypeId, PoweredUp};
use lego_powered_up::{Hub, HubFilter};
use lego_powered_up::HubMutex;
use lego_powered_up::iodevice::visionsensor::DetectedColor;

use grow::zone;
use grow::zone::tank::TankLevel;
use grow::zone::pump::PumpCmd;

// #[tokio::main]
pub async fn init() -> Result<HubMutex, Box<dyn Error>> {
    // === Single hub ===
    let hub = lego_powered_up::setup::single_hub().await?;

    // let vsensor: IoDevice;
    // let pump: IoDevice;
    // let rot: IoDevice;
    // let extend: IoDevice;
    // {
    //     let lock = hub.mutex.lock().await;
    //     vsensor = lock.io_from_kind(IoTypeId::VisionSensor).await?;
    //     pump = lock.io_from_port(named_port::C).await?;
    //     rot = lock.io_from_port(named_port::A).await?;
    //     extend = lock.io_from_port(named_port::B).await?;
    // }

    
    Ok(hub.mutex.clone())
}

    // Do stuff

    // Cleanup
    // println!("Disconnect from hub `{}`", hub.name);
    // {
    //     let lock = hub.mutex.lock().await;
    //     lock.disconnect().await?;
    // }

    // // === Main hub and RC ===
    // let (main_hub, rc_hub) = lego_powered_up::setup::main_and_rc().await?;
    // let rc: IoDevice;
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     rc = lock.io_from_port(named_port::A).await?;
    // }
    // let (mut rc_rx, _) = rc.remote_connect_with_green().await?;

    // // Do stuff

    // // Cleanup
    // println!("Disconnect from hub `{}`", rc_hub.name);
    // {
    //     let lock = rc_hub.mutex.lock().await;
    //     lock.disconnect().await?;
    // }
    // println!("Disconnect from hub `{}`", main_hub.name);
    // {
    //     let lock = main_hub.mutex.lock().await;
    //     lock.disconnect().await?;
    // }



// let rc_control = tokio::spawn(async move {
//     while let Ok(data) = rc_rx.recv().await {
//         match data {
//             RcButtonState::Aup => {  println!("A released"); }
//             RcButtonState::Aplus => { println!("A plus") }
//             RcButtonState::Ared => { println!("A red"); }
//             RcButtonState::Aminus => { println!("A minus") }
//             RcButtonState::Bup => { println!("B released");
//             RcButtonState::Bplus => { println!("B plus") }
//             RcButtonState::Bred => { println!("B red");  }
//             RcButtonState::Bminus => { println!("B minus") }
//             RcButtonState::Green => { println!("Green pressed") }
//             RcButtonState::GreenUp => { println!("Green released") }
//         }
//     }
// });

pub struct Vsensor {
    id: u8,
    device: IoDevice,
    hub: HubMutex,
    feedback_task: Option<JoinHandle<()>>,
    // color_task: JoinHandle<()>,
    // rx_color: broadcast::Receiver<DetectedColor>,
}
#[async_trait]
impl zone::irrigation::tank::TankSensor for Vsensor {
    fn id(&self) -> u8 {
        self.id
    }
    // fn read_level(&self) -> Result<(TankLevel), Box<dyn Error>> {
    //     Ok(TankLevel::Green)
    // }
    async fn init(
        &mut self,
        tx_tanklevel: tokio::sync::broadcast::Sender<(u8, Option<TankLevel>)>,
    ) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.tank_feedback(tx_tanklevel)
                .await
                .expect("Error initializing feedback task"),
        );
        Ok(())
    }
}
impl Vsensor {
    pub async fn new(id: u8, hub: HubMutex) -> Self {
        let device: IoDevice;
        {
            let lock = hub.lock().await;
            device = lock.io_from_kind(IoTypeId::VisionSensor).await.expect("Error accessing LPU device");
        }
        Self {
            id,
            hub,
            device,
            feedback_task: None,
        }
    }

    async fn tank_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<TankLevel>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let (mut rx_color, _color_task) = 
            self.device.visionsensor_color().await.unwrap();
        Ok(tokio::spawn(async move {
            println!("Spawned tank feedback");
            while let Ok(data) = rx_color.recv().await {
                // println!("Tank color: {:?} ", data,);
                match data {
                    DetectedColor::Blue => {
                        tx.send( (id, Some(TankLevel::Blue)) );
                    }
                    DetectedColor::Green => {
                        tx.send( (id, Some(TankLevel::Green)) );
                    }
                    DetectedColor::Yellow => {
                        tx.send( (id, Some(TankLevel::Yellow)) );
                    }
                    DetectedColor::Red => {
                        tx.send( (id, Some(TankLevel::Red)) );
                    }
                    _ =>  {
                        tx.send( (id, None ) );
                    }
                }
            }

        }))
    }
}


pub struct BrickPump {
    id: u8,
    device: IoDevice,
    hub: HubMutex,
    control_task: Option<JoinHandle<()>>,
    // feedback_task: Option<JoinHandle<()>>,
}
#[async_trait]
impl zone::irrigation::pump::Pump for BrickPump {
    fn id(&self) -> u8 {
        self.id
    }
    fn run_for_secs(&self, secs: u16) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn stop(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    async fn init(
        &mut self,
        rx_pumpcmd: tokio::sync::broadcast::Receiver<(u8, PumpCmd)>,
    ) -> Result<(), Box<dyn Error>> {
        self.control_task = Some(
            self.pump_control(rx_pumpcmd)
                .await
                .expect("Error initializing control task"),
        );
        Ok(())
    }
}
impl BrickPump {
    pub async fn new(id: u8, hub: HubMutex) -> Self {
        let device: IoDevice;
        {
            let lock = hub.lock().await;
            device = lock.io_from_port(PUMP_ADDR).await.expect("Error accessing LPU device");
        }
        Self {
            id,
            hub,
            device,
            control_task: None,
        }
    }

    async fn pump_control(
        &self,
        mut rx_cmd: broadcast::Receiver<(u8, PumpCmd)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let device = self.device.clone();
        // let (mut rx_color, _color_task) = 
        //     self.device.visionsensor_color().await.unwrap();  // TODO Speed feedback
        Ok(tokio::spawn(async move {
            println!("Spawned pump control");
            while let Ok(data) = rx_cmd.recv().await {
                // println!("Tank color: {:?} ", data,);
                match data {
                    (_id, PumpCmd::RunForSec(secs)) => { 
                        device.start_speed(50, 100);
                        sleep(Duration::from_secs(secs as u64)).await;
                        device.start_power(Power::Float);
                    }
                    (_id, PumpCmd::Stop) => { 
                        device.start_power(Power::Brake);
                    }
                    // _ =>  {
                    //     tx.send( (id, None ) );
                    // }
                }
            }

        }))
    }
}