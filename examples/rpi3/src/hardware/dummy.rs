// use async_trait::async_trait;
// use std::sync::Arc;
// use tokio::sync::Mutex;
use core::error::Error;
use core::fmt::Debug;
use core::result::Result;
use core::time::Duration;
// use parking_lot::RwLock;
// use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
// use tokio_util::sync::CancellationToken;

// use super::conf::*;
use grow::zone;
// use grow::zone::light::LampState;

#[derive(Debug)]
pub struct DummyMoistureSensor {
    id: u8,
    value: f32,
    feedback_task: Option<JoinHandle<()>>,
}
impl zone::water::MoistureSensor for DummyMoistureSensor {
    fn id(&self) -> u8 {
        self.id
    }
    fn read(&self) -> Result<f32, Box<dyn Error + '_>> {
        Ok(self.value)
    }
    fn init(
        &mut self,
        tx_moist: tokio::sync::broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<(), Box<dyn Error>> {
        self.feedback_task = Some(
            self.moist_feedback(tx_moist)
                .expect("Error initializing feedback task"),
        );

        Ok(())
    }
}
impl DummyMoistureSensor {
    pub fn new(id: u8, value: f32) -> Self {
        Self {
            id,
            value,
            feedback_task: None,
        }
    }
    fn moist_feedback(
        &self,
        tx: broadcast::Sender<(u8, Option<f32>)>,
    ) -> Result<JoinHandle<()>, Box<dyn Error>> {
        let id = self.id;
        let value = self.value;
        Ok(tokio::spawn(async move {
            loop {
                let _ = tx.send((id, Some(value)));
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        }))
    }
}