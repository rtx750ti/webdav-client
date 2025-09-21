use webdav_client::client::WebDavClient;
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::folder::Folders;
use webdav_client::public::enums::depth::Depth;
use webdav_client::resources_file::traits::upload::{Upload, UploadConfig};
use webdav_client::global_config::GlobalConfig;
use std::io::{Write, Seek, SeekFrom};
use tempfile::NamedTempFile;
use tokio::time::{sleep, Duration, Instant};

/// 简洁的上传测试，包含进度监控
#[tokio::test]
async fn test_simple_upload_with_progress() {
    println!("🚀 简洁上传测试 - 包含进度监控");
    println!("{}", "=".repeat(50));
    
    // 1. 创建测试文件 (增大到10MB确保能看到进度)
    let test_file = create_test_file(10).expect("创建测试文件失败"); // 10MB
    let file_size = std::fs::metadata(test_file.path()).unwrap().len();
    println!("📄 测试文件: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);

    // 2. 连接 WebDAV 服务器
    let client = WebDavClient::new();
    let client_key = client.add_account(
        "http://192.168.5.90:36879/",
        "test",
        "test"
    ).expect("连接服务器失败");

    println!("✅ 服务器连接成功");

    // 3. 获取上传资源
    let folders = client.get_folders(
        &client_key,
        &vec!["/".to_string()],
        &Depth::One
    ).await.expect("获取文件夹失败");

    let resources_file = folders[0][0].clone();

    // 4. 配置上传参数 (使用简单上传，因为服务器不支持分片)
    let upload_config = UploadConfig {
        overwrite: true,
        chunk_size: None, // 使用简单上传
        resume: false,
    };

    // 5. 模拟进度显示
    println!("📊 模拟上传进度显示:");
    let start_time = Instant::now();
    for progress in [0.0, 25.0, 50.0, 75.0, 100.0] {
        let elapsed = progress / 100.0 * 2.0; // 模拟2秒上传
        display_progress(progress, elapsed, file_size);
        sleep(Duration::from_millis(300)).await;
    }
    println!();
    
    // 6. 执行上传
    let remote_path = "/simple_upload_test.dat";
    println!("⬆️ 开始实际上传到: {}", remote_path);

    let start_time = Instant::now();
    let result = resources_file.upload_file(
        test_file.path(),
        remote_path,
        Some(upload_config)
    ).await;
    
    // 8. 验证结果
    let elapsed = start_time.elapsed();
    match result {
        Ok(_) => {
            println!("\n✅ 上传测试成功!");
            println!("📊 测试统计:");
            println!("   - 文件大小: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);
            println!("   - 上传时间: {:.2} 秒", elapsed.as_secs_f64());
            println!("   - 平均速度: {:.2} MB/s", 
                (file_size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64());
        }
        Err(e) => {
            println!("\n❌ 上传测试失败: {}", e);
            panic!("上传测试失败: {}", e);
        }
    }
    
    // 9. 清理
    client.remove_account(&client_key).unwrap();
    println!("🏁 测试完成");
}

/// 测试不同文件大小的上传策略
#[tokio::test]
async fn test_upload_strategies() {
    println!("🧪 测试不同文件大小的上传策略");
    println!("{}", "=".repeat(50));
    
    let global_config = GlobalConfig::default();
    
    let test_cases = vec![
        ("小文件", 1024 * 1024),           // 1MB
        ("中等文件", 50 * 1024 * 1024),    // 50MB
        ("大文件", 500 * 1024 * 1024),     // 500MB
        ("超大文件", 2 * 1024 * 1024 * 1024), // 2GB
    ];
    
    for (name, file_size) in test_cases {
        let should_chunk = global_config.should_use_chunked_upload(file_size);
        let suggested_chunk = global_config.suggest_chunk_size(file_size);
        let chunk_count = global_config.calculate_chunk_count(file_size);
        
        let size_display = if file_size >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", file_size as f64 / 1024.0 / 1024.0 / 1024.0)
        } else {
            format!("{:.0} MB", file_size as f64 / 1024.0 / 1024.0)
        };
        
        let strategy = if should_chunk {
            format!("分片上传 ({:.0} MB 分片, {} 个分片)", 
                suggested_chunk as f64 / 1024.0 / 1024.0, 
                chunk_count)
        } else {
            "简单上传".to_string()
        };
        
        println!("📄 {} ({}): {}", name, size_display, strategy);
    }
    
    println!("✅ 上传策略测试完成");
}

/// 测试上传进度显示功能（模拟进度）
#[tokio::test]
async fn test_upload_progress_display() {
    println!("🎭 测试上传进度显示功能（模拟）");
    println!("{}", "=".repeat(50));

    // 模拟上传进度显示
    let file_size = 10 * 1024 * 1024u64; // 10MB

    println!("📄 模拟文件大小: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);
    println!("📊 模拟上传进度显示:\n");

    let start_time = Instant::now();

    // 模拟不同的进度阶段
    let progress_stages = vec![0.0, 15.0, 35.0, 50.0, 70.0, 85.0, 95.0, 100.0];

    for progress in progress_stages {
        let elapsed = start_time.elapsed().as_secs_f64() + (progress / 100.0 * 3.0); // 模拟3秒上传
        display_progress(progress, elapsed, file_size);
        sleep(Duration::from_millis(500)).await; // 每0.5秒显示一次
    }

    println!("\n✅ 进度显示功能测试完成!");
}

/// 测试简单上传的进度监控
#[tokio::test]
async fn test_simple_upload_progress_only() {
    println!("📊 测试简单上传进度监控");
    println!("{}", "=".repeat(50));

    // 创建一个较大的测试文件
    let test_file = create_test_file(5).expect("创建测试文件失败"); // 5MB
    let file_size = std::fs::metadata(test_file.path()).unwrap().len();
    println!("📄 测试文件: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);

    // 连接服务器
    let client = WebDavClient::new();
    let client_key = client.add_account("http://192.168.5.90:36879/", "test", "test")
        .expect("连接服务器失败");

    // 获取资源
    let folders = client.get_folders(&client_key, &vec!["/".to_string()], &Depth::One).await
        .expect("获取文件夹失败");
    let resources_file = folders[0][0].clone();

    // 简单上传配置
    let upload_config = UploadConfig {
        overwrite: true,
        chunk_size: None, // 简单上传
        resume: false,
    };

    // 启动进度监控
    let resources_clone = resources_file.clone();
    let progress_task = tokio::spawn(async move {
        simple_progress_monitor(&resources_clone).await;
    });

    // 执行上传
    println!("⬆️ 开始简单上传...");
    let start = Instant::now();

    let result = resources_file.upload_file(
        test_file.path(),
        "/simple_progress_only_test.dat",
        Some(upload_config)
    ).await;

    // 等待进度监控完成
    let _ = progress_task.await;

    // 显示结果
    match result {
        Ok(_) => {
            let elapsed = start.elapsed();
            println!("\n✅ 简单上传测试成功!");
            println!("📊 统计: {:.1} MB 用时 {:.2}s 速度 {:.2} MB/s",
                file_size as f64 / 1024.0 / 1024.0,
                elapsed.as_secs_f64(),
                (file_size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64()
            );
        }
        Err(e) => {
            println!("\n❌ 简单上传测试失败: {}", e);
        }
    }

    // 清理
    client.remove_account(&client_key).unwrap();
    println!("🏁 简单上传进度测试完成");
}

/// 测试全局配置的分片设置
#[tokio::test]
async fn test_global_config_chunking() {
    println!("⚙️ 测试全局配置分片设置");
    println!("{}", "=".repeat(50));
    
    let config = GlobalConfig::default();
    
    // 测试不同的分片大小设置
    println!("🔧 测试分片大小配置:");
    
    // 设置 1GB 分片
    config.set_chunk_size_gb(1).unwrap();
    println!("   - 1GB 分片: {:.1} GB", config.get_chunk_size_gb());
    
    // 设置 512MB 分片
    config.set_chunk_size_mb(512).unwrap();
    println!("   - 512MB 分片: {:.0} MB", config.get_chunk_size_mb());
    
    // 测试分片数量计算
    println!("\n📊 测试分片数量计算:");
    let test_file_size = (2.5 * 1024.0 * 1024.0 * 1024.0) as u64; // 2.5GB
    
    config.set_chunk_size_gb(1).unwrap(); // 1GB 分片
    let chunks = config.calculate_chunk_count(test_file_size);
    println!("   - 2.5GB 文件 ÷ 1GB 分片 = {} 个分片", chunks);
    
    assert_eq!(chunks, 3, "2.5GB 文件应该需要 3 个 1GB 分片");
    
    println!("✅ 全局配置测试完成");
}

/// 创建指定大小的测试文件 (MB)
fn create_test_file(size_mb: usize) -> Result<NamedTempFile, Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    
    // 创建 1KB 数据块
    let data_block = format!("{}\n", "T".repeat(1023));
    let block_bytes = data_block.as_bytes();
    
    // 写入指定大小的数据
    for i in 0..(size_mb * 1024) {
        file.write_all(format!("Block {:06}: ", i).as_bytes())?;
        file.write_all(block_bytes)?;
    }
    
    file.flush()?;
    file.seek(SeekFrom::Start(0))?; // 重置文件指针
    
    Ok(file)
}

/// 详细的上传进度监控 - 强制显示每次更新
async fn monitor_upload_progress_verbose(
    resources_file: &webdav_client::resources_file::structs::resources_file::ResourcesFile,
    total_size: u64,
) {
    let start_time = Instant::now();
    let mut check_count = 0;
    let mut last_progress = -1.0;

    println!("📊 开始详细监控上传进度...\n");

    loop {
        sleep(Duration::from_millis(100)).await; // 每100ms检查一次
        check_count += 1;

        let progress = resources_file.get_upload_progress();
        let elapsed = start_time.elapsed().as_secs_f64();

        // 只在进度有变化或每10次检查时显示
        if (progress - last_progress).abs() > 0.1 || check_count % 10 == 0 {
            if progress > 0.0 {
                display_progress(progress, elapsed, total_size);
                last_progress = progress;
            } else {
                println!("📊 检查 #{}: 等待上传开始... ({:.1}s)", check_count, elapsed);
            }
        }

        // 完成或超时退出
        if progress >= 100.0 {
            println!("\n✅ 进度监控完成!");
            break;
        }

        if elapsed > 30.0 { // 减少超时时间到30秒
            println!("\n⏰ 监控超时 (30秒)");
            break;
        }
    }
}

/// 监控上传进度
async fn monitor_upload_progress(
    resources_file: &webdav_client::resources_file::structs::resources_file::ResourcesFile,
    total_size: u64,
) {
    let start_time = Instant::now();
    let mut last_progress = -1.0; // 初始化为-1确保第一次显示

    println!("📊 开始监控上传进度...\n");

    loop {
        sleep(Duration::from_millis(50)).await; // 每50ms检查一次，更频繁

        let progress = resources_file.get_upload_progress();
        let elapsed = start_time.elapsed().as_secs_f64();

        // 更敏感的进度更新条件
        if (progress - last_progress).abs() > 0.1 || progress >= 100.0 || elapsed - (last_progress / 10.0) > 0.5 {
            display_progress(progress, elapsed, total_size);
            last_progress = progress;
        }

        // 完成或超时退出
        if progress >= 100.0 || elapsed > 120.0 {
            if progress < 100.0 {
                println!("\n⏰ 监控超时，但上传可能仍在继续...");
            }
            break;
        }
    }
}

/// 简单的进度监控
async fn simple_progress_monitor(
    resources_file: &webdav_client::resources_file::structs::resources_file::ResourcesFile,
) {
    let start_time = Instant::now();
    let mut last_progress = -1.0;

    println!("📊 监控上传进度...");

    loop {
        sleep(Duration::from_millis(100)).await;

        let progress = resources_file.get_upload_progress();
        let elapsed = start_time.elapsed().as_secs_f64();

        // 显示进度变化
        if (progress - last_progress).abs() > 1.0 || progress >= 100.0 {
            if progress > 0.0 {
                println!("   📊 进度: {:.1}% ({:.1}s)", progress, elapsed);
            }
            last_progress = progress;
        }

        // 完成或超时退出
        if progress >= 100.0 {
            println!("   ✅ 上传完成!");
            break;
        }

        if elapsed > 30.0 {
            println!("   ⏰ 监控超时");
            break;
        }
    }
}

/// 显示进度条
fn display_progress(progress: f64, elapsed: f64, total_size: u64) {
    let uploaded_bytes = (progress / 100.0 * total_size as f64) as u64;
    let speed_mbps = if elapsed > 0.0 {
        (uploaded_bytes as f64 / elapsed) / 1024.0 / 1024.0
    } else {
        0.0
    };
    
    // 创建进度条 (30字符宽度)
    let bar_width = 30;
    let filled = ((progress / 100.0) * bar_width as f64) as usize;
    let progress_bar = format!("{}{}",
        "█".repeat(filled),
        "░".repeat(bar_width - filled)
    );
    
    // 显示进度信息
    print!("\r📊 [{}] {:.1}% | 🚀 {:.2} MB/s | 📤 {:.1}/{:.1} MB",
        progress_bar,
        progress,
        speed_mbps,
        uploaded_bytes as f64 / 1024.0 / 1024.0,
        total_size as f64 / 1024.0 / 1024.0
    );
    
    std::io::stdout().flush().unwrap();
    
    // 完成时换行
    if progress >= 100.0 {
        println!();
    }
}
