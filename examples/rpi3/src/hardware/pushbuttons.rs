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
    btn1: Arc<RwLock<InputPin>>,
    btn2: Arc<RwLock<InputPin>>,

}
impl ButtonPanel for PushButtons {
    fn init(
        &mut self,
        from_buttons: tokio::sync::broadcast::Sender<ButtonInput>,
    ) -> Result<(), Box<dyn Error>> {
        let cancel = self.cancel.clone();
        let btn_1 = self.btn1.clone();
        let btn_2 = self.btn2.clone();
        
        self.button_handler = Some(tokio::spawn(async move {
            let from_btn1 = from_buttons.clone();
            let btn1 = ( btn_1.clone(), btn_1.clone() );
            let _ = btn1.0.write().set_async_interrupt(Trigger::Both, move |l| { 
                match debounce(btn1.1.clone(), l) {
                    DebounceResponse::Held => {
                        let _ = from_btn1.send(ButtonInput::OneDown);
                    }   
                    DebounceResponse::Released => {
                        let _ = from_btn1.send(ButtonInput::OneUp);
                    }
                    _ => ()   
                }
                // println!("Btn 1: {:?}", &l);
            });

            let from_btn2 = from_buttons.clone();
            let btn2 = ( btn_2.clone(), btn_2.clone() );
            let _ = btn2.0.write().set_async_interrupt(Trigger::Both, move |l| { 
                match debounce(btn2.1.clone(), l) {
                    DebounceResponse::Held => {
                        let _ = from_btn2.send(ButtonInput::TwoDown);
                    }   
                    DebounceResponse::Released => {
                        let _ = from_btn2.send(ButtonInput::TwoUp);
                    }
                    _ => ()   
                }
                println!("Btn 2: {:?}", &l);
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
            btn1: setup_pin(BUTTON_1_PIN),
            btn2: setup_pin(BUTTON_2_PIN),
        }
    }
}

fn setup_pin(pin: u8) -> Arc<RwLock<InputPin>> {
    let btn = Gpio::new().expect("GPIO error")
        .get(pin).expect("GPIO pin error")
        .into_input_pulldown();

    Arc::new(RwLock::new(btn))
}
fn debounce(btn: Arc<RwLock<InputPin>>, l: Level) -> DebounceResponse {
    match l {
        Level::High => {
            std::thread::sleep(DEBOUNCE_DELAY);
            if btn.read().is_high() { DebounceResponse::Held }
            else { DebounceResponse::Ignore }
        }
        Level::Low => {
            std::thread::sleep(DEBOUNCE_DELAY);
            if btn.read().is_low() { DebounceResponse::Released }
            else { DebounceResponse::Ignore }
        }
    }
}
enum DebounceResponse {
    Ignore,
    // Single,
    // Double,
    // Long,
    Held,
    Released,
}