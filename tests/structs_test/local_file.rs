use webdav_client::local_file::public::get_local_folders::get_local_folders;
use webdav_client::local_file::structs::local_file::LocalFile;

// ==================== 全局测试常量 ====================

/// 测试目录路径
const TEST_DIR: &str = r"C:\project\rust\quick-sync\temp-download-files";

/// 测试文件路径（Cargo.toml）
const TEST_FILE_CARGO_TOML: &str = r"C:\project\rust\quick-sync\webdav-client\Cargo.toml";

/// 测试可执行文件路径（core.exe）
const TEST_FILE_CORE_EXE: &str = r"C:\project\rust\quick-sync\temp-download-files\core.exe";

/// 不存在的测试路径
const TEST_NONEXISTENT_PATH: &str = r"C:\this\path\does\not\exist\at\all";

// ======================================================

/// 测试 get_local_folders 函数
///
/// 该测试验证：
/// 1. 能够成功读取指定目录
/// 2. 返回的 LocalFile 列表不为空
/// 3. 能够访问每个 LocalFile 的响应式属性
#[tokio::test]
async fn test_get_local_folders() -> Result<(), String> {
    // 使用 get_local_folders 函数读取文件夹（非递归，只读取一层）
    let local_files = get_local_folders(TEST_DIR).await?;

    // 打印结果
    println!("\n========== 测试 get_local_folders ==========");
    println!("目录路径: {}", TEST_DIR);
    println!("找到 {} 个文件/目录:", local_files.len());

    for (index, local_file) in local_files.iter().enumerate() {
        let state = local_file.get_reactive_state();
        let name = state.get_reactive_name().watch();
        let name_value = name.borrow();

        // 获取文件元数据
        let data = local_file.get_data();
        let meta = data.get_meta().await?;

        // 使用 LocalFile 的 is_dir() 方法
        let file_type = if local_file.is_dir() { "目录" } else { "文件" };
        let size_info = if local_file.is_dir() {
            String::new()
        } else {
            format!(" ({} 字节)", meta.len)
        };

        println!(
            "  [{}] {} - {}{}",
            index + 1,
            file_type,
            name_value.as_ref().unwrap_or(&"未知".to_string()),
            size_info
        );
    }
    println!("==========================================\n");

    // 验证至少找到了一些文件或目录
    assert!(
        !local_files.is_empty(),
        "temp-download-files 文件夹应该包含文件或目录"
    );

    Ok(())
}

/// 测试 get_local_folders 对不存在路径的错误处理
#[tokio::test]
async fn test_get_local_folders_nonexistent_path() {
    let result = get_local_folders(TEST_NONEXISTENT_PATH).await;

    // 应该返回错误
    assert!(result.is_err(), "不存在的路径应该返回错误");

    if let Err(e) = result {
        println!("预期的错误信息: {}", e);
        assert!(e.contains("路径不存在"), "错误信息应该包含'路径不存在'");
    }
}

/// 测试 get_local_folders 对文件路径（非目录）的错误处理
#[tokio::test]
async fn test_get_local_folders_file_path() -> Result<(), String> {
    // 使用 Cargo.toml 作为测试文件（肯定存在）
    let result = get_local_folders(TEST_FILE_CARGO_TOML).await;

    // 应该返回错误，因为这是一个文件而不是目录
    assert!(result.is_err(), "文件路径应该返回错误");

    if let Err(e) = result {
        println!("预期的错误信息: {}", e);
        assert!(e.contains("不是目录"), "错误信息应该包含'不是目录'");
    }

    Ok(())
}

/// 测试 LocalFile 的 is_dir() 和 is_file() 方法
#[tokio::test]
async fn test_local_file_is_dir_and_is_file() -> Result<(), String> {
    println!("\n========== 测试 is_dir() 和 is_file() 方法 ==========");

    // 测试文件
    let local_file = LocalFile::new(TEST_FILE_CARGO_TOML).await?;

    println!("测试文件: {}", TEST_FILE_CARGO_TOML);
    println!("  is_file(): {}", local_file.is_file());
    println!("  is_dir(): {}", local_file.is_dir());

    assert!(local_file.is_file(), "Cargo.toml 应该是文件");
    assert!(!local_file.is_dir(), "Cargo.toml 不应该是目录");

    // 测试目录
    let local_dir = LocalFile::new(TEST_DIR).await?;

    println!("\n测试目录: {}", TEST_DIR);
    println!("  is_file(): {}", local_dir.is_file());
    println!("  is_dir(): {}", local_dir.is_dir());

    assert!(local_dir.is_dir(), "temp-download-files 应该是目录");
    assert!(!local_dir.is_file(), "temp-download-files 不应该是文件");

    println!("==========================================\n");

    Ok(())
}

