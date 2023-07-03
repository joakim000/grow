use lego_powered_up::PoweredUp;
use std::sync::Arc;

use tokio::sync::Mutex as TokioMutex;
use tokio_util::sync::CancellationToken;

use crate::hardware;
use grow::ops;
use grow::ops::manager::Manager;

use grow::House;
use grow::zone::*;
use grow::HouseMutex;
use grow::ManagerMutex;
use grow::ops::OpsChannelsTx;
use grow::zone::ZoneChannelsTx;

pub async fn init(cancel: CancellationToken)  -> (HouseMutex, ManagerMutex) {
    let (zone_tx, zone_rx) = grow::zone::zone_channels();
    let (ops_tx, ops_rx) = grow::ops::ops_channels();
    
    let mut house =
        ops::conf::read_file_into_house("grow-conf.js", zone_tx.clone(), ops_tx.clone());
    let (house, pu) = house_hardware_init(house, cancel.clone()).await;    
    let house = Arc::new(TokioMutex::new(house));

    let manager = manager_hardware_init(
            house.clone(), cancel.clone(), 
            zone_tx.clone(), ops_tx.clone(), pu).await;    
    let manager = Arc::new(TokioMutex::new(manager));
    
    manager.lock().await.init(zone_rx, ops_rx, manager.clone()).await;
    house.lock().await.init().await;
   
    (house, manager)
}

pub async fn house_hardware_init(
    mut house: House,
    cancel: CancellationToken,
// ) -> (HouseMutex, ManagerMutex) {
) -> (House,  Arc<TokioMutex<PoweredUp>>) {
    let pu = Arc::new(TokioMutex::new(
        PoweredUp::init()
            .await
            .expect("Error from PoweredUp::init()"),
    ));
    let lpu_hub = hardware::lpu::init(pu.clone(), cancel.clone())
        .await
        .expect("Error from lpu::init()"); //thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: BluetoothError(DeviceNotFound)', src/init.rs:19:64
    let adc_1 = hardware::pcf8591::Adc::new(cancel.clone());

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
  
    (house, pu)
}

pub async fn manager_hardware_init(
    house: HouseMutex,
    cancel: CancellationToken,
    zone_tx: ZoneChannelsTx,
    ops_tx: OpsChannelsTx,
    pu: Arc<TokioMutex<PoweredUp>>,
) -> Manager {
    let manager = Manager::new(
        house, //_mutex.clone(),
        Box::new(hardware::regshift_leds::Shiftreg::new(cancel.clone())),
        Box::new(hardware::ssd1306::Oled::new(cancel.clone())),
        Box::new(hardware::lpu_remote::LpuRemote::new(pu, cancel.clone())),
        Box::new(hardware::pushbuttons::PushButtons::new(cancel.clone())),
        ops_tx, //.clone(),
        zone_tx, //.clone(),
    );
 
    manager
}
