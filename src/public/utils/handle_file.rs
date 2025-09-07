const DEFAULT_MAX_CONCURRENT_CHUNKS: u64 = 4; // 最大并发分片数

pub fn computed_semaphore_count(size: Option<u64>) -> usize {
    if let Some(size) = size {
        if size > 1000 * 1024 * 1024 {
            8
        } else if size > 750 * 1024 * 1024 {
            7
        } else if size > 500 * 1024 * 1024 {
            6
        } else if size > 250 * 1024 * 1024 {
            5
        } else if size > 100 * 1024 * 1024 {
            4
        } else if size > 50 * 1024 * 1024 {
            3
        } else if size > 25 * 1024 * 1024 {
            2
        } else {
            1
        }
    } else {
        DEFAULT_MAX_CONCURRENT_CHUNKS as usize
    }
}
