use core::error::Error;

use grow::ops::input::ButtonInput;
use grow::ops::input::ButtonPanel;
use tokio::task::JoinHandle;

pub struct PushButtons {
    // id: u8,
    // hub: HubMutex,
    feedback_task: Option<JoinHandle<()>>,
}
impl ButtonPanel for PushButtons {
    // fn id(&self) -> u8;
    fn init(
        &mut self,
        _tx_rc: tokio::sync::broadcast::Sender<ButtonInput>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl PushButtons {
    pub fn new() -> Self {
        Self {
            // id,
            feedback_task: None,
        }
    }
}
