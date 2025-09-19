use crate::reactive::ReactiveProperty;

#[derive(Debug, Clone)]
pub struct ReactiveFileProperty {
    pub name: ReactiveProperty<String>,
    pub download_bytes: ReactiveProperty<usize>,
}

impl ReactiveFileProperty {
    pub fn new(name: String) -> Self {
        Self {
            name: ReactiveProperty::new(name),
            download_bytes: ReactiveProperty::new(0),
        }
    }

    /// 获取名称响应式属性
    pub fn get_reactive_name(&self) -> &ReactiveProperty<String> {
        &self.name
    }

    pub fn get_download_bytes(&self) -> &ReactiveProperty<usize> {
        &self.download_bytes
    }
}
