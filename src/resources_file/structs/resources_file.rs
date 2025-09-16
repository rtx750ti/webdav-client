use crate::client::structs::client_key::TClientKey;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use reqwest::Client;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResourceFileUniqueKey {
    pub client_key: TClientKey,
    pub relative_path: String,
}

#[derive(Debug, Clone)]
pub struct ResourcesFile {
    data: Arc<ResourceFileData>,
    http_client: Client,
}

impl ResourcesFile {
    pub fn new(data: ResourceFileData, http_client: Client) -> Self {
        Self { data: Arc::new(data), http_client }
    }

    /// 获取资源文件的元数据
    pub fn get_data(&self) -> Arc<ResourceFileData> {
        self.data.clone()
    }

    /// 获取 HTTP 客户端
    pub fn get_http_client(&self) -> &Client {
        &self.http_client
    }
}
