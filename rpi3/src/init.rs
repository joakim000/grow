use crate::hardware;
use grow::ops;
use grow::zone::*;

// pub async fn house_init(lpu_hub: lego_powered_up::HubMutex) -> grow::House {
pub async fn house_init() -> grow::House {
    let mut house = ops::conf::Conf::read_test_into_house();
    let lpu_hub = crate::hardware::lpu::init().await.unwrap();
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
                interface.tank_sensor =
                    Some(Box::new(hardware::lpu::Vsensor::new(*id, lpu_hub.clone())));
            }
            Zone::Pump {
                id,
                settings: _,
                status: _,
                interface,
                runner,
            } => {
                interface.pump = Some(Box::new(
                    hardware::lpu::BrickPump::new(*id, lpu_hub.clone()).await,
                ));
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
                interface.arm = Some(Box::new(
                    hardware::lpu::BrickArm::new(*id, lpu_hub.clone()).await,
                ));
            }
            _ => (),
        }
    }
    println!("Added hw to house:");
    // dbg!(&house);
    house.init().await;
    println!("After house init:");
    // dbg!(&house);

    // Test commands
    // house.set_lamp_state(1, light::LampState::On);
    // house.read_light_value(1);
    // house.arm_goto_x(1, 30).await;
    // house.arm_goto_y(1, 30).await;
    // house.run_pump(1, 2).await;
    // house.read_temperature_value(1);
    // house.set_lamp_state(1, light::LampState::Off);

    house
}

pub fn runner_init(house: grow::HouseMutex) -> grow::ops::running::Manager {
    let board = Box::new(hardware::regshift_leds::Shiftreg::new());
    let display = Box::new(hardware::ssd1306::Oled::new());

    grow::ops::running::Manager {
        house,
        board, 
        display
    }
}   