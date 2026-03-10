use aether_matrix::command::{CommandContext, CommandContextArgs, CommandHandler, Permission};
use aether_matrix::modules::persona::PersonaHandler;
use aether_matrix::store::{Database, PersonaStore};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[tokio::test]
    async fn test_persona_handler_name() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        let db = Database::new(&db_path).unwrap();
        let store = PersonaStore::new(db.conn().clone());
        let handler = PersonaHandler::new(store);
        
        assert_eq!(handler.name(), "persona");
    }
    
    #[tokio::test]
    async fn test_persona_handler_description() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        let db = Database::new(&db_path).unwrap();
        let store = PersonaStore::new(db.conn().clone());
        let handler = PersonaHandler::new(store);
        
        assert_eq!(handler.description(), "人设管理命令");
    }
    
    #[tokio::test]
    async fn test_persona_handler_usage() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        let db = Database::new(&db_path).unwrap();
        let store = PersonaStore::new(db.conn().clone());
        let handler = PersonaHandler::new(store);
        
        assert_eq!(handler.usage(), "persona <set|list|off|info|create|delete>");
    }
    
    #[tokio::test]
    async fn test_persona_handler_permission() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        let db = Database::new(&db_path).unwrap();
        let store = PersonaStore::new(db.conn().clone());
        let handler = PersonaHandler::new(store);
        
        assert_eq!(handler.permission(), Permission::Anyone);
    }
    
    #[tokio::test]
    async fn test_persona_handler_usage_contains_all_subcommands() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        let db = Database::new(&db_path).unwrap();
        let store = PersonaStore::new(db.conn().clone());
        let handler = PersonaHandler::new(store);
        
        let usage = handler.usage();
        assert!(usage.contains("set"));
        assert!(usage.contains("list"));
        assert!(usage.contains("off"));
        assert!(usage.contains("info"));
        assert!(usage.contains("create"));
        assert!(usage.contains("delete"));
    }
}

#[cfg(test)]
mod store_tests {
    use super::*;
    use aether_matrix::ui::{error, info_card, success, warning};
    
    struct TestContext {
        db: Database,
        _temp_dir: TempDir,
    }
    
    impl TestContext {
        fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
            let db = Database::new(&db_path).unwrap();
            
            let store = PersonaStore::new(db.conn().clone());
            store.init_builtin_personas().unwrap();
            
            Self {
                db,
                _temp_dir: temp_dir,
            }
        }
        
        fn create_handler(&self) -> PersonaHandler {
            let store = PersonaStore::new(self.db.conn().clone());
            PersonaHandler::new(store)
        }
    }
    
    #[tokio::test]
    async fn test_list_command_shows_all_builtin_personas() {
        let ctx = TestContext::new();
        let store = PersonaStore::new(ctx.db.conn().clone());
        let personas = store.get_all().unwrap();
        
        assert_eq!(personas.len(), 4);
        let expected_ids = vec!["sarcastic-dev", "cyber-zen", "wiki-chan", "neko-chan"];
        for (i, id) in expected_ids.iter().enumerate() {
            assert_eq!(personas[i].id, *id);
            assert!(personas[i].is_builtin);
        }
    }
    
    #[tokio::test]
    async fn test_info_command_returns_correct_details() {
        let ctx = TestContext::new();
        let store = PersonaStore::new(ctx.db.conn().clone());
        
        let persona = store.get_by_id("sarcastic-dev").unwrap().unwrap();
        assert_eq!(persona.name, "毒舌程序员");
        assert_eq!(persona.avatar_emoji, Some("💻".to_string()));
        assert!(persona.system_prompt.contains("20年经验"));
        assert!(persona.is_builtin);
    }
    
    #[tokio::test]
    async fn test_set_room_persona_works() {
        let ctx = TestContext::new();
        let store = PersonaStore::new(ctx.db.conn().clone());
        
        store.set_room_persona("!test:matrix.org", "sarcastic-dev", "@user:matrix.org").unwrap();
        
        let persona = store.get_room_persona("!test:matrix.org").unwrap().unwrap();
        assert_eq!(persona.id, "sarcastic-dev");
        assert_eq!(persona.name, "毒舌程序员");
    }
    
    #[tokio::test]
    async fn test_disable_room_persona_works() {
        let ctx = TestContext::new();
        let store = PersonaStore::new(ctx.db.conn().clone());
        
        store.set_room_persona("!test2:matrix.org", "cyber-zen", "@user:matrix.org").unwrap();
        let before = store.get_room_persona("!test2:matrix.org").unwrap();
        assert!(before.is_some());
        
        store.disable_room_persona("!test2:matrix.org").unwrap();
        let after = store.get_room_persona("!test2:matrix.org").unwrap();
        assert!(after.is_none());
    }
    
    #[tokio::test]
    async fn test_create_custom_persona_works() {
        let ctx = TestContext::new();
        let store = PersonaStore::new(ctx.db.conn().clone());
        
        let custom_persona = aether_matrix::store::Persona {
            id: "custom-test".to_string(),
            name: "Custom Test".to_string(),
            system_prompt: "Custom test prompt".to_string(),
            avatar_emoji: Some("🎯".to_string()),
            is_builtin: false,
            created_by: Some("@user:matrix.org".to_string()),
        };
        
        store.create_persona(&custom_persona).unwrap();
        
        let retrieved = store.get_by_id("custom-test").unwrap().unwrap();
        assert_eq!(retrieved.name, "Custom Test");
        assert_eq!(retrieved.avatar_emoji, Some("🎯".to_string()));
        assert!(!retrieved.is_builtin);
        assert_eq!(retrieved.created_by, Some("@user:matrix.org".to_string()));
    }
    
    #[tokio::test]
    async fn test_delete_custom_persona_works() {
        let ctx = TestContext::new();
        let store = PersonaStore::new(ctx.db.conn().clone());
        
        let custom_persona = aether_matrix::store::Persona {
            id: "to-delete".to_string(),
            name: "To Delete".to_string(),
            system_prompt: "Will be deleted".to_string(),
            avatar_emoji: None,
            is_builtin: false,
            created_by: None,
        };
        
        store.create_persona(&custom_persona).unwrap();
        let exists_before = store.get_by_id("to-delete").unwrap();
        assert!(exists_before.is_some());
        
        let deleted = store.delete_persona("to-delete").unwrap();
        assert!(deleted);
        
        let exists_after = store.get_by_id("to-delete").unwrap();
        assert!(exists_after.is_none());
    }
    
    #[tokio::test]
    async fn test_cannot_delete_builtin_persona() {
        let ctx = TestContext::new();
        let store = PersonaStore::new(ctx.db.conn().clone());
        
        let deleted = store.delete_persona("sarcastic-dev").unwrap();
        assert!(!deleted);
        
        let still_exists = store.get_by_id("sarcastic-dev").unwrap();
        assert!(still_exists.is_some());
    }
}

