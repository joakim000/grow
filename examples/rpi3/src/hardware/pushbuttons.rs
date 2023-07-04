use core::error::Error;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use core::time::Duration;
use rppal::gpio::{Gpio, Trigger, Level};

use super::conf::{BUTTON_1_PIN, BUTTON_2_PIN};
use grow::ops::io::{ButtonPanel, ButtonInput};

const DEBOUNCE_DELAY: Duration = Duration::from_micros(50);
pub struct PushButtons {
    button_handler: Option<JoinHandle<()>>,
    cancel: CancellationToken,
}
// #[async_trait]
impl ButtonPanel for PushButtons {
    // fn id(&self) -> u8;
    fn init(
        &mut self,
        from_buttons: tokio::sync::broadcast::Sender<ButtonInput>,
    ) -> Result<(), Box<dyn Error>> {

        let cancel = self.cancel.clone();
        self.button_handler = Some(tokio::spawn(async move {
            let mut btn_1 = Gpio::new().expect("GPIO error").get(BUTTON_1_PIN).expect("GPIO pin error").into_input_pulldown();
            let mut btn_2 = Gpio::new().expect("GPIO error").get(BUTTON_2_PIN).expect("GPIO pin error").into_input_pulldown();
            let _from_btn1 = from_buttons.clone();
            let from_btn2 = from_buttons.clone();
            let _ = btn_1.set_async_interrupt(Trigger::Both, move |l| { 
                match l {
                    Level::High => {
                        std::thread::sleep(DEBOUNCE_DELAY);
                        if &btn_1.is_high().clone() { let _ = from_btn1.send(ButtonInput::OneDown); }
                    }
                    Level::Low => {
                        std::thread::sleep(DEBOUNCE_DELAY);
                        // if *(&btn_1.is_low()) { let _ = from_btn1.send(ButtonInput::OneUp); }
                    }
                }
                // println!("Btn 1: {:?}", &l);
            });
            let _ = btn_2.set_async_interrupt(Trigger::Both, move |l| { 
                match l {
                    Level::High => {let _ = from_btn2.send(ButtonInput::TwoDown); }
                    Level::Low => {let _ = from_btn2.send(ButtonInput::TwoUp); }
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
