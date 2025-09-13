use crate::resources_file::structs::resource_file_data::ResourceFileData;
use crate::resources_file::traits::download::TDownloadConfig;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;

pub struct NotChunkedDownloadArgs {
    pub(crate) http_client: Client,
    pub(crate) resource_file_data: Arc<ResourceFileData>,
    pub(crate) save_absolute_path: PathBuf,
    pub(crate) download_config: TDownloadConfig,
}

pub async fn not_chunked_download(
    args: NotChunkedDownloadArgs,
) -> Result<(), String> {
    let resp = args
        .http_client
        .get(&args.resource_file_data.absolute_path)
        .send()
        .await
        .map_err(|e| format!("[http_client] {}", e.to_string()))?;

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("[bytes] {}", e.to_string()))?;

    tokio::fs::write(&args.save_absolute_path, &bytes).await.map_err(
        |e| {
            format!(
                "[write] {} {} {}",
                e.to_string(),
                args.resource_file_data.name,
                args.save_absolute_path.to_string_lossy()
            )
        },
    )?;

    Ok(())
}
