pub mod get_folders;
pub mod webdav_request;

use crate::client::structs::client_key::ClientKey;
use crate::client::structs::raw_file_xml::MultiStatus;

use crate::client::{THttpClientArc, WebDavClient};
use crate::global_config::global_config::GlobalConfig;

use crate::client::enums::depth::Depth;
use crate::client::impl_traits::impl_folders::webdav_request::get_folders_with_client;
use crate::client::traits::_self::account::Account;
use crate::client::traits::_self::url_format::UrlFormat;
use crate::remote_file::traits::to_remote_file_data::ToRemoteFileData;
use async_trait::async_trait;
use futures_util::future::join_all;
use reqwest::Url;
use crate::client::traits::remote::folders::{Folders, GetFoldersError, TRemoteFileCollectionList};

#[derive(Debug)]
pub struct HandleResultArgs {
    pub(crate) results: Vec<Result<MultiStatus, GetFoldersError>>,
    pub(crate) http_client_arc: THttpClientArc,
    pub(crate) base_url: Url,
    pub(crate) global_config: GlobalConfig,
}

pub fn handle_result(
    arg: HandleResultArgs,
) -> Result<TRemoteFileCollectionList, GetFoldersError> {
    let mut all_files = Vec::new();

    for res in arg.results {
        match res {
            Ok(multi_status) => {
                let mut remote_files = Vec::new();
                let remote_file_data_list =
                    multi_status.to_remote_file_data(&arg.base_url)?;

                for remote_file_data in remote_file_data_list {
                    remote_files.push(remote_file_data.to_remote_file(
                        arg.http_client_arc.get_client(),
                        arg.global_config.clone(),
                    ))
                }
                all_files.push(remote_files)
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    Ok(all_files)
}

#[async_trait]
impl Folders for WebDavClient {
    async fn get_folders(
        &self,
        key: &ClientKey,
        paths: &Vec<String>,
        depth: &Depth,
    ) -> Result<TRemoteFileCollectionList, GetFoldersError> {
        let http_client_arc = self.get_http_client(key)?;

        // 构建所有任务（这里只做并发请求）
        let tasks = paths.iter().map(|path| {
            let http_client_entity = http_client_arc.get_client();

            async move {
                let url = self.format_url_path(key, path)?;

                // 调用已有的单次请求函数
                get_folders_with_client(http_client_entity, &url, depth)
                    .await
            }
        });

        // 并发执行所有任务
        let results: Vec<Result<MultiStatus, GetFoldersError>> =
            join_all(tasks).await;

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
