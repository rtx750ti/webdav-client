use async_trait::async_trait;
use std::sync::Arc;
use crate::global_config::GlobalConfig;

pub type TDownloadConfig = GlobalConfig;

#[async_trait]
pub trait Download {
    async fn download(
        self,
        output_absolute_path: &str,
    ) -> Result<Arc<Self>, String>;
}
