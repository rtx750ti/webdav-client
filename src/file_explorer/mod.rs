use crate::client::structs::client_key::ClientKey;
use crate::resource_collector::ResourceCollector;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

pub mod enums;
pub mod structs;
pub mod traits;
mod traits_impl;

#[derive(Debug, Clone)]
pub enum ChannelEvent {
    Ping,
}

#[derive(Debug, Clone)]
pub enum BroadcastEvent {}

pub type TBroadcastSender = broadcast::Sender<BroadcastEvent>;
pub type TBroadcastReceiver = broadcast::Receiver<BroadcastEvent>;
pub type TReplySender = mpsc::Sender<ChannelEvent>;

const BroadcastBufferSize: usize = 2000;

const ChannelBufferSize: usize = 2000;

/// 资源管理器
///
/// 用于管理整个Webdav客户端的全部资源和收集器
///
#[derive(Debug, Clone)]
pub struct FileExplorer {
    /// 广播发送器
    ///
    /// 用于给子结构体向下发送广播命令，仅能在本结构体中使用
    broadcast_sender: TBroadcastSender,
    /// 汇报发送器
    ///
    /// 用于给子结构体向上发送事件使用，本结构体不允许使用
    reply_sender: Option<TReplySender>,
    /// 资源收集器表
    ///
    /// 记录不同客户端对应的资源收集器，一个客户端对应一个
    resource_collector_map: HashMap<ClientKey, ResourceCollector>,
}

impl FileExplorer {
    pub fn new() -> Self {
        let (broadcast_sender, _) =
            broadcast::channel(BroadcastBufferSize as usize);
        Self {
            broadcast_sender,
            resource_collector_map: HashMap::new(),
            reply_sender: None,
        }
    }

    pub fn get_resource_collector(
        &self,
        key: &ClientKey,
    ) -> Option<&ResourceCollector> {
        self.resource_collector_map.get(key)
    }

    pub fn start(&mut self) {
        let (sender, receiver) =
            mpsc::channel::<ChannelEvent>(ChannelBufferSize);

        self.reply_sender = Some(sender);
    }

    /// 获取汇报发送器
    pub fn get_reply_sender(&self) -> Option<TReplySender> {
        self.reply_sender.clone()
    }

    /// 广播一个事件给所有监听者
    pub fn broadcast_event(
        &self,
        event: BroadcastEvent,
    ) -> Result<(), String> {
        let _ = self.broadcast_sender.send(event);
        Ok(())
    }

    /// 获取一个事件接收器
    pub fn subscribe(&self) -> TBroadcastReceiver {
        self.broadcast_sender.subscribe()
    }
}
