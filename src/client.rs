use crate::constants::USER_AGENT;
use anyhow::Result;
use reqwest::{Client, header};
use std::sync::Arc;

pub fn create_client(url: &str) -> Result<Client> {
    let cookie_store = Arc::new(reqwest::cookie::Jar::default());

    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static(USER_AGENT));
    headers.insert("Referer", header::HeaderValue::from_str(url).unwrap());

    let client = Client::builder()
        .cookie_provider(cookie_store)
        .redirect(reqwest::redirect::Policy::limited(10))
        .default_headers(headers)
        .build()?;

    Ok(client)
}
