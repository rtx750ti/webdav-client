use crate::downloader::enums::download_task::DownloadTask;
use async_trait::async_trait;
use crate::resources_file::traits::download::TDownloadConfig;

#[async_trait]
pub trait BatchDownload {
    /// 批量下载
    async fn download(
        &self,
        save_absolute_path: &str,
        download_task: DownloadTask,
        download_config: TDownloadConfig,
    ) -> Result<(), String>;
}
