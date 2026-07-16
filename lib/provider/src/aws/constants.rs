use aws_config::{
    BehaviorVersion,
    timeout::TimeoutConfig,
};
use std::time::Duration;

pub const MAX_ATTEMPTS: u32 = 20;

pub const INITIAL_BACKOFF: Duration = Duration::from_secs(3);

pub const MAX_BACKOFF: Duration = Duration::from_secs(10);

const OPERATION_TIMEOUT: Duration = Duration::from_secs(300);

const OPERATION_ATTEMPT_TIMEOUT: Duration = Duration::from_millis(80000);

pub fn timeout_config() -> TimeoutConfig {
    TimeoutConfig::builder()
        .operation_timeout(OPERATION_TIMEOUT)
        .operation_attempt_timeout(OPERATION_ATTEMPT_TIMEOUT)
        .build()
}

pub fn behavior_version() -> BehaviorVersion {
    BehaviorVersion::latest()
}
