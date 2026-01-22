use crate::_client::traits::local::folders::{
    LocalFileCollections, LocalFiles,
};
use crate::client::enums::Depth;
use crate::client::structs::ClientKey;
use crate::client::traits::client::Account;
use crate::client::traits::local::folders::LocalFolders;
use crate::client::{THttpClientArc, WebDavClient};
use crate::global_config::GlobalConfig;
use crate::local_file::structs::LocalFile;
use async_trait::async_trait;
use std::collections::HashMap;

fn process_local_file(
    http_client: THttpClientArc,
    paths: &[&str],
    global_config: GlobalConfig,
) -> Vec<Result<LocalFile, String>> {
    let mut local_files: Vec<Result<LocalFile, String>> = Vec::new();

    for path in paths {
        let http_client = http_client.get_client(); // 这里可以放心复制，不会有损耗

        let local_file_new_result =
            LocalFile::new(http_client, path, global_config.clone());

        match local_file_new_result {
            Ok(local_file) => {
                local_files.push(Ok(local_file));
            }
            Err(error) => {
                println!("[process_local_file] {}", error);
                local_files.push(Err(error));
            }
        }
    }

    local_files
}

#[async_trait]
impl LocalFolders for WebDavClient {
    async fn get_local_folders(
        &self,
        key: &ClientKey,
        paths: &[&str],
    ) -> Result<LocalFileCollections, String> {
        let http_client =
            self.get_http_client(key).map_err(|e| e.to_string())?;

        let global_config = self.get_global_config();

        // 只把成功的拿出来，失败的需要用时再拿
        let local_files: Vec<LocalFile> =
            process_local_file(http_client, paths, global_config)
                .into_iter()
                .filter_map(|result| result.ok())
                .collect();

        // 构建 LocalFileCollections（使用 HashMap 自动去重）
        let mut file_collections = LocalFileCollections::new();

        for local_file in local_files {
            let path =
                local_file.get_data().path.to_string_lossy().to_string();

            // 使用 insert 直接覆盖，实现去重
            let mut local_files_vec = LocalFiles::new();
            local_files_vec.files.push(local_file);
            file_collections.insert(path, local_files_vec);
        }

        Ok(file_collections)
    }
}
