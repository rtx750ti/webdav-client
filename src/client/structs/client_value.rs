use crate::client::THttpClientArc;
use crate::public::utils::{
    encrypt_str, format_base_url, gen_http_client,
};
use reqwest::{Client, Url};
use std::sync::Arc;

pub struct HttpClient {
    client: Client, // 这个客户端本身的clone就已经在内部实现了Rc，所以就不用Arc了
    base_url: Url,
    encrypted_username: String,
    encrypted_password: String,
}

impl HttpClient {
    pub fn new(
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, String> {
        let encrypted_username = encrypt_str(username);
        let encrypted_password = encrypt_str(password);
        let base_url = format_base_url(base_url)
            .map_err(|e| e.to_string())?;
        let client = gen_http_client(username, password)
            .unwrap_or(Client::default());

        Ok(Self {
            client,
            base_url,
            encrypted_username,
            encrypted_password,
        })
    }

    pub fn get_base_url(&self) -> Url {
        self.base_url.to_owned()
    }

    pub fn into(self) -> THttpClientArc {
        Arc::new(self)
    }

    /// 这个函数获取的是一个客户端实体，但是它是被Arc内部克隆的，所以并不会有资源损耗
    pub fn get_client(&self) -> Client {
        self.client.clone()
    }
}

impl PartialEq for HttpClient {
    fn eq(&self, other: &Self) -> bool {
        self.encrypted_username.eq(&other.encrypted_username)
            && self.encrypted_password.eq(&other.encrypted_password)
    }
}
