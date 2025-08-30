use crate::public::enums::depth::Depth;
use crate::public::utils::get_folders_public_impl::get_folders_with_client;
use crate::resources_file::structs::download_config::DownloadConfig;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::traits::download::Download;
use crate::resources_file::traits::to_resource_file_data::ToResourceFileData;
use reqwest::Client;
use reqwest::header::RANGE;
use std::cmp::min;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

/// 分片黑名单，这些厂商不讲武德，拒绝分片请求，甚至拿1比特数据都要算下载了整个文件的流量
const CHUNKED_DOWNLOAD_BLACKLIST: [&str; 1] =
    ["https://dav.jianguoyun.com/"];

/// 查找地址是否在分片黑名单里
pub fn is_chunked_download_blacklisted(base_url: &str) -> bool {
    CHUNKED_DOWNLOAD_BLACKLIST
        .iter()
        .any(|blacklisted_url| base_url.starts_with(blacklisted_url))
}

/*
#[derive(Debug, Clone)]
pub struct ResourceFileData {
    pub base_url: Url,
    pub relative_root_path: String, // 文件的相对路径（相对根目录）
    pub absolute_path: String,      // 文件的完整路径（从 href 拿到）
    pub name: String,               // 友好化的文件或目录名
    pub is_dir: bool,               // 是否目录
    pub size: Option<u64>,          // 文件大小（字节）
    pub last_modified: Option<DateTime<FixedOffset>>, // 原始时间
    pub mime: Option<String>,       // MIME 类型
    pub owner: Option<String>,      // 所有者
    pub etag: Option<String>,       // 清理后的 ETag
    pub privileges: Vec<String>,    // 权限列表
}
*/

/*
#[derive(Debug, Clone)]
pub struct ResourcesFile {
    data: ResourceFileData,
    http_client: Client,
}
*/

const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

pub async fn chunked_download(
    resource_file_data: &ResourceFileData,
    http_client: &Client,
    save_absolute_path: &PathBuf,
    _download_config: &DownloadConfig,
) -> Result<(), String> {
    let total_size = resource_file_data.size.ok_or_else(|| {
        format!(
            "文件大小未知，无法分片下载 {}",
            resource_file_data.absolute_path
        )
    })?;

    // 检查是否已有部分文件（断点续传）
    let mut local_size: u64 = 0;
    if let Ok(meta) = tokio::fs::metadata(save_absolute_path).await {
        local_size = meta.len();
        if local_size > total_size {
            // 本地文件比远程大，说明出错，删掉重新下
            tokio::fs::remove_file(save_absolute_path)
                .await
                .map_err(|e| e.to_string())?;
            local_size = 0;
        } else if local_size == total_size {
            // 已经下载完成
            return Ok(());
        }
    }

    // 打开文件（续传时用 append + write）
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(save_absolute_path)
        .await
        .map_err(|e| e.to_string())?;

    let mut start = local_size;
    let file_url = &resource_file_data.absolute_path;

    while start < total_size {
        let end = min(start + CHUNK_SIZE - 1, total_size - 1);
        let range_header = format!("bytes={}-{}", start, end);

        let resp = http_client
            .get(file_url)
            .header(RANGE, range_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!(
                "分片下载失败: {} - {}",
                resp.status(),
                file_url
            ));
        }

        let chunk = resp.bytes().await.map_err(|e| e.to_string())?;

        file.seek(std::io::SeekFrom::Start(start))
            .await
            .map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;

        start += CHUNK_SIZE;
    }

    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn not_chunked_download(
    http_client: &Client,
    file_absolute_url: &str,
    save_absolute_path: &PathBuf,
    download_config: &DownloadConfig,
) -> Result<(), String> {
    let resp = http_client
        .get(file_absolute_url)
        .send()
        .await
        .map_err(|e| format!("[http_client] {}", e.to_string()))?;

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("[bytes] {}", e.to_string()))?;

    tokio::fs::write(&save_absolute_path, &bytes)
        .await
        .map_err(|e| format!("[write] {}", e.to_string()))?;

    Ok(())
}

pub async fn recursive_directory(
    resource_file_data: &ResourceFileData,
    save_absolute_path: &PathBuf,
    http_client: &Client,
    download_config: &DownloadConfig,
) -> Result<(), String> {
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

        // 递归
        let _ = children_resource_file
            .download(
                &save_absolute_path.to_string_lossy().to_string(),
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
        if download_config.auto_download_folder {
            // 先不处理递归
            // let _ = recursive_directory(
            //     resource_file_data,
            //     save_absolute_path,
            //     http_client,
            //     download_config,
            // )
            // .await?;
        } else {
            return Ok(());
        }
    }

    if is_chunked_download_blacklisted(
        &resource_file_data.base_url.to_string(),
    ) {
        let _ = not_chunked_download(
            http_client,
            &resource_file_data.absolute_path,
            save_absolute_path,
            download_config,
        )
        .await
        .map_err(|e| {
            format!("[not_chunked_download] {}", e.to_string())
        })?;

        return Ok(());
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
