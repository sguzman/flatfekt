use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("runtime not implemented yet")]
    NotImplemented,
}

pub struct Runtime;

impl Runtime {
    #[instrument(level = "info", skip_all)]
    pub fn new() -> Self {
        Self
    }

    #[instrument(level = "info", skip_all)]
    pub fn run(self) -> Result<(), RuntimeError> {
        Err(RuntimeError::NotImplemented)
    }
}

