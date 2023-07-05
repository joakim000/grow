use grow::zone;

use core::time::Duration;
use std::time::Instant;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use tokio::task::JoinHandle;

use core::error::Error;
use core::result::Result;

use rppal::gpio::{Gpio, Trigger, Level, InputPin};
use rppal::pwm::Pwm;
use grow::zone::air::FanSetting;
use std::sync::Arc;
use parking_lot::Mutex;
use parking_lot::RwLock;
use super::conf::*;

#[allow(unused)]
pub struct PwmFan {
    id: u8,
    pwm_channel: Arc<Mutex<Pwm>>,
    rpm_pin: Arc<RwLock<InputPin>>,
    _fan_setting: FanSetting,
    feedback_task: Option<JoinHandle<()>>,
    control_task: Option<JoinHandle<()>>,
    cancel: CancellationToken,
    rpm: Arc<RwLock<Option<f32>>>,
}
impl zone::air::Fan for PwmFan {
    fn read(&mut self) -> Result<Option<f32>, Box<dyn Error + '_>> {
        // Ok(self.get_rpm())
        Ok(*self.rpm.read())
    }
    fn to_high(&self) -> Result<(), Box<dyn Error + '_>> {
        // println!("Fan set to high");
        let lock = self.pwm_channel.lock();
        Ok(lock.set_duty_cycle(1.0)?)
    }
    fn to_low(&self) -> Result<(), Box<dyn Error + '_>> {
        // println!("Fan set to low");
        let lock = self.pwm_channel.lock();
        Ok(lock.set_duty_cycle(0.5)?)
    }
    fn set_duty_cycle(
        &self,
        duty_cycle: f64,
    ) -> Result<(), Box<dyn Error + '_>> {
        // println!("Fan set to {:?}", &duty_cycle);
        let lock = self.pwm_channel.lock();
        Ok(lock.set_duty_cycle(duty_cycle)?)
    }
    // TODO: return result
    fn init(
        &mut self,
        tx_rpm: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
        rx_control: tokio::sync::broadcast::Receiver<FanSetting>,
    ) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.fan_feedback(tx_rpm)
                .expect("Error initializing feedback task"),
        );
        self.control_task = Some(
            self.fan_control(rx_control)
                .expect("Error initializing control task"),
        );
        Ok(())
    }

    fn id(&self) -> u8 {
        self.id
    }
}

impl PwmFan {
    pub fn new(id: u8, cancel: CancellationToken) -> Self {
        let mut rpm_pin = Gpio::new()
            .expect("Error on new pin")
            .get(PIN_FAN_1_RPM)
            .expect("Error on get pin")
            .into_input_pullup();
        let _ = rpm_pin.set_interrupt(Trigger::Both);
        let rpm_mutex = Arc::new(RwLock::new(rpm_pin));
        let pwm_channel = Pwm::with_frequency(
            PWM_FAN_1,
            PWM_FREQ_FAN_1,
            0.2,
            PWM_POLARITY_FAN_1,
            true,
        )
        .expect("Error setting up pwm");
        let pwm_mutex = Arc::new(Mutex::new(pwm_channel));
        Self {
            id,
            rpm_pin: rpm_mutex,
            pwm_channel: pwm_mutex,
            _fan_setting: FanSetting::High, // Initial
            feedback_task: None,
            control_task: None,
            cancel,
            rpm: Arc::new(RwLock::new(None)),
        }
    }
 
    // Uses poll interrupt
    #[allow(unused)]
    fn fan_feedback(
        &mut self,
        tx: broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let rpm_pin = self.rpm_pin.clone();

        let mut pulse_start: Instant = Instant::now();
        let mut pulse_duration: Duration = pulse_start.elapsed();
        let mut fan_rpm: Option<f32> = Default::default();
        let mut current = self.rpm.clone();
        let cancel = self.cancel.clone();

        let id = self.id;
        Ok(tokio::spawn(async move {
            loop {
                let mut fan_pulse_detected = true;
                {
                    let mut pin = rpm_pin.write();
                    let rpm_pulse = pin
                        .poll_interrupt(true, Some(Duration::from_millis(100)));
                    match rpm_pulse {
                        Ok(level_opt) => match level_opt {
                            None => fan_pulse_detected = false,
                            Some(_level) => pulse_start = Instant::now(),
                        },
                        Err(err) => {
                            println!("Error reading rpm: {}", err);
                        }
                    };
                    let rpm_pulse = pin
                        .poll_interrupt(true, Some(Duration::from_millis(100)));
                    match rpm_pulse {
                        Ok(level_opt) => match level_opt {
                            None => fan_pulse_detected = false,
                            Some(_level) => {
                                pulse_duration = pulse_start.elapsed()
                            }
                        },
                        Err(err) => {
                            println!("Error reading rpm: {}", err);
                        }
                    };
                }
                if fan_pulse_detected {
                    fan_rpm = Some(
                        (Duration::from_secs(60).as_micros() as f32
                            / pulse_duration.as_micros() as f32
                            / PULSES_PER_ROTATION)
                            .round(),
                    );
                } else {
                    fan_rpm = None;
                }
                // print!("Fan 1 duty cycle: {:?}   ", pwm.duty_cycle().unwrap());
                // print!("RPM pulse duration: {:?}   ", pulse_duration);
                // println!("Fan 1 RPM: {:?}", fan_rpm);

                match fan_rpm {
                    Some(rpm) => {
                        if current.read().is_none() {
                            let _ = tx.send((id, fan_rpm));
                        } 
                        else if (rpm - current.read().unwrap()).abs() >= FAN_1_DELTA {
                            let _ = tx.send((id, fan_rpm));
                        }
                    }
                    None => {
                        if current.read().is_some() {
                            let _ = tx.send((id, fan_rpm));
                        }
                    }
                }
                *current.write() = fan_rpm;
                if cancel.is_cancelled() {break}
                tokio::time::sleep(Duration::from_secs(DELAY_FAN_1)).await;
            }
        }))
    }

