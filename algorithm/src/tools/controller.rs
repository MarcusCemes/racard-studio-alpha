use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use thiserror::Error;

#[derive(Clone, Default)]
pub struct ExecutionController(Arc<AtomicBool>);

#[derive(Debug, Error)]
#[error("Interrupted")]
pub struct InterruptRequest;

impl ExecutionController {
    pub fn request_stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn assert(&self) -> Result<(), InterruptRequest> {
        match self.0.load(Ordering::Relaxed) {
            true => Err(InterruptRequest),
            false => Ok(()),
        }
    }
}
