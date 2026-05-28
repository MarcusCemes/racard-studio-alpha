use std::sync::Arc;

use tokio::sync::Mutex;

use algorithm::ExecutionController;

#[derive(Default)]
pub struct Handle<P>(pub Mutex<Option<(Arc<P>, ExecutionController)>>);
