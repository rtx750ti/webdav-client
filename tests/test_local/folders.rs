use crate::{load_account, WEBDAV_ENV_PATH_1};
use memory_stats::memory_stats;
use rand::{thread_rng, RngCore};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::Instant;
use std::time::Duration;
use webdav_client::client::traits::client::Account;
use webdav_client::client::traits::local::folders::LocalFolders;
use webdav_client::client::WebDavClient;
use webdav_client::local_file::structs::LocalFile;

#[tokio::test]
async fn test_get_local_folders() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试单个目录路径（基础测试）
    let paths = vec!["C:\\project\\rust\\quick-sync\\temp-download-files"];
    let result = client.get_local_folders(&key, &paths).await?;

    println!("获取的本地文件夹结果：{:?}", result);
    println!("文件夹数量：{}", result.file_collections.len());

    for (path, files) in result.file_collections {
        println!("路径: {}", path);
        println!("  文件数量: {}", files.files.len());
        for file in &files.files {
            let data = file.get_data();
            println!("  - 文件名: {:?}, 是否目录: {}", data.file_name, data.is_dir);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_single_file() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试单个文件路径
    let paths = vec!["C:\\project\\rust\\quick-sync\\temp-download-files\\hula.exe"];
    let result = client.get_local_folders(&key, &paths).await?;

    println!("测试单个文件：");
    println!("处理的路径数量：{}", result.file_collections.len());

    for (path, files) in result.file_collections.iter() {
        println!("路径: {}", path);
        println!("  文件数量: {}", files.files.len());
        assert_eq!(files.files.len(), 1, "每个路径应该对应1个LocalFile对象");
        
        let file = &files.files[0];
        let data = file.get_data();
        println!("  - 文件名: {:?}, 是否目录: {}", data.file_name, data.is_dir);
        assert_eq!(data.is_dir, false, "hula.exe 应该是文件");
    }

    assert_eq!(result.file_collections.len(), 1, "应该有1个路径");

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_single_dir() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试单个目录路径
    let paths = vec!["C:\\project\\rust\\quick-sync\\temp-download-files"];
    let result = client.get_local_folders(&key, &paths).await?;

    println!("测试单个目录：");
    println!("处理的路径数量：{}", result.file_collections.len());

    for (path, files) in result.file_collections.iter() {
        println!("路径: {}", path);
        println!("  文件数量: {}", files.files.len());
        assert_eq!(files.files.len(), 1, "每个路径应该对应1个LocalFile对象");
        
        let file = &files.files[0];
        let data = file.get_data();
        println!("  - 文件名: {:?}, 是否目录: {}", data.file_name, data.is_dir);
        assert_eq!(data.is_dir, true, "temp-download-files 应该是目录");
    }

    assert_eq!(result.file_collections.len(), 1, "应该有1个路径");

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_mixed_file_and_dir() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试文件和目录混用
    let paths = vec![
        "C:\\project\\rust\\quick-sync\\temp-download-files",           // 目录
        "C:\\project\\rust\\quick-sync\\temp-download-files\\hula.exe", // 文件
        "C:\\project\\rust\\quick-sync\\temp-download-files\\新建 文本文档.txt", // 文件
    ];
    let result = client.get_local_folders(&key, &paths).await?;

    println!("测试文件和目录混用：");
    println!("处理的路径数量：{}", result.file_collections.len());
    assert_eq!(result.file_collections.len(), 3, "应该有3个路径");

    for (path, files) in result.file_collections.iter() {
        println!("路径: {}", path);
        println!("  文件数量: {}", files.files.len());
        assert_eq!(files.files.len(), 1, "每个路径对应1个LocalFile对象");
        
        let file = &files.files[0];
        let data = file.get_data();
        println!("  - 文件名: {:?}, 是否目录: {}", data.file_name, data.is_dir);
    }

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_multiple_paths() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试多个不同路径
    let paths = vec![
        "C:\\project\\rust\\quick-sync\\temp-download-files",
        "C:\\project\\rust\\quick-sync\\webdav-client\\tests",
        "C:\\project\\rust\\quick-sync\\temp-download-files\\hula.exe",
    ];
    let result = client.get_local_folders(&key, &paths).await?;

    println!("测试多个路径：");
    println!("处理的路径数量：{}", result.file_collections.len());
    assert_eq!(result.file_collections.len(), 3, "应该有3个路径");

    for (path, files) in result.file_collections.iter() {
        println!("路径: {}", path);
        println!("  文件数量: {}", files.files.len());
        assert_eq!(files.files.len(), 1, "每个路径对应1个LocalFile对象");
    }

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_with_invalid_paths() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试包含无效路径的情况（文件、目录、无效路径混合）
    let paths = vec![
        "C:\\project\\rust\\quick-sync\\temp-download-files\\hula.exe",  // 有效文件
        "C:\\invalid\\path\\that\\does\\not\\exist",                      // 无效路径
        "C:\\project\\rust\\quick-sync\\temp-download-files",            // 有效目录
        "C:\\also\\invalid\\file.txt",                                     // 无效文件
    ];
    let result = client.get_local_folders(&key, &paths).await?;

    println!("测试包含无效路径的混合情况：");
    println!("成功处理的路径数量：{}", result.file_collections.len());
    assert_eq!(result.file_collections.len(), 2, "应该有2个有效路径");

    for (path, files) in result.file_collections.iter() {
        println!("成功处理的路径: {}", path);
        println!("  文件数量: {}", files.files.len());
        assert_eq!(files.files.len(), 1, "每个路径对应1个LocalFile对象");
    }

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_duplicate_paths() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试重复路径
    let paths = vec![
        "C:\\project\\rust\\quick-sync\\temp-download-files\\hula.exe",
        "C:\\project\\rust\\quick-sync\\temp-download-files\\hula.exe", // 重复
        "C:\\project\\rust\\quick-sync\\temp-download-files",
    ];
    let result = client.get_local_folders(&key, &paths).await?;

    println!("测试重复路径：");
    println!("处理的路径数量：{}", result.file_collections.len());
    // HashMap 会自动去重，重复的路径会被覆盖
    assert_eq!(result.file_collections.len(), 2, "应该有2个不同的路径（重复被去除）");

    for (path, files) in result.file_collections.iter() {
        println!("路径: {}", path);
        println!("  文件数量: {}", files.files.len());
        assert_eq!(files.files.len(), 1, "每个路径只有最后一次的LocalFile对象");
    }

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_empty_paths() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试空路径数组
    let paths: Vec<&str> = vec![];
    let result = client.get_local_folders(&key, &paths).await?;

    println!("测试空路径数组：");
    println!("处理的路径数量：{}", result.file_collections.len());

    assert_eq!(result.file_collections.len(), 0, "空数组应该返回空结果");

    Ok(())
}

/// 生成255字节的随机内容
fn random_content_255() -> String {
    let mut buf = vec![0u8; 255];
    thread_rng().fill_bytes(&mut buf);
    // 转换为可打印字符
    buf.iter()
        .map(|&b| (b % 94 + 33) as char) // ASCII 可打印字符范围
        .collect()
}

/// 性能测试：并发批量调用get_local_folders（每组100个路径）
// cargo test --test lib test_get_local_folders_concurrent_batch -- --nocapture
#[tokio::test]
async fn test_get_local_folders_concurrent_batch() -> Result<(), String> {
    let client = Arc::new(WebDavClient::new());
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 从环境变量读取文件数量，默认1万个
    let file_count = std::env::var("QS_LOCAL_FILES_COUNT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10_000);

    let test_dir = PathBuf::from("C:\\project\\rust\\quick-sync\\temp-download-files\\test-concurrent-batch");

    println!("\n开始并发性能测试：批量调用get_local_folders");
    println!("总文件数: {}", file_count);
    println!("每批路径数: 100");
    println!("测试目录: {:?}", test_dir);

    // 1. 创建测试目录
    println!("\n[步骤1] 创建测试目录...");
    if test_dir.exists() {
        println!("  目录已存在，清理旧文件...");
        fs::remove_dir_all(&test_dir).map_err(|e| format!("删除目录失败: {}", e))?;
    }
    fs::create_dir_all(&test_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    println!("  ✓ 测试目录创建成功");

    // 2. 生成测试文件
    println!("\n[步骤2] 生成 {} 个测试文件...", file_count);
    let create_start = Instant::now();
    let mut created_files = Vec::with_capacity(file_count);

    for i in 0..file_count {
        let file_name = format!("test_file_{:05}.txt", i);
        let file_path = test_dir.join(&file_name);
        let content = random_content_255();

        fs::write(&file_path, content)
            .map_err(|e| format!("写入文件 {} 失败: {}", file_name, e))?;

        created_files.push(file_path.to_string_lossy().to_string());

        // 每1000个文件输出一次进度
        if (i + 1) % 1000 == 0 {
            println!("  进度: {}/{} ({:.1}%)", i + 1, file_count, ((i + 1) as f64 / file_count as f64) * 100.0);
        }
    }

    let create_duration = create_start.elapsed();
    println!("  ✓ 文件创建完成，耗时: {:.2?}", create_duration);
    println!("  ✓ 实际创建文件数: {}", created_files.len());

    // 手动确认点：让用户检查文件
    println!("\n⏸️  [暂停] 请检查测试目录中的文件是否真实存在...");
    println!("  目录: {:?}", test_dir);
    println!("  按任意键继续...");
    
    // 等待用户按键（这段时间不计入性能统计）
    use std::io::{self, BufRead};
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut input = String::new();
    handle.read_line(&mut input).map_err(|e| format!("读取输入失败: {}", e))?;
    println!("  ✓ 继续执行...");

    // 3. 并发批量调用 get_local_folders
    println!("\n[步骤3] 并发批量调用 get_local_folders...");
    println!("  每批100个路径，共 {} 批", (file_count + 99) / 100);
    
    let concurrent_start = Instant::now();
    
    // 将文件路径分成每组100个
    let batch_size = 100;
    let mut tasks = vec![];
    
    for chunk in created_files.chunks(batch_size) {
        let client_clone = Arc::clone(&client);
        let key_clone = key.clone();
        let paths: Vec<String> = chunk.to_vec();
        
        let task = tokio::spawn(async move {
            let paths_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            client_clone.get_local_folders(&key_clone, &paths_refs).await
        });
        
        tasks.push(task);
    }
    
    // 等待所有任务完成
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await);
    }
    let concurrent_duration = concurrent_start.elapsed();
    
    println!("  ✓ 并发批量调用完成，耗时: {:.2?}", concurrent_duration);

    // 4. 统计结果
    println!("\n[步骤4] 统计结果...");
    let mut total_success = 0;
    let mut total_failed = 0;
    
    for (idx, result) in results.iter().enumerate() {
        match result {
            Ok(Ok(collections)) => {
                total_success += collections.file_collections.len();
            }
            Ok(Err(e)) => {
                println!("  批次 {} 失败: {}", idx, e);
                total_failed += 1;
            }
            Err(e) => {
                println!("  批次 {} 任务异常: {}", idx, e);
                total_failed += 1;
            }
        }
    }
    
    println!("  ✓ 成功处理的文件数: {}", total_success);
    println!("  ✓ 失败的批次数: {}", total_failed);

    // 5. 性能统计
    println!("\n[步骤5] 性能统计...");
    let avg_create_micros = create_duration.as_micros() as f64 / file_count as f64;
    let create_per_sec = file_count as f64 / create_duration.as_secs_f64();

    println!("  文件系统写入性能（fs::write）:");
    println!("    总耗时: {:.2?}", create_duration);
    println!("    吞吐量: {:.0} 个/秒", create_per_sec);
    println!("    平均耗时: {:.2} μs/个", avg_create_micros);

    println!("  get_local_folders 并发批量调用性能:");
    println!("    总耗时: {:.2?}", concurrent_duration);
    let concurrent_per_sec = file_count as f64 / concurrent_duration.as_secs_f64();
    let avg_concurrent_micros = concurrent_duration.as_micros() as f64 / file_count as f64;
    println!("    吞吐量: {:.0} 个/秒", concurrent_per_sec);
    println!("    平均耗时: {:.2} μs/个", avg_concurrent_micros);
    println!("    总批次数: {}", (file_count + 99) / 100);

    // 输出内存使用情况
    if let Some(stats) = memory_stats() {
        println!("  内存使用:");
        println!("    物理内存: {:.2} MB", stats.physical_mem as f64 / 1024.0 / 1024.0);
        println!("    虚拟内存: {:.2} MB", stats.virtual_mem as f64 / 1024.0 / 1024.0);
    }

    // 6. 清理测试文件
    println!("\n[步骤6] 清理测试文件...");
    let cleanup_start = Instant::now();
    fs::remove_dir_all(&test_dir)
        .map_err(|e| format!("清理测试目录失败: {}", e))?;
    let cleanup_duration = cleanup_start.elapsed();
    println!("  ✓ 清理完成，耗时: {:.2?}", cleanup_duration);

    // 7. 性能断言
    println!("\n[步骤7] 性能断言...");
    assert_eq!(total_success, file_count, "应该成功处理所有文件");
    assert_eq!(total_failed, 0, "不应该有失败的批次");
    
    assert!(
        concurrent_duration < Duration::from_secs(60),
        "并发批量调用超时: {:.2?}，期望 < 60s",
        concurrent_duration
    );
    
    assert!(
        concurrent_per_sec > 500.0,
        "并发批量调用吞吐量过低: {:.0} 个/秒，期望 > 500 个/秒",
        concurrent_per_sec
    );
    println!("  ✓ 所有断言通过");

    println!("\n✅ 并发性能测试完成！");
    Ok(())
}

/// 性能测试：读取包含1万个小文件的目录
// cargo test --test lib test_get_local_folders_with_many_files -- --nocapture
#[tokio::test]
async fn test_get_local_folders_with_many_files() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 从环境变量读取文件数量，默认1万个
    let file_count = std::env::var("QS_LOCAL_FILES_COUNT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10_000);

    let test_dir = PathBuf::from("C:\\project\\rust\\quick-sync\\temp-download-files\\test-many-files");

    println!("\n开始性能测试：读取包含 {} 个小文件的目录", file_count);
    println!("测试目录: {:?}", test_dir);

    // 1. 创建测试目录
    println!("\n[步骤1] 创建测试目录...");
    if test_dir.exists() {
        println!("  目录已存在，清理旧文件...");
        fs::remove_dir_all(&test_dir).map_err(|e| format!("删除目录失败: {}", e))?;
    }
    fs::create_dir_all(&test_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    println!("  ✓ 测试目录创建成功");

    // 2. 生成测试文件
    println!("\n[步骤2] 生成 {} 个测试文件...", file_count);
    let create_start = Instant::now();
    let mut created_files = Vec::with_capacity(file_count);

    for i in 0..file_count {
        let file_name = format!("test_file_{:05}.txt", i);
        let file_path = test_dir.join(&file_name);
        let content = random_content_255();

        fs::write(&file_path, content)
            .map_err(|e| format!("写入文件 {} 失败: {}", file_name, e))?;

        created_files.push(file_path);

        // 每1000个文件输出一次进度
        if (i + 1) % 1000 == 0 {
            println!("  进度: {}/{} ({:.1}%)", i + 1, file_count, ((i + 1) as f64 / file_count as f64) * 100.0);
        }
    }

    let create_duration = create_start.elapsed();
    println!("  ✓ 文件创建完成，耗时: {:.2?}", create_duration);
    println!("  ✓ 实际创建文件数: {}", created_files.len());

    // 手动确认点：让用户检查文件
    println!("\n⏸️  [暂停] 请检查测试目录中的文件是否真实存在...");
    println!("  目录: {:?}", test_dir);
    println!("  按任意键继续...");
    
    // 等待用户按键（这段时间不计入性能统计）
    use std::io::{self, BufRead};
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut input = String::new();
    handle.read_line(&mut input).map_err(|e| format!("读取输入失败: {}", e))?;
    println!("  ✓ 继续执行...");

    // 3. 读取目录
    println!("\n[步骤3] 读取目录...");
    let test_dir_str = test_dir.to_string_lossy().to_string();
    let paths = vec![test_dir_str.as_str()];

    let read_start = Instant::now();
    let result = client.get_local_folders(&key, &paths).await?;
    let read_duration = read_start.elapsed();

    println!("  ✓ 目录读取完成，耗时: {:.2?}", read_duration);

    // 4. 读取目录内所有文件并创建 LocalFile 对象
    println!("\n[步骤4] 读取目录内所有文件...");
    let read_all_start = Instant::now();
    
    // 获取目录下所有文件路径
    let mut all_file_paths = Vec::new();
    for entry in fs::read_dir(&test_dir).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            all_file_paths.push(path.to_string_lossy().to_string());
        }
    }
    
    // 批量创建所有文件的 LocalFile 对象
    let file_paths_refs: Vec<&str> = all_file_paths.iter().map(|s| s.as_str()).collect();
    let all_files_result = client.get_local_folders(&key, &file_paths_refs).await?;
    
    let read_all_duration = read_all_start.elapsed();
    println!("  ✓ 读取完成，耗时: {:.2?}", read_all_duration);
    println!("  ✓ 读取的文件数: {}", all_file_paths.len());
    println!("  ✓ 返回的 LocalFile 对象数: {}", all_files_result.file_collections.len());

    // 5. 验证结果
    println!("\n[步骤5] 验证结果...");
    assert_eq!(result.file_collections.len(), 1, "应该有1个路径");

    let files = result.file_collections.get(test_dir_str.as_str())
        .ok_or("未找到目录结果")?;

    println!("  ✓ 返回的文件数: {}", files.files.len());
    assert_eq!(files.files.len(), 1, "应该返回1个LocalFile对象（目录本身）");

    // 验证是目录
    let local_file = &files.files[0];
    let data = local_file.get_data();
    assert_eq!(data.is_dir, true, "应该是目录");
    println!("  ✓ 确认返回的是目录对象");

    // 6. 打印前20个和后20个文件名
    println!("\n[步骤6] 文件列表预览...");
    println!("  前20个文件:");
    for (i, file_path) in created_files.iter().take(20).enumerate() {
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        println!("    [{:2}] {}", i + 1, file_name);
    }

    println!("  ...");
    println!("  后20个文件:");
    let start_idx = created_files.len().saturating_sub(20);
    for (i, file_path) in created_files.iter().skip(start_idx).enumerate() {
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        println!("    [{:2}] {}", start_idx + i + 1, file_name);
    }

    // 7. 性能统计
    println!("\n[步骤7] 性能统计...");
    let avg_create_micros = create_duration.as_micros() as f64 / file_count as f64;
    let create_per_sec = file_count as f64 / create_duration.as_secs_f64();

    println!("  文件系统写入性能（fs::write）:");
    println!("    总耗时: {:.2?}", create_duration);
    println!("    吞吐量: {:.0} 个/秒", create_per_sec);
    println!("    平均耗时: {:.2} μs/个", avg_create_micros);

    println!("  get_local_folders 读取单个目录性能:");
    println!("    耗时: {:.2?}", read_duration);

    println!("  get_local_folders 批量读取1万个文件性能:");
    println!("    总耗时: {:.2?}", read_all_duration);
    let read_all_per_sec = file_count as f64 / read_all_duration.as_secs_f64();
    let avg_read_micros = read_all_duration.as_micros() as f64 / file_count as f64;
    println!("    吞吐量: {:.0} 个/秒", read_all_per_sec);
    println!("    平均耗时: {:.2} μs/个", avg_read_micros);

    // 输出内存使用情况
    if let Some(stats) = memory_stats() {
        println!("  内存使用:");
        println!("    物理内存: {:.2} MB", stats.physical_mem as f64 / 1024.0 / 1024.0);
        println!("    虚拟内存: {:.2} MB", stats.virtual_mem as f64 / 1024.0 / 1024.0);
    }

    // 8. 清理测试文件
    println!("\n[步骤8] 清理测试文件...");
    let cleanup_start = Instant::now();
    fs::remove_dir_all(&test_dir)
        .map_err(|e| format!("清理测试目录失败: {}", e))?;
    let cleanup_duration = cleanup_start.elapsed();
    println!("  ✓ 清理完成，耗时: {:.2?}", cleanup_duration);

    // 9. 性能断言
    println!("\n[步骤9] 性能断言...");
    assert!(
        read_duration < Duration::from_secs(5),
        "目录读取超时: {:.2?}，期望 < 5s",
        read_duration
    );
    
    assert!(
        read_all_duration < Duration::from_secs(30),
        "批量读取1万个文件超时: {:.2?}，期望 < 30s",
        read_all_duration
    );
    
    assert!(
        read_all_per_sec > 1000.0,
        "批量读取吞吐量过低: {:.0} 个/秒，期望 > 1000 个/秒",
        read_all_per_sec
    );
    println!("  ✓ 所有断言通过");

    println!("\n✅ 测试完成！");
    Ok(())
}

