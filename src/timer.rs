use tokio::time::{self, Duration, Sleep};
// use tokio::sync::mpsc;
use std::pin::Pin;

pub struct ResettableTimer {
    sleep: Option<Pin<Box<Sleep>>>,
}

impl ResettableTimer {
    pub fn new() -> Self {
        Self { sleep: None }
    }

    pub fn reset(&mut self, duration: Duration) {
        self.sleep = Some(Box::pin(time::sleep(duration)));
    }

    pub async fn wait(&mut self) {
        if let Some(sleep) = self.sleep.as_mut() {
            sleep.as_mut().await;
        }
    }

    pub fn clear(&mut self) {
        self.sleep = None;
    }

    pub fn is_active(&self) -> bool {
        self.sleep.is_some()
    }
}