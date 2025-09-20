/// 分片黑名单，这些厂商不讲武德，拒绝分片请求，甚至拿1比特数据都要算下载了整个文件的流量
pub const CHUNKED_DOWNLOAD_BLACKLIST: [&str; 2] =
    ["https://dav.jianguoyun.com/", "https://aki.teracloud.jp/"];

/// 查找地址是否在分片黑名单里
pub fn is_chunked_download_blacklisted(base_url: &str) -> bool {
    CHUNKED_DOWNLOAD_BLACKLIST
        .iter()
        .any(|blacklisted_url| base_url.starts_with(blacklisted_url))
}
