use crate::global_config::global_config::GlobalConfig;
use crate::resource_file::structs::resource_config::ResourceConfig;
use crate::resource_file::structs::resource_file_property::ResourceFileProperty;
use crate::resource_file::structs::resource_file_data::ResourceFileData;
use crate::resource_file::traits::download::TDownloadConfig;
use crate::resource_file::impl_traits::impl_download::chunked_download::black_list::is_chunked_download_blacklisted;
use crate::resource_file::impl_traits::impl_download::chunked_download::{chunked_download, ChunkedDownloadArgs, ChunkedDownloadError};
use crate::resource_file::impl_traits::impl_download::not_chunked_download::{not_chunked_download, NotChunkedDownloadArgs};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetLargeFileThresholdError {
    #[error("全局配置未初始化")]
    GlobalConfigUninitialized,
}

/// 统一获取大文件阈值
fn get_large_file_threshold(
    config: &TDownloadConfig,
) -> Result<u64, GetLargeFileThresholdError> {
    config.get_current().map(|c| c.large_file_threshold).ok_or_else(|| {
        GetLargeFileThresholdError::GlobalConfigUninitialized
    })
}

#[derive(Debug, Error)]
pub enum DownloadWithoutChunkingError {
    #[error("not_chunked_download 出错: {0}")]
    NotChunkedDownloadError(String),
}

/// 统一处理非分片下载
async fn download_without_chunking(
    args: HandleDownloadArgs,
) -> Result<(), DownloadWithoutChunkingError> {
    let not_chunked_download_args = NotChunkedDownloadArgs {
        http_client: args.http_client,
        resource_file_data: args.resource_file_data,
        save_absolute_path: args.save_absolute_path,
        global_config: args.global_config,
        inner_state: args.inner_state,
        inner_config: args.inner_config,
    };

    not_chunked_download(not_chunked_download_args).await.map_err(|e| {
        DownloadWithoutChunkingError::NotChunkedDownloadError(e)
    })
}

#[derive(Debug, Error)]
pub enum HandleDownloadError {
    #[error(transparent)]
    GetLargeFileThresholdError(#[from] GetLargeFileThresholdError),

    #[error(transparent)]
    DownloadWithoutChunkingError(#[from] DownloadWithoutChunkingError),

    #[error("chunked_download 出错: {0}")]
    ChunkedDownloadError(#[from] ChunkedDownloadError),
}

pub struct HandleDownloadArgs {
    pub(crate) resource_file_data: Arc<ResourceFileData>,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) http_client: Client,
    pub(crate) global_config: GlobalConfig,
    pub(crate) inner_state: ResourceFileProperty,
    pub(crate) inner_config: ResourceConfig,
}

pub async fn handle_download(
    args: HandleDownloadArgs,
) -> Result<(), HandleDownloadError> {
    // 这里不再处理任何文件夹的递归逻辑，交由库的使用者来处理递归情况
    if args.resource_file_data.is_dir {
        return Ok(());
    }

    // 黑名单检查
    if is_chunked_download_blacklisted(
        &args.resource_file_data.base_url.to_string(),
    ) {
        return Ok(download_without_chunking(args).await?);
    }

    // 文件大小阈值检查
    if let Some(size) = args.resource_file_data.size {
        let threshold = get_large_file_threshold(&args.global_config)?;
        if size < threshold {
            return Ok(download_without_chunking(args).await?);
        }
    }

    // 默认使用分片下载
    let chunked_download_args = ChunkedDownloadArgs {
        resource_file_data: args.resource_file_data,
        http_client: args.http_client,
        save_absolute_path: args.save_absolute_path,
        global_config: args.global_config,
        inner_state: args.inner_state,
        inner_config: args.inner_config,
    };

    chunked_download(chunked_download_args).await?;

    Ok(())
}
