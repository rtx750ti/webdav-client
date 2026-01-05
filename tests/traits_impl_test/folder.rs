use crate::{WEBDAV_ENV_PATH_1, load_account};
use webdav_client::client::WebDavClient;
use webdav_client::client::enums::Depth;
use webdav_client::client::traits::client::Account;
use webdav_client::client::traits::remote::{
    Folders, FoldersError, GetFoldersError,
};

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

    let data = client.get_folders(&key, &["./"], &Depth::One).await?;

    println!("获取的结果：{:?}", data);

    Ok(())
}
