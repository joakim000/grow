use grow::ops;
use grow::zone::*;
use crate::hardware;

pub fn house_init() -> grow::House {
    let mut house = ops::conf::Conf::read_test_into_house();

    let adc_1 = crate::hardware::pcf8591::Adc::new();


    for zone in house.zones() {
        match zone {
            Zone::Air {
             id,
             settings: _,
             status: _,
             interface,
             runner,
            } => {
                interface.fan = Some(Box::new(hardware::pwmfan::PwmFan::new(*id)));
                interface.thermo = Some(Box::new(hardware::pcf8591::Thermistor::new(*id, adc_1.new_mutex())));
            }
            Zone::Irrigation {
                id,
                settings: _,
                status: _,
                interface,
                runner,
            } => {
               interface.moist = Some(Box::new(hardware::pcf8591::CapacitiveMoistureSensor::new(*id, adc_1.new_mutex())));
            }
            // Zone::Light {
            //     id,
            //     settings: _,
            //     status: _,
            //     interface,
            //     runner,
            // } => {
            //    interface. = Some(Box::new(hardware::pcf8591::CapacitiveMoistureSensor::new(*id, adc_1.new_mutex())));
            // }  

            _ => (),

        }
    }
    println!("Added hw to house:");
    dbg!(&house);
    house.init();
    println!("After house init:");
    dbg!(&house);

    house
}