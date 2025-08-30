pub mod enums;
pub mod structs;
pub mod traits;
mod traits_impl;

pub struct Downloader {}

impl Downloader {
    pub fn new() -> Self {
        Self {}
    }
}
