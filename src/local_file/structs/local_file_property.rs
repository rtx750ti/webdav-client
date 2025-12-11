use crate::reactive::reactive::ReactiveProperty;

#[derive(Debug, Clone)]
pub struct LocalFileProperty {
    upload_bytes: ReactiveProperty<usize>,
    file_lock: ReactiveProperty<bool>,
}

impl LocalFileProperty {
    pub fn new() -> Self {
        Self {
            upload_bytes: ReactiveProperty::new(0),
            file_lock: ReactiveProperty::new(false),
        }
    }

    /// 获取上传进度（响应式属性）
    pub fn get_upload_bytes(&self) -> &ReactiveProperty<usize> {
        &self.upload_bytes
    }

    /// 获取文件锁（响应式属性）
    pub fn get_file_lock(&self) -> &ReactiveProperty<bool> {
        &self.file_lock
    }
}
