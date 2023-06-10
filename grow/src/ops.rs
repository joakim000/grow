#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{Vec, IntoIter};
use crate::zone::Zone;
use crate::zone;
use anyhow::*;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Conf {}
impl Conf {
    pub fn read_test_into(mut h: Vec<Zone>) -> Vec<Zone> { 
        // Return hardcoded config

        h.push(Zone::Air { id: 0, 
            set: zone::air::Settings {
                temp_fan_low: 25.0,
                temp_fan_high: 30.0,
                temp_warning: 35.0,
            }, 
            status: zone::air::Status {
                temp: None,
                fan_rpm: None,
            }
        });
        h.push(Zone::Light { id: 0, set: zone::light::Settings {}, status: zone::light::Status {}  });
        h.push(Zone::Irrigation { id: 0, 
            set: zone::irrigation::Settings {
                moisture_limit_water: 50,
                moisture_limit_low_warning: 30,
                moisture_limit_high_warning: 70,
                pump_id: 0,
                position: Some(zone::arm::Move {arm_id:0, x:20, y:100}),
            }, 
            status: zone::irrigation::Status {} 
        });
        h.push(Zone::Pump { id: 0, 
            set: zone::pump::Settings {
                run_for_secs: 10,
                rest_secs: 60,

        }, status: zone::pump::Status {} });
        h.push(Zone::Tank { id: 0, set: zone::tank::Settings {}, status: zone::tank::Status {} });
        h.push(Zone::Arm { id: 0, set: zone::arm::Settings {}, status: zone::arm::Status {} });
     
        h 
    }
    pub fn read_file_into(mut h: Vec<Zone>) -> Vec<Zone> { 
     
        h 
    }

}

pub mod running {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {}

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}

pub mod warning {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {}

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}


// mod conf;
// mod running;
// mod warning;