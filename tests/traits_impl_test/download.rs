use crate::{WEBDAV_ENV_PATH_1, load_account};
use webdav_client::client::WebDavClient;
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::folder::Folders;
use webdav_client::public::enums::depth::Depth;
use webdav_client::resources_file::structs::download_config::DownloadConfig;
use webdav_client::resources_file::traits::download::Download;

#[tokio::test]
async fn test_download() -> Result<(), String> {
    let mut client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    let data = client
        .get_folders(
            &key,
            &vec!["./".to_string(), "./书签".to_string()],
            &Depth::One,
        )
        .await
        .map_err(|e| e.to_string())?;

    let config = DownloadConfig::default();

    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            let _resources_file_arc = resources_file
                .download("C:\\project\\rust\\", &config)
                .await?
                .stop()
                .await?
                .start()
                .await?
                .stop()
                .await?
                .restart()
                .await?;
        }
    }

    // println!("获取的结果：{:?}", data);

    Ok(())
}
