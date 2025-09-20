use crate::client::structs::client_key::ClientKey;
use crate::client::traits::account::{Account, AccountError};
use crate::client::{THttpClientArc, WebDavClient};

impl Account for WebDavClient {
    fn add_account(
        &self,
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<ClientKey, AccountError> {
        let key = self
            .child_clients
            .add_account(base_url, username, password)?;

        Ok(key)
    }

    fn remove_account(&self, key: &ClientKey) -> Result<(), AccountError> {
        self.child_clients.remove_account(key)
    }

    fn get_http_client(
        &self,
        key: &ClientKey,
    ) -> Result<THttpClientArc, AccountError> {
        self.child_clients.get_http_client(key)
    }

    fn remove_account_force(
        &self,
        key: &ClientKey,
    ) -> Result<(), AccountError> {
        self.child_clients.remove_account_force(key)
    }
}
