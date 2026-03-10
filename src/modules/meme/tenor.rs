//! Tenor GIF API 客户端。

use anyhow::{Context, Result};
use serde::Deserialize;

/// Tenor API 客户端。
///
/// 用于搜索 GIF 图片。
pub struct TenorClient {
    api_key: String,
    limit: u32,
    http_client: reqwest::Client,
}

/// Tenor 搜索响应。
#[derive(Debug, Deserialize)]
struct TenorSearchResponse {
    results: Vec<TenorResult>,
}

#[derive(Debug, Deserialize)]
struct TenorResult {
    media_formats: MediaFormats,
}

#[derive(Debug, Deserialize)]
struct MediaFormats {
    gif: MediaFormat,
    #[serde(default)]
    tinygif: Option<MediaFormat>,
}

#[derive(Debug, Deserialize)]
struct MediaFormat {
    url: String,
}

/// GIF 搜索结果。
#[derive(Debug, Clone)]
pub struct GifResult {
    pub url: String,
}

impl TenorClient {
    /// 创建新的 Tenor 客户端。
    pub fn new(api_key: String, limit: u32) -> Self {
        Self {
            api_key,
            limit,
            http_client: reqwest::Client::new(),
        }
    }

    /// 搜索 GIF。
    ///
    /// # Arguments
    ///
    /// * `query` - 搜索关键词
    ///
    /// # Returns
    ///
    /// 返回随机一个 GIF 结果，如果没有结果则返回 None。
    pub async fn search(&self, query: &str) -> Result<Option<GifResult>> {
        let encoded_query = urlencoding::encode(query);
        let url = format!(
            "https://tenor.googleapis.com/v2/search?q={}&key={}&limit={}&random=true&media_filter=gif,tinygif",
            encoded_query, self.api_key, self.limit
        );

        tracing::debug!("Tenor API 请求: {}", url.replace(&self.api_key, "API_KEY"));

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("Tenor API 请求失败")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Tenor API 返回错误: {} - {}", status, body);
        }

        let search_response: TenorSearchResponse = response
            .json()
            .await
            .context("解析 Tenor API 响应失败")?;

        if search_response.results.is_empty() {
            return Ok(None);
        }

        // 随机选择一个结果
        use rand::prelude::IndexedRandom;
        let result = search_response
            .results
            .choose(&mut rand::rng())
            .context("没有可用的 GIF 结果")?;

        // 优先使用 tinygif（更小的文件），如果没有则使用普通 gif
        let url = result
            .media_formats
            .tinygif
            .as_ref()
            .map(|t| t.url.as_str())
            .unwrap_or(&result.media_formats.gif.url)
            .to_string();

        Ok(Some(GifResult { url }))
    }
}