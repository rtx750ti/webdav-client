use crate::reactive::ReactiveProperty;

#[derive(Debug, Clone)]
pub struct ReactiveFileProperty {
    pub name: ReactiveProperty<String>,
}

impl ReactiveFileProperty {
    pub fn new(name: String) -> Self {
        Self { name: ReactiveProperty::new(name) }
    }

    /// 获取名称响应式属性
    pub fn get_reactive_name(&self) -> &ReactiveProperty<String> {
        &self.name
    }
}
