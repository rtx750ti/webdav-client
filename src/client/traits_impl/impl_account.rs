use crate::client::structs::client_key::ClientKey;
use crate::client::structs::client_value::HttpClient;
use crate::client::traits::account::{
    Account, AccountError, AddAccountError, GetHttpClientError,
    RemoveAccountError, RemoveAccountForceError,
};
use crate::client::{THttpClientArc, WebDavClient};
use async_trait::async_trait;

#[async_trait]
impl Account for WebDavClient {
    fn add_account(
        &mut self,
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<ClientKey, AccountError> {
        let key = ClientKey::new(base_url, username)
            .map_err(|e| AddAccountError::CreateKeyError(e))?;

        let http_client = HttpClient::new(base_url, username, password)
            .map_err(|e| AddAccountError::CreateHttpClientError(e))?;

        self.clients.insert(key.to_owned(), http_client.into());
        Ok(key)
    }

    fn remove_account(
        &mut self,
        key: &ClientKey,
    ) -> Result<(), AccountError> {
        let http_client_arc = self.get_http_client(key)?;

        let base_msg = format!(
            "[remove_account] 地址为'{}',账号为'{}'的客户端",
            key.get_base_url(),
            key.get_username()
        );

        if !Self::can_modify_value(&http_client_arc) {
            return Err(AccountError::RemoveAccountError(
                RemoveAccountError::ClientInUse(format!(
                    "{}未释放",
                    base_msg
                )),
            ));
        }

        match self.clients.remove(&key) {
            Some(_) => Ok(()),
            None => Err(AccountError::RemoveAccountError(
                RemoveAccountError::ClientInUse(format!(
                    "{}删除失败",
                    base_msg
                )),
            )),
        }
    }

    fn get_http_client(
        &self,
        key: &ClientKey,
    ) -> Result<THttpClientArc, AccountError> {
        let client = match self.clients.get(key).cloned() {
            Some(c) => Ok(c),
            None => Err(AccountError::GetHttpClientError(
                GetHttpClientError::NotFindClient(format!(
                    "获取不到地址为'{}',账号为'{}'的客户端",
                    key.get_base_url(),
                    key.get_username()
                )),
            )),
        }?;

        Ok(client)
    }

    fn remove_account_force(
        &mut self,
        key: &ClientKey,
    ) -> Result<(), AccountError> {
        match self.clients.remove(&key) {
            Some(_) => Ok(()),
            None => Err(AccountError::RemoveAccountForceError(
                RemoveAccountForceError::RemoveError(format!(
                    "地址为'{}',账号为'{}'的客户端强制删除失败",
                    key.get_base_url(),
                    key.get_username()
                )),
            )),
        }
    }
}
