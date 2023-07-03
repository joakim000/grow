use lego_powered_up::PoweredUp;
use std::sync::Arc;

use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;

use crate::hardware;
use grow::ops;
use grow::ops::manager::Manager;

use grow::zone::*;
use grow::HouseMutex;
use grow::ManagerMutex;

pub async fn hardware_init(
    cancel: CancellationToken,
) -> (HouseMutex, ManagerMutex) {
    let pu = Arc::new(TokioMutex::new(
        PoweredUp::init()
            .await
            .expect("Error from PoweredUp::init()"),
    ));
    let lpu_hub = hardware::lpu::init(pu.clone(), cancel.clone())
        .await
        .expect("Error from lpu::init()"); //thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: BluetoothError(DeviceNotFound)', src/init.rs:19:64
    let adc_1 = hardware::pcf8591::Adc::new(cancel.clone());

    // Create main channels
    let (zone_tx, zone_rx) = grow::zone::zone_channels();
    let (ops_tx, ops_rx) = grow::ops::ops_channels();

    let mut house =
        ops::conf::Conf::read_test_into_house(zone_tx.clone(), ops_tx.clone());
    for zone in house.zones() {
        match zone {
            Zone::Air {id, interface, ..} if id == &1 => {
                interface.fan =
                    Some(Box::new(hardware::pwmfan::PwmFan::new(*id)));
                interface.thermo = Some(Box::new(
                    hardware::pcf8591::Thermistor::new(*id, adc_1.new_mutex()),
                ));
            }
            Zone::Air {id, interface, ..} if id == &2 => {
                interface.fan = None;
                interface.thermo = Some(Box::new(
                    hardware::lpu::LpuTemp::new(*id, lpu_hub.clone()),
                ));
            }
            Zone::Aux {id, interface, ..} => {
                interface.aux_device =
                    Some(Box::new(hardware::lpu::LpuHub::new(
                        *id,
                        lpu_hub.clone(),
                        cancel.clone(),
                    )));
            }
            Zone::Light {id, interface, ..} => {
                interface.lightmeter =
                    Some(Box::new(hardware::pcf8591::Photoresistor::new(
                        *id,
                        adc_1.new_mutex(),
                    )));
                interface.lamp = Some(Box::new(hardware::pcf8591::Led::new(
                    *id,
                    adc_1.new_mutex(),
                )));
            }
            Zone::Water {id, interface, ..} => {
                interface.moist = Some(Box::new(
                    hardware::pcf8591::CapacitiveMoistureSensor::new(
                        *id,
                        adc_1.new_mutex(),
                    ),
                ));
            }
            Zone::Tank {id, interface, ..} => {
                interface.tank_sensor = Some(Box::new(
                    hardware::lpu::Vsensor::new(*id, lpu_hub.clone()),
                ));
            }
            Zone::Pump {id, interface, ..} => {
                interface.pump = Some(Box::new(
                    hardware::lpu::BrickPump::new(*id, lpu_hub.clone()).await,
                ));
            }
            Zone::Arm {id, interface, ..} => {
                interface.arm = Some(Box::new(
                    hardware::lpu::BrickArm::new(*id, lpu_hub.clone()).await,
                ));
            } 
            _ => (),
        }
    }
    // dbg!(&house);
    let house_mutex = Arc::new(TokioMutex::new(house));

    let manager = Manager::new(
        house_mutex.clone(),
        Box::new(hardware::regshift_leds::Shiftreg::new(cancel.clone())),
        Box::new(hardware::ssd1306::Oled::new(cancel.clone())),
        Box::new(hardware::lpu_remote::LpuRemote::new(pu, cancel.clone())),
        Box::new(hardware::pushbuttons::PushButtons::new(cancel.clone())),
        ops_tx.clone(),
        zone_tx.clone(),
    );
    let manager_mutex = Arc::new(TokioMutex::new(manager));
    {
        let mut lock = manager_mutex.lock().await;
        lock.init(zone_rx, ops_rx, manager_mutex.clone()).await;
        // dbg!(&lock);
    }
    {
        let mut lock = house_mutex.lock().await;
        lock.init().await;
        // dbg!(&lock);
    }
    (house_mutex, manager_mutex)
}
