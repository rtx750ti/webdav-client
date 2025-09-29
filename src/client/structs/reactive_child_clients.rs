use crate::client::structs::client_key::ClientKey;
use crate::client::THttpClientArc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::watch;

pub type TWebDavChildClients = HashMap<ClientKey, THttpClientArc>;

pub type TReactiveChildClientsReceiver =
    watch::Receiver<TWebDavChildClients>;
pub type TReactiveChildClientsSender = watch::Sender<TWebDavChildClients>;

#[derive(Debug, Clone)]
pub struct ReactiveChildClients {
    pub(crate) sender: TReactiveChildClientsSender,
    pub(crate) receiver: TReactiveChildClientsReceiver,
}

impl ReactiveChildClients {
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(HashMap::new());
        Self { sender, receiver }
    }

    pub fn insert(&self, key: ClientKey, client: THttpClientArc) {
        let mut map = self.receiver.borrow().clone();
        map.insert(key, client);
        let _ = self.sender.send(map);
    }

    pub(crate) fn can_modify_value<T>(arc_client: &Arc<T>) -> bool {
        let strong = Arc::strong_count(&arc_client);
        if strong > 2 { false } else { true }
    }

    pub fn get_reactive_receiver(&self) -> TReactiveChildClientsReceiver {
        self.receiver.clone()
    }
}
