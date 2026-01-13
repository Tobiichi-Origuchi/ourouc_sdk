use crate::constants::CAS_LOGIN_URL;
use anyhow::{Context, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use url::{Url, form_urlencoded};

/// 登录
pub async fn login(client: &Client, username: &str, password: &str, url: &str) -> Result<()> {
    // 构建登录url
    let mut login_url = Url::parse(CAS_LOGIN_URL).expect("url 解析错误");
    login_url.query_pairs_mut().append_pair("service", url);

    // 提取 flowId
    let resp_get = client.get(login_url.as_str()).send().await?;
    let html_text = resp_get.text().await?;
    let flow_id = extract_flow_id(&html_text).context("无法提取 flowId")?;

    // 构建表单
    let params = [
        ("username", username),
        ("password", password),
        ("loginType", "username_password"),
        ("flowId", &flow_id),
        ("submit", "登录"),
        ("captcha", ""),
        ("delegator", ""),
        ("tokenCode", ""),
    ];
    let body_str = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params)
        .finish();

    // 发送登录请求
    let resp_login = client
        .post(login_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body_str)
        .send()
        .await?;

    // 验证登录结果
    let final_url = resp_login.url().to_string();
    let resp_text = resp_login.text().await?;

    // 停留在登录页
    if final_url.contains("id.ouc.edu.cn/sso/login") {
        // 用户名或密码错误
        if resp_text.contains(r"\u8D26\u53F7\u6216\u5BC6\u7801\u9519\u8BEF") {
            return Err(anyhow::anyhow!("登录失败：用户名或密码错误"));
        }

        // 触发人机验证
        if resp_text.contains(r"\u9700\u8981\u6821\u9A8C\u7801") {
            return Err(anyhow::anyhow!("登录失败：登录频繁，请稍后再试"));
        }

        return Err(anyhow::anyhow!("登录失败：未知原因"));
    };

    // 登录成功
    if final_url.contains(url) {
        println!("登录成功");
        return Ok(());
    };

    Err(anyhow::anyhow!("登录失败：未知原因"))
}

// 辅助函数
fn extract_flow_id(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("input[name='flowId']").ok()?;
    if let Some(element) = document.select(&selector).next() {
        return element.value().attr("value").map(|s| s.to_string());
    }
    None
}
