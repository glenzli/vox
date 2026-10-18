use super::sdk::{InferRuntimeSdk, SdkAdapterError};
use infer_runtime_client::{AudioBytesResponse, JobSnapshot, SpeechRequest};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
pub(super) struct FakeSdk {
    pub speeches: Mutex<VecDeque<Result<AudioBytesResponse, SdkAdapterError>>>,
    pub jobs: Mutex<VecDeque<Result<JobSnapshot, SdkAdapterError>>>,
    pub seen_speeches: Arc<Mutex<Vec<SpeechRequest>>>,
}
impl FakeSdk {
    pub(super) fn new() -> Self {
        Self {
            speeches: Mutex::new(VecDeque::new()),
            jobs: Mutex::new(VecDeque::new()),
            seen_speeches: Arc::default(),
        }
    }
    pub(super) fn speech(self, r: AudioBytesResponse) -> Self {
        self.speeches.lock().unwrap().push_back(Ok(r));
        self
    }
    pub(super) fn job(self, r: JobSnapshot) -> Self {
        self.jobs.lock().unwrap().push_back(Ok(r));
        self
    }
}
impl InferRuntimeSdk for FakeSdk {
    fn synthesize_speech(
        &self,
        r: &SpeechRequest,
        _: Duration,
    ) -> Result<AudioBytesResponse, SdkAdapterError> {
        self.seen_speeches.lock().unwrap().push(r.clone());
        self.speeches
            .lock()
            .unwrap()
            .pop_front()
            .expect("fake speech")
    }
    fn job(&self, _: &str) -> Result<JobSnapshot, SdkAdapterError> {
        self.jobs.lock().unwrap().pop_front().expect("fake job")
    }
}
