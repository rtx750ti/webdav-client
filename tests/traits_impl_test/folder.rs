use crate::{WEBDAV_ENV_PATH_1, WEBDAV_ENV_PATH_2, load_account};
use webdav_client::client::WebDavClient;
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::folder::Folders;
use webdav_client::public::enums::depth::Depth;

#[tokio::test]
async fn test_get_folders() -> Result<(), String> {
    let mut client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client.add_account(
        &webdav_account.url,
        &webdav_account.username,
        &webdav_account.password,
    )?;

    let data = client
        .get_folders(
            &key,
            &vec!["./".to_string(), "./书签".to_string()],
            &Depth::One,
        )
        .await?;

    println!("获取的结果：{:?}", data);

    Ok(())
}
