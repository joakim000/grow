use grow::zone;

use core::time::Duration;
use rppal::gpio::InputPin;
use std::time::Instant;
use tokio::sync::broadcast;

use tokio::task::JoinHandle;

use core::error::Error;
use core::result::Result;

use rppal::gpio::{Gpio, Trigger};
use rppal::pwm::{Pwm};

use grow::zone::air::FanSetting;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
pub type FanMutex = Arc<Mutex<Box<dyn Fan>>>;
pub type PwmMutex = Arc<Mutex<Pwm>>;
pub type RpmMutex = Arc<Mutex<InputPin>>;
use super::conf::*;
use grow::zone::air::Fan;

pub struct PwmFan {
    id: u8,
    // pwm_channel: Pwm,
    pwm_channel: PwmMutex,
    rpm_pin: RpmMutex,
    fan_setting: FanSetting,
    feedback_task: Option<JoinHandle<()>>,
    control_task: Option<JoinHandle<()>>,
}
impl zone::air::Fan for PwmFan {
    fn read(&mut self) -> Result<Option<f32>, Box<dyn Error + '_>> {
        Ok(self.get_rpm())
    }
    fn to_high(&self) -> Result<(), Box<dyn Error + '_>> {
        println!("Fan set to high");
        let lock = self.pwm_channel.lock()?;
        Ok(lock.set_duty_cycle(1.0)?)
    }
    fn to_low(&self) -> Result<(), Box<dyn Error + '_>> {
        println!("Fan set to low");
        let lock = self.pwm_channel.lock()?;
        Ok(lock.set_duty_cycle(0.5)?)
    }
    fn set_duty_cycle(&self, duty_cycle: f64) -> Result<(), Box<dyn Error + '_>> {
        println!("Fan set to {:?}", &duty_cycle);
        let lock = self.pwm_channel.lock()?;
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
    pub fn new(id: u8) -> Self {
        let mut rpm_pin = Gpio::new()
            .expect("Error on new pin")
            .get(PIN_FAN_1_RPM)
            .expect("Error on get pin")
            .into_input_pullup();
        let _ = rpm_pin.set_interrupt(Trigger::Both);
        let rpm_mutex = Arc::new(Mutex::new(rpm_pin));
        let pwm_channel =
            Pwm::with_frequency(PWM_FAN_1, PWM_FREQ_FAN_1, 0.2, PWM_POLARITY_FAN_1, true)
                .expect("Error setting up pwm");
        let pwm_mutex = Arc::new(Mutex::new(pwm_channel));
        Self {
            id,
            rpm_pin: rpm_mutex,
            pwm_channel: pwm_mutex,
            fan_setting: FanSetting::High, // Initial
            feedback_task: None,
            control_task: None,
        }
    }
    pub fn mutex(self) {
        let _m: FanMutex = Arc::new(Mutex::new(Box::new(self)));
    }

    fn fan_feedback(
        &mut self,
        tx: broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let rpm_pin = self.rpm_pin.clone();

        let mut pulse_start: Instant = Instant::now();
        let mut pulse_duration: Duration = pulse_start.elapsed();
        let mut fan_rpm: Option<f32> = Default::default();

        let id = self.id;
        let mut previous = f32::MAX;
        Ok(tokio::spawn(async move {
            loop {
                let mut fan_pulse_detected = true;
                {
                    let mut pin = rpm_pin.lock().unwrap();
                    let rpm_pulse = pin.poll_interrupt(true, Some(Duration::from_millis(100)));
                    match rpm_pulse {
                        Ok(level_opt) => match level_opt {
                            None => fan_pulse_detected = false,
                            Some(_level) => pulse_start = Instant::now(),
                        },
                        Err(err) => {
                            println!("Error reading rpm: {}", err);
                        }
                    };
                    let rpm_pulse = pin.poll_interrupt(true, Some(Duration::from_millis(100)));
                    match rpm_pulse {
                        Ok(level_opt) => match level_opt {
                            None => fan_pulse_detected = false,
                            Some(_level) => pulse_duration = pulse_start.elapsed(),
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
                        // println!("Fanrpm {:?}   reading {:?}   previous {:?}   delta {:?}", &id, &rpm, &previous, (&rpm-&previous).abs());
                        if (rpm - previous).abs() >= FAN_1_DELTA {
                            previous = rpm;
                            tx.send((id, fan_rpm));
                            // println!("sent rpm: {:?}", &fan_rpm);
                        }
                    }
                    None => {
                        tx.send((id, fan_rpm));
                        // println!("sent rpm: {:?}", &fan_rpm);
                    }
                }
                tokio::time::sleep(Duration::from_secs(DELAY_FAN_1)).await;
            }
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
                        let _ = pwm.lock().unwrap().set_duty_cycle(0.0);
                    }
                    FanSetting::Low => {
                        let _ = pwm.lock().unwrap().set_duty_cycle(0.2);
                    }
                    FanSetting::Medium => {
                        let _ = pwm.lock().unwrap().set_duty_cycle(0.5);
                    }
                    FanSetting::High => {
                        let _ = pwm.lock().unwrap().set_duty_cycle(1.0);
                    }
                }
                // println!("Current duty cycle: {:?}", pwm.lock().unwrap().duty_cycle());
            }
        }))
    }

    fn get_rpm(&mut self) -> Option<f32> {
        let mut pulse_start: Instant = Instant::now();
        let mut pulse_duration: Duration = pulse_start.elapsed();
        let mut fan_pulse_detected = true;
        let mut pin = self.rpm_pin.lock().unwrap();

        let rpm_pulse = pin.poll_interrupt(true, Some(Duration::from_millis(100)));
        match rpm_pulse {
            Ok(level_opt) => match level_opt {
                None => fan_pulse_detected = false,
                Some(_level) => pulse_start = Instant::now(),
            },
            Err(err) => {
                eprintln!("Error reading rpm: {}", err);
            }
        };
        let rpm_pulse = pin.poll_interrupt(true, Some(Duration::from_millis(100)));
        match rpm_pulse {
            Ok(level_opt) => match level_opt {
                None => fan_pulse_detected = false,
                Some(_level) => pulse_duration = pulse_start.elapsed(),
            },
            Err(err) => {
                eprintln!("Error reading rpm: {}", err);
            }
        };
        match fan_pulse_detected {
            true => Some(
                (Duration::from_secs(60).as_micros() as f32
                    / pulse_duration.as_micros() as f32
                    / PULSES_PER_ROTATION)
                    .round(),
            ),
            false => None,
        }
    }
}
