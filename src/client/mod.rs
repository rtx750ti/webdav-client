pub mod error;
pub mod structs;
pub mod traits;
pub mod traits_impl;
use crate::client::structs::client_key::ClientKey;
use crate::client::structs::client_value::HttpClient;
use std::collections::HashMap;
use std::sync::Arc;
pub type THttpClientArc = Arc<HttpClient>; // 这里的Arc是共享的，并且永远不会被修改，只会被删除，所以可以设计无锁结构

/// WebDav客户端对象
/// - clients就是存储的账号
/// - Key就用来定位到客户端
/// - Value就是一个对应账号的http服务器
pub struct WebDavClient {
    clients: HashMap<ClientKey, THttpClientArc>,
}

impl WebDavClient {
    pub fn new() -> Self {
        Self { clients: HashMap::new() }
    }
}
