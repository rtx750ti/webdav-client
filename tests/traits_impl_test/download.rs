use crate::{WEBDAV_ENV_PATH_2, load_account};
use memory_stats::memory_stats;
use rand::{RngCore, thread_rng};
use std::time::Duration;
use tokio::time::Instant;
use webdav_client::client::WebDavClient;
use webdav_client::client::enums::depth::Depth;
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::folders::Folders;
use webdav_client::resource_file::traits::download::Download;

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
        .get_folders(&key, &vec!["./测试文件夹".to_string()], &Depth::One)
        .await
        .map_err(|e| e.to_string())?;

    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            let _resources_file_arc = resources_file
                .download(
                    "C:\\project\\rust\\quick-sync\\temp-download-files\\",
                )
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_download_progress_monitoring() -> Result<(), String> {
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
            &vec!["./测试文件夹/新建文件夹/hula.exe".to_string()],
            &Depth::One,
        )
        .await
        .map_err(|e| e.to_string())?;

    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            // 假设这里有你现成的获取方法：请替换为你代码里真实存在的方法名
            let state = resources_file.get_reactive_state();

            let mut watcher = state.get_download_bytes().watch();
            let total = resources_file.get_data().size.unwrap();

            // 启动监听
            tokio::spawn({
                // 可选：拿个名字快照用于打印
                let name = state
                    .get_reactive_name()
                    .get_current()
                    .unwrap_or_default();

                async move {
                    while let Ok(bytes) = watcher.changed().await {
                        println!(
                            "文件 [{}] 进度: {} bytes ({:.2}%)",
                            name,
                            bytes,
                            (bytes as f64 / total as f64) * 100.0
                        );
                    }
                }
            });

            // 调用现有的 download，不改签名
            let _ = resources_file
                .download(
                    "C:\\project\\rust\\quick-sync\\temp-download-files\\",
                )
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_download_pause() -> Result<(), String> {
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
            &vec!["./测试文件夹/新建文件夹/hula.exe".to_string()],
            &Depth::One,
        )
        .await
        .map_err(|e| e.to_string())?;

    let global_config = client.get_global_config();

    global_config.enable_pause_switch().map_err(|e| e.to_string())?;
    println!("配置：{:?}", global_config.get_current());

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;

        global_config.try_pause().unwrap();

        tokio::time::sleep(Duration::from_secs(6)).await;

        global_config.try_resume().unwrap();
    });

    for vec_resources_files in data {
        for resources_file in vec_resources_files {
            // 假设这里有你现成的获取方法：请替换为你代码里真实存在的方法名
            let state = resources_file.get_reactive_state();

            let mut watcher = state.get_download_bytes().watch();
            let total = resources_file.get_data().size.unwrap();

            let config = resources_file.get_reactive_config();
            let _config_watcher = config.watch();

            // 启动监听
            tokio::spawn({
                // 可选：拿个名字快照用于打印
                let name = state
                    .get_reactive_name()
                    .get_current()
                    .unwrap_or_default();

                async move {
                    while let Ok(bytes) = watcher.changed().await {
                        println!(
                            "文件 [{}] 进度: {} bytes ({:.2}%)",
                            name,
                            bytes,
                            (bytes as f64 / total as f64) * 100.0
                        );
                    }
                }
            });

            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(3)).await;

                config.update_field(|data| data.pause = true).unwrap();

                let d = config.get_current().unwrap();

                println!("内部配置：{:?}", d);

                tokio::time::sleep(Duration::from_secs(2)).await;

                config.update_field(|data| data.pause = false).unwrap();
            });

            // 调用现有的 download，不改签名
            let _ = resources_file
                .download(
                    "C:\\project\\rust\\quick-sync\\temp-download-files\\",
                )
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub fn random_bytes_2mb() -> Vec<u8> {
    const SIZE: usize = 25;
    let mut buf = vec![0u8; SIZE];
    thread_rng().fill_bytes(&mut buf);
    buf
}

#[test]
fn generates_2mb() {
    let s = random_bytes_2mb();
    assert_eq!(s.len(), 255);
}

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
                    Ok(_new_value) => {
                        // println!("监听器 {} 收到新名称: {}", i, new_value);
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
        for _i in 0..1_000_000 {
            let new_name = format!("{:?}", random_bytes_2mb());
            name.update(new_name.clone())
                .map_err(|e| format!("更新失败: {}", e))?;
            // 验证名称是否更新
            let current_name = name.get_current_borrow();
            assert_eq!(
                current_name.as_ref().unwrap(),
                &new_name,
                "名称更新失败"
            );
        }

        let duration = start.elapsed();

        // 等待监听任务完成（最多1秒）
        tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .map_err(|_| "监听器超时".to_string())?
            .ok_or("通道关闭".to_string())?;

        println!(
            "1_000_000 次 30 个监听对象名称，每个名称长度255字符，更新总耗时: {:.2?}",
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
