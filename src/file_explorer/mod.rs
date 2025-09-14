use crate::client::structs::client_key::ClientKey;
use crate::client::structs::reactive_child_clients::TReactiveChildClientsReceiver;
use crate::file_explorer::structs::reactive_resource_collector::ReactiveResourceCollectors;
use crate::resource_collector::ResourceCollector;
use crate::resources_file::structs::resources_file::ResourceFileUniqueKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, watch};

pub mod enums;
pub mod structs;
pub mod traits;
mod traits_impl;

pub type TVersion = u32;

pub type TUniqueKey = String;

pub type TBroadcastCommand = BroadcastCommand;

#[derive(Debug, Clone)]
pub enum BroadcastCommand {
    Pause(TUniqueKey),
    PauseAll,
    Restart(TUniqueKey),
    RestartAll,
}

pub type RejectCause = String;
pub type ResolveMessage = String;

#[derive(Debug, Clone)]
pub enum ReplyStatus {
    Reject(RejectCause),
    Resolve(ResolveMessage),
    Ack,
    Activated(ResourceFileUniqueKey),
}

#[derive(Debug, Clone)]
pub struct ReplyEvent {
    pub(crate) reply_status: ReplyStatus,
    pub(crate) version: TVersion,
}

#[derive(Debug, Clone)]
pub struct BroadcastEvent {
    pub(crate) command: BroadcastCommand,
    pub(crate) version: TVersion,
}

pub type TBroadcastSender = broadcast::Sender<BroadcastEvent>;
pub type TBroadcastReceiver = broadcast::Receiver<BroadcastEvent>;
pub type TReplySender = mpsc::Sender<ReplyEvent>;

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

    pub(crate) reactive_resource_collectors: ReactiveResourceCollectors,
}

impl FileExplorer {
    pub fn new() -> Self {
        let (broadcast_sender, _) =
            broadcast::channel(BroadcastBufferSize as usize);

        let reactive_resource_collectors =
            ReactiveResourceCollectors::new(None);

        Self {
            broadcast_sender,
            reply_sender: None,
            reactive_resource_collectors,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let (sender, mut receiver) =
            mpsc::channel::<ReplyEvent>(ChannelBufferSize);
        
        self.reply_sender = Some(sender.clone());
        
        self.reactive_resource_collectors.update_reply_sender(sender)?;

        Ok(())
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