    // Uses async interrupt
    #[allow(unused)]
    fn fan_feedback2(
        &mut self,
        tx: broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let cancel = self.cancel.clone();
        let rpm_pin = self.rpm_pin.clone();
        let current = self.rpm.clone();

        let mut pulse_start: Instant = Instant::now();
        let mut pulse_duration: Duration = pulse_start.elapsed();
        let mut fan_rpm: Option<f32> = Default::default();
        Ok(tokio::spawn(async move {
            let _ = rpm_pin.write().set_async_interrupt(Trigger::Both, move |l| { 
                match l {
                    Level::High => {
                        pulse_start = Instant::now();
                    }
                    Level::Low => {
                        pulse_duration = pulse_start.elapsed();
                        fan_rpm = Some(
                            (Duration::from_secs(60).as_micros() as f32
                                / pulse_duration.as_micros() as f32
                                / PULSES_PER_ROTATION)
                                .round(),
                        );
                        if current.read().is_none() {
                            let _ = tx.send((id, fan_rpm));
                        } 
                        else if (fan_rpm.unwrap() - current.read().unwrap()).abs() >= FAN_1_DELTA
                        {
                            let _ = tx.send((id, fan_rpm));
                        
                        }
                        *current.write() = fan_rpm;
                    }
                }
            });
            
            println!("Fan speed initialized");
            cancel.cancelled().await;
        }))
    }
   


    fn fan_control(
        &self,
        mut rx: broadcast::Receiver<FanSetting>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let pwm = self.pwm_channel.clone();
        Ok(tokio::spawn(async move {
            while let Ok(data) = rx.recv().await {
                // println!("Received fansetting: {:?}", data);
                match data {
                    FanSetting::Off => {
                        let _ = pwm.lock().set_duty_cycle(0.0);
                    }
                    FanSetting::Low => {
                        let _ = pwm.lock().set_duty_cycle(0.2);
                    }
                    FanSetting::Medium => {
                        let _ = pwm.lock().set_duty_cycle(0.5);
                    }
                    FanSetting::High => {
                        let _ = pwm.lock().set_duty_cycle(1.0);
                    }
                }
                // println!("Current duty cycle: {:?}", pwm.lock().unwrap().duty_cycle());
            }
        }))
    }

    // fn get_rpm(&mut self) -> Option<f32> {
    //     let mut pulse_start: Instant = Instant::now();
    //     let mut pulse_duration: Duration = pulse_start.elapsed();
    //     let mut fan_pulse_detected = true;
    //     let mut pin = self.rpm_pin.lock().unwrap();

    //     let rpm_pulse =
    //         pin.poll_interrupt(true, Some(Duration::from_millis(100)));
    //     match rpm_pulse {
    //         Ok(level_opt) => match level_opt {
    //             None => fan_pulse_detected = false,
    //             Some(_level) => pulse_start = Instant::now(),
    //         },
    //         Err(err) => {
    //             eprintln!("Error reading rpm: {}", err);
    //         }
    //     };
    //     let rpm_pulse =
    //         pin.poll_interrupt(true, Some(Duration::from_millis(100)));
    //     match rpm_pulse {
    //         Ok(level_opt) => match level_opt {
    //             None => fan_pulse_detected = false,
    //             Some(_level) => pulse_duration = pulse_start.elapsed(),
    //         },
    //         Err(err) => {
    //             eprintln!("Error reading rpm: {}", err);
    //         }
    //     };
    //     match fan_pulse_detected {
    //         true => Some(
    //             (Duration::from_secs(60).as_micros() as f32
    //                 / pulse_duration.as_micros() as f32
    //                 / PULSES_PER_ROTATION)
    //                 .round(),
    //         ),
    //         false => None,
    //     }
    // }
}
