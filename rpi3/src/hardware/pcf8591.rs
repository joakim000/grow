use super::conf::*;
use pcf8591::{PCF8591, Pin};
use std::sync::Arc;
use tokio::sync::Mutex;
// use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use grow::zone;
use grow::zone::Handles; 
use core::error::Error;
use core::result::Result;
use core::time::Duration;
pub type AdcMutex = Arc<Mutex<PCF8591>>;

// let mut adc_control = PCF8591::new(ADC_1_BUS, ADC_1_ADDR, ADC_1_VREF)?;
// let light_1: u8 = light_from_byte(adc_control.analog_read_byte(LIGHT_SENSOR_1)? as &u8);

// let moisture_1: i16 = moist_from_byte(adc_control.analog_read_byte(MOIST_SENSOR_1)? as &i16);
// let moisture_2: i16 = moist_from_byte(adc_control.analog_read_byte(MOIST_SENSOR_2)? as &i16);

// pub struct Adc (PCF8591);
pub struct Adc {
    // control: PCF8591,
    mutex: AdcMutex,
}


impl Adc {
    pub fn new() -> Self {
        let control = PCF8591::new(ADC_1_BUS, ADC_1_ADDR, ADC_1_VREF).unwrap();
        let mutex = Arc::new(Mutex::new(control));
        Self {
            // control: control,
            mutex: mutex, 
        }
    }
    pub fn new_mutex(&self) -> AdcMutex {
        self.mutex.clone()
    }
}

pub struct Led {
    id: u8,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::light::Lamp for Led {
    fn on(&self) -> Result<(), Box<dyn Error>> { Ok(())}
    fn off(&self) -> Result<(), Box<dyn Error>> {Ok(())}
    fn init(&self) -> Result<(), Box<dyn Error>> {Ok(())}
}

pub struct Thermistor {
    id: u8,
    adc: AdcMutex,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::air::Thermometer for Thermistor {
    fn read_temp(&self) -> Result<(i32), Box<dyn Error>> {Ok(100i32)}
    // fn init(&self) -> Result<(), Box<dyn Error>> {Ok(())}
    fn init(&mut self,  tx_temp: tokio::sync::broadcast::Sender<(u8, Option<f64>)>) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.temp_feedback(tx_temp)
                .expect("Error initializing feedback task"),
        );
        
        Ok(())
    }
}
impl Thermistor {
    pub fn new(id: u8, adc: AdcMutex) -> Self {
        Self {
            id,
            adc,
            feedback_task: None,
        }
    }

    // let temperature_1: f64 = celcius_from_byte(adc_control.analog_read_byte(TEMP_SENSOR_1)? as &f64);
    fn temp_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<f64>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id.clone();
        let adc = self.adc.clone();
        Ok(tokio::spawn(async move {
            let mut previous: Option<f64> = None;
            loop {
                let mut lock = adc.lock().await;
                let reading = celcius_from_byte(lock.analog_read_byte(TEMP_SENSOR_1).unwrap() as f64);
                // let reading = celcius_from_byte(adc.lock().unwrap().analog_read_byte(TEMP_SENSOR_1).unwrap() as &f64);  
                if let Some(p) = previous {
                    if reading != p {
                        tx.send( (id, Some(reading)) );
                    }
                else {
                    tx.send( (id, Some(reading)) );
                }
                }
                // Check unwrap on reading
                previous = Some(reading);
                
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }))
    }

}

pub struct Photoresistor {
    id: u8,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::light::Lightmeter for Photoresistor {
    fn init(&self) -> Result<(), Box<dyn Error>> {Ok(())}
}

pub struct CapacitiveMoistureSensor {
    id: u8,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::irrigation::MoistureSensor for CapacitiveMoistureSensor {
    fn init(&self) -> Result<(), Box<dyn Error>> {Ok(())}
    // fn init(&mut self,  tx_moist: tokio::sync::broadcast::Sender<(u8, Option<f32>)>) -> Result<(), Box<dyn Error>> {
    //     self.feedback_task = Some(
    //         self.moist_feedback(tx_moist)
    //             .expect("Error initializing feedback task"),
    //     );
        
    //     Ok(())
    // }
}




fn celcius_from_byte(value: f64) -> f64 {
    let coeff_b: f64 = 3950.0; // thermistor coefficient
    let res_r0: f64 = 10000.0; // resistance @ room temperature
    let res_r1: f64 = 1000.0; // resistance of R1
    let room_temperature_in_kelvin: f64 = 297.15;

    let res_r6: f64 = (res_r1 * value) / (256.0 - value);
    let kelvin: f64 =
        1.0 / ((1.0 / room_temperature_in_kelvin) + (1.0 / coeff_b) * (res_r6 / res_r0).ln());
    kelvin - 273.15
}

fn moist_from_byte(value: &i16) -> i16 {
    // 115 = 100% moist, 215 = 0% moist
    0 - value + 215
}

fn light_from_byte(value: &u8) -> u8 {
    // 15(240) = dark, 40 = 5v LED up close, 208(47) = very light,
    255 - value
}