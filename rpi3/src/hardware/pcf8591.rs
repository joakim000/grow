use super::conf::*;
use pcf8591::{LinuxI2CError, Pin, PCF8591};
use std::sync::Arc;
// use tokio::sync::Mutex;
use core::fmt::Debug;
use grow::zone;
use grow::zone::Handles;
use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use parking_lot::RwLock;
use core::error::Error;
use core::result::Result;
use core::time::Duration;
pub type AdcMutex = Arc<Mutex<PCF8591>>;
use tokio_util::sync::CancellationToken;
use grow::zone::light::LampState;


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
        let adc = mutex.clone();

        // tokio::spawn(async move {
        // loop {
        // let reading: f32;
        // {
        // println!("ADC lock req for ADC ");
        // let mut lock = adc.lock().await;
        // let mut lock = adc.lock().unwrap();
        // println!("ADC lock aquired for ADC");

        // let v0 = lock.analog_read_byte(Pin::AIN0); // photoresistor
        // let v1 = lock.analog_read_byte(Pin::AIN1); // thermistor
        // let v2 = lock.analog_read_byte(Pin::AIN2); // capacitive soil moisture 1
        // let v3 = lock.analog_read_byte(Pin::AIN3); // capacitive soil moisture 2
        // println!("Light {:?}  Temp {:?}    Moist 1 {:?}     Moist 2 {:?} ",&v0, &v1, &v2, &v3);

        //     let c0 = light_from_byte(v0.unwrap().into());
        //     let c1 = celcius_from_byte(v1.unwrap().into());
        //     let c2 = moist_from_byte(v2.unwrap().into());
        //     let c3 = moist_from_byte(v3.unwrap().into());
        //     println!("Light {:?}  Temp {:?}    Moist 1 {:?}     Moist 2 {:?} ",c0, c1, c2, c3);
        // }
        // // println!("ADC lock drop for ADC");
        // tokio::time::sleep(Duration::from_millis(10000)).await;
        // }
        // });

        Self { mutex }
    }
    pub fn new_mutex(&self) -> AdcMutex {
        self.mutex.clone()
    }
}

pub struct Led {
    id: u8,
    adc: AdcMutex,
    state: Arc<RwLock<LampState>>,
    control_task: Option<JoinHandle<()>>,
}
impl zone::light::Lamp for Led {
    fn id(&self) -> u8 {
        self.id
    }
    fn init(
        &mut self,
        rx_control: tokio::sync::broadcast::Receiver<(u8, bool)>,
    ) -> Result<(), Box<dyn Error>> {
        self.set_state(LampState::Off);
        self.control_task = Some(
            self.lamp_control(rx_control)
                .expect("Error initializing control task"),
        );
        Ok(())
    }
    fn set_state(&self, state: zone::light::LampState) -> Result<(), Box<dyn Error + '_>> {
        match state {
            zone::light::LampState::On => {
                let mut lock = self.adc.lock()?;
                *self.state.write() = LampState::On;
                Ok(lock.analog_write_byte(255)?)
            }
            zone::light::LampState::Off => {
                let mut lock = self.adc.lock()?;
                *self.state.write() = LampState::Off;
                Ok(lock.analog_write_byte(0)?)
            }
        }
    }
    fn state(&self) -> Result<LampState, Box<dyn Error>> {
        Ok((*self.state.read()))
    }
   
}
impl Debug for Led {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LED:")
    }
}
impl Led {
    pub fn new(id: u8, adc: AdcMutex) -> Self {
        Self {
            id,
            adc,
            control_task: None,
            state: Arc::new(RwLock::new(LampState::Off)), 
        }
    }
    fn lamp_control(
        &self,
        mut rx: broadcast::Receiver<(u8, bool)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let adc = self.adc.clone();
        Ok(tokio::spawn(async move {
            while let Ok(data) = rx.recv().await {
                println!("Received lamp command: {:?}", data);
                match data {
                    (id, true) => {
                        let mut lock = adc.lock().unwrap();
                        lock.analog_write_byte(255);
                    }
                    (id, false) => {
                        let mut lock = adc.lock().unwrap();
                        lock.analog_write_byte(0);
                    }
                }
            }
        }))
    }
}

