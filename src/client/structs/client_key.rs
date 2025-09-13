use std::sync::Arc;
use crate::public::utils::format_base_url;
use reqwest::Url;

pub type TClientKey = Arc<ClientKey>;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct ClientKey {
    base_url: Url,
    username: String,
}

impl ClientKey {
    pub fn new(base_url: &str, username: &str) -> Result<Self, String> {
        let base_url =
            format_base_url(base_url).map_err(|e| e.to_string())?;

        let username = username.to_string();
        Ok(Self { base_url, username })
    }

    pub fn get_base_url(&self) -> Url {
        self.base_url.to_owned()
    }

    pub fn get_username(&self) -> String {
        self.username.to_owned()
    }
}
