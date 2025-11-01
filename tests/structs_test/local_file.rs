use webdav_client::local_file::public::get_local_folders::get_local_folders;
use webdav_client::local_file::structs::local_file::LocalFile;

/// 测试 get_local_folders 函数
///
/// 该测试验证：
/// 1. 能够成功读取指定目录
/// 2. 返回的 LocalFile 列表不为空
/// 3. 能够访问每个 LocalFile 的响应式属性
#[tokio::test]
async fn test_get_local_folders() -> Result<(), String> {
    let dir_path = r"C:\project\rust\quick-sync\temp-download-files";

    // 使用 get_local_folders 函数读取文件夹（非递归，只读取一层）
    let local_files = get_local_folders(dir_path).await?;

    // 打印结果
    println!("\n========== 测试 get_local_folders ==========");
    println!("目录路径: {}", dir_path);
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
    let dir_path = r"C:\this\path\does\not\exist\at\all";

    let result = get_local_folders(dir_path).await;

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
    let file_path = r"C:\project\rust\quick-sync\webdav-client\Cargo.toml";

    let result = get_local_folders(file_path).await;

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
    let file_path = r"C:\project\rust\quick-sync\webdav-client\Cargo.toml";
    let local_file = LocalFile::new(file_path).await?;

    println!("测试文件: {}", file_path);
    println!("  is_file(): {}", local_file.is_file());
    println!("  is_dir(): {}", local_file.is_dir());

    assert!(local_file.is_file(), "Cargo.toml 应该是文件");
    assert!(!local_file.is_dir(), "Cargo.toml 不应该是目录");

    // 测试目录
    let dir_path = r"C:\project\rust\quick-sync\temp-download-files";
    let local_dir = LocalFile::new(dir_path).await?;

    println!("\n测试目录: {}", dir_path);
    println!("  is_file(): {}", local_dir.is_file());
    println!("  is_dir(): {}", local_dir.is_dir());

    assert!(local_dir.is_dir(), "temp-download-files 应该是目录");
    assert!(!local_dir.is_file(), "temp-download-files 不应该是文件");

    println!("==========================================\n");

    Ok(())
}
