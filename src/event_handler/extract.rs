//! 消息内容提取。

use matrix_sdk::Room;
use matrix_sdk::ruma::events::{
    AnySyncTimelineEvent,
    room::message::{OriginalSyncRoomMessageEvent, Relation},
};

use crate::traits::AiServiceTrait;

/// 消息提取方法。
impl<T: AiServiceTrait> super::EventHandler<T> {
    /// 从原始消息文本中提取纯净的消息内容。
    ///
    /// 移除命令前缀和 @提及，返回修剪后的文本。
    ///
    /// # Arguments
    ///
    /// * `text` - 原始消息文本
    ///
    /// # Returns
    ///
    /// 移除前缀和提及后的纯净文本
    pub fn extract_message(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 移除命令前缀（如 `!ai`）
        if result.starts_with(&self.command_prefix) {
            result = result[self.command_prefix.len()..].to_string();
        }

        // 移除 @提及（兼容旧客户端）
        result = result.replace(&self.bot_user_id.to_string(), "");

        result.trim().to_string()
    }

    /// 从消息中提取引用内容。
    ///
    /// 如果消息是回复消息（包含 `relates_to` 字段），则通过 Room API 获取
    /// 被引用的事件，并提取其消息文本。
    ///
    /// # Arguments
    ///
    /// * `room` - 消息所在的房间
    /// * `original` - 原始消息事件
    ///
    /// # Returns
    ///
    /// 成功时返回被引用消息的文本内容，失败或无引用时返回 `None`。
    pub async fn extract_reply_content(
        &self,
        room: &Room,
        original: &OriginalSyncRoomMessageEvent,
    ) -> Option<String> {
        // 检查是否是回复消息
        let in_reply_to = original.content.relates_to.as_ref().and_then(|r| match r {
            Relation::Reply { in_reply_to } => Some(&in_reply_to.event_id),
            _ => None,
        })?;

        // 通过 Room API 获取被引用的事件
        let event = room.load_or_fetch_event(in_reply_to, None).await.ok()?;

        // 反序列化事件
        let timeline_event = event.into_raw().deserialize().ok()?;

        // 提取消息文本
        match timeline_event {
            AnySyncTimelineEvent::MessageLike(msg) => {
                // 获取事件的原始内容（非删除状态）
                msg.original_content().and_then(|c| match c {
                    matrix_sdk::ruma::events::AnyMessageLikeEventContent::RoomMessage(m) => {
                        Some(m.msgtype.body().to_string())
                    }
                    _ => None,
                })
            }
            _ => None,
        }
    }
}
