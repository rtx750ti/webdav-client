use crate::client::structs::client_key::ClientKey;
use crate::client::structs::raw_file_xml::MultiStatus;
use crate::client::traits::account::Account;
use crate::client::traits::folder::{
    Folders, TResourcesFileCollectionList,
};
use crate::client::{THttpClientArc, WebDavClient};
use crate::global_config::global_config::GlobalConfig;
use crate::client::webdav_request::get_folders_with_client::{
    get_folders_with_client, GetFoldersError,
};
use crate::resource_file::traits::to_resource_file_data::ToResourceFileData;
use async_trait::async_trait;
use futures_util::future::join_all;
use reqwest::Url;
use crate::client::enums::depth::Depth;
use crate::client::traits::url_format::UrlFormat;

#[derive(Debug)]
pub struct HandleResultArgs {
    pub(crate) results: Vec<Result<MultiStatus, GetFoldersError>>,
    pub(crate) http_client_arc: THttpClientArc,
    pub(crate) base_url: Url,
    pub(crate) global_config: GlobalConfig,
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
                            arg.global_config.clone(),
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
        paths: &Vec<String>,
        depth: &Depth,
    ) -> Result<TResourcesFileCollectionList, GetFoldersError> {
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
