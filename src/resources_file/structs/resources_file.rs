#[cfg(feature = "activate")]
use crate::file_explorer::TReplySender;
use crate::resources_file::structs::resource_file_data::ResourceFileData;
use reqwest::Client;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ResourcesFile {
    data: Arc<ResourceFileData>,
    http_client: Client,
    /// 给上层发送消息的汇报发送器
    #[cfg(feature = "activate")]
    pub reply_sender: TReplySender,
}

impl ResourcesFile {
    #[cfg(feature = "activate")]
    pub fn new(
        data: ResourceFileData,
        http_client: Client,
        reply_sender: TReplySender,
    ) -> Self {
        Self { data: Arc::new(data), http_client, reply_sender }
    }

    #[cfg(not(feature = "activate"))]
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

    #[cfg(feature = "activate")]
    pub fn get_reply_sender(&self) -> TReplySender {
        self.reply_sender.clone()
    }
}
