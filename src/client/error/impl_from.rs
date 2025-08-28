use crate::client::error::WebDavClientError;
use std::io::Error;
use tokio::sync::TryLockError;

impl From<std::io::Error> for WebDavClientError {
    fn from(value: Error) -> Self {
        Self::StdIoErr(value)
    }
}

impl From<reqwest::Error> for WebDavClientError {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestErr(value)
    }
}

impl From<String> for WebDavClientError {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<TryLockError> for WebDavClientError {
    fn from(value: TryLockError) -> Self {
        Self::TryLockError(value)
    }
}
