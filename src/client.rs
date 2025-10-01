pub mod enums;
mod format_base_url;
mod impl_traits;
pub mod structs;
pub mod traits;
pub mod webdav_request;

use crate::client::structs::client_value::HttpClient;
use crate::client::structs::reactive_child_clients::ReactiveChildClients;
use crate::global_config::global_config::GlobalConfig;
use std::sync::Arc;

pub type THttpClientArc = Arc<HttpClient>; // 这里的Arc是共享的，并且永远不会被修改，只会被删除，所以可以设计无锁结构

/// WebDav客户端对象
/// - clients就是存储的账号
/// - Key就用来定位到客户端
/// - Value就是一个对应账号的http服务器
pub struct WebDavClient {
    child_clients: ReactiveChildClients,
    global_config: GlobalConfig,
}

impl WebDavClient {
    pub fn new() -> Self {
        let child_clients = ReactiveChildClients::new();
        let global_config = GlobalConfig::default();

        Self { child_clients, global_config }
    }

    pub fn get_global_config(&self) -> GlobalConfig {
        self.global_config.clone()
    }
}
