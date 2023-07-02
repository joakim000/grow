use std::sync::Arc;
// use tokio::sync::Mutex;
use core::error::Error;
use core::fmt::Debug;
use core::result::Result;
use core::time::Duration;
use parking_lot::RwLock;
use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use i2cdev::core::I2CDevice;
use i2cdev_bmp180::{BMP180BarometerThermometer, BMP180PressureMode};

pub type BmpMutex<T> = Arc<RwLock<BMP180BarometerThermometer<T>>>;
use super::conf::*;
use grow::zone;

#[derive( Debug, )]
pub struct BoschSensor<T:i2cdev::core::I2CDevice + Sized> {
    id: u8,
    sensor: BmpMutex<T>,
    // sensor: BMP180BarometerThermometer,
    feedback_task: Option<JoinHandle<()>>,
}
impl<T> zone::air::Thermometer for BoschSensor<T> {
    fn id(&self) -> u8 {
        self.id
    }
    fn init(
        &mut self,
        tx_temp: tokio::sync::broadcast::Sender<(u8, Option<f64>)>,
    ) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.temp_feedback(tx_temp)
                .expect("Error initializing feedback task"),
        );

        Ok(())
    }
    fn read(&self) -> Result<f64, Box<dyn Error + '_>> {
        // let reading_temp: Result<f32, Box<dyn Error>>;
        // let reading_pressure: Result<f32, Box<dyn Error>>;
        // let reading_pressure: f32; 
        // {
        //     let mut sensor = self.sensor.write();
        //     let reading_temp = sensor.temperature_celsius();
        //     let reading_pressure = sensor.pressure_kpa(); 
        // }
        let reading_temp = self.sensor.write().temperature_celsius();
        let reading_pressure = self.sensor.write().pressure_kpa();
        println!("BMP temp:{}  press:{}", reading_temp, reading_pressure);    

        Ok(reading_temp.into())
    }
}
// impl Debug for BoschSensor {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         write!(f, "BoschTempSensor")
//     }
// }
impl<T> BoschSensor<T> {
    pub fn new(id: u8) -> Self {
        let path = format!("/dev/i2c-{}", BMP180_BUS);
        // let i2c = LinuxI2CDevice::new(&path, BMP180_ADDR).unwrap();
        let i2c = i2cdev::linux::LinuxI2CDevice::new(&path, BMP180_ADDR).unwrap();
        let sensor = BMP180BarometerThermometer::new(i2c, BMP180PressureMode::BMP180Standard);
        Self {
            id,
            feedback_task: None,
            sensor: Arc::new(RwLock::new(sensor)),
        }
    }

    fn temp_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<f64>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        Ok(tokio::spawn(async move {
            // let mut previous: Option<f32> = None;
            let mut previous: f32 = f32::MAX;
            loop {
                // let reading_temp: Result<f32, Box<dyn Error>>;
                // let reading_pressure: Result<f32, Box<dyn Error>>;
                // let reading_pressure: f32; 
                // {
                //     let mut sensor = self.sensor.write();
                //     let reading_temp = sensor.temperature_celsius();
                //     let reading_pressure = sensor.pressure_kpa(); 
                // }
                let reading_temp = self.sensor.write().temperature_celsius();
                let reading_pressure = self.sensor.write().pressure_kpa();
                println!("BMP temp:{}  press:{}", reading_temp, reading_pressure);    
                // match read_result {
                //     Ok(raw_reading) => {
                //         reading = celcius_from_byte(raw_reading.into());
                //         // println!("Temp {:?}   reading {:?}   previous {:?}", &id, &reading, &previous);
                //         if reading != previous {
                //             let _ = tx.send((id, Some(reading)));
                //             previous = reading;
                //         }
                //     }
                //     Err(_e) => {
                //         let _ = tx.send((id, None));
                //     }
                // }
                tokio::time::sleep(Duration::from_secs(DELAY_TEMP_2)).await;
            }
        }))
    }
}
