use anyhow::Result;
use matrix_sdk::{
    Client, Room,
    ruma::{
        OwnedUserId,
        events::room::member::{MembershipState, StrippedRoomMemberEvent},
    },
};
use tracing::{debug, info, warn};

use crate::ai_service::AiService;
use crate::config::Config;

#[derive(Clone)]
pub struct EventHandler {
    ai_service: AiService,
    bot_user_id: OwnedUserId,
    command_prefix: String,
}

impl EventHandler {
    pub fn new(ai_service: AiService, bot_user_id: OwnedUserId, config: &Config) -> Self {
        Self {
            ai_service,
            bot_user_id,
            command_prefix: config.command_prefix.clone(),
        }
    }

    /// 处理房间邀请
    pub async fn handle_invite(
        ev: StrippedRoomMemberEvent,
        client: Client,
        room: Room,
    ) -> Result<()> {
        if ev.content.membership != MembershipState::Invite {
            return Ok(());
        }

        let user_id = &ev.state_key;
        let my_user_id = client.user_id().expect("user_id should be available");
        if user_id != my_user_id {
            return Ok(()); // 不是邀请机器人
        }

        let room_id = room.room_id();
        info!("收到房间邀请: {}", room_id);

        match client.join_room_by_id(room_id).await {
            Ok(_) => info!("成功加入房间: {}", room_id),
            Err(e) => warn!("加入房间失败: {}", e),
        }

        Ok(())
    }

    /// 处理消息
    pub async fn handle_message(
        &self,
        ev: matrix_sdk::ruma::events::room::message::SyncRoomMessageEvent,
        room: Room,
    ) -> Result<()> {
        use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

        // 使用 as_original() 获取原始消息事件
        let original = match ev.as_original() {
            Some(o) => o,
            None => return Ok(()), // 忽略已删除的消息
        };

        // 跳过自己发送的消息
        if original.sender == self.bot_user_id {
            return Ok(());
        }

        // 获取消息文本
        let text = original.content.body();

        let room_id = room.room_id();

        // 判断是否应该响应
        let is_direct = room.is_direct().await.unwrap_or(false);
        let should_respond = if is_direct {
            // 私聊：总是响应
            true
        } else {
            // 房间：检查命令前缀或 @提及
            text.starts_with(&self.command_prefix) || text.contains(&self.bot_user_id.to_string())
        };

        if !should_respond {
            return Ok(());
        }

        // 处理命令
        let clean_text = self.extract_message(text);

        if clean_text == "!reset" {
            let session_id = room_id.to_string();
            self.ai_service.reset_conversation(&session_id).await;
            room.send(RoomMessageEventContent::text_plain("会话历史已清除"))
                .await?;
            return Ok(());
        }

        if clean_text == "!help" {
            let help_text = format!(
                "可用命令:\n{} <消息> - 与 AI 对话\n!reset - 清除会话历史\n!help - 显示帮助",
                self.command_prefix
            );
            room.send(RoomMessageEventContent::text_plain(help_text))
                .await?;
            return Ok(());
        }

        // 生成会话 ID（私聊用用户 ID，房间用房间 ID）
        let session_id = if is_direct {
            original.sender.to_string()
        } else {
            room_id.to_string()
        };

        debug!("处理消息 [{}]: {}", session_id, clean_text);

        // 调用 AI
        match self.ai_service.chat(&session_id, &clean_text).await {
            Ok(reply) => {
                room.send(RoomMessageEventContent::text_plain(reply))
                    .await?;
            }
            Err(e) => {
                warn!("AI 调用失败: {}", e);
                room.send(RoomMessageEventContent::text_plain(format!(
                    "AI 服务暂时不可用: {}",
                    e
                )))
                .await?;
            }
        }

        Ok(())
    }

    fn extract_message(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 移除命令前缀
        if result.starts_with(&self.command_prefix) {
            result = result[self.command_prefix.len()..].to_string();
        }

        // 移除 @提及
        result = result.replace(&self.bot_user_id.to_string(), "");

        result.trim().to_string()
    }
}
