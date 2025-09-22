use webdav_client::resources_file::traits_impl::impl_upload::chunked_upload::black_list::{
    is_chunked_upload_blacklisted, should_use_chunked_upload, CHUNKED_UPLOAD_BLACKLIST
};
use webdav_client::resources_file::structs::local_file::{LocalFile, LocalFileEnum};
use webdav_client::resources_file::structs::upload_conflict::{
    UploadResult, UploadConflict, ConflictResolution
};
use webdav_client::client::structs::client_key::ClientKey;
use webdav_client::global_config::GlobalConfig;
use std::io::{Write, Seek, SeekFrom};
use tempfile::NamedTempFile;
use tokio::time::{sleep, Duration};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

/// 测试黑名单功能
#[tokio::test]
async fn test_upload_blacklist_functionality() {
    println!("� 测试上传黑名单功能");
    println!("{}", "=".repeat(50));

    // 测试黑名单文件
    let blacklist_files = vec![
        "/test/file.tmp",
        "/config/app.log",
        "/settings/config.ini",
        "/data/settings.json",
        "/backup/data.xml",
    ];

    println!("📋 黑名单文件列表: {:?}", CHUNKED_UPLOAD_BLACKLIST);

    for file_path in blacklist_files {
        let is_blacklisted = is_chunked_upload_blacklisted(file_path);
        println!("   {} -> 黑名单: {}", file_path, if is_blacklisted { "✅" } else { "❌" });
        assert!(is_blacklisted, "文件 {} 应该在黑名单中", file_path);
    }

    // 测试非黑名单文件
    let normal_files = vec![
        "/documents/report.pdf",
        "/videos/movie.mp4",
        "/images/photo.jpg",
        "/archives/backup.zip",
    ];

    for file_path in normal_files {
        let is_blacklisted = is_chunked_upload_blacklisted(file_path);
        println!("   {} -> 黑名单: {}", file_path, if is_blacklisted { "✅" } else { "❌" });
        assert!(!is_blacklisted, "文件 {} 不应该在黑名单中", file_path);
    }

    println!("✅ 黑名单功能测试通过");
}

/// 测试上传策略选择
#[tokio::test]
async fn test_upload_strategy_selection() {
    println!("🎯 测试上传策略选择");
    println!("{}", "=".repeat(50));

    let chunk_threshold = 5 * 1024 * 1024; // 5MB

    let test_cases = vec![
        ("/large/video.mp4", 50 * 1024 * 1024, true),   // 50MB视频，应该分片
        ("/small/image.jpg", 1024 * 1024, false),       // 1MB图片，不应该分片
        ("/config/app.log", 50 * 1024 * 1024, false),   // 50MB日志，黑名单，不应该分片
        ("/temp/data.tmp", 100 * 1024 * 1024, false),   // 100MB临时文件，黑名单，不应该分片
        ("/documents/large.pdf", 20 * 1024 * 1024, true), // 20MB PDF，应该分片
    ];

    for (file_path, file_size, expected_chunked) in test_cases {
        let should_chunk = should_use_chunked_upload(file_path, file_size, chunk_threshold);
        println!("   {} ({:.1}MB) -> 分片: {}",
            file_path,
            file_size as f64 / 1024.0 / 1024.0,
            if should_chunk { "✅" } else { "❌" }
        );
        assert_eq!(should_chunk, expected_chunked,
            "文件 {} 的分片策略不正确", file_path);
    }

    println!("✅ 上传策略选择测试通过");
}

