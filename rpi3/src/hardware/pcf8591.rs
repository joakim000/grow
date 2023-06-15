use super::conf::*;
use pcf8591::{PCF8591, Pin};
use std::sync::Arc;
use tokio::sync::Mutex;
// use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use grow::zone;
use grow::zone::Handles; 
use core::fmt::Debug;

use core::error::Error;
use core::result::Result;
use core::time::Duration;
pub type AdcMutex = Arc<Mutex<PCF8591>>;


// let mut adc_control = PCF8591::new(ADC_1_BUS, ADC_1_ADDR, ADC_1_VREF)?;
// let temperature_1: f32 = celcius_from_byte(adc_control.analog_read_byte(TEMP_SENSOR_1)? as &f32);

// let light_1: u8 = light_from_byte(adc_control.analog_read_byte(LIGHT_SENSOR_1)? as &u8);
// let moisture_1: i16 = moist_from_byte(adc_control.analog_read_byte(MOIST_SENSOR_1)? as &i16);

// newtype pattern
// struct PCF8591Wrapper(PCF8591);
// impl Debug for PCF8591Wrapper {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         // write!(f, "PCF8591{{{}}}", self.len())
//         write!(f, "PCF8591")
//     }
// }

// #[derive( Debug, )]
pub struct Adc {
    mutex: AdcMutex,
}
impl Adc {
    pub fn new() -> Self {
        let control = PCF8591::new(ADC_1_BUS, ADC_1_ADDR, ADC_1_VREF).unwrap();
        let mutex = Arc::new(Mutex::new(control));
        Self {
            mutex: mutex, 
        }
    }
    pub fn new_mutex(&self) -> AdcMutex {
        self.mutex.clone()
    }
}
// impl Debug for Adc {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         // write!(f, "PCF8591{{{}}}", self.len())
//         write!(f, "PCF8591")
//     }
// }
// impl Debug for AdcMutex {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         // write!(f, "PCF8591{{{}}}", self.len())
//         write!(f, "PCF8591 mutex")
//     }
// }

// #[derive( Debug, )]
pub struct Led {
    id: u8,
    adc: AdcMutex,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::light::Lamp for Led {
    fn on(&self) -> Result<(), Box<dyn Error>> { Ok(())}
    fn off(&self) -> Result<(), Box<dyn Error>> {Ok(())}
    fn init(&self) -> Result<(), Box<dyn Error>> {Ok(())}
}

// #[derive( Debug, )]
pub struct Thermistor {
    id: u8,
    adc: AdcMutex,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::air::Thermometer for Thermistor {
    fn read_temp(&self) -> Result<(i32), Box<dyn Error>> {Ok(100i32)}
    fn init(&mut self,  tx_temp: tokio::sync::broadcast::Sender<(u8, Option<f32>)>) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.temp_feedback(tx_temp)
                .expect("Error initializing feedback task"),
        );
        
        Ok(())
    }
}
impl Debug for Thermistor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Thermistor:")
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

    fn temp_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id.clone();
        let adc = self.adc.clone();
        Ok(tokio::spawn(async move {
            let mut previous: Option<f32> = None;
            loop {
                let reading: f32;
                {
                    let mut lock = adc.lock().await;
                    reading = celcius_from_byte(lock.analog_read_byte(TEMP_SENSOR_1).unwrap() as f32);
                }
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


// #[derive( Debug, )]
pub struct Photoresistor {
    id: u8,
    adc: AdcMutex,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::light::Lightmeter for Photoresistor {
    // fn read_light(&self) -> Result<(i32), Box<dyn Error>> {Ok(100i32)}
    fn init(&mut self,  tx_light: tokio::sync::broadcast::Sender<(u8, Option<f32>)>) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.light_feedback(tx_light)
                .expect("Error initializing feedback task"),
        );
        
        Ok(())
    }
}
impl Photoresistor {
    pub fn new(id: u8, adc: AdcMutex) -> Self {
        Self {
            id,
            adc,
            feedback_task: None,
        }
    }

    fn light_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id.clone();
        let adc = self.adc.clone();
        Ok(tokio::spawn(async move {
            let mut previous: Option<f32> = None;
            loop {
                let reading: f32;
                {
                    let mut lock = adc.lock().await;
                    reading = light_from_byte(lock.analog_read_byte(LIGHT_SENSOR_1).unwrap());
                }
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



// #[derive( Debug, )]
pub struct CapacitiveMoistureSensor {
    id: u8,
    adc: AdcMutex,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::irrigation::MoistureSensor for CapacitiveMoistureSensor {
    // fn read_moist(&self) -> Result<(i32), Box<dyn Error>> {Ok(100i32)}
    fn init(&mut self,  tx_moist: tokio::sync::broadcast::Sender<(u8, Option<f32>)>) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.moist_feedback(tx_moist)
                .expect("Error initializing feedback task"),
        );
        
        Ok(())
    }
}
impl CapacitiveMoistureSensor {
    pub fn new(id: u8, adc: AdcMutex) -> Self {
        Self {
            id,
            adc,
            feedback_task: None,
        }
    }

    fn moist_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id.clone();
        let adc = self.adc.clone();
        let pin = match id {
            1 => MOIST_SENSOR_1,
            2 => MOIST_SENSOR_2,
            _ => Pin::AIN0  // need something
        };
        Ok(tokio::spawn(async move {
            let mut previous: Option<f32> = None;
            loop {
                let reading: f32;
                {
                    let mut lock = adc.lock().await;
                    reading = moist_from_byte(lock.analog_read_byte(pin).unwrap());
                }
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






/// Conversions
fn celcius_from_byte(value: f32) -> f32 {
    let coeff_b = 3950f32; // thermistor coefficient
    let res_r0 = 10000f32; // resistance @ room temperature
    let res_r1 = 1000f32; // resistance of R1
    let room_temperature_in_kelvin = 297.15f32;

    let res_r6: f32 = (res_r1 * value) / (256.0 - value);
    let kelvin: f32 =
        1.0 / ((1.0 / room_temperature_in_kelvin) + (1.0 / coeff_b) * (res_r6 / res_r0).ln());
    
    kelvin - 273.15
}
fn moist_from_byte(value: u8) -> f32 {
    // 115 = 100% moist, 215 = 0% moist
    (0f32 - value as f32 + 215f32) as f32
}
fn light_from_byte(value: u8) -> f32 {
    // 15(240) = dark, 40 = 5v LED up close, 208(47) = very light,
    (255f32 - value as f32) as f32
}