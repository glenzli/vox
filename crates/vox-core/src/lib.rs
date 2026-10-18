//! Script-led voice production. Domain, synthesis and project acceptance have separate owners.
mod domain;
pub use domain::*;
pub mod assembly;
pub mod audio;
mod executor;
mod failure;
pub mod handoff;
pub mod infer_runtime;
pub mod project;
mod provenance;
pub use executor::*;
pub use failure::*;
pub use infer_runtime::{
    INFER_SPEECH_VOICE_CATALOG_REVISION, InferRuntimeSpeechExecutor, SPEECH_PRESETS,
    SpeechSynthesisControl, supported_speech_operation,
};
pub use provenance::*;
