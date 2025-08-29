use crate::resources_file::structs::resource_file_data::ResourceFileData;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct ResourcesFile {
    data: ResourceFileData,
    http_client: Client,
}

impl ResourcesFile {
    pub fn new(data: ResourceFileData, http_client: Client) -> Self {
        Self { data, http_client }
    }

    /// 获取资源文件的元数据
    pub fn get_data(&self) -> &ResourceFileData {
        &self.data
    }

    /// 获取 HTTP 客户端
    pub fn get_http_client(&self) -> &Client {
        &self.http_client
    }
}
