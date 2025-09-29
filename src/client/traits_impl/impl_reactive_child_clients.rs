use crate::client::structs::client_key::ClientKey;
use crate::client::structs::client_value::HttpClient;
use crate::client::structs::reactive_child_clients::ReactiveChildClients;
use crate::client::traits::account::{
    Account, AccountError, AddAccountError, GetHttpClientError,
    RemoveAccountError, RemoveAccountForceError,
};
use crate::client::THttpClientArc;
use std::sync::Arc;

impl Account for ReactiveChildClients {
    fn add_account(
        &self,
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<ClientKey, AccountError> {
        let key = ClientKey::new(base_url, username)
            .map_err(|e| AddAccountError::CreateKeyError(e))?;

        let http_client = HttpClient::new(base_url, username, password)
            .map_err(|e| AddAccountError::CreateHttpClientError(e))?;

        self.insert(key.clone(), Arc::new(http_client));
        Ok(key)
    }

    fn remove_account(&self, key: &ClientKey) -> Result<(), AccountError> {
        let client = self.get_http_client(key)?;

        if !Self::can_modify_value(&client) {
            return Err(RemoveAccountError::ClientInUse(format!(
                "客户端未释放，无法删除：地址='{}', 账号='{}'",
                key.get_base_url(),
                key.get_username()
            ))
            .into());
        }

        let mut map = self.receiver.borrow().clone();
        match map.remove(key) {
            Some(_) => {
                let _ = self.sender.send(map);
                Ok(())
            }
            None => Err(RemoveAccountError::DeleteFailed(format!(
                "删除失败：地址='{}', 账号='{}'",
                key.get_base_url(),
                key.get_username()
            ))
            .into()),
        }
    }

    fn get_http_client(
        &self,
        key: &ClientKey,
    ) -> Result<THttpClientArc, AccountError> {
        match self.receiver.borrow().get(key) {
            Some(client) => Ok(client.clone()),
            None => Err(GetHttpClientError::NotFindClient(format!(
                "未找到客户端：地址='{}', 账号='{}'",
                key.get_base_url(),
                key.get_username()
            ))
            .into()),
        }
    }

    fn remove_account_force(
        &self,
        key: &ClientKey,
    ) -> Result<(), AccountError> {
        let mut map = self.receiver.borrow().clone();
        match map.remove(key) {
            Some(_) => {
                let _ = self.sender.send(map);
                Ok(())
            }
            None => Err(RemoveAccountForceError::RemoveError(format!(
                "强制删除失败：地址='{}', 账号='{}'",
                key.get_base_url(),
                key.get_username()
            ))
            .into()),
        }
    }
}
