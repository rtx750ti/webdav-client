use crate::client::WebDavClient;
use crate::client::structs::client_key::ClientKey;
use crate::client::traits::account::Account;
use crate::client::traits::local_folders::{
    FileBuildError, LocalFolders, LocalFoldersResult,
    TFileBuildFailedList, TLocalFileCollection,
};
use crate::local_file::structs::local_file::LocalFile;
use async_trait::async_trait;
use futures_util::future::join_all;
use reqwest::Client;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

fn create_file_build_error(
    cause: String,
    path: PathBuf,
) -> FileBuildError {
    FileBuildError { cause, path }
}

async fn create_single_file_result(
    http_client: Client,
    absolute_path: &PathBuf,
) -> Result<LocalFoldersResult, String> {
    match LocalFile::new(http_client, &absolute_path).await {
        Ok(local_file) => {
            let file_list = vec![local_file];
            let failed_list: TFileBuildFailedList = Vec::new(); // 此处暂时空，因为 new 成功
            Ok((file_list, failed_list))
        }
        Err(e) => {
            // 构建失败，把 dir_entry 模拟成失败项
            let failed_list = vec![create_file_build_error(
                e.to_string(),
                absolute_path.to_owned(),
            )];
            Ok((Vec::new(), failed_list))
        }
    }
}

const MAX_RETRIES: usize = 3;

async fn process_dir_entry(
    http_client: Client,
    file_path: PathBuf,
    absolute_path: &PathBuf,
    file_list: &mut TLocalFileCollection,
    file_build_failed_list: &mut TFileBuildFailedList,
) {
    let local_file = LocalFile::new(http_client, &file_path).await;

    match local_file {
        Ok(local_file) => {
            file_list.push(local_file);
        }
        Err(e) => {
            file_build_failed_list.push(create_file_build_error(
                e.to_string(),
                absolute_path.to_owned(),
            ));
        }
    }
}

async fn read_directory_entries(
    http_client: Client,
    absolute_path: &PathBuf,
    mut entries: tokio::fs::ReadDir,
) -> LocalFoldersResult {
    let mut file_list: TLocalFileCollection = Vec::new();
    let mut file_build_failed_list: TFileBuildFailedList = Vec::new();
    let mut iter_err_count: usize = 0;

    // 顺序遍历出该层目录的全部文件，这里不需要tokio::spwan，因为本身已经是已步，而且是get_local_folders的子任务
    // 所以分太多子任务就导致内存浪费
    loop {
        let dir_entry_result = entries.next_entry().await;
        match dir_entry_result {
            Ok(dir_entry) => {
                if let Some(dir_entry) = dir_entry {
                    let file_path = dir_entry.path();
                    let http_client_clone = http_client.clone();

                    process_dir_entry(
                        http_client_clone,
                        file_path,
                        absolute_path,
                        &mut file_list,
                        &mut file_build_failed_list,
                    )
                    .await;
                } else {
                    // 没有更多条目，退出循环
                    break;
                }
            }
            Err(e) => {
                // 迭代器 next_entry() 出错了：重试几次后再退出
                iter_err_count += 1;

                // 指数退避算法：2^n * 100ms
                let backoff = Duration::from_millis(100 * (1 << iter_err_count));
                sleep(backoff).await;

                if iter_err_count > MAX_RETRIES {
                    // 超过阈值，退出循环（保留已收集的 file_list）
                    eprintln!(
                        "error: read_dir.next_entry() failed {} times, aborting: {}",
                        iter_err_count,
                        e.to_string()
                    );
                    break;
                } else {
                    // 继续尝试读取下一个 entry
                    continue;
                }
            }
        }
    }

    (file_list, file_build_failed_list)
}

async fn get_local_folder(
    http_client: Client,
    absolute_path: &PathBuf,
) -> Result<LocalFoldersResult, String> {
    // 判断文件夹不存在，则返回空数组
    if !absolute_path.exists() {
        return Ok((Vec::new(), Vec::new()));
    }

    // 读取文件夹
    let entries = tokio::fs::read_dir(absolute_path)
        .await
        .map_err(|e| e.to_string())?;

    // 遍历目录并收集文件
    let result = read_directory_entries(http_client, absolute_path, entries).await;

    Ok(result)
}

#[async_trait]
impl LocalFolders for WebDavClient {
    async fn get_local_folders(
        &self,
        key: &ClientKey,
        paths: &[String],
    ) -> Result<Vec<Result<LocalFoldersResult, String>>, String> {
        let http_client_arc =
            self.get_http_client(key).map_err(|e| e.to_string())?;

        let tasks = paths.iter().map(|path| {
            let http_client_entity = http_client_arc.get_client();
            let absolute_path = PathBuf::from(path);
            async move {
                // 判断该路径是文件
                if absolute_path.is_file() {
                    create_single_file_result(
                        http_client_entity,
                        &absolute_path,
                    )
                    .await
                } else if absolute_path.is_dir() {
                    get_local_folder(http_client_entity, &absolute_path)
                        .await
                } else {
                    unreachable!() // 一般不会进这里
                }
            }
        });

        let results: Vec<Result<LocalFoldersResult, String>> =
            join_all(tasks).await;

        Ok(results)
    }
}
