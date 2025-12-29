use crate::_reactive::reactive::ReactiveProperty;

#[derive(Debug)]
pub struct LocalFileProperty {
    pub file_lock: ReactiveProperty<bool>,
    pub upload_bytes: ReactiveProperty<usize>,
}

impl LocalFileProperty {
    pub fn new() -> Self {
        Self {
            upload_bytes: ReactiveProperty::new(0),
            file_lock: ReactiveProperty::new(false),
        }
    }
}
