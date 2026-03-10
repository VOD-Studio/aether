use aether_matrix::modules::muyu::{
    BagHandler, MeritHandler, MuyuHandler, RankHandler, TitleHandler, MuyuLogic, MuyuStore,
    ConditionKind, DropItem, HitResult, MeritRecord, Rarity, Title,
};
use aether_matrix::command::{CommandContext, CommandHandler, Permission};
use aether_matrix::ui::{error, info_card, leaderboard, success, warning};
use std::time::Duration;
use tempfile::TempDir;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[tokio::test]
    async fn test_muyu_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MuyuHandler::new(store);
        assert_eq!(handler.name(), "木鱼");
    }

    #[tokio::test]
    async fn test_muyu_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MuyuHandler::new(store);
        assert_eq!(handler.description(), "敲一次木鱼，积累功德");
    }

    #[tokio::test]
    async fn test_muyu_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MuyuHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_merit_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MeritHandler::new(store);
        assert_eq!(handler.name(), "功德");
    }

    #[tokio::test]
    async fn test_merit_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MeritHandler::new(store);
        assert_eq!(handler.description(), "查看当前功德值");
    }

    #[tokio::test]
    async fn test_merit_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MeritHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_rank_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = RankHandler::new(store);
        assert_eq!(handler.name(), "功德榜");
    }

    #[tokio::test]
    async fn test_rank_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = RankHandler::new(store);
        assert_eq!(handler.description(), "查看房间功德排行榜");
    }

    #[tokio::test]
    async fn test_rank_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = RankHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_title_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = TitleHandler::new(store);
        assert_eq!(handler.name(), "称号");
    }

    #[tokio::test]
    async fn test_title_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = TitleHandler::new(store);
        assert_eq!(handler.description(), "查看或装备称号");
    }

    #[tokio::test]
    async fn test_title_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = TitleHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_bag_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = BagHandler::new(store);
        assert_eq!(handler.name(), "背包");
    }

    #[tokio::test]
    async fn test_bag_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = BagHandler::new(store);
        assert_eq!(handler.description(), "查看掉落物品背包");
    }

    #[tokio::test]
    async fn test_bag_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = BagHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }
}

#[cfg(test)]
mod logic_tests {
    use super::*;

    #[tokio::test]
    async fn test_normal_hit_earns_merit() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let logic = MuyuLogic::new(store.clone());
        
        let user_id = "@test:example.com";
        let room_id = "!room:example.com";
        
        // First hit should earn 1 merit
        let result = logic.hit(user_id, room_id).unwrap();
        assert_eq!(result.merit_gained, 1);
        assert_eq!(result.merit_total, 1);
        assert_eq!(result.new_combo, 1);
        assert!(!result.is_critical);
        assert_eq!(result.combo_multiplier, 1.0);
    }

    #[tokio::test]
    async fn test_cooldown_prevents_rapid_hits() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let logic = MuyuLogic::new(store.clone());
        
        let user_id = "@test:example.com";
        let room_id = "!room:example.com";
        
        // First hit
        let result1 = logic.hit(user_id, room_id).unwrap();
        assert_eq!(result1.merit_gained, 1);
        
        // Immediate second hit should be blocked by cooldown
        let result2 = logic.hit(user_id, room_id).unwrap();
        assert_eq!(result2.merit_gained, 0);
        
        // Wait for cooldown to expire (500ms + buffer)
        tokio::time::sleep(Duration::from_millis(600)).await;
        
        // Third hit should work again
        let result3 = logic.hit(user_id, room_id).unwrap();
        assert_eq!(result3.merit_gained, 1);
    }

    #[tokio::test]
    async fn test_consecutive_hits_build_combo() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let logic = MuyuLogic::new(store.clone());
        
        let user_id = "@test:example.com";
        let room_id = "!room:example.com";
        
        // First hit
        let result1 = logic.hit(user_id, room_id).unwrap();
        assert_eq!(result1.new_combo, 1);
        
        // Wait briefly and hit again (within combo window)
        tokio::time::sleep(Duration::from_millis(100)).await;
        let result2 = logic.hit(user_id, room_id).unwrap();
        assert_eq!(result2.new_combo, 2);
        assert_eq!(result2.combo_multiplier, 1.0); // Still under 5
    }
        
        // One more hit to reach 6 combo
        tokio::time::sleep(Duration::from_millis(100)).await;
        let result6 = logic.hit(user_id, room_id).unwrap();
        assert_eq!(result6.new_combo, 6);
        assert_eq!(result6.combo_multiplier, 1.5); // Now over 5
    }
}

