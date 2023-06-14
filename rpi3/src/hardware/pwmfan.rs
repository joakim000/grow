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
use tokio::sync::Mutex;
pub type FanMutex = Arc<Mutex<Box<dyn Fan>>>;
use super::conf::*;
use grow::zone::air::Fan;

pub struct PwmFan {
    id: u8,
    pwm_channel: Pwm,
    fan_setting: FanSetting,
    feedback_task: Option<JoinHandle<()>>,
    control_task: Option<JoinHandle<()>>,
}
impl zone::air::Fan for PwmFan {
    fn to_high(&self) -> Result<(), Box<dyn Error>> {
        println!("Fan set to high");
        Ok(self.pwm_channel.set_duty_cycle(1.0)?)
    }
    fn to_low(&self) -> Result<(), Box<dyn Error>> {
        println!("Fan set to low");
        Ok(self.pwm_channel.set_duty_cycle(0.5)?)
    }
    // TODO: return result
    fn init(
        &mut self,
        tx_rpm: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
        rx_control: tokio::sync::broadcast::Receiver<FanSetting>,
    ) -> FanMutex {
        self.feedback_task = Some(
            self.fan_feedback(tx_rpm)
                .expect("Error initializing feedback task"),
        );
        let m: FanMutex = Arc::new(Mutex::new(Box::new(*self)));
        m
    }
}
impl PwmFan {
    // pub fn new(id: u8) -> Self {
    //     Self {
    //         id,
    //         pwm_channel: Pwm::with_frequency(
    //             PWM_FAN_1,
    //             PWM_FREQ_FAN_1,
    //             0.0,
    //             PWM_POLARITY_FAN_1,
    //             true,
    //         )
    //         .expect("Error setting up pwm"),
    //         fan_setting: FanSetting::High, // Initial
    //         feedback_task: None,
    //         control_task: None,
    //     }
    // }

    pub fn new(id: u8) -> FanMutex {
        let s= Self {
            id,
            pwm_channel: Pwm::with_frequency(
                PWM_FAN_1,
                PWM_FREQ_FAN_1,
                0.0,
                PWM_POLARITY_FAN_1,
                true,
            )
            .expect("Error setting up pwm"),
            fan_setting: FanSetting::High, // Initial
            feedback_task: None,
            control_task: None,
        };
        Arc::new(Mutex::new(Box::new(s)))

    }
    pub fn mutex(self) {
        let m: FanMutex = Arc::new(Mutex::new(Box::new(self)));
    }
    // pub fn feedback_rx(&self) -> tokio::sync::broadcast::Receiver<Option<f32>> {
    //     self.rpm_tx.as_ref().expect("No receiver found").subscribe()
    // }

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

        let id = self.id.clone();

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

                tx.send((id, fan_rpm));
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }))
    }

    // async fn fan_control(&self, rx: mpsc::Receiver<FanSetting>) -> Result<JoinHandle<()>> {
    //     Ok(tokio::spawn(async move {
    //         while let Ok(data) = rx.recv().await {
    //             println!("Received fansetting: {:?}", data);
    //             match data {
    //                 FanSetting::Off => {
    //                     let _ = pwm.set_duty_cycle(0.0)?;
    //                 },
    //                 FanSetting::Low => {
    //                     let _ = pwm.set_duty_cycle(0.5)?;
    //                 },
    //                 FanSetting::High => {
    //                     let _ = pwm.set_duty_cycle(1.0)?;
    //                 },
    //             }
    //         }
    //     }))
    // }
}
