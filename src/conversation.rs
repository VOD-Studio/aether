use std::collections::HashMap;

use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
};

pub struct ConversationManager {
    conversations: HashMap<String, Vec<ChatCompletionRequestMessage>>,
    system_prompt: Option<String>,
    max_history: usize,
}

impl ConversationManager {
    pub fn new(system_prompt: Option<String>, max_history: usize) -> Self {
        Self {
            conversations: HashMap::new(),
            system_prompt,
            max_history,
        }
    }

    pub fn add_user_message(&mut self, session_id: &str, content: &str) {
        let history = self
            .conversations
            .entry(session_id.to_string())
            .or_default();

        history.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: content.to_string().into(),
                name: None,
            },
        ));

        // 限制历史长度
        if history.len() > self.max_history * 2 {
            *history = history.split_off(history.len() - self.max_history * 2);
        }
    }

    pub fn add_assistant_message(&mut self, session_id: &str, content: &str) {
        if let Some(history) = self.conversations.get_mut(session_id) {
            let msg = ChatCompletionRequestAssistantMessage {
                content: Some(
                    async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(
                        content.to_string(),
                    ),
                ),
                ..Default::default()
            };
            history.push(ChatCompletionRequestMessage::Assistant(msg));
        }
    }

    pub fn get_messages(&self, session_id: &str) -> Vec<ChatCompletionRequestMessage> {
        let mut messages = Vec::new();

        // 添加系统提示词
        if let Some(ref prompt) = self.system_prompt {
            messages.push(ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: prompt.clone().into(),
                    name: None,
                },
            ));
        }

        // 添加历史消息
        if let Some(history) = self.conversations.get(session_id) {
            messages.extend(history.clone());
        }

        messages
    }

    pub fn reset(&mut self, session_id: &str) {
        self.conversations.remove(session_id);
    }
}
