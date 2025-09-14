use crate::client::structs::client_key::ClientKey;
use crate::client::traits::account::{
    Account, AccountError, AddAccountError,
};
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

        #[cfg(feature = "activate")]
        {
            let _ = self
                .file_explorer
                .reactive_resource_collectors
                .insert(&key)
                .map_err(|e| {
                    AddAccountError::InsertResourceCollectorError(e)
                })?;
        }

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
