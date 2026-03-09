//! # HTTP/SSE 传输实现
//!
//! 与远程 HTTP/SSE MCP 服务器通信的传输层实现。

use rmcp::client::Client;
use rmcp::transport::streamable_http::reqwest::ReqwestStreamableHttpTransport;
use anyhow::{Context, Result};
use reqwest::Client as ReqwestClient;

use super::super::config::ExternalServerConfig;

/// HTTP/SSE 传输客户端
pub struct HttpTransport {
    client: Client,
    config: ExternalServerConfig,
}

impl HttpTransport {
    /// 创建新的 HTTP 传输客户端
    pub async fn new(config: &ExternalServerConfig) -> Result<Self> {
        let url = config.url.as_ref()
            .context("HTTP transport requires 'url' field in configuration")?;
        
        // 创建 HTTP 客户端
        let http_client = ReqwestClient::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Aether-Matrix-Bot/1.0")
            .build()
            .context("Failed to create HTTP client")?;
        
        // 创建 HTTP 传输
        let transport = ReqwestStreamableHttpTransport::new(
            url.parse().context("Invalid MCP server URL")?,
            http_client,
        );
        
        // 创建 MCP 客户端
        let client = Client::builder()
            .transport(transport)
            .name("aether-matrix-bot")
            .version(env!("CARGO_PKG_VERSION"))
            .build()
            .await
            .context("Failed to create MCP client")?;
        
        Ok(Self {
            client,
            config: config.clone(),
        })
    }
    
    /// 获取 MCP 客户端引用
    pub fn client(&self) -> &Client {
        &self.client
    }
    
    /// 获取服务器配置
    pub fn config(&self) -> &ExternalServerConfig {
        &self.config
    }
    
    /// 关闭连接
    pub async fn close(self) -> Result<()> {
        self.client.close().await?;
        Ok(())
    }
}