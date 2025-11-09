use crate::{load_account, WEBDAV_ENV_PATH_1};
use webdav_client::client::traits::account::Account;
use webdav_client::client::traits::local_folders::LocalFolders;
use webdav_client::client::WebDavClient;

#[tokio::test]
async fn test_get_local_folders() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试路径：可以根据实际情况修改
    let paths = vec![
        "C:\\project\\rust\\quick-sync".to_string(), // 文件夹路径
    ];

    let results = client.get_local_folders(&key, &paths).await?;

    println!("获取的结果数量：{}", results.len());

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok((files, failed)) => {
                println!(
                    "路径 {} 成功: {} 个文件, {} 个失败",
                    i,
                    files.len(),
                    failed.len()
                );

                // 打印前5个文件信息
                for (j, file) in files.iter().take(5).enumerate() {
                    println!("  文件 {}: {:?}", j + 1, file);
                }

                // 打印失败的文件
                for (j, error) in failed.iter().enumerate() {
                    println!("  失败 {}: {:?} - {}", j + 1, error.path, error.cause);
                }
            }
            Err(e) => {
                println!("路径 {} 失败: {}", i, e);
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_with_file() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试单个文件路径
    let paths = vec![
        "C:\\project\\rust\\quick-sync\\Cargo.toml".to_string(), // 文件路径
    ];

    let results = client.get_local_folders(&key, &paths).await?;

    println!("获取的结果数量：{}", results.len());

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok((files, failed)) => {
                println!(
                    "路径 {} 成功: {} 个文件, {} 个失败",
                    i,
                    files.len(),
                    failed.len()
                );
                assert_eq!(files.len(), 1, "单个文件应该返回1个文件");
            }
            Err(e) => {
                println!("路径 {} 失败: {}", i, e);
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_get_local_folders_not_exist() -> Result<(), String> {
    let client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client
        .add_account(
            &webdav_account.url,
            &webdav_account.username,
            &webdav_account.password,
        )
        .map_err(|e| e.to_string())?;

    // 测试不存在的路径
    let paths = vec![
        "C:\\not_exist_path_12345".to_string(), // 不存在的路径
    ];

    let results = client.get_local_folders(&key, &paths).await?;

    println!("获取的结果数量：{}", results.len());

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok((files, failed)) => {
                println!(
                    "路径 {} 成功: {} 个文件, {} 个失败",
                    i,
                    files.len(),
                    failed.len()
                );
                assert_eq!(files.len(), 0, "不存在的路径应该返回0个文件");
                assert_eq!(failed.len(), 0, "不存在的路径应该返回0个失败");
            }
            Err(e) => {
                println!("路径 {} 失败: {}", i, e);
            }
        }
    }

    Ok(())
}
