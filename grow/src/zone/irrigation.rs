use super::Zone;
use core::error::Error;
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


pub trait MoistureSensor {
    fn init(&self) -> Result<(), Box<dyn Error>>;
}