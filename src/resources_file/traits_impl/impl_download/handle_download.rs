use crate::global_config::GlobalConfig;
use crate::resources_file::structs::reactive_config::ReactiveConfig;
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::traits::download::TDownloadConfig;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use crate::resources_file::traits_impl::impl_download::chunked_download::{chunked_download, ChunkedDownloadArgs};
use crate::resources_file::traits_impl::impl_download::chunked_download::black_list::is_chunked_download_blacklisted;
use crate::resources_file::traits_impl::impl_download::not_chunked_download::{not_chunked_download, NotChunkedDownloadArgs};

/// 统一获取大文件阈值
fn get_large_file_threshold(
    config: &TDownloadConfig,
) -> Result<u64, String> {
    #[cfg(not(feature = "reactive"))]
    {
        Ok(config.large_file_threshold)
    }

    #[cfg(feature = "reactive")]
    {
        config
            .get_current()
            .map(|c| c.large_file_threshold)
            .ok_or_else(|| "全局配置未初始化".to_string())
    }
}

/// 统一处理非分片下载
async fn download_without_chunking(
    args: HandleDownloadArgs,
) -> Result<(), String> {
    let not_chunked_download_args = NotChunkedDownloadArgs {
        http_client: args.http_client,
        resource_file_data: args.resource_file_data,
        save_absolute_path: args.save_absolute_path,
        global_config: args.global_config,
        inner_state: args.inner_state,
        inner_config: args.inner_config,
    };

    not_chunked_download(not_chunked_download_args)
        .await
        .map_err(|e| format!("[not_chunked_download] {}", e))
}

pub struct HandleDownloadArgs {
    pub(crate) resource_file_data: Arc<ResourceFileData>,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) http_client: Client,
    pub(crate) global_config: GlobalConfig,
    pub(crate) inner_state: ReactiveFileProperty,
    pub(crate) inner_config: ReactiveConfig,
}

pub async fn handle_download(
    args: HandleDownloadArgs,
) -> Result<(), String> {
    // 这里不再处理任何文件夹的递归逻辑，交由库的使用者来处理递归情况
    if args.resource_file_data.is_dir {
        return Ok(());
    }

    // 黑名单检查
    if is_chunked_download_blacklisted(
        &args.resource_file_data.base_url.to_string(),
    ) {
        return download_without_chunking(args).await;
    }

    // 文件大小阈值检查
    if let Some(size) = args.resource_file_data.size {
        let threshold = get_large_file_threshold(&args.global_config)?;
        if size < threshold {
            return download_without_chunking(args).await;
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

    chunked_download(chunked_download_args)
        .await
        .map_err(|e| format!("[chunked_download] {}", e))
}
