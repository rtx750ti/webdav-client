use webdav_client::client::enums::depth::Depth;
use webdav_client::client::traits::_self::account::Account;
use webdav_client::client::traits::remote::folders::{Folders, FoldersError, GetFoldersError};
use crate::{load_account, WEBDAV_ENV_PATH_1};
use webdav_client::client::WebDavClient;

#[tokio::test]
async fn test_get_folders() -> Result<(), FoldersError> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| {
            FoldersError::GetFoldersError(GetFoldersError::AccountError(e))
        })?;

    let data = client
        .get_folders(&key, &vec!["./".to_string()], &Depth::One)
        .await?;

    println!("获取的结果：{:?}", data);

    Ok(())
}
