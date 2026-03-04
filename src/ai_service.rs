use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use anyhow::Result;
use async_openai::{Client, config::OpenAIConfig, types::CreateChatCompletionRequestArgs};
use futures_util::{Stream, StreamExt};

use crate::config::Config;
use crate::conversation::ConversationManager;

#[derive(Clone)]
pub struct AiService {
    inner: Arc<AiServiceInner>,
}

struct AiServiceInner {
    client: Client<OpenAIConfig>,
    model: String,
    conversation: Arc<RwLock<ConversationManager>>,
}

/// 流式响应的状态追踪
pub struct StreamingState {
    pub accumulated: String,
}

impl StreamingState {
    pub fn new() -> Self {
        Self {
            accumulated: String::new(),
        }
    }

    /// 追加新内容
    pub fn append(&mut self, delta: &str) {
        self.accumulated.push_str(delta);
    }

    /// 获取当前累积的完整内容
    pub fn content(&self) -> &str {
        &self.accumulated
    }
}

impl AiService {
    pub fn new(config: &Config) -> Self {
        let openai_config = OpenAIConfig::new()
            .with_api_key(&config.openai_api_key)
            .with_api_base(&config.openai_base_url);

        Self {
            inner: Arc::new(AiServiceInner {
                client: Client::with_config(openai_config),
                model: config.openai_model.clone(),
                conversation: Arc::new(RwLock::new(ConversationManager::new(
                    config.system_prompt.clone(),
                    config.max_history,
                ))),
            }),
        }
    }

    pub async fn chat(&self, session_id: &str, prompt: &str) -> Result<String> {
        // 添加用户消息到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_user_message(session_id, prompt);
        }

        // 获取完整消息历史
        let messages = {
            let conv = self.inner.conversation.read().await;
            conv.get_messages(session_id)
        };

        // 调用 API
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.inner.model)
            .messages(messages)
            .build()?;

        let response = self.inner.client.chat().create(request).await?;

        // 提取回复内容
        let content = response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default();

        // 添加助手回复到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_assistant_message(session_id, &content);
        }

        Ok(content)
    }

    pub async fn reset_conversation(&self, session_id: &str) {
        let mut conv = self.inner.conversation.write().await;
        conv.reset(session_id);
    }

    /// 流式聊天
    /// 返回一个共享状态用于追踪累积内容，以及一个 Stream 用于消费
    pub async fn chat_stream(
        &self,
        session_id: &str,
        prompt: &str,
    ) -> Result<(
        Arc<Mutex<StreamingState>>,
        Pin<Box<dyn Stream<Item = Result<String>> + Send>>,
    )> {
        // 添加用户消息到历史
        {
            let mut conv = self.inner.conversation.write().await;
            conv.add_user_message(session_id, prompt);
        }

        // 获取完整消息历史
        let messages = {
            let conv = self.inner.conversation.read().await;
            conv.get_messages(session_id)
        };

        // 创建流式请求
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.inner.model)
            .messages(messages)
            .stream(true)
            .build()?;

        let stream = self.inner.client.chat().create_stream(request).await?;

        // 创建共享状态
        let state = Arc::new(Mutex::new(StreamingState::new()));
        let state_clone = state.clone();
        let conversation = self.inner.conversation.clone();
        let session_id_owned = session_id.to_string();

        // 包装 stream，在消费时更新状态
        let wrapped_stream = stream.filter_map(move |chunk_result| {
            let state = state_clone.clone();
            let conversation = conversation.clone();
            let session_id_owned = session_id_owned.clone();
            async move {
                match chunk_result {
                    Ok(chunk) => {
                        // 提取 delta 内容
                        if let Some(delta) =
                            chunk.choices.first().and_then(|c| c.delta.content.clone())
                        {
                            // 更新共享状态
                            {
                                let mut s = state.lock().await;
                                s.append(&delta);
                            }
                            Some(Ok(delta))
                        } else {
                            // 检查是否是结束标记
                            if chunk
                                .choices
                                .first()
                                .is_some_and(|c| c.finish_reason.is_some())
                            {
                                // 保存完整回复到会话历史
                                let s = state.lock().await;
                                let content = s.content().to_string();
                                drop(s); // 释放锁
                                let mut conv = conversation.write().await;
                                conv.add_assistant_message(&session_id_owned, &content);
                            }
                            None
                        }
                    }
                    Err(e) => Some(Err(anyhow::anyhow!("流式响应错误: {}", e))),
                }
            }
        });

        Ok((state, Box::pin(wrapped_stream)))
    }
}
