pub mod enums;
pub mod structs;
pub mod traits;
mod traits_impl;

pub struct FileExplorer {}

impl FileExplorer {
    pub fn new() -> Self {
        Self {}
    }
}
