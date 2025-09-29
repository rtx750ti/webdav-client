use crate::reactive::reactive::ReactiveProperty;

#[derive(Debug, Clone)]
pub struct ReactiveFileProperty {
    pub name: ReactiveProperty<String>,
    pub download_bytes: ReactiveProperty<usize>,
    pub file_lock: ReactiveProperty<bool>, // 文件锁，主要用于限制下载时用户尝试修改文件名的操作
}

impl ReactiveFileProperty {
    pub fn new(name: String) -> Self {
        Self {
            name: ReactiveProperty::new(name),
            download_bytes: ReactiveProperty::new(0),
            file_lock: ReactiveProperty::new(false),
        }
    }

    /// 获取名称响应式属性
    pub fn get_reactive_name(&self) -> &ReactiveProperty<String> {
        &self.name
    }

    pub fn get_download_bytes(&self) -> &ReactiveProperty<usize> {
        &self.download_bytes
    }

    pub fn get_file_lock(&self) -> &ReactiveProperty<bool> {
        &self.file_lock
    }
}
