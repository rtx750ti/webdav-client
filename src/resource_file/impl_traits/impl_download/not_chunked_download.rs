use crate::global_config::global_config::GlobalConfig;
use crate::reactive::reactive::ReactivePropertyError;
use crate::resource_file::structs::resource_config::ResourceConfig;
use crate::resource_file::structs::resource_file_data::ResourceFileData;
use crate::resource_file::structs::resource_file_property::ResourceFileProperty;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Error)]
pub enum NotChunkedDownloadError {
    #[error("HTTP 请求失败: {0}")]
    HttpClientError(#[from] reqwest::Error),

    #[error("创建文件失败: {0}")]
    CreateFileError(std::io::Error),

    #[error("下载流出错: {0}")]
    DownloadStreamError(reqwest::Error),

    #[error("写入文件失败: {0}")]
    WriteFileError(tokio::io::Error),

    #[error("更新下载字节数失败: {0}")]
    UpdateBytesError(#[from] ReactivePropertyError), // 或者你自己定义的错误类型
}

pub struct NotChunkedDownloadArgs {
    pub(crate) http_client: Client,
    pub(crate) resource_file_data: Arc<ResourceFileData>,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) global_config: GlobalConfig,
    pub(crate) inner_state: ResourceFileProperty,
    pub(crate) inner_config: ResourceConfig,
}

pub async fn not_chunked_download(
    args: NotChunkedDownloadArgs,
) -> Result<(), NotChunkedDownloadError> {
    let resp = args
        .http_client
        .get(&args.resource_file_data.absolute_path)
        .send()
        .await?;

    let mut download_stream = resp.bytes_stream();

    let mut file = File::create(&args.save_absolute_path)
        .await
        .map_err(|e| NotChunkedDownloadError::CreateFileError(e))?;

    let reactive_downloaded_bytes = args.inner_state.get_download_bytes();

    let global_config = &args.global_config;
    let mut global_config_watch = global_config.watch();

    let inner_config = &args.inner_config;
    let mut inner_config_watch = inner_config.watch();

    while let Some(memory_chunk) = download_stream.next().await {
        while global_config.is_paused() || inner_config.is_paused() {
            if global_config.is_paused() {
                println!("全局暂停");
                let _ = global_config_watch.changed().await;
                println!("全局启动");
            }

            if inner_config.is_paused() {
                println!("内部暂停");
                let _ = inner_config_watch.changed().await;
                println!("内部启动");
            }
        }

        let chunk = memory_chunk.map_err(|e| {
            NotChunkedDownloadError::DownloadStreamError(e)
        })?;

        file.write_all(&chunk)
            .await
            .map_err(|e| NotChunkedDownloadError::WriteFileError(e))?;

        reactive_downloaded_bytes.update_field(|download_bytes| {
            *download_bytes += chunk.len()
        })?;
    }

    Ok(())
}
