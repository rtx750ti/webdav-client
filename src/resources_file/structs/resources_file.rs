use crate::client::structs::client_key::TClientKey;
#[cfg(feature = "activate")]
use crate::file_explorer::ReplyStatus;
#[cfg(feature = "activate")]
use crate::file_explorer::TReplySender;
#[cfg(feature = "activate")]
use crate::file_explorer::{BroadcastCommand, ReplyEvent};
#[cfg(feature = "activate")]
use crate::resource_collector;
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
    /// 给上层发送消息的汇报发送器
    #[cfg(feature = "activate")]
    reply_sender: TReplySender,
    #[cfg(feature = "activate")]
    unique_key: ResourceFileUniqueKey,
}

impl ResourcesFile {
    #[cfg(feature = "activate")]
    pub fn new(
        data: ResourceFileData,
        http_client: Client,
        reply_sender: TReplySender,
        client_key: TClientKey,
    ) -> Self {
        let relative_path =
            data.etag.clone().unwrap_or(data.relative_root_path.clone());

        Self {
            data: Arc::new(data),
            http_client,
            reply_sender,
            unique_key: ResourceFileUniqueKey {
                client_key,
                relative_path,
            },
        }
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

#[cfg(feature = "activate")]
impl ResourcesFile {
    pub async fn activate(
        &self,
        mut broadcast_receiver: resource_collector::TBroadcastReceiver,
    ) {
        let reply_sender = self.reply_sender.clone();

        let _ = reply_sender
            .send(ReplyEvent {
                reply_status: ReplyStatus::Activated(
                    self.unique_key.clone(),
                ),
                version: 0,
            })
            .await;

        tokio::spawn(async move {
            while let Ok(event) = broadcast_receiver.recv().await {
                let version = &event.version;
                match &event.command {
                    BroadcastCommand::Pause(unique_key) => {}
                    BroadcastCommand::PauseAll => {}
                    BroadcastCommand::Restart(unique_key) => {}
                    BroadcastCommand::RestartAll => {}
                }
            }
        });
    }
}