impl TestContext {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        let db = Database::new(&db_path).unwrap();
        
        let store = PersonaStore::new(db.conn().clone());
        store.init_builtin_personas().unwrap();
        
        let sent_messages = Arc::new(Mutex::new(Vec::new()));
        
        Self {
            db,
            _temp_dir: temp_dir,
            sent_messages,
        }
    }
    
    fn create_mock_room(&self, room_id: &str) -> MockRoom {
        MockRoom {
            room_id: room_id.to_string(),
            sent_messages: self.sent_messages.clone(),
        }
    }
    
    fn create_mock_client(&self) -> MockClient {
        MockClient {}
    }
    
    fn get_sent_messages(&self) -> Vec<String> {
        self.sent_messages.lock().unwrap().clone()
    }
    
    fn clear_sent_messages(&self) {
        self.sent_messages.lock().unwrap().clear();
    }
}

struct MockRoom {
    room_id: String,
    sent_messages: Arc<Mutex<Vec<String>>>,
}

impl MockRoom {
    fn room_id(&self) -> &RoomId {
        RoomId::try_from(self.room_id.as_str()).unwrap()
    }
    
    async fn send(&self, content: matrix_sdk::ruma::events::room::message::RoomMessageEventContent) -> Result<matrix_sdk::send_message_event::v3::Response, matrix_sdk::matrix_sdk_base::RoomNotJoined> {
        if let Some(html) = content.as_original().and_then(|e| e.formatted.as_ref()) {
            self.sent_messages.lock().unwrap().push(html.body.clone());
        } else if let Some(text) = content.as_original().map(|e| e.body.as_str()) {
            self.sent_messages.lock().unwrap().push(text.to_string());
        }
        Ok(matrix_sdk::send_message_event::v3::Response {
            event_id: matrix_sdk::ruma::event_id!("$test_event"),
        })
    }
}

struct MockClient {}

impl MockClient {
    fn account(&self) -> MockAccount {
        MockAccount {}
    }
}

struct MockAccount {}

impl MockAccount {
    async fn get_display_name(&self) -> Result<Option<String>, matrix_sdk::Error> {
        Ok(Some("Test Bot".to_string()))
    }
    
    async fn set_display_name(&self, name: Option<&str>) -> Result<(), matrix_sdk::Error> {
        Ok(())
    }
    
    async fn set_avatar_url(&self, url: Option<&str>) -> Result<(), matrix_sdk::Error> {
        Ok(())
    }
}

fn create_test_context() -> (PersonaHandler, TestContext) {
    let test_ctx = TestContext::new();
    let store = PersonaStore::new(test_ctx.db.conn().clone());
    let handler = PersonaHandler::new(store);
    (handler, test_ctx)
}

fn create_command_context(
    test_ctx: &TestContext,
    room_id: &str,
    sender: &str,
    args: Vec<&str>,
    bot_owners: &[String],
) -> CommandContext {
    let client = test_ctx.create_mock_client();
    let room = test_ctx.create_mock_room(room_id);
    let sender_id: OwnedUserId = UserId::parse(sender).unwrap().into();
    
    CommandContext::new(CommandContextArgs {
        client: &client,
        room: room.into(),
        sender: sender_id,
        args,
        bot_owners,
    })
}