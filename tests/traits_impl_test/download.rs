use crate::{load_account, WEBDAV_ENV_PATH_2};
use memory_stats::memory_stats;
use rand::{thread_rng, RngCore};
use webdav_client::remote_file::impl_traits::impl_download::handle_download::HandleDownloadError;
use std::time::Duration;
use tokio::time::Instant;
use webdav_client::client::enums::depth::Depth;
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::folders::Folders;
use webdav_client::client::WebDavClient;
use webdav_client::remote_file::traits::download::{
    Download, DownloadError,
};

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

    for vec_remote_files in data {
        for remote_file in vec_remote_files {
            let _remotes_file_arc = remote_file
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

    for vec_remote_files in data {
        for remote_file in vec_remote_files {
            // 假设这里有你现成的获取方法：请替换为你代码里真实存在的方法名
            let state = remote_file.get_reactive_state();

            let mut watcher = state.get_download_bytes().watch();
            let total = remote_file.get_data().size.unwrap();

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
            let _ = remote_file
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

    for vec_remote_files in data {
        for remote_file in vec_remote_files {
            // 假设这里有你现成的获取方法：请替换为你代码里真实存在的方法名
            let state = remote_file.get_reactive_state();

            let mut watcher = state.get_download_bytes().watch();
            let total = remote_file.get_data().size.unwrap();

            let config = remote_file.get_reactive_config();
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
            let _ = remote_file
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

#[tokio::test(flavor = "current_thread")]
async fn test_performance() -> Result<(), String> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Barrier;
    use webdav_client::remote_file::structs::remote_file_property::RemoteFileProperty;

    let watcher_count = std::env::var("QS_REACTIVE_WATCHERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100_0000);
    let update_count = std::env::var("QS_REACTIVE_UPDATES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1_0000);

    let state = RemoteFileProperty::new("reactive-bench".to_string());
    let download_bytes = state.get_download_bytes().clone();

    let barrier = Arc::new(Barrier::new(watcher_count + 1));
    let ready = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(watcher_count);
    for _ in 0..watcher_count {
        let mut watcher = download_bytes.watch();
        let barrier = barrier.clone();
        let ready = ready.clone();

        handles.push(tokio::spawn(async move {
            ready.fetch_add(1, Ordering::SeqCst);
            barrier.wait().await;

            let mut last = watcher.borrow().unwrap_or_default();
            loop {
                let v = watcher.changed().await.map_err(|e| e.to_string())?;
                if v < last {
                    return Err(format!(
                        "download_bytes 非单调递增: {} -> {}",
                        last, v
                    ));
                }
                last = v;
                if v >= update_count {
                    break;
                }
            }
            Ok::<(), String>(())
        }));
    }

    while ready.load(Ordering::SeqCst) < watcher_count {
        tokio::task::yield_now().await;
    }

    barrier.wait().await;

    let start = Instant::now();
    for v in 1..=update_count {
        download_bytes
            .update(v)
            .map_err(|e| format!("更新失败: {e}"))?;

        if v % 256 == 0 {
            tokio::task::yield_now().await;
        }
    }
    let update_duration = start.elapsed();

    for handle in handles {
        let join = tokio::time::timeout(Duration::from_secs(10), handle)
            .await
            .map_err(|_| "监听器等待超时".to_string())?;
        join.map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
    }

    let total_duration = start.elapsed();
    let updates_per_sec = (update_count as f64) / update_duration.as_secs_f64();

    println!(
        "ReactiveProperty响应式属性性能测试(flavor = \"current_thread\")\n 监听器数量={} 每个监听器更新次数={}\n 更新耗时={:.2?} ({:.0} 次修改/s)\n全量收敛耗时={:.2?}",
        watcher_count,
        update_count,
        update_duration,
        updates_per_sec,
        total_duration
    );

    if let Some(stats) = memory_stats() {
        println!("物理内存使用: {} bytes", stats.physical_mem);
        println!("虚拟内存使用: {} bytes", stats.virtual_mem);
    }

    assert!(
        total_duration < Duration::from_secs(10),
        "响应式状态收敛过慢: {total_duration:.2?}"
    );

    Ok(())
}

/// 测试重复下载
#[tokio::test]
async fn test_download_repeat() -> Result<(), String> {
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
            &vec![
                "./测试文件夹/新建 文本文档.txt".to_string(),
                "./测试文件夹/新建 文本文档.txt".to_string(),
                "./测试文件夹/新建 文本文档.txt".to_string(),
                "./测试文件夹/新建 文本文档.txt".to_string(),
                "./测试文件夹/新建 文本文档.txt".to_string(),
            ],
            &Depth::One,
        )
        .await
        .map_err(|e| e.to_string())?;

    // 并发下载
    let mut handles = Vec::new();

    // 记录失败的次数
    let mut existing_files = 0;

    for vec_remote_files in data {
        for remote_file in vec_remote_files {
            let handle = tokio::spawn(async move {
                let res = remote_file
                    .download("C:\\project\\rust\\quick-sync\\temp-download-files\\")
                    .await;
                if let Err(err) = res {
                    match err {
                        DownloadError::HandleDownloadError(e) => match e {
                            HandleDownloadError::PathExists(_) => {
                                existing_files += 1;
                                existing_files
                            }
                            _ => 0,
                        },
                        _ => 0,
                    }
                } else {
                    0
                }
            });
            handles.push(handle);
        }
    }

    // 等待所有下载完成
    for handle in handles {
        let _ = handle.await.map_err(|e| e.to_string())?;
    }

    // 检查是否有跳过的下载任务
    assert_eq!(existing_files, 4, "跳过4个文件，成功下载1个");

    Ok(())
}
