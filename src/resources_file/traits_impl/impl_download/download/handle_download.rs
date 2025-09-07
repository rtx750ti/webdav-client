use crate::download_config::DownloadConfig;
use crate::public::enums::depth::Depth;
use crate::public::utils::get_folders_public_impl::get_folders_with_client;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::traits::download::Download;
use crate::resources_file::traits::to_resource_file_data::ToResourceFileData;
use reqwest::Client;
use std::path::PathBuf;
use crate::resources_file::traits_impl::impl_download::download::chunked_download::{chunked_download, is_chunked_download_blacklisted};
use crate::resources_file::traits_impl::impl_download::download::not_chunked_download::not_chunked_download;

pub async fn recursive_directory(
    resource_file_data: &ResourceFileData,
    save_absolute_path: &PathBuf,
    http_client: &Client,
    download_config: &DownloadConfig,
) -> Result<(), String> {
    // 拼接当前目录路径
    let current_dir = save_absolute_path.join(&resource_file_data.name);

    // 确保本地目录存在
    if let Err(e) = tokio::fs::create_dir_all(&current_dir).await {
        return Err(format!(
            "创建目录失败: {:?}, 错误: {}",
            current_dir,
            e.to_string()
        ));
    }

    // 请求子文件/目录
    let children = get_folders_with_client(
        http_client.clone(),
        &resource_file_data.absolute_path,
        &Depth::One,
    )
    .await
    .map_err(|e| e.to_string())?;

    let children_resource_files_data = children
        .to_resource_file_data(&resource_file_data.base_url)
        .map_err(|e| e.to_string())?;

    for data in children_resource_files_data {
        let children_resource_file =
            data.to_resources_file(http_client.clone());

        // 拼接子文件/目录的本地保存路径
        let child_save_path = current_dir.clone();

        // 递归/下载
        let _ = children_resource_file
            .download(
                &child_save_path.to_string_lossy().to_string(),
                download_config,
            )
            .await?;
    }

    Ok(())
}

pub async fn handle_download(
    resource_file_data: &ResourceFileData,
    save_absolute_path: &PathBuf,
    http_client: &Client,
    download_config: &DownloadConfig,
) -> Result<(), String> {
    if resource_file_data.is_dir {
        return if download_config.auto_download_folder {
            // 先不处理递归
            let _ = recursive_directory(
                resource_file_data,
                save_absolute_path,
                http_client,
                download_config,
            )
            .await?;
            Ok(())
        } else {
            Ok(())
        };
    }

    if is_chunked_download_blacklisted(
        &resource_file_data.base_url.to_string(),
    ) {
        let _ = not_chunked_download(
            http_client,
            resource_file_data,
            save_absolute_path,
            download_config,
        )
        .await
        .map_err(|e| {
            format!("[not_chunked_download] {}", e.to_string())
        })?;

        return Ok(());
    }

    if let Some(size) = resource_file_data.size {
        if size < download_config.large_file_threshold {
            let _ = not_chunked_download(
                http_client,
                resource_file_data,
                save_absolute_path,
                download_config,
            )
            .await
            .map_err(|e| {
                format!("[not_chunked_download] {}", e.to_string())
            })?;

            return Ok(());
        }
    }

    let _ = chunked_download(
        resource_file_data,
        http_client,
        save_absolute_path,
        download_config,
    )
    .await
    .map_err(|e| format!("[chunked_download] {}", e.to_string()))?;

    Ok(())
}
