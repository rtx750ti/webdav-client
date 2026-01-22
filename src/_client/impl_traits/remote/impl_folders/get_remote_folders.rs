use crate::_client::impl_traits::remote::impl_folders::webdav_request::get_folders_raw_data;
use crate::_global_config::global_config::GlobalConfig;
use crate::client::enums::Depth;
use crate::client::structs::{ClientKey, MultiStatus};
use crate::client::traits::client::{Account, UrlFormat, UrlFormatError};
use crate::client::traits::remote::{
    RemoteFolders, GetRemoteFoldersError, RemoteFileCollections, RemoteFiles,
};
use crate::client::{THttpClientArc, WebDavClient};
use crate::remote_file::traits::to_remote_file_data::ToRemoteFileData;
use async_trait::async_trait;
use futures_util::future::join_all;
use reqwest::Url;

#[derive(Debug)]
struct WebdavFolderTaskResult {
    path: String,
    result: MultiStatus,
}

type WebDavTaskResult =
    Vec<Result<WebdavFolderTaskResult, GetRemoteFoldersError>>;

#[derive(Debug)]
struct HandleResultArgs {
    results: WebDavTaskResult,
    http_client_arc: THttpClientArc,
    base_url: Url,
    global_config: GlobalConfig,
}

fn handle_result(
    arg: HandleResultArgs,
) -> Result<RemoteFileCollections, GetRemoteFoldersError> {
    let mut remote_file_collections = RemoteFileCollections::new();

    for res in arg.results {
        match res {
            Ok(webdav_folder_task_result) => {
                let mut remote_files = RemoteFiles::new();
                let remote_file_data_list = webdav_folder_task_result
                    .result
                    .to_remote_file_data(&arg.base_url)?;

                for remote_file_data in remote_file_data_list {
                    let remote_file = remote_file_data.to_remote_file(
                        arg.http_client_arc.get_client(),
                        arg.global_config.clone(),
                    );
                    remote_files.push(remote_file);
                }

                remote_file_collections
                    .insert(webdav_folder_task_result.path, remote_files);
            }
            Err(url_format_error) => {
                eprintln!("{}", url_format_error);
            }
        }
    }

    Ok(remote_file_collections)
}

#[async_trait]
impl RemoteFolders for WebDavClient {
    async fn get_remote_folders(
        &self,
        key: &ClientKey,
        paths: &[&str],
        depth: &Depth,
    ) -> Result<RemoteFileCollections, GetRemoteFoldersError> {
        let http_client_arc = self.get_http_client(key)?;

        // 构建所有任务（这里只做并发请求）
        let tasks = paths.iter().map(|path| {
            let http_client_entity = http_client_arc.get_client();

            async move {
                let url = self.format_url_path(key, path)?;

                // 获取webdav文件夹原始数据
                let folders_raw_data =
                    get_folders_raw_data(http_client_entity, &url, depth)
                        .await?;

                Ok(WebdavFolderTaskResult {
                    path: path.to_string(),
                    result: folders_raw_data,
                })
            }
        });

        // 并发执行所有任务
        let results: WebDavTaskResult = join_all(tasks).await;

        let handle_result_args = HandleResultArgs {
            results,
            http_client_arc,
            base_url: key.get_base_url(),
            global_config: self.get_global_config(),
        };

        let all_files = handle_result(handle_result_args)?;

        Ok(all_files)
    }
}
