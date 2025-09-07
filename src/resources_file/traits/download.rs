use crate::download_config::DownloadConfig;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait Download {
    async fn download(
        self,
        output_absolute_path: &str,
        download_config: &DownloadConfig,
    ) -> Result<Arc<Self>, String>;
}
