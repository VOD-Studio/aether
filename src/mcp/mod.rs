//! # MCP (Model Context Protocol) 集成模块
//!
//! 提供 MCP Client 功能，支持内置工具和外部 MCP 服务器集成。
//!
//! ## 功能特性
//!
//! - **内置工具**: 开箱即用的工具（如 web_fetch）
//! - **外部 MCP**: 支持连接外部 MCP 服务器
//! - **统一接口**: 内置和外部工具使用相同的调用接口
//! - **优雅降级**: MCP 不可用时自动降级
//!
//! ## 模块结构
//!
//! - [`tool_registry`][]: 工具注册表，统一管理所有工具
//! - [`config`][]: MCP 配置管理
//! - [`builtin`][]: 内置工具实现
//! - [`transport`][]: MCP 传输层（Stdio/HTTP/SSE）

pub mod builtin;
pub mod config;
pub mod tool_registry;
pub mod transport;

pub use config::{McpConfig, BuiltinToolsConfig, WebFetchConfig, ExternalServerConfig, TransportType};
pub use tool_registry::{Tool, ToolDefinition, ToolResult, ToolSource, ToolRegistry};