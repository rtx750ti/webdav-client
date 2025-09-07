use std::path::PathBuf;
use reqwest::Client;
use crate::download_config::DownloadConfig;
use crate::resources_file::structs::resource_file_data::ResourceFileData;

pub async fn not_chunked_download(
    http_client: &Client,
    resource_file_data: &ResourceFileData,
    save_absolute_path: &PathBuf,
    download_config: &DownloadConfig,
) -> Result<(), String> {
    let resp = http_client
        .get(&resource_file_data.absolute_path)
        .send()
        .await
        .map_err(|e| format!("[http_client] {}", e.to_string()))?;

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("[bytes] {}", e.to_string()))?;

    tokio::fs::write(&save_absolute_path, &bytes).await.map_err(|e| {
        format!(
            "[write] {} {} {}",
            e.to_string(),
            resource_file_data.name,
            save_absolute_path.to_string_lossy()
        )
    })?;

    Ok(())
}