// #[derive( Debug, )]
pub struct Thermistor {
    id: u8,
    adc: AdcMutex,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::air::Thermometer for Thermistor {
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
    fn read(&self) -> Result<(f64), Box<dyn Error + '_>> {
        let pin = TEMP_SENSOR[self.id as usize - 1];
        let reading: f64;
        {
            let mut lock = self.adc.lock()?;
            reading = celcius_from_byte(lock.analog_read_byte(pin)?.into());
        }

        Ok(reading)
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
        tx: broadcast::Sender<(u8, Option<f64>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let adc = self.adc.clone();
        let pin = TEMP_SENSOR[self.id as usize - 1];
        Ok(tokio::spawn(async move {
            // let mut previous: Option<f32> = None;
            let mut previous: f64 = f64::MAX;
            loop {
                let reading: f64;
                let read_result: Result<u8, LinuxI2CError>;
                {
                    let mut lock = adc.lock().unwrap();
                    read_result = lock.analog_read_byte(pin);
                }
                match read_result {
                    Ok(raw_reading) => {
                        reading = celcius_from_byte(raw_reading.into());
                        // println!("Temp {:?}   reading {:?}   previous {:?}", &id, &reading, &previous);
                        if reading != previous {
                            tx.send((id, Some(reading)));
                            previous = reading;
                        }
                    }
                    Err(e) => {
                        tx.send((id, None));
                    }
                }
                tokio::time::sleep(Duration::from_secs(DELAY_TEMP_1)).await;
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
    fn id(&self) -> u8 {
        self.id
    }
    fn init(
        &mut self,
        tx_light: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.light_feedback(tx_light)
                .expect("Error initializing feedback task"),
        );

        Ok(())
    }
    fn read(&self) -> Result<(f32), Box<dyn Error + '_>> {
        let pin: Pin;
        let pin = LIGHT_SENSOR[self.id as usize - 1];
        let reading: f32;
        {
            let mut lock = self.adc.lock()?;
            reading = light_from_byte(lock.analog_read_byte(pin)?);
        }
        Ok(reading)
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
        let id = self.id;
        let adc = self.adc.clone();
        let pin = LIGHT_SENSOR[self.id as usize - 1];
        Ok(tokio::spawn(async move {
            let mut previous = f32::MAX;
            loop {
                let reading: f32;
                let read_result: Result<u8, LinuxI2CError>;
                {
                    let mut lock = adc.lock().unwrap();
                    read_result = lock.analog_read_byte(pin);
                }
                match read_result {
                    Ok(raw_reading) => {
                        reading = light_from_byte(raw_reading.into());
                        // println!("Light {:?}   reading {:?}   previous {:?}", &id, &reading, &previous);
                        if reading != previous {
                            tx.send((id, Some(reading)));
                            previous = reading;
                        }
                    }
                    Err(e) => {
                        tx.send((id, None));
                    }
                }

                tokio::time::sleep(Duration::from_secs(DELAY_LIGHT_1)).await;
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
    fn id(&self) -> u8 {
        self.id
    }
    fn read(&self) -> Result<(f32), Box<dyn Error + '_>> {
        let pin = MOIST_SENSOR[self.id as usize - 1];
        let reading: f32;
        {
            let mut lock = self.adc.lock()?;
            reading = moist_from_byte(lock.analog_read_byte(pin)?);
        }

        Ok(reading)
    }
    fn init(
        &mut self,
        tx_moist: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<(), Box<dyn Error>> {
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
        let id = self.id;
        let adc = self.adc.clone();
        let pin = MOIST_SENSOR[self.id as usize - 1];
        Ok(tokio::spawn(async move {
            let mut previous = f32::MAX;
            loop {
                tokio::time::sleep(Duration::from_secs(DELAY_MOIST_1)).await;
                let reading: f32;
                let read_result: Result<u8, LinuxI2CError>;
                {
                    let mut lock = adc.lock().unwrap();
                    read_result = lock.analog_read_byte(pin);
                }
                match read_result {
                    Ok(raw_reading) => {
                        reading = moist_from_byte(raw_reading.into());
                        // println!("Moist {:?}   reading {:?}   previous {:?}", &id, &reading, &previous);
                        if reading != previous {
                            tx.send((id, Some(reading)));
                            previous = reading;
                        }
                    }
                    Err(e) => {
                        tx.send((id, None));
                    }
                }
            }
        }))
    }
}

/// Conversions
fn celcius_from_byte(value: f64) -> f64 {
    let coeff_b = 3950f64; // thermistor coefficient
    let res_r0 = 10000f64; // resistance @ room temperature
    let res_r1 = 1000f64; // resistance of R1
    let room_temperature_in_kelvin = 297.15f64;

    let res_r6: f64 = (res_r1 * value) / (256.0 - value);
    let kelvin: f64 =
        1.0 / ((1.0 / room_temperature_in_kelvin) + (1.0 / coeff_b) * (res_r6 / res_r0).ln());

    kelvin - 273.15
}
fn moist_from_byte(value: u8) -> f32 {
    // 115 = 100% moist, 215 = 0% moist
    // moist at 4v: 41-174                                  255-41=214  255-174=81  
    // (0f32 - value as f32 + 215f32)    
    // value as f32
    (255f32 - value as f32) - 75f32         // Värden från ca 5 - 140                
}
fn light_from_byte(value: u8) -> f32 {
    // 15(240) = dark, 40 = 5v LED up close, 208(47) = very light,
    (255f32 - value as f32)
}