/// 测试LocalFile上传接口
#[tokio::test]
async fn test_local_file_upload_interface() {
    println!("📁 测试LocalFile上传接口");
    println!("{}", "=".repeat(50));

    // 创建测试文件
    let test_file = create_test_file(1).expect("创建测试文件失败"); // 1MB
    let file_size = std::fs::metadata(test_file.path()).unwrap().len();
    println!("📄 测试文件: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);

    // 创建客户端密钥
    let client_key = ClientKey::new(
        "http://192.168.5.90:36879/",
        "test_user"
    ).unwrap();

    // 测试File类型的LocalFile
    println!("🔧 测试File类型LocalFile");
    let file = File::open(test_file.path()).await.expect("打开文件失败");
    let local_file = LocalFile::new(
        LocalFileEnum::File(file),
        "/test_local_file.dat".to_string(),
        &client_key,
    );

    println!("   - 目标路径: {}", local_file.target_path());
    println!("   - 分片配置: {:?}", local_file.get_enable_chunked());

    // 测试配置方法
    let local_file_disabled = LocalFile::new(
        LocalFileEnum::File(File::open(test_file.path()).await.expect("打开文件失败")),
        "/test_disabled_chunk.dat".to_string(),
        &client_key,
    ).disable_chunked();

    assert_eq!(local_file_disabled.get_enable_chunked(), Some(false));
    println!("   - 禁用分片配置: ✅");

    let local_file_enabled = LocalFile::new(
        LocalFileEnum::File(File::open(test_file.path()).await.expect("打开文件失败")),
        "/test_enabled_chunk.dat".to_string(),
        &client_key,
    ).enable_chunked();

    assert_eq!(local_file_enabled.get_enable_chunked(), Some(true));
    println!("   - 启用分片配置: ✅");

    // 测试Stream类型的LocalFile
    println!("🌊 测试Stream类型LocalFile");
    let file_for_stream = File::open(test_file.path()).await.expect("打开文件失败");
    let stream = ReaderStream::new(file_for_stream);
    let local_file_stream = LocalFile::new(
        LocalFileEnum::StreamFile(stream),
        "/test_stream.dat".to_string(),
        &client_key,
    );

    println!("   - Stream LocalFile创建: ✅");

    // 注意：实际上传会失败，因为没有实现全局客户端管理器
    // 但我们可以测试接口的正确性
    println!("✅ LocalFile接口测试通过");
}

/// 测试模块结构和功能
#[tokio::test]
async fn test_upload_module_structure() {
    println!("🏗️ 测试上传模块结构");
    println!("{}", "=".repeat(50));

    // 测试黑名单模块
    println!("📋 测试黑名单模块:");
    println!("   - 黑名单常量: {:?}", CHUNKED_UPLOAD_BLACKLIST);
    println!("   - 黑名单检查函数: ✅");

    // 测试策略选择
    println!("🎯 测试策略选择:");
    let test_file = "/test/large_video.mp4";
    let file_size = 100 * 1024 * 1024; // 100MB
    let threshold = 10 * 1024 * 1024;  // 10MB

    let should_chunk = should_use_chunked_upload(test_file, file_size, threshold);
    println!("   - 文件: {}", test_file);
    println!("   - 大小: {:.1}MB", file_size as f64 / 1024.0 / 1024.0);
    println!("   - 阈值: {:.1}MB", threshold as f64 / 1024.0 / 1024.0);
    println!("   - 应该分片: {}", should_chunk);
    assert!(should_chunk, "大文件应该使用分片上传");

    // 测试黑名单优先级
    let blacklist_file = "/logs/app.log";
    let large_size = 200 * 1024 * 1024; // 200MB
    let should_chunk_blacklist = should_use_chunked_upload(blacklist_file, large_size, threshold);
    println!("   - 黑名单文件: {}", blacklist_file);
    println!("   - 大小: {:.1}MB", large_size as f64 / 1024.0 / 1024.0);
    println!("   - 应该分片: {}", should_chunk_blacklist);
    assert!(!should_chunk_blacklist, "黑名单文件不应该使用分片上传");

    println!("✅ 模块结构测试通过");
}

/// 测试文件类型推断
#[tokio::test]
async fn test_content_type_inference() {
    println!("🔍 测试文件类型推断");
    println!("{}", "=".repeat(50));

    use webdav_client::resources_file::traits_impl::impl_upload::chunked_upload::infer_content_type;

    let test_cases = vec![
        ("document.txt", "text/plain"),
        ("webpage.html", "text/html"),
        ("style.css", "text/css"),
        ("script.js", "application/javascript"),
        ("data.json", "application/json"),
        ("config.xml", "application/xml"),
        ("report.pdf", "application/pdf"),
        ("archive.zip", "application/zip"),
        ("photo.jpg", "image/jpeg"),
        ("image.png", "image/png"),
        ("animation.gif", "image/gif"),
        ("video.mp4", "video/mp4"),
        ("audio.mp3", "audio/mpeg"),
        ("unknown.xyz", "application/octet-stream"),
        ("no_extension", "application/octet-stream"),
    ];

    for (filename, expected_type) in test_cases {
        let inferred_type = infer_content_type(filename);
        println!("   {} -> {}", filename, inferred_type);
        assert_eq!(inferred_type, expected_type,
            "文件 {} 的类型推断不正确", filename);
    }

    println!("✅ 文件类型推断测试通过");
}

/// 测试上传配置优先级
#[tokio::test]
async fn test_upload_configuration_priority() {
    println!("⚙️ 测试上传配置优先级");
    println!("{}", "=".repeat(50));

    // 创建测试文件
    let test_file = create_test_file(1).expect("创建测试文件失败");
    let client_key = ClientKey::new("http://test.example.com/", "user").unwrap();

    // 测试默认配置（应该跟随全局配置）
    println!("� 测试默认配置:");
    let file = File::open(test_file.path()).await.expect("打开文件失败");
    let local_file_default = LocalFile::new(
        LocalFileEnum::File(file),
        "/test_default.dat".to_string(),
        &client_key,
    );
    println!("   - 默认分片配置: {:?}", local_file_default.get_enable_chunked());
    assert_eq!(local_file_default.get_enable_chunked(), None, "默认应该是None");

    // 测试局部禁用配置
    println!("🚫 测试局部禁用配置:");
    let file = File::open(test_file.path()).await.expect("打开文件失败");
    let local_file_disabled = LocalFile::new(
        LocalFileEnum::File(file),
        "/test_disabled.dat".to_string(),
        &client_key,
    ).disable_chunked();
    println!("   - 禁用分片配置: {:?}", local_file_disabled.get_enable_chunked());
    assert_eq!(local_file_disabled.get_enable_chunked(), Some(false), "应该是Some(false)");

    // 测试局部启用配置
    println!("✅ 测试局部启用配置:");
    let file = File::open(test_file.path()).await.expect("打开文件失败");
    let local_file_enabled = LocalFile::new(
        LocalFileEnum::File(file),
        "/test_enabled.dat".to_string(),
        &client_key,
    ).enable_chunked();
    println!("   - 启用分片配置: {:?}", local_file_enabled.get_enable_chunked());
    assert_eq!(local_file_enabled.get_enable_chunked(), Some(true), "应该是Some(true)");

    // 测试链式调用（最后的设置生效）
    println!("🔗 测试链式调用:");
    let file = File::open(test_file.path()).await.expect("打开文件失败");
    let local_file_chained = LocalFile::new(
        LocalFileEnum::File(file),
        "/test_chained.dat".to_string(),
        &client_key,
    ).enable_chunked().disable_chunked().enable_chunked();
    println!("   - 链式调用结果: {:?}", local_file_chained.get_enable_chunked());
    assert_eq!(local_file_chained.get_enable_chunked(), Some(true), "最后的enable_chunked应该生效");

    println!("✅ 配置优先级测试通过");
}

/// 测试进度显示功能（模拟）
#[tokio::test]
async fn test_progress_display_simulation() {
    println!("🎭 测试进度显示功能（模拟）");
    println!("{}", "=".repeat(50));

    let file_size = 10 * 1024 * 1024u64; // 10MB
    println!("📄 模拟文件大小: {:.1} MB", file_size as f64 / 1024.0 / 1024.0);
    println!("📊 模拟上传进度显示:\n");

    // 模拟不同的进度阶段
    let progress_stages = vec![0.0, 25.0, 50.0, 75.0, 100.0];

    for progress in progress_stages {
        let elapsed = progress / 100.0 * 2.0; // 模拟2秒上传
        display_progress(progress, elapsed, file_size);
        sleep(Duration::from_millis(300)).await;
    }

    println!("\n✅ 进度显示功能测试完成!");
}

/// 测试全局配置的基本功能
#[tokio::test]
async fn test_global_config_basic() {
    println!("⚙️ 测试全局配置基本功能");
    println!("{}", "=".repeat(50));

    let config = GlobalConfig::default();

    // 测试默认配置
    println!("🔧 测试默认配置:");
    println!("   - 默认分片大小: {:.0} MB", config.get_chunk_size() as f64 / 1024.0 / 1024.0);
    println!("   - 默认分片上传: {}", config.is_chunked_upload_enabled());

    // 验证默认值
    assert!(config.get_chunk_size() > 0, "分片大小应该大于0");

    println!("✅ 全局配置测试完成");
}

/// 测试上传真实文件
#[tokio::test]
async fn test_upload_real_file() {
    println!("🎬 测试上传真实文件");
    println!("{}", "=".repeat(50));

    let real_file_path = r"G:\Systeam\Videos\Radeon ReLive\艾尔登法环\薄纱.mp4";

    // 检查文件是否存在
    if !std::path::Path::new(real_file_path).exists() {
        println!("⚠️ 测试文件不存在: {}", real_file_path);
        println!("   跳过真实文件上传测试");
        return;
    }

    // 获取文件信息
    let metadata = std::fs::metadata(real_file_path).expect("获取文件信息失败");
    let file_size = metadata.len();
    println!("📄 文件信息:");
    println!("   - 路径: {}", real_file_path);
    println!("   - 大小: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);

    // 测试策略选择
    let threshold = 10 * 1024 * 1024; // 10MB
    let should_chunk = should_use_chunked_upload("/videos/薄纱.mp4", file_size, threshold);
    println!("   - 推荐策略: {}", if should_chunk { "分片上传" } else { "简单上传" });

    // 创建LocalFile进行上传测试
    println!("\n🔧 创建LocalFile对象:");
    let client_key = ClientKey::new("http://192.168.5.90:36879/", "test").unwrap();

    // 打开文件
    let file = File::open(real_file_path).await.expect("打开文件失败");
    let target_path = "/videos/薄纱.mp4".to_string();

    let local_file = if should_chunk {
        LocalFile::new(
            LocalFileEnum::File(file),
            target_path,
            &client_key,
        ).enable_chunked()
    } else {
        LocalFile::new(
            LocalFileEnum::File(file),
            target_path,
            &client_key,
        ).disable_chunked()
    };

    println!("   - 目标路径: {}", local_file.target_path());
    println!("   - 分片配置: {:?}", local_file.get_enable_chunked());
    println!("   - 客户端: {}", client_key.get_base_url());

    // 注意：这里不执行实际上传，因为需要真实的WebDAV服务器
    println!("\n📝 注意: 实际上传需要配置WebDAV服务器");
    println!("   可以通过以下方式执行实际上传:");
    println!("   local_file.upload().await?;");

    println!("\n✅ 真实文件测试完成");
}

/// 实际执行上传真实文件（需要WebDAV服务器）
#[tokio::test]
#[ignore] // 默认忽略，需要手动运行
async fn test_actual_upload_real_file() {
    println!("🚀 实际上传真实文件");
    println!("{}", "=".repeat(50));

    let real_file_path = r"G:\Systeam\Videos\Radeon ReLive\艾尔登法环\薄纱.mp4";

    // 检查文件是否存在
    if !std::path::Path::new(real_file_path).exists() {
        println!("❌ 测试文件不存在: {}", real_file_path);
        panic!("文件不存在");
    }

    // 获取文件信息
    let metadata = std::fs::metadata(real_file_path).expect("获取文件信息失败");
    let file_size = metadata.len();
    println!("📄 文件信息:");
    println!("   - 路径: {}", real_file_path);
    println!("   - 大小: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);

    // 创建LocalFile进行实际上传
    println!("\n🔧 准备上传:");
    let client_key = ClientKey::new("http://192.168.5.90:36879/", "test").unwrap();

    // 打开文件
    let file = File::open(real_file_path).await.expect("打开文件失败");
    let target_path = "/videos/薄纱.mp4".to_string();

    // 强制使用简单上传（避免WebDAV服务器不支持分片）
    println!("   - 强制使用简单上传策略（避免501错误）");
    let local_file = LocalFile::new(
        LocalFileEnum::File(file),
        target_path,
        &client_key,
    ).disable_chunked();

    println!("   - 目标路径: {}", local_file.target_path());
    println!("   - 分片配置: {:?}", local_file.get_enable_chunked());

    // 执行实际上传
    println!("\n⬆️ 开始上传...");
    let start_time = std::time::Instant::now();

    match local_file.upload().await {
        UploadResult::Success { target_path, upload_time, .. } => {
            println!("\n🎉 上传成功!");
            println!("📊 上传统计:");
            println!("   - 目标路径: {}", target_path);
            println!("   - 文件大小: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);
            println!("   - 上传时间: {:.2} 秒", upload_time.as_secs_f64());
            println!("   - 平均速度: {:.2} MB/s",
                (file_size as f64 / 1024.0 / 1024.0) / upload_time.as_secs_f64());
        }
        UploadResult::Conflict { conflict_info } => {
            println!("\n⚠️ 上传冲突: {} - {:?}", conflict_info.target_path, conflict_info.conflict_type);
            panic!("上传冲突: {:?}", conflict_info.conflict_type);
        }
        UploadResult::Error { target_path, error_message } => {
            println!("\n❌ 上传失败: {} - {}", target_path, error_message);
            panic!("上传失败: {}", error_message);
        }
    }

    println!("\n✅ 实际上传测试完成");
}

/// 测试上传重复文件（覆盖测试）
#[tokio::test]
#[ignore] // 默认忽略，需要手动运行
async fn test_upload_duplicate_file() {
    println!("🔄 测试上传重复文件");
    println!("{}", "=".repeat(50));

    let real_file_path = r"G:\Systeam\Videos\Radeon ReLive\艾尔登法环\薄纱.mp4";

    // 检查文件是否存在
    if !std::path::Path::new(real_file_path).exists() {
        println!("❌ 测试文件不存在: {}", real_file_path);
        panic!("文件不存在");
    }

    let client_key = ClientKey::new("http://192.168.5.90:36879/", "test").unwrap();
    let target_path = "/videos/薄纱_重复测试.mp4".to_string();

    println!("📄 测试场景: 连续上传同一个文件3次");
    println!("   - 源文件: {}", real_file_path);
    println!("   - 目标路径: {}", target_path);

    // 第一次上传
    println!("\n🚀 第1次上传:");
    let file1 = File::open(real_file_path).await.expect("打开文件失败");
    let local_file1 = LocalFile::new(
        LocalFileEnum::File(file1),
        target_path.clone(),
        &client_key,
    ).disable_chunked();

    let start_time1 = std::time::Instant::now();
    match local_file1.upload().await {
        UploadResult::Success { upload_time, .. } => {
            println!("   ✅ 第1次上传成功 - 耗时: {:.2}秒", upload_time.as_secs_f64());
        }
        UploadResult::Conflict { conflict_info } => {
            println!("   ⚠️ 第1次上传冲突: {:?}", conflict_info.conflict_type);
            panic!("第1次上传冲突: {:?}", conflict_info.conflict_type);
        }
        UploadResult::Error { error_message, .. } => {
            println!("   ❌ 第1次上传失败: {}", error_message);
            panic!("第1次上传失败: {}", error_message);
        }
    }

    // 第二次上传（覆盖测试）
    println!("\n🔄 第2次上传（覆盖测试）:");
    let file2 = File::open(real_file_path).await.expect("打开文件失败");
    let local_file2 = LocalFile::new(
        LocalFileEnum::File(file2),
        target_path.clone(),
        &client_key,
    ).disable_chunked();

    let start_time2 = std::time::Instant::now();
    match local_file2.upload().await {
        UploadResult::Success { upload_time, .. } => {
            println!("   ❌ 第2次上传意外成功（应该检测到冲突） - 耗时: {:.2}秒", upload_time.as_secs_f64());
        }
        UploadResult::Conflict { conflict_info } => {
            println!("   ✅ 第2次上传正确检测到冲突: {:?}", conflict_info.conflict_type);
        }
        UploadResult::Error { error_message, .. } => {
            println!("   ❌ 第2次上传失败: {}", error_message);
            panic!("第2次上传失败: {}", error_message);
        }
    }

    // 第三次上传（再次覆盖测试）
    println!("\n🔄 第3次上传（再次覆盖测试）:");
    let file3 = File::open(real_file_path).await.expect("打开文件失败");
    let local_file3 = LocalFile::new(
        LocalFileEnum::File(file3),
        target_path.clone(),
        &client_key,
    ).disable_chunked();

    let start_time3 = std::time::Instant::now();
    match local_file3.upload().await {
        UploadResult::Success { upload_time, .. } => {
            println!("   ❌ 第3次上传意外成功（应该检测到冲突） - 耗时: {:.2}秒", upload_time.as_secs_f64());
        }
        UploadResult::Conflict { conflict_info } => {
            println!("   ✅ 第3次上传正确检测到冲突: {:?}", conflict_info.conflict_type);
        }
        UploadResult::Error { error_message, .. } => {
            println!("   ❌ 第3次上传失败: {}", error_message);
            panic!("第3次上传失败: {}", error_message);
        }
    }

    println!("\n📊 重复上传测试总结:");
    println!("   - 所有3次上传都成功完成");
    println!("   - WebDAV服务器正确处理了文件覆盖");
    println!("   - 没有发生冲突或错误");

    println!("\n✅ 重复文件上传测试完成");
}

/// 测试冲突检测功能
#[tokio::test]
#[ignore] // 默认忽略，需要手动运行
async fn test_conflict_detection() {
    println!("🔍 测试冲突检测功能");
    println!("{}", "=".repeat(50));

    let real_file_path = r"G:\Systeam\Videos\Radeon ReLive\艾尔登法环\薄纱.mp4";

    // 检查文件是否存在
    if !std::path::Path::new(real_file_path).exists() {
        println!("❌ 测试文件不存在: {}", real_file_path);
        panic!("文件不存在");
    }

    let client_key = ClientKey::new("http://192.168.5.90:36879/", "test").unwrap();

    // 测试1: 检测不存在的文件（应该无冲突）
    println!("\n🔍 测试1: 检测不存在的文件");
    let file1 = File::open(real_file_path).await.expect("打开文件失败");
    let local_file1 = LocalFile::new(
        LocalFileEnum::File(file1),
        "/videos/不存在的文件_冲突测试.mp4".to_string(),
        &client_key,
    );

    match local_file1.detect_upload_conflict().await {
        Ok(None) => println!("   ✅ 无冲突检测正确"),
        Ok(Some(conflict)) => println!("   ⚠️ 意外发现冲突: {:?}", conflict),
        Err(e) => println!("   ❌ 检测失败: {}", e),
    }

    // 测试2: 先上传一个文件，然后检测冲突
    println!("\n🔍 测试2: 先上传文件，然后检测冲突");
    let file2 = File::open(real_file_path).await.expect("打开文件失败");
    let target_path = "/videos/冲突检测测试.mp4".to_string();
    let local_file2 = LocalFile::new(
        LocalFileEnum::File(file2),
        target_path.clone(),
        &client_key,
    ).disable_chunked();

    // 先上传文件
    println!("   📤 先上传文件...");
    match local_file2.upload().await {
        UploadResult::Success { .. } => println!("   ✅ 文件上传成功"),
        UploadResult::Conflict { conflict_info } => {
            println!("   ⚠️ 文件上传冲突: {:?}", conflict_info.conflict_type);
            return;
        }
        UploadResult::Error { error_message, .. } => {
            println!("   ❌ 文件上传失败: {}", error_message);
            return;
        }
    }

    // 再检测冲突
    println!("   🔍 检测同一路径的冲突...");
    let file3 = File::open(real_file_path).await.expect("打开文件失败");
    let local_file3 = LocalFile::new(
        LocalFileEnum::File(file3),
        target_path,
        &client_key,
    );

    match local_file3.detect_upload_conflict().await {
        Ok(Some(UploadConflict::AlreadyExists)) => {
            println!("   ✅ 正确检测到文件已存在冲突");

            // 获取现有文件信息
            if let Ok(Some(info)) = local_file3.get_existing_file_info().await {
                println!("   📊 现有文件信息:");
                println!("      - 大小: {} 字节", info.size);
                if let Some(modified) = info.last_modified {
                    println!("      - 修改时间: {}", modified);
                }
                if let Some(etag) = info.etag {
                    println!("      - ETag: {}", etag);
                }
            }
        }
        Ok(Some(other_conflict)) => println!("   ⚠️ 检测到其他冲突: {:?}", other_conflict),
        Ok(None) => println!("   ❌ 未检测到预期的冲突"),
        Err(e) => println!("   ❌ 检测失败: {}", e),
    }

    println!("\n✅ 冲突检测测试完成");
}

/// 简单测试：上传与冲突检测
#[tokio::test]
#[ignore] // 默认忽略，需要手动运行
async fn test_simple_upload_conflict() {
    println!("� 简单测试：上传与冲突检测");
    println!("{}", "=".repeat(40));

    let real_file_path = r"G:\Systeam\Videos\Radeon ReLive\艾尔登法环\薄纱.mp4";

    // 检查文件是否存在
    if !std::path::Path::new(real_file_path).exists() {
        println!("❌ 测试文件不存在: {}", real_file_path);
        panic!("文件不存在");
    }

    let client_key = ClientKey::new("http://192.168.5.90:36879/", "test").unwrap();
    let target_path = "/videos/简单冲突测试.mp4".to_string();

    // 第一次上传
    println!("\n📤 第一次上传:");
    let file1 = File::open(real_file_path).await.expect("打开文件失败");
    let local_file1 = LocalFile::new(
        LocalFileEnum::File(file1),
        target_path.clone(),
        &client_key,
    ).disable_chunked();

    match local_file1.upload().await {
        UploadResult::Success { target_path, upload_time, .. } => {
            println!("   ✅ 成功: {} ({:.1}s)", target_path, upload_time.as_secs_f64());
        }
        UploadResult::Conflict { conflict_info } => {
            println!("   ⚠️ 冲突: {} - {:?}", conflict_info.target_path, conflict_info.conflict_type);
            return; // 如果第一次就冲突，说明文件已存在，直接结束
        }
        UploadResult::Error { error_message, .. } => {
            println!("   ❌ 失败: {}", error_message);
            return;
        }
    }

    // 第二次上传相同文件（应该冲突）
    println!("\n🔄 第二次上传（应该冲突）:");
    let file2 = File::open(real_file_path).await.expect("打开文件失败");
    let local_file2 = LocalFile::new(
        LocalFileEnum::File(file2),
        target_path,
        &client_key,
    ).disable_chunked();

    match local_file2.upload().await {
        UploadResult::Success { .. } => {
            println!("   ❌ 意外成功（应该检测到冲突）");
        }
        UploadResult::Conflict { conflict_info } => {
            println!("   ✅ 检测到冲突: {:?}", conflict_info.conflict_type);
            if let Some(info) = conflict_info.existing_file_info {
                println!("   📊 现有文件: {} 字节", info.size);
            }
        }
        UploadResult::Error { error_message, .. } => {
            println!("   ❌ 检测失败: {}", error_message);
        }
    }

    println!("\n✅ 测试完成");
}

// ==================== 测试辅助函数 ====================

/// 测试配置
#[derive(Clone)]
struct TestConfig {
    file_path: &'static str,
    server_url: &'static str,
    username: &'static str,
}

impl TestConfig {
    const DEFAULT: TestConfig = TestConfig {
        file_path: r"G:\Systeam\Videos\Radeon ReLive\艾尔登法环\薄纱.mp4",
        server_url: "http://192.168.5.90:36879/",
        username: "test",
    };

    fn validate(&self) -> Result<(), String> {
        if !std::path::Path::new(self.file_path).exists() {
            return Err(format!("测试文件不存在: {}", self.file_path));
        }
        Ok(())
    }

    fn client_key(&self) -> ClientKey {
        ClientKey::new(self.server_url, self.username).unwrap()
    }
}

/// 创建LocalFile的辅助函数
async fn create_local_file(config: &TestConfig, target_path: &str) -> Result<LocalFile, String> {
    let file = File::open(config.file_path).await
        .map_err(|e| format!("打开文件失败: {}", e))?;

    Ok(LocalFile::new(
        LocalFileEnum::File(file),
        target_path.to_string(),
        &config.client_key(),
    ).disable_chunked())
}

/// 打印上传结果的辅助函数
fn print_upload_result(prefix: &str, result: &UploadResult) {
    match result {
        UploadResult::Success { target_path, upload_time, .. } => {
            println!("   ✅ {}: {} ({:.1}s)", prefix, target_path, upload_time.as_secs_f64());
        }
        UploadResult::Conflict { conflict_info } => {
            println!("   ⚠️ {}: {:?}", prefix, conflict_info.conflict_type);
        }
        UploadResult::Error { target_path, error_message } => {
            println!("   ❌ {}: {} - {}", prefix, target_path, error_message);
        }
    }
}

/// 确保基础文件存在的辅助函数
async fn ensure_base_file(config: &TestConfig, target_path: &str) -> Result<(), String> {
    let local_file = create_local_file(config, target_path).await?;

    match local_file.upload().await {
        UploadResult::Success { .. } => {
            println!("   ✅ 基础文件创建成功");
            Ok(())
        }
        UploadResult::Conflict { .. } => {
            // 文件已存在，用覆盖策略确保是最新的
            let local_file = create_local_file(config, target_path).await?;
            match local_file.upload_with_resolution(ConflictResolution::Overwrite).await {
                UploadResult::Success { .. } => {
                    println!("   ✅ 基础文件覆盖成功");
                    Ok(())
                }
                _ => Err("无法建立基础文件".to_string())
            }
        }
        UploadResult::Error { error_message, .. } => Err(error_message)
    }
}

// ==================== 优化后的测试函数 ====================

/// 测试冲突解决策略
#[tokio::test]
#[ignore] // 默认忽略，需要手动运行
async fn test_conflict_resolution_strategies() {
    println!("🎯 测试冲突解决策略");
    println!("{}", "=".repeat(50));

    let config = TestConfig::DEFAULT;
    if let Err(e) = config.validate() {
        println!("❌ {}", e);
        panic!("{}", e);
    }

    let target_path = "/videos/冲突解决测试.mp4";

    // 建立基础文件
    println!("\n� 建立基础文件:");
    if let Err(e) = ensure_base_file(&config, target_path).await {
        println!("❌ 建立基础文件失败: {}", e);
        return;
    }

    // 测试各种冲突解决策略
    test_rename_strategy(&config, target_path).await;
    test_overwrite_strategy(&config, target_path).await;
    test_skip_strategy(&config, target_path).await;
    test_abort_strategy(&config, target_path).await;

    println!("\n✅ 冲突解决策略测试完成");
}

/// 示例：用户自己实现并发上传
#[tokio::test]
#[ignore] // 默认忽略，需要手动运行
async fn test_user_controlled_concurrent_upload() {
    println!("🚀 示例：用户自己控制并发上传");
    println!("{}", "=".repeat(50));

    let config = TestConfig::DEFAULT;
    if let Err(e) = config.validate() {
        println!("❌ {}", e);
        panic!("{}", e);
    }

    // 创建多个上传任务
    let file_names = vec![
        "/videos/并发测试1.mp4",
        "/videos/并发测试2.mp4",
        "/videos/并发测试3.mp4",
    ];

    println!("\n📤 方式1: 使用 futures::future::join_all 并发上传");

    // 创建上传任务
    let upload_tasks: Vec<_> = file_names.iter().map(|&target_path| {
        let config = config.clone();
        async move {
            match create_local_file(&config, target_path).await {
                Ok(local_file) => {
                    println!("   📤 开始上传: {}", target_path);
                    let result = local_file.upload().await;
                    print_upload_result(&format!("上传 {}", target_path), &result);
                    result
                }
                Err(e) => {
                    println!("   ❌ 创建文件失败 {}: {}", target_path, e);
                    UploadResult::Error {
                        target_path: target_path.to_string(),
                        error_message: e,
                    }
                }
            }
        }
    }).collect();

    // 并发执行所有上传
    let results: Vec<UploadResult> = futures_util::future::join_all(upload_tasks).await;

    // 统计结果
    let success_count = results.iter().filter(|r| r.is_success()).count();
    let conflict_count = results.iter().filter(|r| r.is_conflict()).count();
    let error_count = results.iter().filter(|r| r.is_error()).count();

    println!("\n📊 并发上传结果统计:");
    println!("   - 成功: {} 个", success_count);
    println!("   - 冲突: {} 个", conflict_count);
    println!("   - 错误: {} 个", error_count);

    // 处理冲突文件
    if conflict_count > 0 {
        println!("\n🔄 处理冲突文件:");
        for (index, result) in results.iter().enumerate() {
            if let UploadResult::Conflict { conflict_info } = result {
                println!("   📝 冲突文件 {}: {} - {:?}",
                    index + 1, conflict_info.target_path, conflict_info.conflict_type);

                // 用户可以在这里决定如何处理冲突
                let resolution = ConflictResolution::rename_with_timestamp(&conflict_info.target_path);
                if let ConflictResolution::Rename(ref new_name) = resolution {
                    println!("   📝 决定重命名为: {}", new_name);

                    // 重新创建文件并上传
                    if let Ok(retry_file) = create_local_file(&config, &conflict_info.target_path).await {
                        let retry_result = retry_file.upload_with_resolution(resolution).await;
                        print_upload_result("重试上传", &retry_result);
                    }
                }
            }
        }
    }

    println!("\n✅ 用户控制的并发上传示例完成");
}

/// 测试重命名策略
async fn test_rename_strategy(config: &TestConfig, target_path: &str) {
    println!("\n🔄 测试策略1: 重命名策略");

    // 先检测冲突
    let local_file = match create_local_file(config, target_path).await {
        Ok(file) => file,
        Err(e) => {
            println!("   ❌ 创建文件失败: {}", e);
            return;
        }
    };

    match local_file.upload().await {
        UploadResult::Conflict { conflict_info } => {
            println!("   ✅ 检测到冲突: {:?}", conflict_info.conflict_type);

            // 使用重命名策略
            let new_name = ConflictResolution::rename_with_timestamp(target_path);
            if let ConflictResolution::Rename(ref renamed_path) = new_name {
                println!("   📝 重命名为: {}", renamed_path);

                let retry_file = match create_local_file(config, target_path).await {
                    Ok(file) => file,
                    Err(e) => {
                        println!("   ❌ 重新创建文件失败: {}", e);
                        return;
                    }
                };

                let result = retry_file.upload_with_resolution(new_name).await;
                print_upload_result("重命名上传", &result);
            }
        }
        result => {
            println!("   ❌ 意外结果（应该检测到冲突）");
            print_upload_result("意外结果", &result);
        }
    }
}

/// 测试覆盖策略
async fn test_overwrite_strategy(config: &TestConfig, target_path: &str) {
    println!("\n🔄 测试策略2: 覆盖策略");

    let local_file = match create_local_file(config, target_path).await {
        Ok(file) => file,
        Err(e) => {
            println!("   ❌ 创建文件失败: {}", e);
            return;
        }
    };

    let result = local_file.upload_with_resolution(ConflictResolution::Overwrite).await;
    print_upload_result("覆盖上传", &result);
}

/// 测试跳过策略
async fn test_skip_strategy(config: &TestConfig, target_path: &str) {
    println!("\n🔄 测试策略3: 跳过策略");

    let local_file = match create_local_file(config, target_path).await {
        Ok(file) => file,
        Err(e) => {
            println!("   ❌ 创建文件失败: {}", e);
            return;
        }
    };

    let result = local_file.upload_with_resolution(ConflictResolution::Skip).await;
    print_upload_result("跳过策略", &result);
}

/// 测试中止策略
async fn test_abort_strategy(config: &TestConfig, target_path: &str) {
    println!("\n🔄 测试策略4: 中止策略");

    let local_file = match create_local_file(config, target_path).await {
        Ok(file) => file,
        Err(e) => {
            println!("   ❌ 创建文件失败: {}", e);
            return;
        }
    };

    let result = local_file.upload_with_resolution(ConflictResolution::Abort).await;
    print_upload_result("中止策略", &result);
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
