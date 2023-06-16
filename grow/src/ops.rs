#![allow(unused)]

extern crate alloc;
use super::House;
// use super::HouseMapped;
use crate::zone;
use crate::zone::Zone;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use core::error::Error;
pub mod conf;
pub mod running;
// mod warning;
use tokio::task::JoinHandle;


pub mod warning {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {}

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}

pub struct Manager {

}

// pub fn Manager(house: House) -> Result<JoinHandle<()>, Box<dyn Error>> {
//     Ok(tokio::spawn(async move {
//         println!("Spawned manager");
//         loop {
//             tokio::select! {
//                 Ok(data) = rx_rpm.recv() => {
//                 }
//                 Ok(data) = rx_temp.recv() => {
//                 }
//                 else => { break }
//             };
//         }

//     }))
// }