use crate::global_config::GlobalConfig;
use crate::resources_file::structs::reactive_config::ReactiveConfig;
use crate::resources_file::structs::reactive_file_property::ReactiveFileProperty;
use bytes::Bytes;
use futures_util::StreamExt;
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

pub struct HandleBytesStreamArgs<'a> {
    pub chunk: Bytes,
    pub current_file_seek_start: u64,
    pub file: &'a mut File,
    pub inner_state: ReactiveFileProperty,
}

async fn handle_bytes_stream<'a>(
    args: HandleBytesStreamArgs<'a>,
) -> Result<(), String> {
    args.file
        .seek(SeekFrom::Start(args.current_file_seek_start))
        .await
        .map_err(|e| {
            format!(
                "[download_stream.next()]→[seek(SeekFrom::Start(current_file_seek_start))]{}",
                e
            )
        })?;

    args.file.write_all(&args.chunk).await.map_err(|e| {
        format!(
            "[download_stream.next()]→[args.file.write_all(&chunk))]{}",
            e
        )
    })?;

    // 读取一次，避免报未使用错误
    let _ = args.current_file_seek_start;

    // 更新大小
    args.inner_state.download_bytes.update_field(|download_bytes| {
        *download_bytes += args.chunk.len()
    })?;

    Ok(())
}


pub struct DownloadRangeFileArgs<'a> {
    pub(crate) http_client: &'a Client,
    pub range_header_str: &'a str,
    pub file_url: &'a str,
    pub file: &'a mut File,
    pub start: u64,
    pub inner_state: ReactiveFileProperty,
    #[allow(dead_code)]
    pub global_config: GlobalConfig,
    #[allow(dead_code)]
    pub inner_config: ReactiveConfig,
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

    // 把它转成数据流
    let mut download_stream = resp.bytes_stream();

    let mut current_file_seek_start = args.start;

    while let Some(downloaded_chunk) = download_stream.next().await {
        let chunk = downloaded_chunk
            .map_err(|e| format!("[download_stream.next()]{}", e))?;

        let chunk_length = chunk.len() as u64;

        let handle_bytes_stream_args = HandleBytesStreamArgs {
            chunk,
            current_file_seek_start,
            file: args.file,
            inner_state: args.inner_state.clone(), // 如果 ReactiveFileProperty 可 Clone
        };

        handle_bytes_stream(handle_bytes_stream_args).await?;

        current_file_seek_start += chunk_length;
    }

    Ok(())
}
