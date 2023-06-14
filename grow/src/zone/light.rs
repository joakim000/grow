use core::error::Error;


// struct Lamp {}
// struct Sensor{}
// struct Timer{}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status {}


pub trait Lamp {
    fn on(&self) -> Result<(), Box<dyn Error>>;
    fn off(&self) -> Result<(), Box<dyn Error>>;
    fn init(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Lightmeter {
    fn init(&self) -> Result<(), Box<dyn Error>>;
}