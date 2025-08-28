use tokio::sync::TryLockError;
mod impl_display;
mod impl_from;

#[derive(Debug)]
pub enum WebDavClientError {
    RequestErr(reqwest::Error),
    StdIoErr(std::io::Error),
    String(String),
    InvalidHeaderValue(String), // 这个就不用装http库了，直接输出string就行
    SerdeJsonErr(serde_json::Error),
    SerdeErr(String),
    ParseUrlErr(String),
    TryLockError(TryLockError),
    NotFindClient(String)
}
