//! 命令路由核心

use std::sync::Arc;

use anyhow::Result;
use matrix_sdk::Room;
use matrix_sdk::ruma::{OwnedEventId, OwnedUserId};
use tracing::debug;

use super::context::CommandContext;
use super::parser::Parser;
use super::registry::CommandRegistry;
use crate::ui;

/// 命令网关，负责路由分发
#[derive(Clone)]
pub struct CommandGateway {
    /// 命令解析器
    parser: Parser,
    /// 命令注册表（使用 Arc 支持共享）
    registry: Arc<CommandRegistry>,
    /// Bot 所有者列表
    bot_owners: Vec<String>,
}

impl CommandGateway {
    /// 创建新的命令网关
    pub fn new(prefix: String, bot_owners: Vec<String>) -> Self {
        Self {
            parser: Parser::new(prefix),
            registry: Arc::new(CommandRegistry::new()),
            bot_owners,
        }
    }

    /// 注册命令处理器
    pub fn register(&mut self, handler: Arc<dyn super::registry::CommandHandler>) {
        // 由于使用 Arc，需要创建新的 Registry 来注册
        let mut registry = (*self.registry).clone();
        registry.register(handler);
        self.registry = Arc::new(registry);
    }

    /// 获取命令解析器
    pub fn parser(&self) -> &Parser {
        &self.parser
    }

    /// 获取命令解析器（可变）
    pub fn parser_mut(&mut self) -> &mut Parser {
        &mut self.parser
    }

    /// 检查消息是否是命令
    pub fn is_command(&self, msg: &str) -> bool {
        self.parser.is_command(msg)
    }

    /// 分发命令
    pub async fn dispatch(
        &self,
        client: &matrix_sdk::Client,
        room: Room,
        sender: OwnedUserId,
        msg: &str,
        event_id: OwnedEventId,
    ) -> Result<()> {
        // 解析命令
        let parsed = match self.parser.parse(msg) {
            Some(p) => p,
            None => return Ok(()),
        };

        debug!("解析命令: cmd={}, args={:?}", parsed.cmd, parsed.args);

        // 处理内置命令
        if parsed.cmd == "help" {
            self.handle_help(&room).await?;
            return Ok(());
        }

        // 查找命令处理器
        let handler = match self.registry.get(parsed.cmd) {
            Some(h) => h,
            None => {
                // 未知命令
                let html = ui::error(&format!("未知命令: !{}", parsed.cmd));
                send_html_message(&room, &html, &format!("未知命令: !{}", parsed.cmd)).await?;
                return Ok(());
            }
        };

        // 权限检查
        let permission = handler.permission();
        if !permission.check(&room, &sender, &self.bot_owners).await {
            let html = ui::error(&format!("权限不足: 需要 {}", permission.display_name()));
            send_html_message(&room, &html, &format!("权限不足: 需要 {}", permission.display_name())).await?;
            return Ok(());
        }

        // 创建上下文并执行
        let ctx = CommandContext::new(
            client,
            room,
            sender,
            parsed.cmd,
            parsed.args,
            parsed.raw_msg,
            event_id,
            &self.bot_owners,
        );

        handler.execute(&ctx).await
    }

    /// 处理 help 命令
    async fn handle_help(&self, room: &Room) -> Result<()> {
        let html = self.registry.generate_help_html();
        let plain = self.registry.generate_help();
        send_html_message(room, &html, &plain).await
    }
}

/// 发送 HTML 消息
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
    fn test_gateway_parser() {
        let mut gateway = CommandGateway::new("!".to_string(), vec![]);
        gateway.parser_mut().set_prefix("!!".to_string());

        assert!(gateway.is_command("!!help"));
        assert!(!gateway.is_command("!help"));
    }
}