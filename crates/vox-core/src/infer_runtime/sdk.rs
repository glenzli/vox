//! Synchronous adapter for the official SDK, called only outside the GUI thread.
use infer_runtime_client::{
    AudioBytesResponse, Client, DiscoveryResolver, Error, JobSnapshot, SpeechRequest,
};
use std::{future::Future, path::PathBuf, sync::Mutex, time::Duration};

#[derive(Debug)]
pub(super) enum SdkAdapterError {
    Sdk(Error),
    Timeout,
    InvalidEndpoint,
    RuntimeUnavailable,
}
pub(super) trait InferRuntimeSdk: Send + Sync {
    fn synthesize_speech(
        &self,
        request: &SpeechRequest,
        timeout: Duration,
    ) -> Result<AudioBytesResponse, SdkAdapterError>;
    fn job(&self, id: &str) -> Result<JobSnapshot, SdkAdapterError>;
}
pub(super) struct OfficialSdkClient {
    client: Client,
    runtime: Mutex<tokio::runtime::Runtime>,
}
impl OfficialSdkClient {
    pub(super) fn new(
        resolver: DiscoveryResolver,
        credential: Option<PathBuf>,
    ) -> Result<Self, SdkAdapterError> {
        let builder = Client::with_discovery(resolver);
        let builder = if let Some(path) = credential {
            builder.credential_file(path)
        } else {
            builder
        };
        Ok(Self {
            client: builder.build().map_err(SdkAdapterError::Sdk)?,
            runtime: Mutex::new(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|_| SdkAdapterError::RuntimeUnavailable)?,
            ),
        })
    }
    fn run<T>(
        &self,
        timeout: Duration,
        future: impl Future<Output = infer_runtime_client::Result<T>>,
    ) -> Result<T, SdkAdapterError> {
        self.runtime
            .lock()
            .map_err(|_| SdkAdapterError::RuntimeUnavailable)?
            .block_on(async {
                tokio::time::timeout(timeout, future)
                    .await
                    .map_err(|_| SdkAdapterError::Timeout)?
                    .map_err(SdkAdapterError::Sdk)
            })
    }
}
impl InferRuntimeSdk for OfficialSdkClient {
    fn synthesize_speech(
        &self,
        r: &SpeechRequest,
        t: Duration,
    ) -> Result<AudioBytesResponse, SdkAdapterError> {
        self.run(t, self.client.synthesize_speech(r))
    }
    fn job(&self, id: &str) -> Result<JobSnapshot, SdkAdapterError> {
        self.run(Duration::from_secs(10), self.client.job(id))
    }
}
pub(super) fn execution_failure(error: SdkAdapterError) -> (String, bool) {
    let (code, retry) = match error {
        SdkAdapterError::InvalidEndpoint => ("infer_invalid_endpoint".into(), false),
        SdkAdapterError::RuntimeUnavailable => ("infer_unavailable".into(), true),
        SdkAdapterError::Timeout | SdkAdapterError::Sdk(Error::Transport(_)) => {
            ("infer_unavailable".into(), true)
        }
        SdkAdapterError::Sdk(Error::Credential(_)) => ("credential_unavailable".into(), false),
        SdkAdapterError::Sdk(Error::Api { status, code, .. }) => {
            (code, status.is_server_error() || status.as_u16() == 429)
        }
        SdkAdapterError::Sdk(_) => ("infer_invalid_response".into(), false),
    };
    (code, retry)
}
