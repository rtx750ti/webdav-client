use crate::{WEBDAV_ENV_PATH_1, WEBDAV_ENV_PATH_2, load_account};
use webdav_client::client::WebDavClient;
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::folder::Folders;
use webdav_client::download_config::DownloadConfig;
use webdav_client::downloader::Downloader;
use webdav_client::downloader::enums::download_task::DownloadTask;
use webdav_client::downloader::traits::batch_download::BatchDownload;
use webdav_client::public::enums::depth::Depth;
use webdav_client::resources_file::traits::download::Download;

#[tokio::test]
async fn test_download() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_2);

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
            &vec!["./测试文件夹/测试文件3.txt".to_string()],
            &Depth::One,
        )
        .await
        .map_err(|e| e.to_string())?;

    println!("内容：{:?}", data);

    let config = DownloadConfig::default();

    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            let _resources_file_arc = resources_file
                .download(
                    "C:\\project\\rust\\quick-sync\\temp-download-files\\",
                    &config,
                )
                .await?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_downloader_multiply_task() -> Result<(), String> {
    let client = WebDavClient::new();

    let webdav_account = load_account(WEBDAV_ENV_PATH_2);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    {}

    let downloader = Downloader::new();

    let data = client
        .get_folders(&key, &vec!["./测试文件夹".to_string()], &Depth::One)
        .await
        .map_err(|e| e.to_string())?;

    let download_task = DownloadTask::from(data.clone());

    let mut config = DownloadConfig::default();
    config.auto_download_folder = true;

    downloader
        .download(
            "C:\\project\\rust\\quick-sync\\temp-download-files\\",
            download_task,
            &config,
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_downloader_single_task() -> Result<(), String> {
    let client = WebDavClient::new();

    let webdav_account = load_account(WEBDAV_ENV_PATH_2);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    {}

    let downloader = Downloader::new();

    let data = client
        .get_folders(&key, &vec!["./测试文件夹".to_string()], &Depth::One)
        .await
        .map_err(|e| e.to_string())?;

    let download_task = DownloadTask::from(data.clone());

    let config = DownloadConfig::default();

    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            if !resources_file.get_data().is_dir {
                let download_task = DownloadTask::from(resources_file);
                let _ =
                    downloader.download("C:\\project\\rust\\quick-sync\\temp-download-files\\单个下载", download_task, &config).await;
                return Ok(());
            }
        }
    }
    Ok(())
}
