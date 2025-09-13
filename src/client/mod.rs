pub mod structs;
pub mod traits;
pub mod traits_impl;
use crate::client::structs::client_value::HttpClient;
use crate::client::structs::ref_web_dav_child_clients::RefWebDavChildClients;
#[cfg(feature = "activate")]
use crate::file_explorer::FileExplorer;
use std::sync::Arc;

pub type THttpClientArc = Arc<HttpClient>; // 这里的Arc是共享的，并且永远不会被修改，只会被删除，所以可以设计无锁结构

/// WebDav客户端对象
/// - clients就是存储的账号
/// - Key就用来定位到客户端
/// - Value就是一个对应账号的http服务器
pub struct WebDavClient {
    child_clients: RefWebDavChildClients,
    #[cfg(feature = "activate")]
    file_explorer: FileExplorer,
}

impl WebDavClient {
    pub fn new() -> Self {
        let child_clients = RefWebDavChildClients::new();
        #[cfg(feature = "activate")]
        {
            Self { child_clients, file_explorer: FileExplorer::new() }
        }

        #[cfg(not(feature = "activate"))]
        {
            Self { child_clients }
        }
    }
}
