//! # MCP 配置
//!
//! 管理 MCP 功能的所有配置项，包括内置工具和外部 MCP 服务器。

use serde::{Deserialize, Serialize};

/// MCP 总配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    /// 是否启用 MCP 功能
    #[serde(default)]
    pub enabled: bool,

    /// 内置工具配置
    #[serde(default)]
    pub builtin_tools: BuiltinToolsConfig,

    /// 外部 MCP 服务器配置
    #[serde(default)]
    pub external_servers: Vec<ExternalServerConfig>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            builtin_tools: BuiltinToolsConfig::default(),
            external_servers: Vec::new(),
        }
    }
}

/// 内置工具配置
#[derive(Debug, Clone, Deserialize)]
pub struct BuiltinToolsConfig {
    /// 是否启用内置工具
    #[serde(default = "default_builtin_enabled")]
    pub enabled: bool,

    /// web_fetch 工具配置
    #[serde(default)]
    pub web_fetch: WebFetchConfig,
}

fn default_builtin_enabled() -> bool {
    true
}

impl Default for BuiltinToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            web_fetch: WebFetchConfig::default(),
        }
    }
}

/// Web Fetch 工具配置
#[derive(Debug, Clone, Deserialize)]
pub struct WebFetchConfig {
    /// 是否启用 web_fetch 工具
    #[serde(default = "default_web_fetch_enabled")]
    pub enabled: bool,

    /// 最大内容长度（字符数）
    #[serde(default = "default_max_length")]
    pub max_length: usize,

    /// HTTP 请求超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_web_fetch_enabled() -> bool {
    true
}

fn default_max_length() -> usize {
    10000
}

fn default_timeout() -> u64 {
    10
}

impl Default for WebFetchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_length: 10000,
            timeout: 10,
        }
    }
}

/// 外部 MCP 服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct ExternalServerConfig {
    /// 服务器名称
    pub name: String,

    /// 传输类型
    pub transport: TransportType,

    /// 是否启用
    #[serde(default = "default_server_enabled")]
    pub enabled: bool,

    /// Stdio 传输：命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Stdio 传输：参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    /// HTTP/SSE 传输：URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

fn default_server_enabled() -> bool {
    true
}

/// 传输类型
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    /// Stdio 传输（通过标准输入输出）
    Stdio,
    /// HTTP 传输
    Http,
    /// SSE (Server-Sent Events) 传输
    Sse,
}

impl McpConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let enabled = std::env::var("MCP_ENABLED")
            .ok()
            .map(|s| s.to_lowercase() != "false")
            .unwrap_or(true);

        let builtin_tools = BuiltinToolsConfig::from_env()?;

        let external_servers = if let Ok(servers_json) = std::env::var("MCP_EXTERNAL_SERVERS") {
            serde_json::from_str(&servers_json).unwrap_or_else(|e| {
                tracing::warn!("Failed to parse MCP_EXTERNAL_SERVERS: {}", e);
                Vec::new()
            })
        } else {
            Vec::new()
        };

        Ok(Self {
            enabled,
            builtin_tools,
            external_servers,
        })
    }
}

impl BuiltinToolsConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let enabled = std::env::var("MCP_BUILTIN_TOOLS_ENABLED")
            .ok()
            .map(|s| s.to_lowercase() != "false")
            .unwrap_or(true);

        let web_fetch = WebFetchConfig::from_env()?;

        Ok(Self { enabled, web_fetch })
    }
}

impl WebFetchConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let enabled = std::env::var("MCP_BUILTIN_WEB_FETCH_ENABLED")
            .ok()
            .map(|s| s.to_lowercase() != "false")
            .unwrap_or(true);

        let max_length = std::env::var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000);

        let timeout = std::env::var("MCP_BUILTIN_WEB_FETCH_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        Ok(Self {
            enabled,
            max_length,
            timeout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert!(config.enabled);
        assert!(config.builtin_tools.enabled);
        assert!(config.builtin_tools.web_fetch.enabled);
    }

    #[test]
    fn test_web_fetch_config_defaults() {
        let config = WebFetchConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_length, 10000);
        assert_eq!(config.timeout, 10);
    }

    #[test]
    fn test_transport_type_deserialization() {
        let json = r#""stdio""#;
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Stdio);

        let json = r#""http""#;
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Http);

        let json = r#""sse""#;
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Sse);
    }
}
