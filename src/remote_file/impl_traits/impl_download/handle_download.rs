use crate::global_config::global_config::GlobalConfig;
use crate::remote_file::structs::remote_file_config::RemoteConfig;
use crate::remote_file::structs::remote_file_property::RemoteFileProperty;
use crate::remote_file::structs::remote_file_data::RemoteFileData;
use crate::remote_file::traits::download::TDownloadConfig;
use crate::remote_file::impl_traits::impl_download::chunked_download::black_list::is_chunked_download_blacklisted;
use crate::remote_file::impl_traits::impl_download::chunked_download::{chunked_download, ChunkedDownloadArgs, ChunkedDownloadError};
use crate::remote_file::impl_traits::impl_download::not_chunked_download::{not_chunked_download, NotChunkedDownloadArgs, NotChunkedDownloadError};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;

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
    NotChunkedDownloadError(#[from] NotChunkedDownloadError),
}

/// 统一处理非分片下载
async fn download_without_chunking(
    args: HandleDownloadArgs,
) -> Result<(), DownloadWithoutChunkingError> {
    let not_chunked_download_args = NotChunkedDownloadArgs {
        http_client: args.http_client,
        remote_file_data: args.remote_file_data,
        save_absolute_path: args.save_absolute_path,
        global_config: args.global_config,
        inner_state: args.inner_state,
        inner_config: args.inner_config,
    };

    not_chunked_download(not_chunked_download_args).await?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum HandleDownloadError {
    #[error(transparent)]
    GetLargeFileThresholdError(#[from] GetLargeFileThresholdError),

    #[error(transparent)]
    DownloadWithoutChunkingError(#[from] DownloadWithoutChunkingError),

    #[error("chunked_download 出错: {0}")]
    ChunkedDownloadError(#[from] ChunkedDownloadError),

    #[error("跳过下载路径: {0} ，因为该文件或文件夹已存在")]
    PathExists(PathBuf),

    #[error("创建文件夹失败 : {0}")]
    CreateDirError(std::io::Error),
}

pub(crate) struct HandleDownloadArgs {
    pub(crate) remote_file_data: Arc<RemoteFileData>,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) http_client: Client,
    pub(crate) global_config: GlobalConfig,
    pub(crate) inner_state: RemoteFileProperty,
    pub(crate) inner_config: RemoteConfig,
}

async fn handle_dir(
    args: HandleDownloadArgs,
) -> Result<(), HandleDownloadError> {
    // 判断它在不在本地
    if !args.save_absolute_path.exists() {
        // 创建文件夹
        fs::create_dir(args.save_absolute_path)
            .await
            .map_err(|e| HandleDownloadError::CreateDirError(e))?;
        Ok(())
    } else {
        Err(HandleDownloadError::PathExists(args.save_absolute_path))
    }
}

async fn handle_file(
    args: HandleDownloadArgs,
) -> Result<(), HandleDownloadError> {
    // 首先判断本地文件中是否有该文件存在，如果有则不下载
    if args.save_absolute_path.exists() {
        return Err(HandleDownloadError::PathExists(
            args.save_absolute_path,
        ));
    }

    // 这里不再处理任何文件夹的递归逻辑，交由库的使用者来处理递归情况
    if args.remote_file_data.is_dir {
        return Ok(());
    }

    // 黑名单检查
    if is_chunked_download_blacklisted(
        &args.remote_file_data.base_url.to_string(),
    ) {
        return Ok(download_without_chunking(args).await?);
    }

    // 文件大小阈值检查
    if let Some(size) = args.remote_file_data.size {
        let threshold = get_large_file_threshold(&args.global_config)?;
        if size < threshold {
            return Ok(download_without_chunking(args).await?);
        }
    }

    // 默认使用分片下载
    let chunked_download_args = ChunkedDownloadArgs {
        remote_file_data: args.remote_file_data,
        http_client: args.http_client,
        save_absolute_path: args.save_absolute_path,
        global_config: args.global_config,
        inner_state: args.inner_state,
        inner_config: args.inner_config,
    };

    chunked_download(chunked_download_args).await?;

    Ok(())
}

pub(crate) async fn handle_download(
    args: HandleDownloadArgs,
) -> Result<(), HandleDownloadError> {
    if args.remote_file_data.is_dir {
        handle_dir(args).await
    } else {
        handle_file(args).await
    }
}
