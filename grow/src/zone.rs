#![allow(unused)]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::{Vec, IntoIter};
use anyhow;
// use core::result::Result;



// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Zone {
    Light {id:u8, set:light::Settings, status:light::Status},
    Tank {id:u8, set:tank::Settings, status:tank::Status},
    Irrigation {id:u8, set:irrigation::Settings, status:irrigation::Status},
    Pump {id:u8, set:pump::Settings, status:pump::Status},
    Arm {id:u8, set:arm::Settings, status:arm::Status},
    Air {id:u8, set:air::Settings, status:air::Status},
}

pub mod light {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {}

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}

pub mod tank {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {}

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}

pub mod irrigation {
    use super::Zone;
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {
        pub moisture_limit_water: i16,
        pub moisture_limit_low_warning: i16,
        pub moisture_limit_high_warning: i16,
        pub pump_id: u8,
        pub position: Option<super::arm::Move>,

    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}

pub mod pump {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {
        pub run_for_secs: u16,
        pub rest_secs: u16,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}
}

pub mod arm {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Settings {}

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Move{
        pub arm_id: u8,
        pub x: i32,
        pub y: i32 
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Status {}

    trait Arm {
        fn goto(&self, x: i32, y: i32) -> anyhow::Result<()>; 
        fn confirm(&self, x: i32, y: i32) -> anyhow::Result<()>;
    }
}

pub mod air {   
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Settings {
        pub temp_fan_low: f64,
        pub temp_fan_high: f64,
        pub temp_warning: f64,

    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Status {
        pub temp: Option<f64>,
        pub fan_rpm: Option<f64>
    }

    trait Fan {
        fn to_high(&self) -> anyhow::Result<()>;
        fn to_low(&self) -> anyhow::Result<()>;
        fn read_rpm(&self) -> anyhow::Result<u32>;
    }
    trait Thermometer {
        fn read_temp(&self) -> anyhow::Result<u32>;
    }
}


// mod light;
// mod pump;  
// mod tank;
// mod irrigation;
// mod air;