pub trait UpdateLocalFile {
    fn update_name(&self, new_name: &str) -> Result<(), ()>;
}
