use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{code}: {message}")]
pub struct ExecutionFailure {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub executor_job_id: Option<String>,
}
impl ExecutionFailure {
    pub fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
            executor_job_id: None,
        }
    }
    pub fn with_executor_job_id(mut self, job_id: impl Into<String>) -> Self {
        self.executor_job_id = Some(job_id.into());
        self.retryable = false;
        self
    }
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Invalid capability")]
    InvalidCapabilityId,
    #[error("Invalid executor identity")]
    InvalidExecutorIdentity,
    #[error("Invalid media type")]
    InvalidMediaType,
    #[error("Input bytes do not match their content identity")]
    InvalidInput,
}
