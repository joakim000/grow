#![allow(unused)]

extern crate alloc;
use super::House;
use crate::ops::OpsChannelsTx;
use crate::zone;
use crate::zone::Zone;
use crate::zone::ZoneChannelsTx;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
use anyhow::*;
use core::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Conf {}
impl Conf {
    pub fn read_file_into(mut h: Vec<Zone>) -> Vec<Zone> {
        h
    }

    pub fn read_test_into_house(
        zone_tx: ZoneChannelsTx,
        ops_tx: OpsChannelsTx,
    ) -> House {
        // Return hardcoded config
        let mut h = House::new(zone_tx, ops_tx);

        h.zones.push(zone::air::new(
            1,
            zone::air::Settings {
                temp_fan_low: 25.0,
                temp_fan_high: 30.0,
                temp_high_yellow_warning: 35.0,
                temp_high_red_alert: 40.0,
                fan_rpm_low_red_alert: 10.0,
            },
        ));
        h.zones.push(zone::water::new(
            1,
            zone::water::Settings {
                moisture_limit_water: 50.0,
                moisture_low_red_alert: 30.0,
                moisture_high_red_alert: 70.0,
                moisture_low_yellow_warning: 40.0,
                moisture_high_yellow_warning: 65.0,
                pump_id: 1,
                tank_id: 1,
                pump_time: Duration::from_secs(2),
                settling_time: Duration::from_secs(60),
                position: zone::water::arm::Position {
                    arm_id: 1,
                    x: 84,
                    y: 3872,
                    z: 0,
                },
            },
        ));
        h.zones.push(zone::water::new(
            2,
            zone::water::Settings {
                moisture_limit_water: 50.0,
                moisture_low_red_alert: 30.0,
                moisture_high_red_alert: 70.0,
                moisture_low_yellow_warning: 40.0,
                moisture_high_yellow_warning: 65.0,
                pump_id: 1,
                tank_id: 1,
                pump_time: Duration::from_secs(2),
                settling_time: Duration::from_secs(60),
                position: zone::water::arm::Position {
                    arm_id: 1,
                    x: 210,
                    y: 3653,
                    z: 0,
                },
            },
        ));
        h.zones.push(zone::light::new(
            1,
            zone::light::Settings {
                lightlevel_low_yellow_warning: 100.0,
                lightlevel_low_red_alert: 80.0,
                // lamp on time
                // lamp off time
            },
        ));
        h.zones
            .push(zone::water::arm::new(1, zone::water::arm::Settings {}));
        h.zones.push(zone::water::pump::new(
            1,
            zone::water::pump::Settings {
                run_for_secs: 10,
                rest_secs: 60,
            },
        ));
        h.zones
            .push(zone::water::tank::new(1, zone::water::tank::Settings {}));
        h.zones
            .push(zone::auxiliary::new(1, zone::auxiliary::Settings {}));
        // h.zones
        //     .push(zone::auxiliary::new(2, zone::auxiliary::Settings {}));

        h
    }
}
