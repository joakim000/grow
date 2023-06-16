use crate::hardware;
use grow::ops;
use grow::zone::*;

pub async fn house_init(lpu_hub: lego_powered_up::HubMutex) -> grow::House {
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
                interface.thermo = Some(Box::new(hardware::pcf8591::Thermistor::new(
                    *id,
                    adc_1.new_mutex(),
                )));
            }
            Zone::Light {
                id,
                settings: _,
                status: _,
                interface,
                runner,
            } => {
                interface.lightmeter = Some(Box::new(hardware::pcf8591::Photoresistor::new(
                    *id,
                    adc_1.new_mutex(),
                )));
                interface.lamp = Some(Box::new(hardware::pcf8591::Led::new(
                    *id,
                    adc_1.new_mutex(),
                )));
            }
            Zone::Irrigation {
                id,
                settings: _,
                status: _,
                interface,
                runner,
            } => {
                interface.moist = Some(Box::new(hardware::pcf8591::CapacitiveMoistureSensor::new(
                    *id,
                    adc_1.new_mutex(),
                )));
            }
            Zone::Tank {
                id,
                settings: _,
                status: _,
                interface,
                runner,
            } => {
                interface.tank_sensor = Some(Box::new(hardware::lpu::Vsensor::new(
                *id,
                lpu_hub.clone(),
                ).await
                
                ));
            }
            Zone::Pump {
                id,
                settings: _,
                status: _,
                interface,
                runner,
            } => {
                // interface.pump_cmd = Some(Box::new(hardware::lpu::Pump::new(
                // *id,
                // )));
                // interface.pump_feedback = Some(Box::new(hardware::lpu::Pump::new(
                // *id,
                // )));
            }
            Zone::Arm {
                id,
                settings: _,
                status: _,
                interface,
                runner,
            } => {
                // interface.arm_cmd = Some(Box::new(hardware::lpu::Arm::new(
                // *id,
                // )));
                // interface.arm_feedback = Some(Box::new(hardware::lpu::Arm::new(
                // *id,
                // )));
            }

            _ => (),
        }
    }
    println!("Added hw to house:");
    // dbg!(&house);
    house.init().await;
    println!("After house init:");
    // dbg!(&house);

    house
}