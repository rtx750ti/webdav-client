pub mod get_folders_public_impl;

use base64::Engine;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, Url};
use sha2::{Digest, Sha256};

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

pub fn format_base_url(url: &str) -> Result<Url, String> {
    if url.is_empty() {
        return Err("路径为空".to_string());
    }

    let mut base_url = Url::parse(url).map_err(|e| e.to_string())?;

    if !base_url.path().ends_with('/') {
        let new_path = format!("{}/", base_url.path());
        base_url.set_path(&new_path);
    }

    Ok(base_url)
}

pub fn encrypt_str(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}
