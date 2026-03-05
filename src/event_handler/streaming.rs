//! 流式响应处理。

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use futures_util::{Stream, StreamExt};
use matrix_sdk::Room;
use matrix_sdk::ruma::{
    OwnedEventId,
    events::room::message::{ReplacementMetadata, RoomMessageEventContent},
};
use tokio::sync::Mutex;
use tracing::warn;

use crate::traits::StreamingState;

/// 流式响应处理器。
///
/// 使用混合节流策略处理流式输出：
/// - 时间触发：超过配置的最小间隔
/// - 字符触发：累积超过配置的最小字符数
///
/// 支持两种场景：
/// - 普通流式响应：首次发送新消息，后续编辑
/// - 图片流式响应：基于已有的处理中消息编辑
#[derive(Clone)]
pub struct StreamingHandler {
    /// 流式更新的最小时间间隔
    pub min_interval: Duration,
    /// 流式更新的最小字符数阈值
    pub min_chars: usize,
}

impl StreamingHandler {
    /// 创建新的流式处理器。
    pub fn new(min_interval: Duration, min_chars: usize) -> Self {
        Self {
            min_interval,
            min_chars,
        }
    }

    /// 处理流式响应（普通场景）。
    ///
    /// 首次发送新消息，后续使用 Matrix 消息编辑 API 更新内容。
    ///
    /// # Arguments
    ///
    /// * `room` - 目标房间
    /// * `state` - 共享的流状态
    /// * `stream` - 文本片段流
    pub async fn handle(
        &self,
        room: &Room,
        state: Arc<Mutex<StreamingState>>,
        stream: impl Stream<Item = Result<String>> + Send + Unpin,
    ) -> Result<()> {
        self.handle_internal(room, state, stream, None).await
    }

    /// 处理流式响应（图片场景）。
    ///
    /// 基于已有的处理中消息进行编辑更新。
    ///
    /// # Arguments
    ///
    /// * `room` - 目标房间
    /// * `state` - 共享的流状态
    /// * `stream` - 文本片段流
    /// * `initial_event_id` - 初始消息事件 ID（处理中消息）
    pub async fn handle_with_initial_event(
        &self,
        room: &Room,
        state: Arc<Mutex<StreamingState>>,
        stream: impl Stream<Item = Result<String>> + Send + Unpin,
        initial_event_id: OwnedEventId,
    ) -> Result<()> {
        self.handle_internal(room, state, stream, Some(initial_event_id))
            .await
    }

    /// 内部流处理实现。
    async fn handle_internal(
        &self,
        room: &Room,
        state: Arc<Mutex<StreamingState>>,
        mut stream: impl Stream<Item = Result<String>> + Send + Unpin,
        initial_event_id: Option<OwnedEventId>,
    ) -> Result<()> {
        let mut event_id = initial_event_id;
        let mut chars_since_update: usize = 0;
        let mut last_update = Instant::now();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(delta) => {
                    chars_since_update += delta.chars().count();

                    // 混合节流策略：时间或字符数任一满足即更新
                    let time_elapsed = last_update.elapsed() >= self.min_interval;
                    let chars_accumulated = chars_since_update >= self.min_chars;

                    if time_elapsed || chars_accumulated {
                        let content = {
                            let s = state.lock().await;
                            s.content().to_string()
                        };

                        event_id = self.send_or_edit(room, &content, event_id).await?;

                        chars_since_update = 0;
                        last_update = Instant::now();
                    }
                }
                Err(e) => {
                    warn!("流式响应错误: {}", e);
                    self.handle_error(room, &state, event_id, &e.to_string())
                        .await?;
                    return Ok(());
                }
            }
        }

        // 流结束，确保发送最终内容
        let final_content = {
            let s = state.lock().await;
            s.content().to_string()
        };

        if !final_content.is_empty() {
            self.send_final(room, &final_content, event_id).await?;
        }

        Ok(())
    }

    /// 发送新消息或编辑已有消息。
    async fn send_or_edit(
        &self,
        room: &Room,
        content: &str,
        event_id: Option<OwnedEventId>,
    ) -> Result<Option<OwnedEventId>> {
        if let Some(original_event_id) = event_id {
            // 编辑已有消息
            let metadata = ReplacementMetadata::new(original_event_id.clone(), None);
            let msg_content =
                RoomMessageEventContent::text_plain(content).make_replacement(metadata);
            room.send(msg_content).await?;
            Ok(Some(original_event_id))
        } else {
            // 发送新消息
            let response = room
                .send(RoomMessageEventContent::text_plain(content))
                .await?;
            Ok(Some(response.event_id))
        }
    }

    /// 处理流错误。
    async fn handle_error(
        &self,
        room: &Room,
        state: &Arc<Mutex<StreamingState>>,
        event_id: Option<OwnedEventId>,
        error_msg: &str,
    ) -> Result<()> {
        let content = {
            let s = state.lock().await;
            s.content().to_string()
        };

        if !content.is_empty() {
            // 已有内容，追加错误信息
            let error_content = format!("{}\n\n[错误: {}]", content, error_msg);
            if let Some(original_event_id) = event_id {
                let metadata = ReplacementMetadata::new(original_event_id, None);
                let msg_content =
                    RoomMessageEventContent::text_plain(&error_content).make_replacement(metadata);
                room.send(msg_content).await?;
            } else {
                room.send(RoomMessageEventContent::text_plain(&error_content))
                    .await?;
            }
        } else {
            // 无内容，仅显示错误
            let error_text = format!("AI 服务暂时不可用: {}", error_msg);
            if let Some(original_event_id) = event_id {
                let metadata = ReplacementMetadata::new(original_event_id, None);
                let msg_content =
                    RoomMessageEventContent::text_plain(&error_text).make_replacement(metadata);
                room.send(msg_content).await?;
            } else {
                room.send(RoomMessageEventContent::text_plain(&error_text))
                    .await?;
            }
        }

        Ok(())
    }

    /// 发送最终内容。
    async fn send_final(
        &self,
        room: &Room,
        content: &str,
        event_id: Option<OwnedEventId>,
    ) -> Result<()> {
        if let Some(original_event_id) = event_id {
            let metadata = ReplacementMetadata::new(original_event_id, None);
            let msg_content =
                RoomMessageEventContent::text_plain(content).make_replacement(metadata);
            room.send(msg_content).await?;
        }
        Ok(())
    }
}
