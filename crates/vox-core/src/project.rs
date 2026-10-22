//! Independent .vox bundle, working drafts, immutable sources and explicit audio acceptance.
pub mod materials;
mod model;
mod render;
mod segments;
pub use segments::*;
pub mod shape_transfer;
pub(crate) mod store;
pub use model::*;
pub use store::Project;
#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
#[error("{code}")]
pub struct Error {
    pub code: String,
}
impl Error {
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::new("file_error")
    }
}
impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::new("invalid_project")
    }
}
impl From<crate::ExecutionFailure> for Error {
    fn from(e: crate::ExecutionFailure) -> Self {
        Self::new(e.code)
    }
}
pub type Result<T> = std::result::Result<T, Error>;
