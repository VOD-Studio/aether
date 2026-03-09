//! # MCP 传输层
//!
//! 提供与 MCP 服务器通信的传输实现。
//!
//! ## 支持的传输方式
//!
//! - **Stdio**: 通过标准输入输出通信
//! - **HTTP**: 通过 HTTP API 通信
//! - **SSE**: 通过 Server-Sent Events 通信

pub mod stdio;
// pub mod http; 待后续实现，解决reqwest依赖冲突
// SSE 传输已包含在 http 模块中，由 rmcp 自动处理

pub use stdio::StdioTransport;
// pub use http::HttpTransport;
