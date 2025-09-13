use crate::file_explorer::{
    BroadcastEvent, ReplyEvent, ReplyStatus, TBroadcastSender,
    TReplySender,
};
use tokio::sync::{broadcast, mpsc};

pub mod enums;
pub mod structs;
pub mod traits;
mod traits_impl;

const BroadcastBufferSize: usize = 2000;

const ChannelBufferSize: usize = 2000;

pub type TBroadcastReceiver = broadcast::Receiver<BroadcastEvent>;

/// 资源收集器
///
/// 用于收集激活后的资源文件的活动数据
#[derive(Debug, Clone)]
pub struct ResourceCollector {
    /// 广播发送器
    ///
    /// 用于给子结构体向下发送广播命令，仅能在本结构体中使用
    broadcast_sender: TBroadcastSender,
    /// 汇报发送器
    ///
    /// 用于给子结构体向上发送事件使用，本结构体不允许使用
    child_reply_sender: TReplySender,
    /// 给上层发送消息的汇报发送器
    self_reply_sender: TReplySender,
}

impl ResourceCollector {
    pub fn new(self_reply_sender: TReplySender) -> Self {
        let (broadcast_sender, _) =
            broadcast::channel(BroadcastBufferSize);

        let (child_reply_sender, mut reply_receiver) =
            mpsc::channel::<ReplyEvent>(ChannelBufferSize);

        tokio::spawn(async move {
            while let Some(reply_event) = reply_receiver.recv().await {
                let reply_version = &reply_event.version;
                match &reply_event.reply_status {
                    ReplyStatus::Reject(reject_cause) => {}
                    ReplyStatus::Resolve(resolve_message) => {}
                    ReplyStatus::Ack => {}
                    ReplyStatus::Activated(unique_key) => {}
                }
            }
        });

        Self { broadcast_sender, child_reply_sender, self_reply_sender }
    }

    /// 获取汇报发送器
    pub fn get_reply_sender(&self) -> TReplySender {
        self.child_reply_sender.clone()
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
    pub fn subscribe(&self) -> crate::file_explorer::TBroadcastReceiver {
        self.broadcast_sender.subscribe()
    }
}
