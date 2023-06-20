use crate::House;
use crate::Zone;
use crate::HouseMutex;

use core::error::Error;
use core::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use std::sync::Mutex;
use std::sync::Arc;
use core::fmt::Debug;

use super::display::ZoneDisplay;


// pub struct Runner {
//     house: Vec<Zone>,

// }

pub struct Manager {
    pub house: HouseMutex,
    pub board: Box<dyn Board>,
    pub display: Box<dyn TextDisplay>,
}
impl Manager {
    pub fn new(house: HouseMutex, board: Box<dyn Board>, display: Box<dyn TextDisplay>) -> Self {     
        Self {
            house,
            board,
            display,
        }
    }
    
    pub fn update_board(&self) {

    }

    pub fn run(&self) -> (Result<(), Box<dyn Error>>) {

        Ok(())
    }
}

pub trait Board : Send {
    fn init(
        &mut self,
        rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Indicator board: {{{}}}", 0)
    }
}
pub trait TextDisplay : Send {
    fn init(
        &mut self,
        rx: tokio::sync::broadcast::Receiver<Vec<ZoneDisplay>>,
    ) -> Result<(), Box<dyn Error>>;
}
impl Debug for dyn TextDisplay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Text display: {{{}}}", 0)
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
