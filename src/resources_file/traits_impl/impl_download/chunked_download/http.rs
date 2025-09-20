use reqwest::header::RANGE;
use reqwest::{Client, Response};
use std::io::SeekFrom;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

pub struct FetchRangeArgs<'a> {
    pub(crate) http_client: &'a Client,
    pub range_header_str: &'a str,
    pub file_url: &'a str,
}

pub async fn fetch_range_method<'a>(
    args: FetchRangeArgs<'a>,
) -> Result<Response, String> {
    args.http_client
        .get(args.file_url)
        .header(RANGE, args.range_header_str)
        .send()
        .await
        .map_err(|e| e.to_string())
}

pub struct DownloadRangeFileArgs<'a> {
    pub(crate) http_client: &'a Client,
    pub range_header_str: &'a str,
    pub file_url: &'a str,
    pub file: &'a mut File,
    pub start: u64,
}

pub async fn download_range_file<'a>(
    args: DownloadRangeFileArgs<'a>,
) -> Result<(), String> {
    let fetch_range_args = FetchRangeArgs {
        http_client: args.http_client,
        range_header_str: args.range_header_str,
        file_url: args.file_url,
    };

    let resp = fetch_range_method(fetch_range_args).await?;

    if !resp.status().is_success() {
        return Err(format!(
            "分片下载失败: {} - {}",
            resp.status(),
            &args.file_url
        ));
    }

    let chunk = resp.bytes().await.map_err(|e| e.to_string())?;

    args.file
        .seek(SeekFrom::Start(args.start))
        .await
        .map_err(|e| e.to_string())?;

    args.file.write_all(&chunk).await.map_err(|e| e.to_string())?;
    Ok(())
}
