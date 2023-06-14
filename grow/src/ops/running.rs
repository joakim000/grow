use crate::House;
use crate::Zone;

use core::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// pub struct Runner {
//     house: Vec<Zone>,

// }

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
