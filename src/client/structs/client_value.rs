use crate::client::THttpClientArc;
use base64::Engine;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, Url};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use crate::client::format_base_url::format_base_url;
use std::fmt;

pub fn encrypt_str(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Clone)]
pub struct HttpClient {
    client: Client, // 这个客户端本身的clone就已经在内部实现了Rc，所以就不用Arc了
    base_url: Url,
    encrypted_username: String,
    encrypted_password: String,
}

impl fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpClient")
            .field("client", &"<Client with hidden authorization>")
            .field("base_url", &self.base_url)
            .field("encrypted_username", &self.encrypted_username)
            .field("encrypted_password", &self.encrypted_password)
            .finish()
    }
}

impl HttpClient {
    pub fn new(
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, String> {
        let encrypted_username = encrypt_str(username);
        let encrypted_password = encrypt_str(password);
        let base_url =
            format_base_url(base_url).map_err(|e| e.to_string())?;
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

pub fn gen_http_client(
    username: &str,
    password: &str,
) -> Result<Client, String> {
    let mut headers = HeaderMap::new();

    let token = base64::engine::general_purpose::STANDARD
        .encode(format!("{username}:{password}"));

    let auth_val = HeaderValue::from_str(&format!("Basic {token}"))
        .map_err(|e| e.to_string())?;

    headers.insert(AUTHORIZATION, auth_val);

    let client = Client::builder()
        .http1_only()
        .default_headers(headers)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(client)
}