#[cfg(test)]
mod store_tests {
    use super::*;

    #[tokio::test]
    async fn test_merit_accumulation_and_storage() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        
        let user_id = "@test:example.com";
        let room_id = "!room:example.com";
        
        // Initial state should be None
        let initial = store.get_merit(user_id, room_id).unwrap();
        assert!(initial.is_none());
        
        // Add some merit (normal hit = 1 merit)
        let record = store
            .update_merit(user_id, room_id, 1, 1, false)
            .unwrap();
        assert_eq!(record.merit_total, 1);
        assert_eq!(record.merit_today, 1); // First hit sets merit_today to 1
        assert_eq!(record.hits_today, 1);
        assert_eq!(record.combo, 1);
        assert_eq!(record.max_combo, 1);
        
        // Add more merit (another normal hit = 1 more merit)
        let record2 = store
            .update_merit(user_id, room_id, 1, 2, false)
            .unwrap();
        assert_eq!(record2.merit_total, 2);
        assert_eq!(record2.merit_today, 2); // Should accumulate
        assert_eq!(record2.hits_today, 2);
        assert_eq!(record2.combo, 2);
        assert_eq!(record2.max_combo, 2);
    }

    #[tokio::test]
    async fn test_leaderboard_functionality() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        
        let room_id = "!room:example.com";
        
        // Add merit for multiple users
        store
            .update_merit("@user1:example.com", room_id, 100, 1, false)
            .unwrap();
        store
            .update_merit("@user2:example.com", room_id, 50, 1, false)
            .unwrap();
        store
            .update_merit("@user3:example.com", room_id, 200, 1, false)
            .unwrap();
        
        // Get leaderboard
        let rankings = store.get_leaderboard(room_id, 10).unwrap();
        assert_eq!(rankings.len(), 3);
        
        // Should be sorted by merit_total descending
        assert_eq!(rankings[0].user_id, "@user3:example.com");
        assert_eq!(rankings[0].merit_total, 200);
        assert_eq!(rankings[1].user_id, "@user1:example.com");
        assert_eq!(rankings[1].merit_total, 100);
        assert_eq!(rankings[2].user_id, "@user2:example.com");
        assert_eq!(rankings[2].merit_total, 50);
    }

    #[tokio::test]
    async fn test_title_unlocking_based_on_conditions() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        
        let user_id = "@test:example.com";
        let room_id = "!room:example.com";
        
        // Create a mock merit record that should unlock titles
        let mut record = MeritRecord::default();
        record.user_id = user_id.to_string();
        record.room_id = room_id.to_string();
        record.merit_total = 150; // Should unlock "虔诚信徒" (100) and "初心者" (1)
        record.hits_today = 60; // Should unlock "木鱼狂魔" (50)
        record.max_combo = 25; // Should unlock "连击大师" (20)
        record.critical_count = 15; // Should unlock "会心一击者" (10)
        
        let unlocked = store.check_and_unlock_titles(&record).unwrap();
        
        // Should have unlocked multiple titles
        assert!(!unlocked.is_empty());
        
        // Check that specific titles were unlocked
        let unlocked_names: Vec<String> = unlocked.iter().map(|t| t.name.clone()).collect();
        assert!(unlocked_names.contains(&"初心者".to_string()));
        assert!(unlocked_names.contains(&"虔诚信徒".to_string()));
        assert!(unlocked_names.contains(&"木鱼狂魔".to_string()));
        assert!(unlocked_names.contains(&"连击大师".to_string()));
        assert!(unlocked_names.contains(&"会心一击者".to_string()));
    }

    #[tokio::test]
    async fn test_drop_item_functionality() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        
        let user_id = "@test:example.com";
        let room_id = "!room:example.com";
        let item_name = "佛珠";
        let icon = "📿";
        let rarity = Rarity::Rare;
        
        // Add a drop item
        let drop_item = store
            .add_drop(user_id, room_id, item_name, icon, &rarity)
            .unwrap();
        
        assert_eq!(drop_item.item_name, item_name);
        assert_eq!(drop_item.item_icon, Some(icon.to_string()));
        assert_eq!(drop_item.rarity, rarity);
        assert_eq!(drop_item.user_id, user_id);
        assert_eq!(drop_item.room_id, room_id);
        
        // Retrieve drops
        let drops = store.get_drops(user_id, room_id).unwrap();
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].item_name, item_name);
        assert_eq!(drops[0].rarity, rarity);
    }
}

