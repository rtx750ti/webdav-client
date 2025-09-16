use crate::client::structs::client_key::{ClientKey, TClientKey};
use crate::client::structs::raw_file_xml::MultiStatus;
use crate::client::traits::account::Account;
use crate::client::traits::folder::{
    Folders, TResourcesFileCollectionList,
};
use crate::client::{THttpClientArc, WebDavClient};
use crate::public::enums::depth::Depth;
use crate::public::traits::url_format::UrlFormat;
use crate::public::utils::get_folders_public_impl::{
    GetFoldersError, get_folders_with_client,
};
use crate::resources_file::traits::to_resource_file_data::ToResourceFileData;
use async_trait::async_trait;
use futures_util::future::join_all;
use reqwest::Url;
use std::sync::Arc;

#[derive(Debug)]
pub struct HandleResultArgs {
    pub(crate) results: Vec<Result<MultiStatus, GetFoldersError>>,
    pub(crate) http_client_arc: THttpClientArc,
    pub(crate) base_url: Url,
}

pub fn handle_result(
    arg: HandleResultArgs,
) -> Result<TResourcesFileCollectionList, GetFoldersError> {
    let mut all_files = Vec::new();

    for res in arg.results {
        match res {
            Ok(multi_status) => {
                let mut resources_files = Vec::new();
                let resource_data_list =
                    multi_status.to_resource_file_data(&arg.base_url)?;

                for resource_file_data in resource_data_list {
                    resources_files.push(
                        resource_file_data.to_resources_file(
                            arg.http_client_arc.get_client(),
                        ),
                    )
                }
                all_files.push(resources_files)
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
        reactive_paths: &Vec<String>,
        depth: &Depth,
    ) -> Result<TResourcesFileCollectionList, GetFoldersError> {
        let http_client_arc = self.get_http_client(key)?;

        // 构建所有任务（这里只做并发请求）
        let tasks = reactive_paths.iter().map(|path| {
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
        };

        let all_files = handle_result(handle_result_args)?;

        Ok(all_files)
    }
}
