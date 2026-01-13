use anyhow::Result;
use ourouc_sdk::api::schedule::download_schedule;
use ourouc_sdk::auth::login::login;
use ourouc_sdk::client::create_client;
use ourouc_sdk::constants::JWGL_URL;

#[tokio::main]
async fn main() -> Result<()> {
    println!("请输入学号:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    println!("请输入密码:");
    let password = rpassword::read_password()?;

    println!("正在初始化 HTTP 客户端...");
    let client = create_client(JWGL_URL)?;

    println!("正在登录...");
    login(&client, &username, &password, JWGL_URL).await?;

    println!("正在下载课表...");
    let data = download_schedule(&client).await;

    match data {
        Ok(response) => {
            println!("请求成功!\n");
            println!("{:#?}", response);
        }
        Err(e) => {
            println!("请求失败或解析错误:\n");
            eprintln!("{:#}", e);
        }
    }

    Ok(())
}
