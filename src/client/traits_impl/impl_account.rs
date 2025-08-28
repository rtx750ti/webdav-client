use crate::client::structs::client_key::ClientKey;
use crate::client::structs::client_value::HttpClient;
use crate::client::traits::account::Account;
use crate::client::{THttpClientArc, WebDavClient};
use async_trait::async_trait;

#[async_trait]
impl Account for WebDavClient {
    fn add_account(
        &mut self,
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<ClientKey, String> {
        let key = ClientKey::new(base_url, username)?;
        let http_client = HttpClient::new(base_url, username, password)?;
        self.clients.insert(key.to_owned(), http_client.into());
        Ok(key)
    }

    fn remove_account(&mut self, key: &ClientKey) -> Result<(), String> {
        let http_client_arc = self.get_http_client(key)?;

        if !Self::can_modify_value(&http_client_arc) {
            return Err("该账号未释放".to_string());
        }

        match self.clients.remove(&key) {
            Some(_) => Ok(()),
            None => Err("[remove_account] 删除失败".to_string()),
        }
    }

    fn get_http_client(
        &self,
        key: &ClientKey,
    ) -> Result<THttpClientArc, String> {
        let client = match self.clients.get(key).cloned() {
            Some(c) => Ok(c),
            None => Err("".to_string()),
        }?;

        Ok(client)
    }

    fn remove_account_force(
        &mut self,
        key: &ClientKey,
    ) -> Result<(), String> {
        match self.clients.remove(&key) {
            Some(_) => Ok(()),
            None => Err("[remove_account] 删除失败".to_string()),
        }
    }
}
