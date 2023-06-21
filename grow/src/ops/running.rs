use crate::House;
use crate::Zone;
use crate::HouseMutex;
use crate::zone::arm;

use core::error::Error;
use core::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::sync::Mutex;
use std::sync::Arc;
use core::fmt::Debug;

use super::display::ZoneDisplay;
use super::TextDisplay;
use super::Board;
use super::remote;
use super::remote::*;
use crate::zone::irrigation::arm::Arm;


// pub struct Runner {
//     house: Vec<Zone>,

// }

pub struct Manager {
    pub house: HouseMutex,
    pub board: Box<dyn Board>,
    pub display: Box<dyn TextDisplay>,
    pub remote: Box<dyn RemoteControl>,
}
impl Manager {
    pub fn new(house: HouseMutex, board: Box<dyn Board>, 
            display: Box<dyn TextDisplay>, remote: Box<dyn RemoteControl>,
            // buttons: Box<dyn Buttons>
        ) -> Self {     
        Self {
            house,
            board,
            display,
            remote,
            // buttons,
        }
    }
    
    pub fn update_board(&self) {

    }

    pub fn run(&self) -> (Result<(), Box<dyn Error>>) {

        Ok(())
    }

    pub async fn position_from_rc(&mut self, zid: u8) -> (i32, i32, i32) {
        println!("Inside pos from rc");
        let (rc_tx, mut rc_rx) = broadcast::channel::<RcInput>(1);
        let _ = self.remote.init(rc_tx).await;
        
        let x = 0;
        let y = 0;
        let z = 0;

        let mutex = self.house.clone();
        // let mut lock = mutex.lock().await;
        // let mut arm_o: Option<Arc<&(dyn Arm + '_)>> = None;
        
        // for z in lock.zones() {
        //     match z {
        //         Zone::Arm {id , settings:_, status:_, interface, runner: _} if id == &zid => {
        //             // let arm = Arc::new(interface.arm.as_deref().expect("Interface not found"));
        //             let arm = interface.arm.as_deref().expect("Interface not found"); 
        //         }
        //         _ => continue
        //     }
        // }
        
        // return Err(Box::new(ZoneError::new("Zone not found")))
        tokio::spawn(async move {
            println!("Spawned position finder task");
            // let mut arm_o: Option<Arc<&(dyn Arm + '_)>> = None;
            let mut arm_o: Option<&(dyn Arm + '_)> = None;
            let mut lock = mutex.lock().await;
            for z in lock.zones() {
                match z {
                    Zone::Arm {id , settings:_, status:_, interface, runner: _} if id == &zid => {
                        // let arm = Arc::new(interface.arm.as_deref().expect("Interface not found"));
                        let arm_o = Some(interface.arm.as_deref().expect("Interface not found")); 
                    }
                    _ => continue
                }
            }
            let arm = arm_o.expect("Zone not found");
            while let Ok(data) = rc_rx.recv().await {
                println!("Runner RC input: {:?} ", data,);
                match data {
                    RcInput::LeftUp | RcInput::RightUp => {
                        arm.stop_x().await;
                    }
                    RcInput::Left => {
                        arm.start_x(-20).await;
                    }
                    RcInput::Right => {
                        arm.start_x(20).await;
                    }
                    RcInput::Up => {
                    }
                    RcInput::Down => {
                    }
                    RcInput::Confirm => {
                    }
                    RcInput::Back => {
                    }
                    RcInput::Mode => {
                    }
                    RcInput::Exit => {
                    }
                   
                    RcInput::RightUp => {
                    }
                    RcInput::ConfirmUp => {
                    }
                    RcInput::DownUp => {
                    }
                    RcInput::UpUp => {
                    }
                    RcInput::BackUp => {
                    }
                    RcInput::ModeUp => {
                    }
                }
            }
        });


        (x, y, z)
    }

}




// // (house: House )

// impl Runner {
//     pub fn new() -> Self {
//         Self {
//             house: Vec::new(),

//         }
//     }

//     pub fn init(&mut self) {
//         // Create house from conf
//         self.house = super::conf::Conf::read_test_into_vec();
//         // Init hw
//         for z in &self.house {
//             match z {
//                 Zone::Air{id, set, status , interface} => {

//                 },
//                 _ => ()
//             }
//         }

//         // Start run
//     }

//     // Running: light scheduler, rx: zone::sensortype, ui (buttons, display, indicators)

//     pub async fn run() -> tokio::task::JoinHandle<()> {

//         tokio::spawn(async move {

//             tokio::time::sleep(Duration::from_secs(1)).await;

//         })
//     }

//     pub fn remote_set_irrigation_position() {}

//     pub async fn shutdown() -> tokio::task::JoinHandle<()> {
//         // Save: LPU position data, indicator statuses?

//         // Reset pins

//         // Disconnect LPU

//         tokio::spawn(async move {

//             tokio::time::sleep(Duration::from_secs(1)).await;

//         })

//     }
// }

// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub struct Settings {}

// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub struct Status {}