/// 测试文件在读写模式下打开后是否能重命名
///
/// 该测试验证：
/// 1. 读取 core.exe 文件
/// 2. 使用 LocalFile::new 打开文件（内部使用读写模式）
/// 3. 在异步任务中持续尝试修改文件名，持续10秒
/// 4. 检查文件名是否真正能够被修改
#[tokio::test]
async fn test_file_rename_while_opened() -> Result<(), String> {
    use tokio::fs;
    use std::path::Path;
    use std::time::Duration;

    println!("\n========== 测试文件在读写模式下打开后能否重命名 ==========");

    // 测试文件路径
    let original_file_path = TEST_FILE_CORE_EXE;
    let renamed_file_path = format!("{}_renamed.exe", &TEST_FILE_CORE_EXE[..TEST_FILE_CORE_EXE.len() - 4]);

    // 检查原文件是否存在
    if !Path::new(original_file_path).exists() {
        return Err(format!("测试文件不存在: {}", original_file_path));
    }

    println!("✅ 找到测试文件: {}", original_file_path);

    // 清理可能存在的重命名文件
    let _ = fs::remove_file(&renamed_file_path).await;

    // 使用 LocalFile::new 打开文件（会以读写模式打开）
    let local_file = LocalFile::new(original_file_path).await?;

    println!("✅ 已打开文件（读写模式）");
    println!("   文件名: {:?}", local_file.get_reactive_state().get_reactive_name().get_current());

    // 克隆路径用于异步任务
    let original_path = original_file_path.to_string();
    let renamed_path = renamed_file_path.clone();

    // 启动异步任务，持续尝试重命名文件，持续10秒
    let rename_task = tokio::spawn(async move {
        let start_time = std::time::Instant::now();
        let mut attempt_count = 0;
        let mut success_count = 0;
        let mut last_state_is_renamed = false;

        println!("🔄 开始异步重命名任务（持续10秒）...");

        while start_time.elapsed() < Duration::from_secs(10) {
            attempt_count += 1;

            // 根据当前状态决定重命名方向
            let (from, to) = if !last_state_is_renamed {
                (original_path.as_str(), renamed_path.as_str())
            } else {
                (renamed_path.as_str(), original_path.as_str())
            };

            // 尝试重命名
            match fs::rename(from, to).await {
                Ok(_) => {
                    success_count += 1;
                    last_state_is_renamed = !last_state_is_renamed;
                    println!(
                        "   ✅ 第 {} 次尝试成功: {} -> {}",
                        attempt_count,
                        Path::new(from).file_name().unwrap().to_string_lossy(),
                        Path::new(to).file_name().unwrap().to_string_lossy()
                    );
                }
                Err(e) => {
                    println!(
                        "   ❌ 第 {} 次尝试失败: {} (错误: {})",
                        attempt_count,
                        Path::new(from).file_name().unwrap().to_string_lossy(),
                        e
                    );
                }
            }

            // 短暂延迟，避免过于频繁
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        println!("⏱️  10秒已到，重命名任务结束");
        println!("📊 统计: 总尝试 {} 次，成功 {} 次", attempt_count, success_count);

        (success_count, last_state_is_renamed)
    });

    // 等待异步任务完成
    let (success_count, last_state_is_renamed) = rename_task.await
        .map_err(|e| format!("异步任务执行失败: {}", e))?;

    // 显式释放 local_file
    drop(local_file);
    println!("✅ 已释放文件句柄");

    // 恢复文件名到原始状态（如果最后状态是重命名的）
    if last_state_is_renamed {
        println!("🔄 恢复文件名到原始状态...");
        fs::rename(&renamed_file_path, original_file_path)
            .await
            .map_err(|e| format!("恢复文件名失败: {}", e))?;
        println!("✅ 文件名已恢复");
    }

    // 判断测试结果
    if success_count > 0 {
        println!("\n✅ 测试结果: 文件在读写模式下打开时，可以被重命名（成功 {} 次）", success_count);
    } else {
        println!("\n❌ 测试结果: 文件在读写模式下打开时，无法被重命名");
        return Err("文件在读写模式下打开时无法重命名，可能被锁定".to_string());
    }

    println!("==========================================\n");

    Ok(())
}

