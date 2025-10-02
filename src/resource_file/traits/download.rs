use crate::global_config::global_config::GlobalConfig;
use crate::resource_file::impl_traits::impl_download::handle_download::HandleDownloadError;
use crate::resource_file::impl_traits::impl_download::{
    HandleMountedError, HandleUnmountedError, PreprocessingSavePathError,
};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error(transparent)]
    PreprocessingSavePathError(#[from] PreprocessingSavePathError),
    #[error(transparent)]
    HandleMountedError(#[from] HandleMountedError),
    #[error(transparent)]
    HandleUnmountedError(#[from] HandleUnmountedError),
    #[error(transparent)]
    HandleDownloadError(#[from] HandleDownloadError),
}

pub type TDownloadConfig = GlobalConfig;

#[async_trait]
pub trait Download {
    async fn download(
        self,
        output_absolute_path: &str,
    ) -> Result<Arc<Self>, DownloadError>;
}
