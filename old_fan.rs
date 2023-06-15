 /*
 
    // Init fan control
    let pwm = Pwm::with_frequency(Channel::Pwm0, FAN_PWM_FREQ, 0.0, Polarity::Inverse, true)?;
    let mut fan_setting = FanSetting::High;
    println!("Initialized PWM fan control, duty cycle 0.0");
    let mut fan_1_rpm_in = Gpio::new()?.get(PIN_FAN_RPM)?.into_input_pullup();
    fan_1_rpm_in.set_interrupt(Trigger::Both)?;
    let pulses_per_rotation: f32 = 4.0;
    let mut pulse_start: Instant = Instant::now(); // TODO box
    let mut pulse_duration: Duration = pulse_start.elapsed(); // TODO box
    let mut fan_1_rpm: f32; // = 0.0;
    let mut rpm_pulse: Result<Option<rppal::gpio::Level>, rppal::gpio::Error>;


     // Fan data from config file
    let temperature_fan_low: f32 = 25.0;
    let temperature_fan_high: f32 = 35.0;
    let temperature_warning: f32 = 40.0;


 // Control fan
        let mut fan_pulse_detected = true;
        rpm_pulse = fan_1_rpm_in.poll_interrupt(true, Some(Duration::from_millis(500)));
        match rpm_pulse {
            Ok(level_opt) => 
                match level_opt{
                    None => fan_pulse_detected = false,
                    Some(_level) => pulse_start = Instant::now(),
                }    
            Err(err) => println!("Error reading rpm: {}", err),
        };
        rpm_pulse = fan_1_rpm_in.poll_interrupt(true, Some(Duration::from_millis(500)));
        match rpm_pulse {
            Ok(level_opt) => 
                match level_opt {
                    None => fan_pulse_detected = false,
                    Some(_level) => pulse_duration = pulse_start.elapsed(),
                }    
            Err(err) => { 
                println!("Error reading rpm: {}", err);
                fan_pulse_detected = false;
            },
        };        
        if fan_pulse_detected {
            fan_1_rpm = (Duration::from_secs(60).as_micros() as f32 / 
                         pulse_duration.as_micros() as f32 / pulses_per_rotation).round();
        } else {
            fan_1_rpm = 0.0;
        }
        print!("Fan 1 duty cycle: {:?}   ", pwm.duty_cycle().unwrap());
        print!("RPM pulse duration: {:?}   ", pulse_duration);
        println!("Fan 1 RPM: {}", fan_1_rpm);

        
        if temperature_1 > temperature_fan_high {
            if fan_setting != FanSetting::High {
                println!("{} Fan HIGH @ {}C", now.to_rfc2822(), temperature_1.round());
            }
            fan_setting = FanSetting::High;
            pwm.set_duty_cycle(1.0)?;
        } else if temperature_1 > temperature_fan_low {
            if fan_setting != FanSetting::Low {
                println!("{} Fan LOW @ {}C", now.to_rfc2822(), temperature_1.round());
            }
            fan_setting = FanSetting::Low;
            pwm.set_duty_cycle(0.5)?;
        } else {
            if fan_setting != FanSetting::Off {
                println!("{} Fan OFF @ {}C", now.to_rfc2822(), temperature_1.round());
            }
            fan_setting = FanSetting::Off;
            pwm.set_duty_cycle(0.0)?;
        }

        if temperature_1 > temperature_warning {
            println!("WARNING temp_1: {} (limit: {}", temperature_1, temperature_warning);
        }


*/