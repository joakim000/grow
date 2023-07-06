use core::error::Error;
use core::time::Duration;
use std::sync::Arc;
use parking_lot::RwLock;
use rppal::gpio::{Gpio, Trigger, Level, InputPin};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::conf::{BUTTON_1_PIN, BUTTON_2_PIN};
use grow::ops::io::{ButtonPanel, ButtonInput};
const DEBOUNCE_DELAY: Duration = Duration::from_micros(50);
pub struct PushButtons {
    button_handler: Option<JoinHandle<()>>,
    cancel: CancellationToken,
}
impl ButtonPanel for PushButtons {
    fn init(
        &mut self,
        from_buttons: tokio::sync::broadcast::Sender<ButtonInput>,
    ) -> Result<(), Box<dyn Error>> {

        let cancel = self.cancel.clone();
        self.button_handler = Some(tokio::spawn(async move {

            let from_btn1 = from_buttons.clone();
            let btn_1 = setup_pin(BUTTON_1_PIN);
            let _ = btn_1.0.write().set_async_interrupt(Trigger::Both, move |l| { 
                match l {
                    Level::High => {
                        std::thread::sleep(DEBOUNCE_DELAY);
                        if btn_1.1.read().is_high() { let _ = from_btn1.send(ButtonInput::OneDown); }
                    }
                    Level::Low => {
                        std::thread::sleep(DEBOUNCE_DELAY);
                        if btn_1.1.read().is_low() { let _ = from_btn1.send(ButtonInput::OneUp); }
                    }
                }
                // println!("Btn 1: {:?}", &l);

            let from_btn2 = from_buttons.clone();
            let btn_2 = setup_pin(BUTTON_2_PIN);
                match l {
                    Level::High => {
                        std::thread::sleep(DEBOUNCE_DELAY);
                        if btn_2.1.read().is_high() { let _ = from_btn2.send(ButtonInput::TwoDown); }
                    }
                    Level::Low => {
                        std::thread::sleep(DEBOUNCE_DELAY);
                        if btn_2.1.read().is_low() { let _ = from_btn2.send(ButtonInput::TwoUp); }
                    }
                }
                // println!("Btn 2: {:?}", &l);

            });
            // println!("Button pins initialized");
            cancel.cancelled().await;
        }));
        
        Ok(())
    }
}

impl PushButtons {
    pub fn new(cancel: CancellationToken) -> Self {
        Self {
            button_handler: None,
            cancel,
        }
    }
}

fn setup_pin(pin: u8) -> (Arc<RwLock<InputPin>>, Arc<RwLock<InputPin>>) {
    let btn = Gpio::new().expect("GPIO error")
        .get(pin).expect("GPIO pin error")
        .into_input_pulldown();
    let btn = Arc::new(RwLock::new(btn));
    let btn_clone = btn.clone();

    (btn, btn_clone)
}