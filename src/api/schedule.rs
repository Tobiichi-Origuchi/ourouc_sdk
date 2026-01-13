use crate::constants::SCHEDULE_API_URL;
use crate::models::schedule::{ApiResponse, CourseMeta, Semester};
use anyhow::{Context, Result};
use infer;
use reqwest::Client;
use scraper::{Html, Selector};
use url::form_urlencoded;

pub async fn fetch_course_meta(client: &Client) -> Result<CourseMeta> {
    let body_str = form_urlencoded::Serializer::new(String::new())
        .append_pair("viweType", "0")
        .finish();

    // 发送请求
    let resp = client
        .post(SCHEDULE_API_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body_str)
        .send()
        .await?;
    let html = resp.text().await?;
    let document = Html::parse_document(&html);

    // 提取 kbjcmsid (时间模式 ID)
    let selector_kb = Selector::parse("select#kbjcmsid option").unwrap();
    let kbjcmsid = document
        .select(&selector_kb)
        .next()
        .context("未找到时间模式 ID (kbjcmsid)")?
        .value()
        .attr("value")
        .unwrap_or("16FD8C2BE55E15F9E0630100007FF6B5") // 如果没找到就回退到默认的崂山鱼山时间
        .to_string();

    // 提取学期列表
    let selector_xq = Selector::parse("select#xnxq01id option").unwrap();
    let mut semesters = Vec::new();

    for element in document.select(&selector_xq) {
        let id = element.value().attr("value").unwrap_or("").to_string();
        let name = element.text().collect::<String>().trim().to_string();
        // 判断是否是当前选中项
        let is_current = element.value().attr("selected").is_some();

        if !id.is_empty() {
            semesters.push(Semester {
                id,
                name,
                is_current,
            });
        }
    }

    Ok(CourseMeta {
        semesters,
        kbjcmsid,
    })
}

pub async fn download_schedule(client: &Client) -> Result<ApiResponse> {
    let meta = fetch_course_meta(client).await?;

    // 构建表单
    let params = [
        ("viweType", "1"),
        ("needData", "1"),
        ("pageNum", "1"),
        ("pageSize", "20"),
        ("baseUrl", "/jsxsd"),
        ("sfykb", "2"),
        ("xsflMapListJsonStr", "讲课学时,实践学时,"),
        ("xnxq01id", &meta.semesters[0].id), // 学期 ID
        ("kbjcmsid", &meta.kbjcmsid),        // 时间模式（崂山鱼山时间（默认））
        ("zc", ""),                          // 周次（空值表示全部）
    ];
    let body_str = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params)
        .finish();

    // 发送请求
    let resp = client
        .post(SCHEDULE_API_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body_str)
        .send()
        .await?;

    // 检查状态码
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("下载失败: 状态码 {}", resp.status()));
    }

    // 下载 Body
    let content = resp.bytes().await?;

    // 使用 infer 库进行 Magic Bytes 检测
    if let Some(kind) = infer::get(&content) {
        if kind.mime_type() == "text/html" {
            let limit = content.len().min(200);
            let snippet = String::from_utf8_lossy(&content[..limit]);
            return Err(anyhow::anyhow!(
                "下载内容检测为 HTML 格式，Session 可能无效。片段:\n{}",
                snippet
            ));
        }
    }

    let api_response: ApiResponse = serde_json::from_slice(&content).context("解析 JSON 失败")?;

    Ok(api_response)
}
