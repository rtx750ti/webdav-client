use crate::{WEBDAV_ENV_PATH_1, load_account};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use webdav_client::client::WebDavClient;
use webdav_client::client::traits::account::{Account, AccountError};

#[tokio::test]
async fn test_add_account() -> Result<(), AccountError> {
    let mut client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let result = client.add_account(
        &webdav_account.url,
        &webdav_account.username,
        &webdav_account.password,
    );

    match result {
        Ok(key) => {
            println!(
                "插入账号测试成功：{}  {}",
                key.get_base_url(),
                key.get_username()
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e);
            Err(e)
        }
    }
}

#[tokio::test]
async fn test_remove_account() -> Result<(), AccountError> {
    let mut client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let result = client.add_account(
        &webdav_account.url,
        &webdav_account.username,
        &webdav_account.password,
    );

    match result {
        Ok(key) => {
            match client.remove_account(&key) {
                Ok(_) => {
                    println!(
                        "删除账号测试成功：{}  {}",
                        key.get_base_url(),
                        key.get_username()
                    );
                }
                Err(e) => {
                    eprintln!("删除账号测试失败，错误信息：{}", e);
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "[test_remove_account] 插入账号测试失败，错误信息：{}",
                e
            );
            Err(e)
        }
    }
}

#[tokio::test]
async fn test_get_http_client() -> Result<(), AccountError> {
    let mut client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    let key = client.add_account(
        &webdav_account.url,
        &webdav_account.username,
        &webdav_account.password,
    )?;

    let result = client.get_http_client(&key);

    match result {
        Ok(client_arc) => {
            println!(
                "获取http客户端测试成功,计数器:{}",
                Arc::strong_count(&client_arc)
            );
        }
        Err(e) => {
            eprintln!("获取http客户端测试失败:{}", e.to_string());
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_remove_account_force() -> Result<(), AccountError> {
    let mut client = WebDavClient::new();
    let webdav_account = load_account(WEBDAV_ENV_PATH_1);

    match client.add_account(
        &webdav_account.url,
        &webdav_account.username,
        &webdav_account.password,
    ) {
        Ok(key) => {
            // 多次获取引用并保留到变量中，让它们在延迟期间都活着
            let _ = client.get_http_client(&key)?;
            let _ = client.get_http_client(&key)?;
            let _ = client.get_http_client(&key)?;
            let http_client = client.get_http_client(&key)?;

            println!(
                "[test_remove_account_force] 测试延迟前计数器: {}",
                Arc::strong_count(&http_client)
            );

            // tokio 异步延迟
            sleep(Duration::from_secs(2)).await;

            println!(
                "[test_remove_account_force] 测试延迟后计数器: {}",
                Arc::strong_count(&http_client)
            );

            match client.remove_account_force(&key) {
                Ok(_) => {
                    println!(
                        "强制删除账号测试成功：{}  {}",
                        key.get_base_url(),
                        key.get_username()
                    );
                }
                Err(e) => {
                    eprintln!("强制删除账号测试失败，错误信息：{}", e);
                }
            }

            // 确保最后变量不立即 drop
            let _ = Arc::clone(&http_client);
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "[test_remove_account_force_with_delay] 插入账号测试失败，错误信息：{}",
                e
            );
            Err(e)
        }
    }
}
