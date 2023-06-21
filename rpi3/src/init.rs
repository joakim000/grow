use crate::hardware;
use grow::ops;
use grow::ops::running::Manager;
use grow::zone::*;

use lego_powered_up::PoweredUp;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
use grow::HouseMutex;
use grow::ManagerMutex;

// pub async fn house_init(lpu_hub: lego_powered_up::HubMutex) -> grow::House {
pub async fn house_init(pu: Arc<TokioMutex<PoweredUp>>) -> HouseMutex {
    let mut house = ops::conf::Conf::read_test_into_house();
    let adc_1 = crate::hardware::pcf8591::Adc::new();

    
    let lpu_hub = crate::hardware::lpu::init(pu.clone()).await.unwrap();

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
            Zone::Aux {
                id,
                settings: _,
                status: _,
                interface,
                runner,
            } => {
                interface.aux_device = Some(Box::new(hardware::lpu::LpuHub::new(*id, lpu_hub.clone())));
             
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

    Arc::new(TokioMutex::new(house))
}

pub fn manager_init(house: grow::HouseMutex, pu: Arc<TokioMutex<PoweredUp>>) -> ManagerMutex {
    let board = Box::new(hardware::regshift_leds::Shiftreg::new());
    let display = Box::new(hardware::ssd1306::Oled::new());
    let remote = Box::new(hardware::lpu_remote::LpuRemote::new(pu));
    let buttons = Box::new(hardware::pushbuttons::PushButtons::new());

    Arc::new(TokioMutex::new(Manager {
        house,
        board, 
        display, 
        remote,
        // buttons,
    }))
}   