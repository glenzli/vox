mod audio;
mod content;
mod ids;
pub mod speech_script;
pub use audio::*;
pub use content::*;
pub use ids::*;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid audio value")]
    InvalidAudioValueContract,
    #[error("Invalid voice alias (limit {max_bytes} bytes)")]
    InvalidPresetVoiceAlias { max_bytes: usize },
    #[error("Invalid voice catalog (limit {max_bytes} bytes)")]
    InvalidVoiceCatalogRevision { max_bytes: usize },
    #[error("Invalid voice authorization (limit {max_bytes} bytes)")]
    InvalidVoiceAuthorization { max_bytes: usize },
    #[error("Invalid synthesis settings (language limit {max_language_bytes} bytes)")]
    InvalidSpeechSynthesisOperation { max_language_bytes: usize },
    #[error("Invalid media type (limit {max_bytes} bytes)")]
    InvalidMediaType { max_bytes: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "contract", rename_all = "snake_case")]
pub enum ArtifactContentContract {
    AudioClip(AudioValueContract),
}
