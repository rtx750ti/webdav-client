use crate::client::THttpClientArc;
use crate::client::structs::client_key::ClientKey;
use crate::client::structs::client_value::HttpClient;
use crate::client::traits::account::{
    Account, AccountError, AddAccountError, GetHttpClientError,
    RemoveAccountError, RemoveAccountForceError,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::watch;

pub type TWebDavChildClients = HashMap<ClientKey, THttpClientArc>;

pub struct RefWebDavChildClients {
    pub(crate) sender: watch::Sender<TWebDavChildClients>,
    pub(crate) receiver: watch::Receiver<TWebDavChildClients>,
}

impl RefWebDavChildClients {
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
}
