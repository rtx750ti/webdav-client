use crate::resources_file::structs::resource_file_data::ResourceFileData;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(feature = "reactive")]
use crate::global_config::DEFAULT_LARGE_FILE_THRESHOLD;
use crate::resources_file::traits::download::TDownloadConfig;
use crate::resources_file::traits_impl::impl_download::download::chunked_download::{chunked_download, is_chunked_download_blacklisted, ChunkedDownloadArgs};
use crate::resources_file::traits_impl::impl_download::download::not_chunked_download::{not_chunked_download, NotChunkedDownloadArgs};

pub struct HandleDownloadArgs {
    pub(crate) resource_file_data: Arc<ResourceFileData>,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) http_client: Client,
    pub(crate) download_config: TDownloadConfig,
}

pub async fn handle_download(
    args: HandleDownloadArgs,
) -> Result<(), String> {
    if args.resource_file_data.is_dir {
        return Ok(());
    }

    let is_founded_blacklist = is_chunked_download_blacklisted(
        &args.resource_file_data.base_url.to_string(),
    );

    let not_chunked_download_args = NotChunkedDownloadArgs {
        http_client: args.http_client.clone(),
        resource_file_data: args.resource_file_data.clone(),
        save_absolute_path: args.save_absolute_path.clone(),
        download_config: args.download_config.clone(),
    };

    if is_founded_blacklist {
        let _ = not_chunked_download(not_chunked_download_args)
            .await
            .map_err(|e| {
                format!("[not_chunked_download] {}", e.to_string())
            })?;

        return Ok(());
    }

    #[cfg(not(feature = "reactive"))]
    {
        if let Some(size) = args.resource_file_data.size {
            if size < args.download_config.large_file_threshold {
                let _ = not_chunked_download(not_chunked_download_args)
                    .await
                    .map_err(|e| {
                        format!("[not_chunked_download] {}", e.to_string())
                    })?;

                return Ok(());
            }
        }
    }

    #[cfg(feature = "reactive")]
    {
        if let Some(size) = args.resource_file_data.size {
            let download_config = args.download_config.get_current();

            if let Some(config) = download_config {
                if size < config.large_file_threshold {
                    let _ =
                        not_chunked_download(not_chunked_download_args)
                            .await
                            .map_err(|e| {
                                format!(
                                    "[not_chunked_download] {}",
                                    e.to_string()
                                )
                            })?;

                    return Ok(());
                }
            } else {
                return Err("全局配置未初始化".to_string());
            }
        }
    }

    let chunked_download_args = ChunkedDownloadArgs {
        resource_file_data: args.resource_file_data,
        http_client: args.http_client,
        save_absolute_path: args.save_absolute_path,
        download_config: args.download_config,
    };

    let _ = chunked_download(chunked_download_args)
        .await
        .map_err(|e| format!("[chunked_download] {}", e.to_string()))?;

    Ok(())
}
