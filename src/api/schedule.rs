use crate::constants::SCHEDULE_API_URL;
use crate::models::schedule::ApiResponse;
use anyhow::{Context, Result};
use infer;
use reqwest::Client;

pub async fn download_schedule(client: &Client) -> Result<ApiResponse> {
    let resp = client.get(SCHEDULE_API_URL).send().await?;

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
