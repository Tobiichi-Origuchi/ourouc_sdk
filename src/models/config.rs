/// 用 otrust webvpn 代理实现校外访问
/// 目前尚未实现
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use url::Url;

// 连接模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConnectMode {
    Intranet, // 内网直连
    Extranet, // OTrust 代理
}

// 服务目标
#[derive(Debug, Clone)]
pub struct ServiceTarget {
    base_url: Url,
}

impl ServiceTarget {
    /// 构造函数
    pub fn new(raw_url: &str) -> Result<Self> {
        let url = Url::parse(raw_url).map_err(|e| anyhow!("无效的 URL 格式: {}", e))?;
        Ok(Self { base_url: url })
    }

    /// 根据模式获取完整的 URL 字符串
    pub fn get_url(&self, mode: ConnectMode) -> String {
        match mode {
            ConnectMode::Intranet => self.base_url.to_string(),
            ConnectMode::Extranet => self.to_otrust_url(),
        }
    }

    /// 根据模式获取 Host 头
    pub fn get_host_header(&self, mode: ConnectMode) -> String {
        match mode {
            ConnectMode::Intranet => self.base_url.host_str().unwrap_or("").to_string(),
            ConnectMode::Extranet => {
                let otrust = self.to_otrust_url();
                if let Ok(u) = Url::parse(&otrust) {
                    u.host_str().unwrap_or("").to_string()
                } else {
                    String::new()
                }
            }
        }
    }

    /// 辅助函数：将原始 URL 转换为 OTrust 格式
    fn to_otrust_url(&self) -> String {
        let original_host = self.base_url.host_str().unwrap_or("");
        let scheme = self.base_url.scheme(); // "http" or "https"

        // 域名替换
        let converted_host = original_host.replace('.', "-");

        // 协议后缀处理
        let suffix = if scheme == "https" { "-s" } else { "-p" };

        // 拼接 Host
        let new_host = format!("{}{}.otrust.ouc.edu.cn", converted_host, suffix);

        // 重组 URL
        let mut new_url = self.base_url.clone();
        let _ = new_url.set_scheme("https");
        let _ = new_url.set_host(Some(&new_host));
        let _ = new_url.set_port(None);

        new_url.to_string()
    }
}
