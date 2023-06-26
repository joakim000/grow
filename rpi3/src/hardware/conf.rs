use pcf8591::{Pin, PCF8591};
use rppal::gpio::{Gpio, Level, Trigger};
use rppal::pwm::{Channel, Polarity, Pwm};

// GPIO
pub const ACTIVITY_LED_PIN: u8 = 4; //BCM 17 phys 11
pub const BUTTON_1_PIN: u8 = 24; // 24p18
pub const BUTTON_2_PIN: u8 = 25; // 25p22
pub const INDICATORS_SR_DATA: u8 = 26; // 5p29
pub const INDICATORS_SR_ENABLE: u8 = 19; // 6p31
pub const INDICATORS_SR_CLK: u8 = 6; // 13p33
pub const INDICATORS_SR_LATCH: u8 = 13; // 19p35
pub const INDICATORS_SR_RESET: u8 = 5; // 26p37

// Fan
pub const PIN_FAN_1_RPM: u8 = 23; //BCM 26 phys 16
pub const PULSES_PER_ROTATION: f32 = 4.0;
pub const PWM_FREQ_FAN_1: f64 = 25000.0; // PWM frequency target 25 kHz
pub const PWM_FAN_1: rppal::pwm::Channel = rppal::pwm::Channel::Pwm0;
pub const PWM_POLARITY_FAN_1: rppal::pwm::Polarity = rppal::pwm::Polarity::Inverse;
pub const FAN_1_DELTA: f32 = 50f32;

// I2C
pub const ADC_1_ADDR: u16 = 0x48;
pub const ADC_1_BUS: &str = "/dev/i2c-1";
pub const ADC_1_VREF: f64 = 5.0;
pub const LIGHT_SENSOR: [Pin; 1] = [Pin::AIN0];
pub const TEMP_SENSOR: [Pin; 1] = [Pin::AIN1];
pub const MOIST_SENSOR: [Pin; 2] = [Pin::AIN2, Pin::AIN3];

pub const DISPLAY_BUS: &str = "/dev/i2c-1";
pub const DISPLAY_ADDR: u16 = 0xc3;

// LPU
pub const HUB_ADDR: &str = "90:84:2B:70:93:75";
pub const REMOTE_ADDR: &str = "E4:E1:12:A0:39:07";
pub const ARM_ROT_ADDR: u8 = 0x00; // Ext hub port A
pub const ARM_EXTENSION_ADDR: u8 = 0x01; // Ext hub port B
pub const PUMP_ADDR: u8 = 0x02; // Ext hub port C
pub const TANK_SENSOR_ADDR: u8 = 0x03; // Ext hub port D

// Poll intervals
pub const DELAY_TEMP_1: u64 = 10;
pub const DELAY_MOIST_1: u64 = 10;
pub const DELAY_MOIST_2: u64 = 10;
pub const DELAY_LIGHT_1: u64 = 2;
pub const DELAY_FAN_1: u64 = 2;
