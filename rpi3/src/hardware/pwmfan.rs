use grow::zone;
use grow::zone::Handles;

// use anyhow;
use core::time::Duration;
use rppal::gpio::InputPin;
use std::time::Instant;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use core::error::Error;
use core::result::Result;

use rppal::gpio::{Gpio, Level, Trigger};
use rppal::pwm::{Channel, Polarity, Pwm};

use grow::zone::air::FanSetting;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::sync::Mutex;
pub type FanMutex = Arc<Mutex<Box<dyn Fan>>>;
pub type PwmMutex = Arc<Mutex<Pwm>>;
use super::conf::*;
use grow::zone::air::Fan;

pub struct PwmFan {
    id: u8,
    // pwm_channel: Pwm,
    pwm_channel: PwmMutex,
    fan_setting: FanSetting,
    feedback_task: Option<JoinHandle<()>>,
    control_task: Option<JoinHandle<()>>,
}
impl zone::air::Fan for PwmFan {
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
        let pwm_channel =
            Pwm::with_frequency(PWM_FAN_1, PWM_FREQ_FAN_1, 0.0, PWM_POLARITY_FAN_1, true)
                .expect("Error setting up pwm");
        let pwm_mutex = Arc::new(Mutex::new(pwm_channel));
        Self {
            id,
            pwm_channel: pwm_mutex,
            fan_setting: FanSetting::High, // Initial
            feedback_task: None,
            control_task: None,
        }
    }
    pub fn mutex(self) {
        let m: FanMutex = Arc::new(Mutex::new(Box::new(self)));
    }

    fn fan_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let mut rpm_pin = Gpio::new()
            .expect("Error on new pin")
            .get(PIN_FAN_1_RPM)
            .expect("Error on get pin")
            .into_input_pullup();
        rpm_pin.set_interrupt(Trigger::Both)?;

        let mut pulse_start: Instant = Instant::now();
        let mut pulse_duration: Duration = pulse_start.elapsed();
        let mut fan_rpm: Option<f32> = Default::default();

        let id = self.id;
        let mut previous = f32::MAX;
        Ok(tokio::spawn(async move {
            loop {
                let mut fan_pulse_detected = true;
                let rpm_pulse = rpm_pin.poll_interrupt(true, Some(Duration::from_millis(100)));
                match rpm_pulse {
                    Ok(level_opt) => match level_opt {
                        None => fan_pulse_detected = false,
                        Some(_level) => pulse_start = Instant::now(),
                    },
                    Err(err) => {
                        println!("Error reading rpm: {}", err);
                    }
                };
                let rpm_pulse = rpm_pin.poll_interrupt(true, Some(Duration::from_millis(100)));
                match rpm_pulse {
                    Ok(level_opt) => match level_opt {
                        None => fan_pulse_detected = false,
                        Some(_level) => pulse_duration = pulse_start.elapsed(),
                    },
                    Err(err) => {
                        println!("Error reading rpm: {}", err);
                    }
                };
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
                        if (rpm-previous).abs() >= FAN_1_DELTA {
                            previous = rpm;
                            tx.send( (id, fan_rpm) );
                        }
                    }
                    None => {
                        tx.send( (id, fan_rpm) );
                    }
                }

                tx.send((id, fan_rpm));
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
                println!("Received fansetting: {:?}", data);
                match data {
                    FanSetting::Off => {
                        let _ = pwm.lock().unwrap().set_duty_cycle(0.0);
                    }
                    FanSetting::Low => {
                        let _ = pwm.lock().unwrap().set_duty_cycle(0.5);
                    }
                    FanSetting::High => {
                        let _ = pwm.lock().unwrap().set_duty_cycle(1.0);
                    }
                }
                println!("Current duty cycle: {:?}", pwm.lock().unwrap().duty_cycle());
            }
        }))
    }
}
