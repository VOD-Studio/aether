use std::{pin::Pin, sync::Arc};
use anyhow::Result;
use futures_util::Stream;
use tokio::sync::Mutex;

use crate::ai_service::StreamingState;

/// AI 服务的 trait 抽象，用于支持 mock 测试
pub trait AiServiceTrait: Clone + Send + Sync + 'static {
    /// 普通聊天
    async fn chat(&self, session_id: &str, prompt: &str) -> Result<String>;

    /// 重置会话
    async fn reset_conversation(&self, session_id: &str);

    /// 流式聊天
    /// 返回共享状态用于追踪累积内容，以及 Stream 用于消费
    async fn chat_stream(
        &self,
        session_id: &str,
        prompt: &str,
    ) -> Result<(
        Arc<Mutex<StreamingState>>,
        Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
    )>;
}