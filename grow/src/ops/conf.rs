#![allow(unused)]

extern crate alloc;
use super::House;
// use super::HouseMapped;
use crate::zone;
use crate::zone::Zone;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use anyhow::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Conf {}
impl Conf {
    pub fn read_file_into(mut h: Vec<Zone>) -> Vec<Zone> {
        h
    }

    pub fn read_test_into_house() -> House {
        // Return hardcoded config
        let mut h = House::new();

        h.zones.push(zone::air::new(
                    1,
                    zone::air::Settings {
                        temp_fan_low: 25.0,
                        temp_fan_high: 30.0,
                        temp_warning: 35.0,
                    },
                ));
        h.zones.push(zone::irrigation::new(
            1,
            zone::irrigation::Settings {
                moisture_limit_water: 50,
                moisture_limit_low_warning: 30,
                moisture_limit_high_warning: 70,
                pump_id: 0,
                position: Some(zone::irrigation::arm::Move {
                    arm_id: 0,
                    x: 20,
                    y: 100,
                    z: 0,
                }),
            },
        )); 
        h.zones.push(zone::irrigation::new(
            2,
            zone::irrigation::Settings {
                moisture_limit_water: 50,
                moisture_limit_low_warning: 30,
                moisture_limit_high_warning: 70,
                pump_id: 0,
                position: Some(zone::irrigation::arm::Move {
                    arm_id: 0,
                    x: 50,
                    y: 200,
                    z: 0,
                }),
            },
        )); 
        h.zones.push(zone::light::new(
            1,
            zone::light::Settings {},
        ));
        h.zones.push(zone::auxiliary::new(
            1,
            zone::auxiliary::Settings {},
        ));
        h.zones.push(zone::irrigation::arm::new(
            1,
            zone::irrigation::arm::Settings {},
        ));
        h.zones.push(zone::irrigation::pump::new(
            1,
            zone::irrigation::pump::Settings {
                run_for_secs: 10,
                rest_secs: 60,
            },
        ));
        h.zones.push(zone::irrigation::tank::new(
            1,
            zone::irrigation::tank::Settings {},
        ));

        h
    }
}

    // pub fn read_test_into_vec() -> Vec<Zone> {
    //     // Return hardcoded config
    //     let mut h = Vec::new();
    //     h.push(zone::air::new(
    //         1,
    //         zone::air::Settings {
    //             temp_fan_low: 25.0,
    //             temp_fan_high: 30.0,
    //             temp_warning: 35.0,
    //         },
    //     ));
    //     h.push(Zone::Light {
    //         id: 0,
    //         settings: zone::light::Settings {},
    //         status: zone::light::Status {},
    //     });
    //     h.push(Zone::Irrigation {
    //         id: 0,
    //         set: zone::irrigation::Settings {
    //             moisture_limit_water: 50,
    //             moisture_limit_low_warning: 30,
    //             moisture_limit_high_warning: 70,
    //             pump_id: 0,
    //             position: Some(zone::arm::Move {
    //                 arm_id: 0,
    //                 x: 20,
    //                 y: 100,
    //                 z: 0,
    //             }),
    //         },
    //         status: zone::irrigation::Status {},
    //     });
    //     h.push(Zone::Pump {
    //         id: 0,
    //         set: zone::pump::Settings {
    //             run_for_secs: 10,
    //             rest_secs: 60,
    //         },
    //         status: zone::pump::Status {},
    //     });
    //     h.push(Zone::Tank {
    //         id: 0,
    //         set: zone::tank::Settings {},
    //         status: zone::tank::Status {},
    //     });
    //     h.push(Zone::Arm {
    //         id: 0,
    //         set: zone::arm::Settings {},
    //         status: zone::arm::Status {},
    //     });

    //     h
    // }

    // pub fn read_test_into_housemapped() -> HouseMapped {
    //     // Return hardcoded config
    //     let mut h = HouseMapped::new();

    //     h.air.insert(
    //         0,
    //         Zone::Air {
    //             id: 0,
    //             set: zone::air::Settings {
    //                 temp_fan_low: 25.0,
    //                 temp_fan_high: 30.0,
    //                 temp_warning: 35.0,
    //             },
    //             status: zone::air::Status {
    //                 temp: None,
    //                 fan_rpm: None,
    //                 indicator: None,
    //             },
    //         },
    //     );
    //     h.light.insert(
    //         0,
    //         Zone::Light {
    //             id: 0,
    //             set: zone::light::Settings {},
    //             status: zone::light::Status {},
    //         },
    //     );
    //     h.irrigation.insert(
    //         0,
    //         Zone::Irrigation {
    //             id: 0,
    //             set: zone::irrigation::Settings {
    //                 moisture_limit_water: 50,
    //                 moisture_limit_low_warning: 30,
    //                 moisture_limit_high_warning: 70,
    //                 pump_id: 0,
    //                 position: Some(zone::arm::Move {
    //                     arm_id: 0,
    //                     x: 20,
    //                     y: 100,
    //                 }),
    //             },
    //             status: zone::irrigation::Status {},
    //         },
    //     );
    //     h.pump.insert(
    //         0,
    //         Zone::Pump {
    //             id: 0,
    //             set: zone::pump::Settings {
    //                 run_for_secs: 10,
    //                 rest_secs: 60,
    //             },
    //             status: zone::pump::Status {},
    //         },
    //     );
    //     h.tank.insert(
    //         0,
    //         Zone::Tank {
    //             id: 0,
    //             set: zone::tank::Settings {},
    //             status: zone::tank::Status {},
    //         },
    //     );
    //     h.arm.insert(
    //         0,
    //         Zone::Arm {
    //             id: 0,
    //             set: zone::arm::Settings {},
    //             status: zone::arm::Status {},
    //         },
    //     );

    //     h
    // }
