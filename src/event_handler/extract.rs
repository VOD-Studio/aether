//! 消息内容提取。

use matrix_sdk::Room;
use matrix_sdk::ruma::events::{
    AnySyncTimelineEvent,
    room::message::{
        ImageMessageEventContent, MessageType, OriginalSyncRoomMessageEvent, Relation,
    },
};

use crate::media::download_image_as_base64;
use crate::traits::AiServiceTrait;

/// 引用消息的上下文内容。
#[derive(Debug, Clone)]
pub struct ReplyContext {
    /// 文本内容（如果有）
    pub text: Option<String>,
    /// 图片 base64 data URL（如果有）
    pub image_data_url: Option<String>,
}

impl ReplyContext {
    /// 是否包含图片
    pub fn has_image(&self) -> bool {
        self.image_data_url.is_some()
    }

    /// 获取完整文本（用于显示）
    pub fn display_text(&self) -> String {
        match (&self.text, &self.image_data_url) {
            (Some(t), None) => t.clone(),
            (None, Some(_)) => "[图片]".to_string(),
            (Some(t), Some(_)) => format!("[图片] {}", t),
            (None, None) => String::new(),
        }
    }
}

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
    /// 被引用的事件，并提取其消息内容（文本或图片）。
    ///
    /// # Arguments
    ///
    /// * `room` - 消息所在的房间
    /// * `original` - 原始消息事件
    ///
    /// # Returns
    ///
    /// 成功时返回被引用消息的上下文，失败或无引用时返回 `None`。
    pub async fn extract_reply_content(
        &self,
        room: &Room,
        original: &OriginalSyncRoomMessageEvent,
    ) -> Option<ReplyContext> {
        // 检查是否是回复消息
        let in_reply_to = original.content.relates_to.as_ref().and_then(|r| match r {
            Relation::Reply { in_reply_to } => Some(&in_reply_to.event_id),
            _ => None,
        })?;

        // 通过 Room API 获取被引用的事件
        let event = room.load_or_fetch_event(in_reply_to, None).await.ok()?;

        // 反序列化事件
        let timeline_event = event.into_raw().deserialize().ok()?;

        // 提取消息内容
        let msg = match timeline_event {
            AnySyncTimelineEvent::MessageLike(msg) => msg,
            _ => return None,
        };

        // 获取事件的原始内容（非删除状态）
        let content = msg.original_content()?;
        let room_msg = match content {
            matrix_sdk::ruma::events::AnyMessageLikeEventContent::RoomMessage(m) => m,
            _ => return None,
        };

        // 根据消息类型构建 ReplyContext
        match &room_msg.msgtype {
            MessageType::Text(text) => Some(ReplyContext {
                text: Some(text.body.clone()),
                image_data_url: None,
            }),
            MessageType::Image(img) => {
                // 下载图片并转 base64
                let data_url = self.extract_image_from_message(img).await;
                Some(ReplyContext {
                    text: if img.body.is_empty() {
                        None
                    } else {
                        Some(img.body.clone())
                    },
                    image_data_url: data_url,
                })
            }
            _ => None,
        }
    }

    /// 从图片消息中提取图片数据。
    ///
    /// 下载图片并转换为 base64 data URL 格式。
    ///
    /// # Arguments
    ///
    /// * `image_msg` - 图片消息内容
    ///
    /// # Returns
    ///
    /// 成功时返回 base64 data URL，失败时返回 `None`。
    async fn extract_image_from_message(
        &self,
        image_msg: &ImageMessageEventContent,
    ) -> Option<String> {
        let mxc_uri = match &image_msg.source {
            matrix_sdk::ruma::events::room::MediaSource::Plain(url) => url,
            matrix_sdk::ruma::events::room::MediaSource::Encrypted(_) => return None,
        };

        let media_type = image_msg
            .info
            .as_ref()
            .and_then(|i| i.mimetype.as_deref())
            .unwrap_or("image/png");

        download_image_as_base64(
            &self.client,
            mxc_uri,
            Some(media_type),
            self.vision_max_image_size,
        )
        .await
        .ok()
    }
}
