//! Official SDK mechanics; Vox alone owns its app identity, credential and product policy.
use infer_runtime_client::{Client, DiscoveryResolver};
use std::path::PathBuf;
mod job_provenance;
mod sdk;
pub mod sound;
pub(crate) mod speech;
#[cfg(test)]
use sdk::SdkAdapterError;
#[cfg(test)]
mod test_support;
pub use speech::{
    INFER_SPEECH_VOICE_CATALOG_REVISION, INFER_SPEECH_VOICE_ZH_BRIGHT_FEMALE_LANGUAGE,
    InferRuntimeSpeechExecutor, SPEECH_PRESETS, SpeechPreset, SpeechSynthesisControl,
    supported_speech_operation,
};
pub const INFER_RUNTIME_CONTRACT_VERSION: &str = infer_runtime_client::CONSUMER_CORE;
pub const INFER_RUNTIME_SPEECH_CAPABILITY: &str = "infer.audio.speech@20260811.1";

fn official_sdk(
    endpoint: &str,
    credential: PathBuf,
) -> Result<sdk::OfficialSdkClient, sdk::SdkAdapterError> {
    let resolver = resolver(endpoint)?;
    sdk::OfficialSdkClient::new(resolver, Some(credential))
}
fn resolver(endpoint: &str) -> Result<DiscoveryResolver, sdk::SdkAdapterError> {
    let resolver = DiscoveryResolver::local();
    if endpoint.is_empty() {
        Ok(resolver)
    } else {
        resolver
            .with_explicit_endpoint(endpoint)
            .map_err(|_| sdk::SdkAdapterError::InvalidEndpoint)
    }
}
pub fn credential_path() -> PathBuf {
    if let Some(path) = std::env::var_os("VOX_INFER_CREDENTIAL_FILE") {
        return path.into();
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".config/vox/infer-credential")
}
pub fn probe(endpoint: &str) -> serde_json::Value {
    let result = (|| {
        let resolver = resolver(endpoint).map_err(|_| "invalid_endpoint")?;
        let resolved = resolver.resolve().map_err(|_| "unavailable")?;
        let client = Client::with_discovery(resolver)
            .credential_file(credential_path())
            .build()
            .map_err(|_| "credential_unavailable")?;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| "unavailable")?;
        rt.block_on(async {
            let contract = tokio::time::timeout(std::time::Duration::from_secs(6),client.contract()).await.map_err(|_| "unavailable")?.map_err(probe_error)?;
            let caps = tokio::time::timeout(std::time::Duration::from_secs(6),client.capabilities()).await.map_err(|_| "unavailable")?.map_err(probe_error)?;
            caps.require_exact(INFER_RUNTIME_SPEECH_CAPABILITY).map_err(|_| "speech_unavailable")?;
            let jobs=tokio::time::timeout(std::time::Duration::from_secs(6),client.jobs(&[("limit","1")])).await.map_err(|_|"unavailable")?.map_err(probe_error)?;
            if jobs.jobs.iter().any(|j|j.app_id!="vox") {return Err("app_identity_mismatch");}
            Ok::<_, &str>(serde_json::json!({"ready": true, "core":contract.core_contract,"endpoint":resolved.endpoint,"credential_present":true,"credential_authenticated":true,"app_id":"vox"}))
        })
    })();
    result.unwrap_or_else(|code| serde_json::json!({"ready":false,"error":code,"credential_present":credential_path().is_file(),"app_id":"vox"}))
}
fn probe_error(error: infer_runtime_client::Error) -> &'static str {
    use infer_runtime_client::Error;
    match error {
        Error::Discovery(_) | Error::Transport(_) => "unavailable",
        Error::Credential(_) => "credential_unavailable",
        Error::ContractMismatch => "incompatible_contract",
        Error::Api { status, .. } if matches!(status.as_u16(), 401 | 403) => {
            "credential_unavailable"
        }
        _ => "invalid_response",
    }
}
