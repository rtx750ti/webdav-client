use crate::{WEBDAV_ENV_PATH_1, WEBDAV_ENV_PATH_2, load_account};
use memory_stats::memory_stats;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::time::Instant;
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
        .get_folders(&key, &vec!["./".to_string()], &Depth::One)
        .await
        .map_err(|e| e.to_string())?;

    let config = DownloadConfig::default();
    let config = Arc::new(config);

    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            let _resources_file_arc = resources_file
                .download(
                    "C:\\project\\rust\\quick-sync\\temp-download-files\\",
                    config.clone(),
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
            Arc::new(config),
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
                    downloader.download("C:\\project\\rust\\quick-sync\\temp-download-files\\单个下载", download_task, Arc::new(config)).await;
                return Ok(());
            }
        }
    }
    Ok(())
}

#[cfg(feature = "reactive")]
#[tokio::test]
async fn test_reactive_data() -> Result<(), String> {
    if let Some(stats) = memory_stats() {
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
            .get_folders(&key, &vec!["./".to_string()], &Depth::One)
            .await
            .map_err(|e| e.to_string())?;

        // 确保有文件可以测试
        if data.is_empty() || data[0].is_empty() {
            return Err("没有找到可测试的文件".to_string());
        }

        let file = &data[0][0];

        // 获取响应式名称属性
        let name = file.get_reactive_name();
        let initial_name = name.get_current().unwrap();
        println!("初始名称: {}", initial_name);

        // 使用 mpsc 通道替代 oneshot
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        // 启动监听任务
        for i in 0..30 {
            let mut watch_clone = name.watch();
            let tx_clone = tx.clone();

            tokio::spawn(async move {
                match watch_clone.changed().await {
                    Ok(new_value) => {
                        println!("监听器 {} 收到新名称: {}", i, new_value);
                        let _ = tx_clone.send(i).await;
                    }
                    Err(e) => {
                        println!("监听器 {} 错误: {}", i, e);
                        let _ = tx_clone.send(i).await;
                    }
                }
            });
        }

        // 等待一小段时间确保监听任务启动
        tokio::time::sleep(Duration::from_millis(1000)).await;

        let start = Instant::now();

        // 更新名称以触发监听器
        for i in 0..1_000_000 {
            let new_name = format!("{}_updated{}", initial_name, i);
            name.update(new_name.clone())
                .map_err(|e| format!("更新失败: {}", e))?;
            // 验证名称是否更新
            let current_name = name.get_current().unwrap();
            assert_eq!(current_name, new_name, "名称更新失败");
        }

        let duration = start.elapsed();

        // 等待监听任务完成（最多1秒）
        tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .map_err(|_| "监听器超时".to_string())?
            .ok_or("通道关闭".to_string())?;

        println!(
            "1_000_000 次 30 个监听对象名称更新总耗时: {:.2?}",
            duration
        );
        println!("物理内存使用: {} bytes", stats.physical_mem);
        println!("虚拟内存使用: {} bytes", stats.virtual_mem);

        println!("测试成功完成");
        Ok(())
    } else {
        Ok(())
    }
}
