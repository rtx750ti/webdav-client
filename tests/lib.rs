#[cfg(test)]
pub mod structs_test;
#[cfg(test)]
pub mod traits_impl_test;
#[cfg(test)]
use dotenvy::from_filename_override;
#[cfg(test)]
use std::env;
#[cfg(test)]
use std::path::Path;

/// 坚果云
#[cfg(test)]
pub const WEBDAV_ENV_PATH_1: &str =
    "C:\\project\\rust\\quick-sync\\.env.jianguoyun";
/// teracloud
#[cfg(test)]
pub const WEBDAV_ENV_PATH_2: &str =
    "C:\\project\\rust\\quick-sync\\.env.teracloud";

#[cfg(test)]
#[derive(Debug)]
pub struct WebDavAccount {
    url: String,
    username: String,
    password: String,
}

#[cfg(test)]
pub fn load_account(path: &str) -> WebDavAccount {
    from_filename_override(Path::new(path)).expect("无法加载 env 文件");
    WebDavAccount {
        url: env::var("WEBDAV_URL").expect("缺少 WEBDAV_URL"),
        username: env::var("WEBDAV_USERNAME")
            .expect("缺少 WEBDAV_USERNAME"),
        password: env::var("WEBDAV_PASSWORD")
            .expect("缺少 WEBDAV_PASSWORD"),
    }
}

#[cfg(test)]
pub fn assert_test_result(
    ok_count: usize,
    err_count: usize,
    expected_ok_count: usize,
    expected_err_count: usize,
    test_name: &str,
) {
    println!("统计结果：正确 {} 个，错误 {} 个", ok_count, err_count);

    if ok_count == expected_ok_count && err_count == expected_err_count {
        println!("测试结果: OK ✅");
    } else {
        panic!("❌ 测试异常[{}]：统计数量不匹配", test_name);
    }
}
