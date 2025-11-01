// use webdav_client::client::traits::account::Account;
// use webdav_client::client::WebDavClient;
// use crate::{load_account, WEBDAV_ENV_PATH_2};

// #[tokio::test]
// async fn test_upload() -> Result<(), String> {
//     let client = WebDavClient::new();
//     let webdav_account = load_account(WEBDAV_ENV_PATH_2);

//     let key = client
//         .add_account(
//             &webdav_account.url,
//             &webdav_account.username,
//             &webdav_account.password,
//         )
//         .map_err(|e| e.to_string())?;
    
//     Ok(())
// }
