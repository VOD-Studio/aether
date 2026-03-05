//! Admin 命令处理器

use anyhow::Result;
use async_trait::async_trait;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::command::{CommandContext, CommandHandler, Permission};
use crate::ui::{info_card, success, warning, error};

/// Bot 信息命令处理器
pub struct BotInfoHandler;

#[async_trait]
impl CommandHandler for BotInfoHandler {
    fn name(&self) -> &str {
        "bot"
    }

    fn description(&self) -> &str {
        "Bot 管理命令"
    }

    fn usage(&self) -> &str {
        "bot <info|name|ping>"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let sub = ctx.sub_command();

        match sub {
            Some("info") => self.handle_info(ctx).await,
            Some("name") => self.handle_name(ctx).await,
            Some("ping") => self.handle_ping(ctx).await,
            _ => self.handle_help(ctx).await,
        }
    }
}

impl BotInfoHandler {
    async fn handle_help(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let items = vec![
            ("!bot info", "查看 Bot 基本信息"),
            ("!bot name <名称>", "修改 Bot 显示名称（需要 Bot 所有者权限）"),
            ("!bot ping", "测试响应延迟"),
            ("!bot leave", "离开当前房间（需要管理员权限）"),
        ];
        let html = info_card("Bot 命令", &items);
        send_html(&ctx.room, &html).await
    }

    async fn handle_info(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let user_id = ctx
            .client
            .user_id()
            .map(|u| u.to_string())
            .unwrap_or_else(|| "未知".to_string());

        let device_id = ctx
            .client
            .device_id()
            .map(|d| d.to_string())
            .unwrap_or_else(|| "未知".to_string());

        let rooms_count = ctx.client.joined_rooms().len();
        let rooms_str = format!("{} 个", rooms_count);

        let items = vec![
            ("用户 ID", user_id.as_str()),
            ("设备 ID", device_id.as_str()),
            ("已加入房间", rooms_str.as_str()),
            ("运行状态", "正常运行中"),
        ];

        let html = info_card("Bot 信息", &items);
        send_html(&ctx.room, &html).await
    }

    async fn handle_name(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // 检查权限 - 需要 BotOwner
        if !Permission::BotOwner.check(&ctx.room, &ctx.sender, ctx.bot_owners).await {
            let html = error("权限不足: 需要 Bot 所有者权限");
            return send_html(&ctx.room, &html).await;
        }

        // 获取新名称参数（子命令后的参数）
        let new_name: String = ctx.sub_args().join(" ");
        if new_name.is_empty() {
            let html = error("请提供新名称: !bot name <名称>");
            return send_html(&ctx.room, &html).await;
        }

        // 调用 Matrix API 设置显示名称
        let account = ctx.client.account();
        match account.set_display_name(Some(&new_name)).await {
            Ok(()) => {
                let html = success(&format!("显示名称已修改为: {}", new_name));
                send_html(&ctx.room, &html).await
            }
            Err(e) => {
                let html = error(&format!("修改显示名称失败: {}", e));
                send_html(&ctx.room, &html).await
            }
        }
    }

    async fn handle_ping(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let html = success("Pong! 机器人响应正常");
        send_html(&ctx.room, &html).await
    }
}

/// Bot 离开房间命令处理器
pub struct BotLeaveHandler;

#[async_trait]
impl CommandHandler for BotLeaveHandler {
    fn name(&self) -> &str {
        "leave"
    }

    fn description(&self) -> &str {
        "让 Bot 离开当前房间"
    }

    fn usage(&self) -> &str {
        "leave"
    }

    fn permission(&self) -> Permission {
        Permission::RoomMod
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let room_id = ctx.room_id();

        // 发送告别消息
        let html = warning(&format!("再见！正在离开房间 {} ...", room_id));
        send_html(&ctx.room, &html).await?;

        // 离开房间
        ctx.room.leave().await?;

        Ok(())
    }
}

/// Bot Ping 命令处理器
pub struct BotPingHandler;

#[async_trait]
impl CommandHandler for BotPingHandler {
    fn name(&self) -> &str {
        "ping"
    }

    fn description(&self) -> &str {
        "测试 Bot 响应"
    }

    fn usage(&self) -> &str {
        "ping"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        let html = success("Pong! 机器人响应正常");
        send_html(&ctx.room, &html).await
    }
}

/// 发送 HTML 消息
async fn send_html(room: &matrix_sdk::Room, html: &str) -> Result<()> {
    // 提取纯文本作为 fallback
    let plain_text = html
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != ' ', "")
        .chars()
        .take(100)
        .collect::<String>();

    let content = RoomMessageEventContent::text_html(plain_text, html);
    room.send(content).await?;
    Ok(())
}