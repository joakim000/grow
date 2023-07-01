/// Hardware config consts
pub mod conf;
pub use conf::*;
/// Lego Powered Up
pub mod lpu;
pub mod lpu_remote;

/// I2C -> ADC/DAC with connected sensors and LED
pub mod pcf8591;

/// I2C -> OLED display
pub mod ssd1306;

/// PWM controlled case fan
pub mod pwmfan;

/// 74HC595 with 8 connected LEDs
pub mod regshift_leds;

// Some GPIO momentary buttons
pub mod pushbuttons;
