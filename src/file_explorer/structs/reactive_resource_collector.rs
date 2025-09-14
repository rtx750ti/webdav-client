use crate::client::structs::client_key::ClientKey;
use crate::file_explorer::TReplySender;
use crate::resource_collector::ResourceCollector;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::watch;

pub type TResourceCollectorMap =
    HashMap<ClientKey, Arc<ResourceCollector>>;

pub type TReactiveResourceCollectorsSender =
    watch::Sender<TResourceCollectorMap>;

pub type TReactiveResourceCollectorsReceiver =
    watch::Receiver<TResourceCollectorMap>;

#[derive(Debug, Clone)]
pub struct ReactiveResourceCollectors {
    sender: TReactiveResourceCollectorsSender,
    receiver: TReactiveResourceCollectorsReceiver,
    reply_sender: Option<TReplySender>,
}

impl ReactiveResourceCollectors {
    pub fn new(reply_sender: Option<TReplySender>) -> Self {
        let empty_resource_collector_map = HashMap::new();

        let (sender, receiver) =
            watch::channel(empty_resource_collector_map);

        Self { sender, receiver, reply_sender }
    }

    pub fn update_reply_sender(
        &mut self,
        reply_sender: TReplySender,
    ) -> Result<(), String> {
        self.reply_sender = Some(reply_sender);
        Ok(())
    }

    pub fn insert(&self, key: &ClientKey) -> Result<(), String> {
        if let Some(reply_sender) = self.reply_sender.clone() {
            let resource_collector = ResourceCollector::new(reply_sender);

            let mut map = self.sender.borrow().clone();

            map.insert(key.clone(), Arc::new(resource_collector));

            self.sender.send(map).map_err(|e| {
                format!("尝试修改响应式资源收集器失败 {}", e.to_string())
            })?;

            Ok(())
        } else {
            Err("reply_sender为None，资源管理器没初始化".to_string())
        }
    }

    pub fn get(&self, key: &ClientKey) -> Option<Arc<ResourceCollector>> {
        let map = self.receiver.borrow();
        map.get(key).cloned()
    }

    pub fn remove(
        &self,
        key: &ClientKey,
    ) -> Option<Arc<ResourceCollector>> {
        let mut map = self.receiver.borrow().clone();
        map.remove(key)
    }
}
