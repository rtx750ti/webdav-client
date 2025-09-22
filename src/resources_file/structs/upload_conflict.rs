// 移除了LocalFile的导入，因为我们不再需要批量上传功能

/// 上传冲突类型
#[derive(Debug, Clone, PartialEq)]
pub enum UploadConflict {
    /// 文件已存在
    AlreadyExists,
    /// 版本冲突（如果服务器支持版本控制）
    VersionMismatch,
    /// 权限不足
    PermissionDenied,
}

/// 冲突解决策略
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    /// 覆盖现有文件
    Overwrite,
    /// 重命名为指定名称
    Rename(String),
    /// 跳过此文件
    Skip,
    /// 中止整个上传过程
    Abort,
}

/// 冲突信息（不包含LocalFile，避免Clone问题）
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub conflict_type: UploadConflict,
    pub target_path: String,
    pub existing_file_info: Option<ExistingFileInfo>,
}

/// 上传结果
#[derive(Debug)]
pub enum UploadResult {
    /// 上传成功
    Success {
        target_path: String,
        file_size: u64,
        upload_time: std::time::Duration,
    },
    /// 发生冲突，需要用户决策
    Conflict {
        conflict_info: ConflictInfo,
    },
    /// 上传失败
    Error {
        target_path: String,
        error_message: String,
    },
}

/// 现有文件信息
#[derive(Debug, Clone)]
pub struct ExistingFileInfo {
    /// 文件大小
    pub size: u64,
    /// 最后修改时间
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    /// ETag（如果服务器支持）
    pub etag: Option<String>,
}

// 移除了ConflictResolutionResult，让用户自己管理冲突解决

impl UploadResult {
    /// 检查是否为冲突结果
    pub fn is_conflict(&self) -> bool {
        matches!(self, UploadResult::Conflict { .. })
    }

    /// 检查是否为成功结果
    pub fn is_success(&self) -> bool {
        matches!(self, UploadResult::Success { .. })
    }

    /// 检查是否为错误结果
    pub fn is_error(&self) -> bool {
        matches!(self, UploadResult::Error { .. })
    }

    /// 获取目标路径
    pub fn target_path(&self) -> &str {
        match self {
            UploadResult::Success { target_path, .. } => target_path,
            UploadResult::Conflict { conflict_info } => &conflict_info.target_path,
            UploadResult::Error { target_path, .. } => target_path,
        }
    }
}

impl ConflictResolution {
    /// 创建重命名策略，自动添加时间戳
    pub fn rename_with_timestamp(original_path: &str) -> Self {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let new_name = if let Some(dot_pos) = original_path.rfind('.') {
            let (name, ext) = original_path.split_at(dot_pos);
            format!("{}_{}{}", name, timestamp, ext)
        } else {
            format!("{}_{}", original_path, timestamp)
        };
        ConflictResolution::Rename(new_name)
    }
    
    /// 创建重命名策略，添加序号
    pub fn rename_with_number(original_path: &str, number: u32) -> Self {
        let new_name = if let Some(dot_pos) = original_path.rfind('.') {
            let (name, ext) = original_path.split_at(dot_pos);
            format!("{}_({}){}", name, number, ext)
        } else {
            format!("{}_({})", original_path, number)
        };
        ConflictResolution::Rename(new_name)
    }
}

// 批量上传功能已移除
// 用户可以使用标准的Rust并发工具来实现批量上传：
//
// 示例1: 使用 futures::future::join_all
// let results: Vec<UploadResult> = futures::future::join_all(
//     files.into_iter().map(|file| file.upload())
// ).await;
//
// 示例2: 使用 tokio::spawn 控制并发数
// let semaphore = Arc::new(Semaphore::new(5)); // 最多5个并发
// let tasks: Vec<_> = files.into_iter().map(|file| {
//     let permit = semaphore.clone();
//     tokio::spawn(async move {
//         let _guard = permit.acquire().await.unwrap();
//         file.upload().await
//     })
// }).collect();
// let results: Vec<UploadResult> = futures::future::join_all(tasks).await
//     .into_iter().map(|r| r.unwrap()).collect();
