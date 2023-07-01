use core::error::Error;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use async_trait::async_trait;

use rppal::gpio::{Gpio, Trigger};

use super::conf::{BUTTON_1_PIN, BUTTON_2_PIN};
use grow::ops::input::ButtonInput;
use grow::ops::input::ButtonPanel;

pub struct PushButtons {
    // id: u8,
    // hub: HubMutex,
    feedback_task: Option<JoinHandle<()>>,
    cancel: CancellationToken,
}
// #[async_trait]
impl ButtonPanel for PushButtons {
    // fn id(&self) -> u8;
    fn init(
        &mut self,
        _from_buttons_tx: tokio::sync::broadcast::Sender<ButtonInput>,
    ) -> Result<(), Box<dyn Error>> {

        let cancel = self.cancel.clone();
        let button_handler = tokio::spawn(async move {
            let mut btn_1 = Gpio::new().expect("GPIO error").get(BUTTON_1_PIN).expect("GPIO pin error").into_input_pulldown();
            let mut btn_2 = Gpio::new().expect("GPIO error").get(BUTTON_2_PIN).expect("GPIO pin error").into_input_pulldown();
            println!("==== Button pins initialized ====");
            btn_1.set_async_interrupt(Trigger::Both, |l| println!("Btn 111: {:?}", l));
            btn_2.set_async_interrupt(Trigger::Both, |l| println!("Btn 2 2 2: {:?}", l));

            cancel.cancelled().await;
        });
        
        Ok(())
    }
}

impl PushButtons {
    pub fn new(cancel: CancellationToken) -> Self {
        
        
        
        Self {
            // id,
            feedback_task: None,
            cancel,
        }
    }
}
