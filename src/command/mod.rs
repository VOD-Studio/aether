//! # 命令系统模块
//!
//! 提供命令解析、权限控制、路由分发等功能。
//!
//! ## 核心类型
//!
//! - [`CommandHandler`][]: 命令处理器 trait，定义命令的执行接口
//! - [`CommandGateway`][]: 命令网关，负责命令路由和分发
//! - [`CommandContext`][]: 命令执行上下文，包含执行所需的所有信息
//! - [`Permission`][]: 权限检查器
//!
//! ## 架构设计
//!
//! ```text
//! 消息 -> Parser -> ParsedCommand
//!                    ↓
//!              CommandGateway
//!                    ↓
//!         Permission Check (Permission)
//!                    ↓
//!              CommandHandler
//!                    ↓
//!              CommandContext
//!                    ↓
//!              执行命令逻辑
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use aether_matrix::command::{CommandGateway, CommandContext, CommandHandler};
//!
//! // 注册命令处理器
//! let mut gateway = CommandGateway::new(config.clone());
//! gateway.register("help", HelpHandler);
//!
//! // 处理命令
//! if let Some(parsed) = parser.parse(msg) {
//!     gateway.handle(&context, parsed.cmd, parsed.args).await;
//! }
//! ```

mod context;
mod gateway;
mod parser;
mod permission;
mod registry;

pub use context::CommandContext;
pub use gateway::CommandGateway;
pub use permission::Permission;
pub use registry::CommandHandler;
