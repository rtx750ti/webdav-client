use crate::reactive::ReactiveProperty;

#[derive(Debug, Clone)]
pub struct ReactiveFileProperty {
    pub name: ReactiveProperty<String>,
    pub download_bytes: ReactiveProperty<usize>,
    pub upload_bytes: ReactiveProperty<u64>,
    pub upload_total_bytes: ReactiveProperty<u64>,
}

impl ReactiveFileProperty {
    pub fn new(name: String) -> Self {
        Self {
            name: ReactiveProperty::new(name),
            download_bytes: ReactiveProperty::new(0),
            upload_bytes: ReactiveProperty::new(0),
            upload_total_bytes: ReactiveProperty::new(0),
        }
    }

    /// 获取名称响应式属性
    pub fn get_reactive_name(&self) -> &ReactiveProperty<String> {
        &self.name
    }

    pub fn get_download_bytes(&self) -> &ReactiveProperty<usize> {
        &self.download_bytes
    }

    /// 设置上传总字节数
    pub fn set_upload_total_bytes(&self, total: u64) -> Result<(), String> {
        self.upload_total_bytes.update(total).map(|_| ())
    }

    /// 设置已上传字节数
    pub fn set_upload_bytes(&self, bytes: u64) -> Result<(), String> {
        self.upload_bytes.update(bytes).map(|_| ())
    }

    /// 增加已上传字节数
    pub fn add_upload_bytes(&self, bytes: u64) -> Result<(), String> {
        if let Some(current) = self.upload_bytes.get_current() {
            self.upload_bytes.update(current + bytes).map(|_| ())
        } else {
            self.upload_bytes.update(bytes).map(|_| ())
        }
    }

    /// 获取上传进度百分比
    pub fn get_upload_progress(&self) -> f64 {
        if let (Some(uploaded), Some(total)) = (
            self.upload_bytes.get_current(),
            self.upload_total_bytes.get_current(),
        ) {
            if total > 0 {
                (uploaded as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}
