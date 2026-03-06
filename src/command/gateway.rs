//! # 命令网关
//!
//! 负责命令的路由分发，是命令系统的核心枢纽。

use std::sync::{Arc, RwLock};

use anyhow::Result;
use matrix_sdk::Room;
use matrix_sdk::ruma::OwnedUserId;
use tracing::debug;

use super::context::{CommandContext, CommandContextArgs};
use super::parser::Parser;
use super::registry::CommandRegistry;
use crate::ui;

/// 命令网关，负责命令的路由分发。
///
/// 作为命令系统的核心枢纽，负责：
/// - 解析消息，提取命令
/// - 查找并调用对应的命令处理器
/// - 执行权限检查
/// - 处理 `help` 命令
///
/// # 线程安全
///
/// 使用 `Arc<RwLock<Parser>>` 支持命令前缀的热更新，
/// 使用 `Arc<CommandRegistry>` 共享命令注册表。
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::CommandGateway;
///
/// let mut gateway = CommandGateway::new("!".to_string(), config.bot_owners);
///
/// // 注册命令处理器
/// gateway.register(Arc::new(HelpHandler));
///
/// // 分发命令
/// gateway.dispatch(&client, room, sender, "!help").await?;
/// ```
#[derive(Clone)]
pub struct CommandGateway {
    /// 命令解析器（使用 RwLock 支持热更新）。
    parser: Arc<RwLock<Parser>>,
    /// 命令注册表（使用 Arc 支持共享）。
    registry: Arc<CommandRegistry>,
    /// Bot 所有者列表，用于权限检查。
    bot_owners: Vec<String>,
}

impl CommandGateway {
    /// 创建新的命令网关。
    ///
    /// # Arguments
    ///
    /// * `prefix` - 命令前缀，如 `"!"` 或 `"!ai "`
    /// * `bot_owners` - Bot 所有者列表，用于权限检查
    ///
    /// # Example
    ///
    /// ```ignore
    /// let gateway = CommandGateway::new("!".to_string(), vec!["@admin:matrix.org".to_string()]);
    /// ```
    pub fn new(prefix: String, bot_owners: Vec<String>) -> Self {
        Self {
            parser: Arc::new(RwLock::new(Parser::new(prefix))),
            registry: Arc::new(CommandRegistry::new()),
            bot_owners,
        }
    }

    /// 注册命令处理器。
    ///
    /// # Arguments
    ///
    /// * `handler` - 命令处理器实例
    ///
    /// # Note
    ///
    /// 如果已存在同名命令，新处理器会覆盖旧的。
    pub fn register(&mut self, handler: Arc<dyn super::registry::CommandHandler>) {
        let mut registry = (*self.registry).clone();
        registry.register(handler);
        self.registry = Arc::new(registry);
    }

    /// 设置命令前缀（支持热更新）。
    ///
    /// 允许在运行时更改命令前缀，无需重启服务。
    ///
    /// # Arguments
    ///
    /// * `prefix` - 新的命令前缀
    #[allow(dead_code)]
    pub fn set_prefix(&self, prefix: String) {
        self.parser.write().unwrap().set_prefix(prefix);
    }

    /// 检查消息是否是命令。
    ///
    /// 快速判断消息是否以命令前缀开头。
    ///
    /// # Arguments
    ///
    /// * `msg` - 原始消息文本
    ///
    /// # Returns
    ///
    /// 如果消息以命令前缀开头返回 `true`，否则返回 `false`。
    pub fn is_command(&self, msg: &str) -> bool {
        self.parser.read().unwrap().is_command(msg)
    }

    /// 分发命令。
    ///
    /// 完整的命令处理流程：
    /// 1. 解析消息，提取命令和参数
    /// 2. 特殊处理 `help` 命令
    /// 3. 查找命令处理器
    /// 4. 执行权限检查
    /// 5. 调用命令处理器执行
    ///
    /// # Arguments
    ///
    /// * `client` - Matrix 客户端
    /// * `room` - 消息来源房间
    /// * `sender` - 命令发送者
    /// * `msg` - 原始消息文本
    ///
    /// # Returns
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    ///
    /// # Errors
    ///
    /// 当命令处理器执行失败时返回错误。
    pub async fn dispatch(
        &self,
        client: &matrix_sdk::Client,
        room: Room,
        sender: OwnedUserId,
        msg: &str,
    ) -> Result<()> {
        let parsed = match self.parser.read().unwrap().parse(msg) {
            Some(p) => p,
            None => return Ok(()),
        };

        debug!("解析命令: cmd={}, args={:?}", parsed.cmd, parsed.args);

        if parsed.cmd == "help" {
            self.handle_help(&room).await?;
            return Ok(());
        }

        let handler = match self.registry.get(parsed.cmd) {
            Some(h) => h,
            None => {
                let html = ui::error(&format!("未知命令: !{}", parsed.cmd));
                send_html_message(&room, &html, &format!("未知命令: !{}", parsed.cmd)).await?;
                return Ok(());
            }
        };

        let permission = handler.permission();
        if !permission.check(&room, &sender, &self.bot_owners).await {
            let html = ui::error(&format!("权限不足: 需要 {}", permission.display_name()));
            send_html_message(
                &room,
                &html,
                &format!("权限不足: 需要 {}", permission.display_name()),
            )
            .await?;
            return Ok(());
        }

        let ctx = CommandContext::new(CommandContextArgs {
            client,
            room,
            sender,
            args: parsed.args,
            bot_owners: &self.bot_owners,
        });

        handler.execute(&ctx).await
    }

    /// 处理 help 命令。
    ///
    /// 生成并显示所有已注册命令的帮助信息。
    async fn handle_help(&self, room: &Room) -> Result<()> {
        let html = self.registry.generate_help_html();
        let plain = self.registry.generate_help();
        send_html_message(room, &html, &plain).await
    }
}

/// 发送 HTML 格式的消息。
///
/// 同时发送 HTML 和纯文本版本，客户端会选择支持的格式显示。
///
/// # Arguments
///
/// * `room` - 目标房间
/// * `html` - HTML 格式的消息内容
/// * `plain_fallback` - 纯文本格式的备选内容
///
/// # Returns
///
/// 成功时返回 `Ok(())`，失败时返回错误。
async fn send_html_message(room: &Room, html: &str, plain_fallback: &str) -> Result<()> {
    use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

    let content = RoomMessageEventContent::text_html(plain_fallback, html);
    room.send(content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_creation() {
        let gateway = CommandGateway::new("!".to_string(), vec!["@admin:matrix.org".to_string()]);
        assert!(gateway.is_command("!help"));
        assert!(!gateway.is_command("help"));
    }

    #[test]
    fn test_gateway_prefix_update() {
        let gateway = CommandGateway::new("!".to_string(), vec![]);
        assert!(gateway.is_command("!help"));
        assert!(gateway.is_command("!!help"));

        gateway.set_prefix("!!".to_string());
        assert!(gateway.is_command("!!help"));
        assert!(!gateway.is_command("!help"));
    }
}
