use crate::downloader::enums::download_task::DownloadTask;
use crate::resources_file::structs::download_config::DownloadConfig;
use async_trait::async_trait;

#[async_trait]
pub trait BatchDownload {
    /// 批量下载
    async fn download(
        &self,
        save_absolute_path: &str,
        download_task: DownloadTask,
        download_config: &DownloadConfig,
    ) -> Result<(), String>;
}
