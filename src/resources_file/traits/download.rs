use crate::download_config::DownloadConfig;
use async_trait::async_trait;
use std::sync::Arc;

pub type TDownloadConfig = Arc<DownloadConfig>;

#[async_trait]
pub trait Download {
    async fn download(
        self,
        output_absolute_path: &str,
        download_config: TDownloadConfig,
    ) -> Result<Arc<Self>, String>;
}
