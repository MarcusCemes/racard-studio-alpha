use tokio::sync::Mutex;

use algorithm::ExecutionController;

pub struct ActiveOperation {
    pub controller: ExecutionController,
}

#[derive(Default)]
pub struct ActiveOperationState(pub Mutex<Option<ActiveOperation>>);