// ============================================================
// 极限性能测试可配置参数
// ============================================================

/// 默认测试文件数量（可通过环境变量 QS_LOCAL_FILES_COUNT 覆盖）
const DEFAULT_FILE_COUNT: usize = 1_000;

/// 每个文件的更新次数（精度 0.001%，从 0% 到 100% 共 100001 次）
const UPDATES_PER_FILE: usize = 100_000;

/// 完成率断言阈值（%）
const ASSERT_ACCURACY_THRESHOLD: f64 = 99.9;

/// 吐吐量断言阈值（次/秒）
const ASSERT_THROUGHPUT_THRESHOLD: f64 = 1_000_000.0;

/// 单次更新耗时断言阈值（微秒 μs）
const ASSERT_UPDATE_TIME_THRESHOLD_US: f64 = 10.0;

/// 文件创建进度报告间隔（每 N 个文件输出一次）
const FILE_PROGRESS_INTERVAL: usize = 1_000;

/// 极限性能测试：1万个文件的响应式状态更新（精度 0.001%，共 10 亿次更新）
// cargo test --test lib test_reactive_state_extreme_performance -- --nocapture
#[tokio::test]
async fn test_reactive_state_extreme_performance() -> Result<(), String> {
    let client = Arc::new(WebDavClient::new());
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 从环境变量读取文件数量，否则使用默认值
    let file_count = std::env::var("QS_LOCAL_FILES_COUNT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_FILE_COUNT);

    // 精度0.001%，从0%到10%即每个文件更新 UPDATES_PER_FILE 次
    let updates_per_file: usize = UPDATES_PER_FILE;
    let total_updates: u64 = file_count as u64 * updates_per_file as u64;

    let test_dir = PathBuf::from("C:\\project\\rust\\quick-sync\\temp-download-files\\test-reactive-extreme");

    println!("\n🚨 极限性能测试：响应式状态更新");
    println!("{}", "=".repeat(60));
    println!("文件数量: {}", file_count);
    println!("每文件更新次数: {} (精度 0.001%)", updates_per_file);
    println!("总更新次数: {} ({:.2} 亿次)", total_updates, total_updates as f64 / 1e8);
    println!("测试目录: {:?}", test_dir);
    println!("{}", "=".repeat(60));

    // 1. 创建测试目录
    println!("\n[步骤1] 创建测试目录...");
    if test_dir.exists() {
        println!("  目录已存在，清理旧文件...");
        fs::remove_dir_all(&test_dir).map_err(|e| format!("删除目录失败: {}", e))?;
    }
    fs::create_dir_all(&test_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    println!("  ✓ 测试目录创建成功");

    // 2. 生成测试文件
    println!("\n[步骤2] 生成 {} 个测试文件...", file_count);
    let create_start = Instant::now();
    let mut created_files = Vec::with_capacity(file_count);

    for i in 0..file_count {
        let file_name = format!("test_file_{:05}.txt", i);
        let file_path = test_dir.join(&file_name);
        let content = random_content_255();

        fs::write(&file_path, content)
            .map_err(|e| format!("写入文件 {} 失败: {}", file_name, e))?;

        created_files.push(file_path.to_string_lossy().to_string());

        if (i + 1) % FILE_PROGRESS_INTERVAL == 0 || i + 1 == file_count {
            println!("  进度: {}/{} ({:.1}%)", i + 1, file_count, ((i + 1) as f64 / file_count as f64) * 100.0);
        }
    }

    let create_duration = create_start.elapsed();
    println!("  ✓ 文件创建完成，耗时: {:.2?}", create_duration);

    // 手动确认点
    println!("\n⏸️  [暂停] 请检查测试目录中的文件是否真实存在...");
    println!("  目录: {:?}", test_dir);
    println!("  按任意键继续...");
    
    use std::io::{self, BufRead};
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut input = String::new();
    handle.read_line(&mut input).map_err(|e| format!("读取输入失败: {}", e))?;
    println!("  ✓ 继续执行...");

    // 3. 获取所有 LocalFile 对象
    println!("\n[步骤3] 获取所有 LocalFile 对象...");
    let load_start = Instant::now();
    
    let file_paths_refs: Vec<&str> = created_files.iter().map(|s| s.as_str()).collect();
    let result = client.get_local_folders(&key, &file_paths_refs).await?;
    
    // 提取所有 LocalFile
    let mut local_files: Vec<LocalFile> = Vec::with_capacity(file_count);
    for (_, files) in result.file_collections {
        for file in files.files {
            local_files.push(file);
        }
    }
    
    let load_duration = load_start.elapsed();
    println!("  ✓ 加载完成，耗时: {:.2?}", load_duration);
    println!("  ✓ LocalFile 对象数: {}", local_files.len());

    // 4. 极限响应式状态更新测试
    println!("\n[步骤4] 极限响应式状态更新测试...");
    println!("  总更新次数: {} ({:.2} 亿次)", total_updates, total_updates as f64 / 1e8);
    println!("  开始并发更新（无间隔）...");
    
    // 记录内存使用情况 - 测试前
    let mem_before = memory_stats();
    
    let update_start = Instant::now();
    
    // 原子计数器用于统计实际更新次数
    let update_counter = Arc::new(AtomicU64::new(0));
    let error_counter = Arc::new(AtomicU64::new(0));
    
    // 并发更新所有文件的响应式状态
    let mut tasks = Vec::new();
    
    for local_file in local_files {
        let update_counter_clone = Arc::clone(&update_counter);
        let error_counter_clone = Arc::clone(&error_counter);
        
        let task = tokio::spawn(async move {
            let reactive_state = local_file.get_reactive_state();
            let mut success_count = 0u64;
            let mut error_count = 0u64;
            
            // 每个文件从 0 更新到 UPDATES_PER_FILE（模拟 0% 到 100%，精度 0.001%）
            for progress in 0..=UPDATES_PER_FILE {
                match reactive_state.upload_bytes.update(progress) {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
            
            update_counter_clone.fetch_add(success_count, Ordering::Relaxed);
            error_counter_clone.fetch_add(error_count, Ordering::Relaxed);
        });
        
        tasks.push(task);
    }
    
    // 进度监控任务
    let progress_counter = Arc::clone(&update_counter);
    let progress_task = tokio::spawn(async move {
        let mut last_count = 0u64;
        let mut last_time = Instant::now();
        
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            let current_count = progress_counter.load(Ordering::Relaxed);
            let elapsed = last_time.elapsed();
            let delta = current_count - last_count;
            let speed = delta as f64 / elapsed.as_secs_f64();
            
            let progress_pct = (current_count as f64 / total_updates as f64) * 100.0;
            
            println!("  进度: {:.3}% | 已完成: {} | 速度: {:.0} 次/秒", 
                     progress_pct, current_count, speed);
            
            if current_count >= total_updates {
                break;
            }
            
            last_count = current_count;
            last_time = Instant::now();
        }
    });
    
    // 等待所有更新任务完成
    for task in tasks {
        let _ = task.await;
    }
    
    let update_duration = update_start.elapsed();
    
    // 停止进度监控
    progress_task.abort();
    
    // 记录内存使用情况 - 测试后
    let mem_after = memory_stats();
    
    let actual_updates = update_counter.load(Ordering::Relaxed);
    let actual_errors = error_counter.load(Ordering::Relaxed);
    
    println!("\n  ✓ 响应式更新完成！");
    println!("  ✓ 总耗时: {:.2?}", update_duration);
    println!("  ✓ 实际更新次数: {}", actual_updates);
    println!("  ✓ 错误次数: {}", actual_errors);

    // 5. 详细性能统计
    println!("\n[步骤5] 详细性能统计...");
    println!("{}", "=".repeat(60));
    
    // 基础指标
    let updates_per_sec = actual_updates as f64 / update_duration.as_secs_f64();
    let avg_update_ns = update_duration.as_nanos() as f64 / actual_updates as f64;
    let avg_update_us = avg_update_ns / 1000.0;
    
    println!("\n  📊 基础性能指标:");
    println!("    总耗时: {:.2?}", update_duration);
    println!("    实际更新次数: {} ({:.2} 亿次)", actual_updates, actual_updates as f64 / 1e8);
    println!("    吐吐量: {:.0} 次/秒 ({:.2} 百万次/秒)", updates_per_sec, updates_per_sec / 1e6);
    println!("    平均单次更新耗时: {:.2} ns ({:.4} μs)", avg_update_ns, avg_update_us);
    
    // 文件维度指标
    let files_per_sec = file_count as f64 / update_duration.as_secs_f64();
    let avg_file_complete_ms = update_duration.as_millis() as f64 / file_count as f64;
    
    println!("\n  📁 文件维度指标:");
    println!("    文件处理速度: {:.2} 文件/秒", files_per_sec);
    println!("    平均单文件完成时间: {:.2} ms", avg_file_complete_ms);
    println!("    每文件更新次数: {}", updates_per_file);
    
    // 并发性能指标
    let theoretical_serial_time_sec = actual_updates as f64 * avg_update_ns / 1e9;
    let parallelism_factor = theoretical_serial_time_sec / update_duration.as_secs_f64();
    
    println!("\n  ⚡ 并发性能指标:");
    println!("    并发任务数: {} (文件数)", file_count);
    println!("    理论串行时间: {:.2} s", theoretical_serial_time_sec);
    println!("    实际并发时间: {:.2} s", update_duration.as_secs_f64());
    println!("    并发加速比: {:.2}x", parallelism_factor);
    
    // 内存指标
    println!("\n  💾 内存使用指标:");
    if let (Some(before), Some(after)) = (mem_before, mem_after) {
        let physical_delta = after.physical_mem as i64 - before.physical_mem as i64;
        let virtual_delta = after.virtual_mem as i64 - before.virtual_mem as i64;
        
        println!("    测试前物理内存: {:.2} MB", before.physical_mem as f64 / 1024.0 / 1024.0);
        println!("    测试后物理内存: {:.2} MB", after.physical_mem as f64 / 1024.0 / 1024.0);
        println!("    物理内存变化: {:+.2} MB", physical_delta as f64 / 1024.0 / 1024.0);
        println!("    测试前虚拟内存: {:.2} MB", before.virtual_mem as f64 / 1024.0 / 1024.0);
        println!("    测试后虚拟内存: {:.2} MB", after.virtual_mem as f64 / 1024.0 / 1024.0);
        println!("    虚拟内存变化: {:+.2} MB", virtual_delta as f64 / 1024.0 / 1024.0);
        
        // 每次更新内存成本
        if physical_delta > 0 {
            let bytes_per_update = physical_delta as f64 / actual_updates as f64;
            println!("    每次更新内存成本: {:.4} bytes", bytes_per_update);
        }
    } else if let Some(stats) = memory_stats() {
        println!("    当前物理内存: {:.2} MB", stats.physical_mem as f64 / 1024.0 / 1024.0);
        println!("    当前虚拟内存: {:.2} MB", stats.virtual_mem as f64 / 1024.0 / 1024.0);
    }
    
    // 精度指标
    let expected_updates = file_count as u64 * (updates_per_file as u64 + 1); // 0 到 100000 是 100001 次
    let accuracy = actual_updates as f64 / expected_updates as f64 * 100.0;
    
    println!("\n  🎯 精度指标:");
    println!("    期望更新次数: {}", expected_updates);
    println!("    实际更新次数: {}", actual_updates);
    println!("    完成率: {:.6}%", accuracy);
    println!("    错误率: {:.6}%", (actual_errors as f64 / expected_updates as f64) * 100.0);
    
    // 压力测试指标
    println!("\n  💪 压力测试指标:");
    println!("    单文件更新密度: {} 次/文件", updates_per_file + 1);
    println!("    平均并发更新速率: {:.0} 次/秒/文件", updates_per_sec / file_count as f64);
    println!("    tokio 任务数: {}", file_count);
    
    println!("\n{}", "=".repeat(60));

    // 6. 清理测试文件
    println!("\n[步骤6] 清理测试文件...");
    let cleanup_start = Instant::now();
    fs::remove_dir_all(&test_dir)
        .map_err(|e| format!("清理测试目录失败: {}", e))?;
    let cleanup_duration = cleanup_start.elapsed();
    println!("  ✓ 清理完成，耗时: {:.2?}", cleanup_duration);

    // 7. 性能断言
    println!("\n[步骤7] 性能断言...");
    
    // 断言完成率
    assert!(
        accuracy > ASSERT_ACCURACY_THRESHOLD,
        "完成率过低: {:.2}%，期望 > {}%",
        accuracy, ASSERT_ACCURACY_THRESHOLD
    );
    println!("  ✓ 完成率断言通过: {:.4}%", accuracy);
    
    // 断言吐吐量
    assert!(
        updates_per_sec > ASSERT_THROUGHPUT_THRESHOLD,
        "吐吐量过低: {:.0} 次/秒，期望 > {:.0} 次/秒",
        updates_per_sec, ASSERT_THROUGHPUT_THRESHOLD
    );
    println!("  ✓ 吐吐量断言通过: {:.0} 次/秒", updates_per_sec);
    
    // 断言单次更新耗时
    assert!(
        avg_update_us < ASSERT_UPDATE_TIME_THRESHOLD_US,
        "单次更新耗时过高: {:.2} μs，期望 < {} μs",
        avg_update_us, ASSERT_UPDATE_TIME_THRESHOLD_US
    );
    println!("  ✓ 单次更新耗时断言通过: {:.4} μs", avg_update_us);
    
    // 断言错误率
    assert_eq!(
        actual_errors, 0,
        "存在更新错误: {} 次",
        actual_errors
    );
    println!("  ✓ 错误率断言通过: 0 错误");
    
    println!("\n  ✓ 所有断言通过！");

    println!("\n🎉 极限性能测试完成！响应式系统表现优异！");
    Ok(())
}
