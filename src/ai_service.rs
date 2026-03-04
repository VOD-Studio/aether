use std::sync::Arc;
use tokio::sync::RwLock;

use anyhow::Result;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::CreateChatCompletionRequestArgs,
};

use crate::config::Config;
use crate::conversation::ConversationManager;

#[derive(Clone)]
pub struct AiService {
    inner: Arc<AiServiceInner>,
}

struct AiServiceInner {
    client: Client<OpenAIConfig>,
    model: String,
    conversation: RwLock<ConversationManager>,
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
                conversation: RwLock::new(ConversationManager::new(
                    config.system_prompt.clone(),
                    config.max_history,
                )),
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
}