use std::collections::HashMap;

use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestUserMessage,
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
                content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                    content.to_string(),
                )),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager_is_empty() {
        let manager = ConversationManager::new(None, 10);
        let messages = manager.get_messages("test-session");
        assert!(messages.is_empty());
    }

    #[test]
    fn test_add_user_message() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Hello");

        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            ChatCompletionRequestMessage::User(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestUserMessageContent::Text(text) => {
                    assert_eq!(text, "Hello");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected user message"),
        }
    }

    #[test]
    fn test_add_assistant_message() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Hello");
        manager.add_assistant_message("session-1", "Hi there!");

        let messages = manager.get_messages("session-1");
        assert_eq!(messages.len(), 2);

        match &messages[1] {
            ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                    assert_eq!(text, "Hi there!");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected assistant message"),
        }
    }

    #[test]
    fn test_history_limit() {
        let mut manager = ConversationManager::new(None, 2); // max_history = 2

        // 添加 5 条消息，第 5 条会触发截断（add_user_message 时检查）
        // u1, a1, u2, a2, u3
        manager.add_user_message("s1", "u1");
        manager.add_assistant_message("s1", "a1");
        manager.add_user_message("s1", "u2");
        manager.add_assistant_message("s1", "a2");
        manager.add_user_message("s1", "u3"); // 触发截断到 4 条：a1, u2, a2, u3

        let messages = manager.get_messages("s1");
        // max_history * 2 = 4 条消息被保留
        assert_eq!(messages.len(), 4);

        // 最新的消息应该被保留（第一条是 a1）
        match &messages[0] {
            ChatCompletionRequestMessage::Assistant(msg) => match &msg.content {
                Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                    assert_eq!(text, "a1");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected assistant message a1"),
        }
    }

    #[test]
    fn test_reset_conversation() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Hello");
        manager.add_user_message("session-2", "World");

        manager.reset("session-1");

        assert_eq!(manager.get_messages("session-1").len(), 0);
        assert_eq!(manager.get_messages("session-2").len(), 1);
    }

    #[test]
    fn test_system_prompt() {
        let mut manager = ConversationManager::new(Some("You are helpful.".to_string()), 10);
        manager.add_user_message("s1", "Hello");

        let messages = manager.get_messages("s1");
        assert_eq!(messages.len(), 2);

        match &messages[0] {
            ChatCompletionRequestMessage::System(msg) => match &msg.content {
                async_openai::types::chat::ChatCompletionRequestSystemMessageContent::Text(
                    text,
                ) => {
                    assert_eq!(text, "You are helpful.");
                }
                _ => panic!("Expected text content"),
            },
            _ => panic!("Expected system message"),
        }
    }

    #[test]
    fn test_multiple_sessions() {
        let mut manager = ConversationManager::new(None, 10);
        manager.add_user_message("session-1", "Hello from s1");
        manager.add_user_message("session-2", "Hello from s2");
        manager.add_assistant_message("session-1", "Response to s1");

        let s1_messages = manager.get_messages("session-1");
        let s2_messages = manager.get_messages("session-2");

        assert_eq!(s1_messages.len(), 2);
        assert_eq!(s2_messages.len(), 1);
    }
}
