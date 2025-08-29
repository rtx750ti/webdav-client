use crate::resources_file::structs::download_config::DownloadConfig;
use crate::resources_file::structs::resources_file::ResourcesFile;
use crate::resources_file::traits::download::Download;
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;

#[async_trait]
impl Download for ResourcesFile {
    async fn download(
        self,
        save_absolute_path: &str,
        download_config: &DownloadConfig,
    ) -> Result<Arc<Self>, String> {
        let http_client = self.get_http_client();
        Ok(Arc::new(self))
    }

    async fn stop(self: Arc<Self>) -> Result<Arc<Self>, String> {
        todo!()
    }

    async fn start(self: Arc<Self>) -> Result<Arc<Self>, String> {
        todo!()
    }

    async fn restart(self: Arc<Self>) -> Result<Arc<Self>, String> {
        todo!()
    }
}
