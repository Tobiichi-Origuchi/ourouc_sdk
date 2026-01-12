use anyhow::{Context, Result};
use reqwest::{Client, header};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    pub msg: char,                 // 都是':'
    pub code: bool,                // 都是0
    pub data: Option<Vec<Course>>, // 选课数据
    pub count: Option<u8>,         // 选课数
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    pub kch: Option<String>,      // 课程号
    pub kc_mc: Option<String>,    // 课程名称
    pub jg0101mc: Option<String>, // 教师姓名
    pub jsgh: Option<String>,     // 教师工号
    pub kt_mc: Option<String>,    // 课堂名称
    pub pkrs: Option<u16>,        // 排课人数
    pub xkrs: Option<u16>,        // 选课人数
    pub kcxz: Option<String>,     // 课程选择
    pub kclb: Option<String>,     // 课程类别
    pub jx0404id: Option<String>, // 选课号
    pub sktime: Option<String>,   // 上课时间
    pub skddmc: Option<String>,   // 上课地点名称
    pub skxqmc: Option<String>,   // 上课校区名称
    pub kkyx: Option<String>,     // 开课院系
    pub zhouxs: Option<u8>,       // 周学时
    pub xf: Option<f32>,          // 学分
    pub zxs: Option<u8>,          // 总学时
    pub khfs: Option<String>,     // 考核方式
    pub xsfl0: Option<u8>,        // 讲课学时
    pub xsfl1: Option<u8>,        // 实践学时
    pub bj: Option<String>,       // 备注
    pub rownum_: Option<u8>,      // 行序号
}

// 登录页 URL
const LOGIN_PAGE_URL: &str =
    "https://id.ouc.edu.cn/sso/login?service=https%3A%2F%2Fjwgl2024.ouc.edu.cn%2F";
// 课表下载 URL
const DOWNLOAD_URL: &str = "https://jwgl2024.ouc.edu.cn/jsxsd/xskb/xskb_print.do?viweType=0&showallprint=0&showkchprint=0&showkink=0&showfzmprint=0&baseUrl=%2Fjsxsd&xsflMapListJsonStr=%E8%AE%B2%E8%AF%BE%E5%AD%A6%E6%97%B6%2C%E5%AE%9E%E8%B7%B5%E5%AD%A6%E6%97%B6%2C&xnxq01id=2025-2026-2&zc=&kbjcmsid=16FD8C2BE55E15F9E0630100007FF6B5";

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 获取用户输入
    println!("请输入学号:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    println!("请输入密码:");
    let password = rpassword::read_password()?;

    println!("正在初始化 HTTP 客户端...");

    // 2. 初始化带 Cookie 容器的 Client
    let cookie_store = Arc::new(reqwest::cookie::Jar::default());

    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36"));
    headers.insert(
        "Referer",
        header::HeaderValue::from_static("https://jwgl2024.ouc.edu.cn/"),
    );

    let client = Client::builder()
        .cookie_provider(cookie_store.clone())
        .redirect(reqwest::redirect::Policy::limited(10))
        .default_headers(headers)
        .build()?;

    // 3. 第一步：GET 登录页，获取 flowId
    println!("正在获取登录页面信息...");
    let resp_get = client.get(LOGIN_PAGE_URL).send().await?;
    let html_text = resp_get.text().await?;

    let flow_id = extract_flow_id(&html_text).context("无法提取 flowId")?;
    println!("获取成功，FlowID: {}", flow_id);

    // 4. 第二步：构建表单并 POST 登录
    println!("正在发送登录请求...");

    // 手动构建 x-www-form-urlencoded 字符串
    let body_str = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("username", &username)
        .append_pair("password", &password)
        .append_pair("loginType", "username_password")
        .append_pair("flowId", &flow_id)
        .append_pair("submit", "登录")
        .append_pair("captcha", "")
        .append_pair("delegator", "")
        .append_pair("tokenCode", "")
        .append_pair("continue", "")
        .append_pair("asserts", "")
        .append_pair("pageFrom", "")
        .finish();

    // 发送 POST
    let resp_login = client
        .post(LOGIN_PAGE_URL)
        .header("Content-Type", "application/x-www-form-urlencoded") // 显式声明内容类型
        .body(body_str) // 直接发送字符串作为 body
        .send()
        .await?;

    // 检查登录结果
    let final_url = resp_login.url().to_string();
    println!("登录后跳转到了: {}", final_url);

    if final_url.contains("sso/login") {
        return Err(anyhow::anyhow!(
            "登录失败！依然停留在登录页，可能是密码错误或有验证码。"
        ));
    }

    // 5. 第三步：下载课表
    println!("登录成功，正在下载课表...");
    let resp_download = client.get(DOWNLOAD_URL).send().await?;

    if resp_download.status().is_success() {
        let content = resp_download.bytes().await?;

        if content.starts_with(b"<!DOCTYPE") || content.starts_with(b"<html") {
            println!("警告：下载内容看起来像 HTML，可能 Session 无效。");
        } else {
            let mut file = File::create("course_schedule.xls")?;
            file.write_all(&content)?;
            println!("--------------------------------------");
            println!("成功！课表已保存为 course_schedule.xls");
            println!("--------------------------------------");
        }
    } else {
        println!("下载失败: {}", resp_download.status());
    }

    Ok(())
}

fn extract_flow_id(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("input[name='flowId']").ok()?;
    if let Some(element) = document.select(&selector).next() {
        return element.value().attr("value").map(|s| s.to_string());
    }
    None
}
