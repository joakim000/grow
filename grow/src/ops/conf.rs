// #![allow(unused)]

use super::House;
use crate::ops::OpsChannelsTx;
use crate::zone::{self, Zone, ZoneChannelsTx, ZoneSave };

use core::error::Error;
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{IntoIter, Vec};
// use anyhow::*;
use core::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use time::Time;


    fn load_settings(path: &str) -> Result<Vec<Zone>, Box<dyn Error>> {
        let readdata = std::fs::read_to_string(path)?;
        let loaddata: Vec<ZoneSave> = serde_json::from_str(&readdata)?;
        let mut zones: Vec<Zone> = Vec::new();
        for zone in loaddata {
            match zone {
                ZoneSave::Air { id, settings } => 
                    zones.push(zone::air::new(id, settings)),
                ZoneSave::Water { id, settings } => 
                    zones.push(zone::water::new(id, settings)),
                ZoneSave::Light { id, settings } => 
                    zones.push(zone::light::new(id, settings)),
                ZoneSave::Arm { id, settings } => 
                    zones.push(zone::arm::new(id, settings)),
                ZoneSave::Pump { id, settings } => 
                    zones.push(zone::pump::new(id, settings)),
                ZoneSave::Tank { id, settings } => 
                    zones.push(zone::tank::new(id, settings)),
                ZoneSave::Aux { id, settings } => 
                    zones.push(zone::auxiliary::new(id, settings)),
                _ => ()
            }
        }
        
        Ok(zones)
    }

    pub fn read_file_into_house( 
        path: &str,
        zone_tx: ZoneChannelsTx,
        ops_tx: OpsChannelsTx
    ) -> House {
        match load_settings(path) {
            Ok(zones) => {
                eprintln!("Load settings from: {}", &path);
                House::new2(zones, zone_tx, ops_tx)
            }
            Err(e) => {
                eprintln!("Load settings error: {}\nLoading demo settings", e);
                read_test_into_house(zone_tx, ops_tx)
            }    
        }
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
                temp_high_yellow_warning: 35.0,
                temp_high_red_alert: 40.0,
                temp_fan_low: 25.0,
                temp_fan_high: 30.0,
                fan_rpm_low_red_alert: 10.0,
            },
        ));
        h.zones.push(zone::air::new(
            2,
            zone::air::Settings {
                temp_high_yellow_warning: 35.0,
                temp_high_red_alert: 40.0,
                temp_fan_low: 25.0,
                temp_fan_high: 30.0,
                fan_rpm_low_red_alert: 10.0,
            },
        ));
        h.zones.push(zone::water::new(
            1,
            zone::water::Settings {
                moisture_low_red_alert: 20.0,
                moisture_low_yellow_warning: 30.0,
                moisture_limit_water: 50.0,
                moisture_high_yellow_warning: 90.0,
                moisture_high_red_alert: 100.0,
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
                lamp_on: Time::from_hms(19, 30, 00).expect("Time parse error"),
                lamp_off: Time::from_hms(20, 45, 00).expect("Time parse error"),
            },
        ));
        h.zones
            .push(zone::water::arm::new(1, zone::water::arm::Settings {}));
        h.zones
            .push(zone::water::pump::new(1, zone::water::pump::Settings {}));
        h.zones
            .push(zone::water::tank::new(1, zone::water::tank::Settings {}));
        h.zones
            .push(zone::auxiliary::new(1, zone::auxiliary::Settings {}));
        // h.zones
        //     .push(zone::auxiliary::new(2, zone::auxiliary::Settings {}));

        h
    }