#[cfg(test)]
mod ui_tests {
    use super::*;

    #[test]
    fn test_ui_message_formats() {
        // Test success message
        let msg = success("Test success");
        assert!(msg.contains("Test success"));
        assert!(msg.contains("✓"));

        // Test error message
        let msg = error("Test error");
        assert!(msg.contains("Test error"));
        assert!(msg.contains("✕"));

        // Test info card message
        let items = vec![("功德", "100")];
        let msg = info_card("功德信息", &items);
        assert!(msg.contains("功德信息"));
        assert!(msg.contains("功德"));
        assert!(msg.contains("100"));

        // Test warning message
        let msg = warning("敲得太快了");
        assert!(msg.contains("敲得太快了"));
        assert!(msg.contains("⚠"));

        // Test leaderboard format
        let headers = ["排名", "用户", "功德"];
        let rows = vec![
            vec!["1", "user1", "100"],
            vec!["2", "user2", "50"],
        ];
        let msg = leaderboard("功德排行榜", &headers, &rows);
        assert!(msg.contains("功德排行榜"));
        assert!(msg.contains("user1"));
        assert!(msg.contains("user2"));
    }
}

// Helper functions
async fn create_test_store(db_path: &std::path::Path) -> MuyuStore {
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    
    // Initialize database with schema
    let conn = Connection::open(db_path).unwrap();
    let conn = Arc::new(Mutex::new(conn));
    
    // Apply migrations manually for testing
    let conn_lock = conn.lock().unwrap();
    conn_lock.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS merit (
            user_id TEXT NOT NULL,
            room_id TEXT NOT NULL,
            merit_total INTEGER DEFAULT 0,
            merit_today INTEGER DEFAULT 0,
            hits_today INTEGER DEFAULT 0,
            last_hit DATETIME,
            combo INTEGER DEFAULT 0,
            max_combo INTEGER DEFAULT 0,
            critical_count INTEGER DEFAULT 0,
            consecutive_days INTEGER DEFAULT 0,
            last_hit_date DATE,
            PRIMARY KEY (user_id, room_id)
        );

        CREATE TABLE IF NOT EXISTS titles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            description TEXT,
            icon TEXT,
            condition_kind TEXT NOT NULL CHECK(condition_kind IN ('total_merit', 'daily_hits', 'combo', 'critical_hits', 'consecutive_days')),
            condition_value INTEGER NOT NULL,
            rarity TEXT NOT NULL CHECK(rarity IN ('common', 'rare', 'epic', 'legendary'))
        );

        CREATE TABLE IF NOT EXISTS user_titles (
            user_id TEXT NOT NULL,
            room_id TEXT NOT NULL,
            title_id INTEGER NOT NULL REFERENCES titles(id) ON DELETE CASCADE,
            obtained_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            equipped INTEGER DEFAULT 0,
            PRIMARY KEY (user_id, room_id, title_id)
        );

        CREATE TABLE IF NOT EXISTS drops (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            room_id TEXT NOT NULL,
            item_name TEXT NOT NULL,
            item_icon TEXT,
            rarity TEXT NOT NULL CHECK(rarity IN ('common', 'rare', 'epic', 'legendary')),
            obtained_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Insert test titles
        INSERT OR IGNORE INTO titles (name, description, icon, condition_kind, condition_value, rarity) VALUES
            ('初心者', '首次敲击木鱼', '🌱', 'total_merit', 1, 'common'),
            ('虔诚信徒', '累计 100 功德', '🙏', 'total_merit', 100, 'common'),
            ('木鱼狂魔', '单日敲击 50 次', '🥁', 'daily_hits', 50, 'rare'),
            ('连击大师', '达成 20 连击', '💥', 'combo', 20, 'rare'),
            ('会心一击者', '触发 10 次会心', '⚡', 'critical_hits', 10, 'epic');
        ",
    ).unwrap();
    drop(conn_lock);
    
    MuyuStore::new(conn)
}

fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}