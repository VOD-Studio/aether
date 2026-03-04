use std::time::{Duration, Instant};

use anyhow::Result;
use futures_util::StreamExt;
use matrix_sdk::{
    Client, Room,
    ruma::{
        OwnedEventId, OwnedUserId,
        events::room::{
            member::{MembershipState, StrippedRoomMemberEvent},
            message::{RoomMessageEventContent, ReplacementMetadata},
        },
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
    // 流式输出配置
    streaming_enabled: bool,
    streaming_min_interval: Duration,
    streaming_min_chars: usize,
}

impl EventHandler {
    pub fn new(ai_service: AiService, bot_user_id: OwnedUserId, config: &Config) -> Self {
        Self {
            ai_service,
            bot_user_id,
            command_prefix: config.command_prefix.clone(),
            streaming_enabled: config.streaming_enabled,
            streaming_min_interval: Duration::from_millis(config.streaming_min_interval_ms),
            streaming_min_chars: config.streaming_min_chars,
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

        // 根据配置选择流式或普通响应
        if self.streaming_enabled {
            self.handle_streaming_response(&room, &session_id, &clean_text)
                .await?;
        } else {
            self.handle_normal_response(&room, &session_id, &clean_text)
                .await?;
        }

        Ok(())
    }

    /// 普通响应（非流式）
    async fn handle_normal_response(
        &self,
        room: &Room,
        session_id: &str,
        clean_text: &str,
    ) -> Result<()> {
        match self.ai_service.chat(session_id, clean_text).await {
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

    /// 流式响应（打字机效果）
    async fn handle_streaming_response(
        &self,
        room: &Room,
        session_id: &str,
        clean_text: &str,
    ) -> Result<()> {
        // 开始流式聊天
        let (state, mut stream) = match self.ai_service.chat_stream(session_id, clean_text).await {
            Ok(result) => result,
            Err(e) => {
                warn!("流式 AI 调用初始化失败: {}", e);
                room.send(RoomMessageEventContent::text_plain(format!(
                    "AI 服务暂时不可用: {}",
                    e
                )))
                .await?;
                return Ok(());
            }
        };

        // 状态追踪
        let mut event_id: Option<OwnedEventId> = None;
        let mut chars_since_update: usize = 0;
        let mut last_update = Instant::now();

        // 消费流
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(delta) => {
                    chars_since_update += delta.chars().count();

                    // 检查是否需要更新消息（混合策略）
                    let time_elapsed = last_update.elapsed() >= self.streaming_min_interval;
                    let chars_accumulated = chars_since_update >= self.streaming_min_chars;

                    if time_elapsed || chars_accumulated {
                        // 获取当前累积的内容
                        let content = {
                            let s = state.lock().await;
                            s.content().to_string()
                        };

                        // 发送或编辑消息
                        if let Some(ref original_event_id) = event_id {
                            // 编辑已有消息
                            let metadata = ReplacementMetadata::new(original_event_id.clone(), None);
                            let msg_content = RoomMessageEventContent::text_plain(&content)
                                .make_replacement(metadata);
                            room.send(msg_content).await?;
                        } else {
                            // 发送新消息
                            let response = room
                                .send(RoomMessageEventContent::text_plain(&content))
                                .await?;
                            event_id = Some(response.event_id);
                        }

                        // 重置计数器
                        chars_since_update = 0;
                        last_update = Instant::now();
                    }
                }
                Err(e) => {
                    warn!("流式响应错误: {}", e);
                    // 如果已经发送了一些内容，显示错误
                    let content = {
                        let s = state.lock().await;
                        s.content().to_string()
                    };

                    if !content.is_empty() {
                        let error_msg = format!("{}\n\n[错误: {}]", content, e);
                        if let Some(ref original_event_id) = event_id {
                            let metadata = ReplacementMetadata::new(original_event_id.clone(), None);
                            let msg_content = RoomMessageEventContent::text_plain(&error_msg)
                                .make_replacement(metadata);
                            room.send(msg_content).await?;
                        } else {
                            room.send(RoomMessageEventContent::text_plain(&error_msg))
                                .await?;
                        }
                    } else {
                        room.send(RoomMessageEventContent::text_plain(format!(
                            "AI 服务暂时不可用: {}",
                            e
                        )))
                        .await?;
                    }
                    return Ok(());
                }
            }
        }

        // 流结束，发送最终消息（如果有残留内容）
        let final_content = {
            let s = state.lock().await;
            s.content().to_string()
        };

        if !final_content.is_empty() {
            if let Some(ref original_event_id) = event_id {
                // 编辑为最终内容
                let metadata = ReplacementMetadata::new(original_event_id.clone(), None);
                let msg_content = RoomMessageEventContent::text_plain(&final_content)
                    .make_replacement(metadata);
                room.send(msg_content).await?;
            }
            // 如果 event_id 为 None，说明从未发送过消息（流立即结束且内容为空或被过滤）
            // 这种情况理论上不应该发生，因为上面已经在消费流时处理了
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