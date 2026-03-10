//! Comprehensive tests for database connection and migration functionality.
//!
//! This test suite covers:
//! - Database connection establishment and configuration
//! - Table structure creation and migration
//! - Concurrent access safety
//! - Error handling (invalid paths, permission issues)
//! - Automatic parent directory creation
//! - Migration idempotency
//! - Multi-threaded concurrent access

use aether_matrix::store::Database;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;
use std::sync::Once;

// Test utilities copied from tests/common/test_utils.rs
static INIT: Once = Once::new();

fn init_test_logging() {
    INIT.call_once(|| {
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_test_writer()
            .init();
    });
}

fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

#[tokio::test]
async fn test_database_connection_establishment() {
    init_test_logging();
    let temp_dir = create_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();

    // Test that database connection can be established
    let db = Database::new(&db_path).expect("Failed to create database");
    
    // Verify the database file exists
    assert!(Path::new(&db_path).exists(), "Database file should exist after creation");
    
    // Test that we can get a connection
    let conn = db.conn().lock().expect("Failed to acquire database lock");
    drop(conn);
}

#[tokio::test]
async fn test_table_structure_creation() {
    init_test_logging();
    let temp_dir = create_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();

    let db = Database::new(&db_path).expect("Failed to create database");
    let conn = db.conn().lock().expect("Failed to acquire database lock");

    // Test that all expected tables exist
    let tables_to_check = vec![
        "personas",
        "room_persona", 
        "chat_history",
        "merit",
        "titles",
        "user_titles",
        "drops"
    ];

    for table_name in tables_to_check {
        let query = format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'", table_name);
        let result: i32 = conn.query_row(&query, [], |row| row.get(0))
            .expect(&format!("Failed to query existence of table {}", table_name));
        assert_eq!(result, 1, "Table {} should exist", table_name);
    }

    // Test that indexes exist
    let indexes_to_check = vec![
        "idx_chat_history_room_id",
        "idx_chat_history_created_at", 
        "idx_merit_room_total",
        "idx_merit_user_room",
        "idx_user_titles_user_room",
        "idx_drops_user_room"
    ];

    for index_name in indexes_to_check {
        let query = format!("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='{}'", index_name);
        let result: i32 = conn.query_row(&query, [], |row| row.get(0))
            .expect(&format!("Failed to query existence of index {}", index_name));
        assert_eq!(result, 1, "Index {} should exist", index_name);
    }
}

#[tokio::test]
async fn test_automatic_parent_directory_creation() {
    init_test_logging();
    let temp_dir = create_temp_dir();
    let nested_path = temp_dir.path().join("subdir1").join("subdir2").join("test.db");
    let db_path = nested_path.to_string_lossy().to_string();

    // Should create parent directories automatically
    let _db = Database::new(&db_path).expect("Failed to create database with nested path");
    
    // Verify both the database file and parent directories exist
    assert!(nested_path.exists(), "Database file should exist");
    assert!(nested_path.parent().unwrap().exists(), "Parent directory should exist");
    assert!(nested_path.parent().unwrap().parent().unwrap().exists(), "Grandparent directory should exist");
}

#[tokio::test]
async fn test_migration_idempotency() {
    init_test_logging();
    let temp_dir = create_temp_dir();
    let db_path = temp_dir.path().join("4a9dcd5c-3e8f-4b7a-8f1a-2c6e8f9a3b2d").to_string_lossy().to_string();

    // Create database (runs migrations)
    let db1 = Database::new(&db_path).expect("First database creation failed");
    
    // Create another instance pointing to same database (should run migrations again safely)
    let db2 = Database::new(&db_path).expect("Second database creation failed");
    
    // Both should work without errors
    let conn1 = db1.conn().lock().expect("Failed to acquire first connection");
    let conn2 = db2.conn().lock().expect("Failed to acquire second connection");
    
    // Verify tables still exist and are accessible
    let count: i32 = conn1.query_row(
        "SELECT COUNT(*) FROM personas", 
        [], 
        |row| row.get(0)
    ).expect("Failed to query personas table");
    
    drop(conn1);
    drop(conn2);
    
    // Count should be reasonable (at least the built-in personas)
    assert!(count >= 0, "Personas table should be accessible after idempotent migrations");
}

#[tokio::test]
async fn test_invalid_database_path_error_handling() {
    init_test_logging();
    
    // Test with a path that cannot be created (permission denied scenario)
    // On Unix-like systems, trying to write to /root/ typically fails for non-root users
    let invalid_path = "/root/protected/test.db";
    
    // This should fail gracefully with an error
    let result = Database::new(invalid_path);
    assert!(result.is_err(), "Database creation should fail for invalid path");
    
    // Check that the error is related to directory creation or file access
    // Be flexible with error messages as they can vary by system
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(error) => {
            let error_str = error.to_string();
            assert!(
                error_str.contains("denied") || 
                error_str.contains("Permission") ||
                error_str.contains("No such file") ||
                error_str.contains("cannot create") ||
                error_str.contains("Read-only") ||
                error_str.contains("access") ||
                error_str.contains("failed to open"),
                "Error should indicate permission, path, or access issue: {}",
                error_str
            );
        }
    }
}

#[tokio::test]
async fn test_concurrent_access_safety_single_thread() {
    init_test_logging();
    let temp_dir = create_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();

    let db = Database::new(&db_path).expect("Failed to create database");
    
    // Test multiple operations on the same connection
    {
        let conn = db.conn().lock().expect("Failed to acquire connection");
        
        // Insert some test data
        conn.execute(
            "INSERT INTO personas (id, name, system_prompt) VALUES (?, ?, ?)",
            ["test_id", "Test Persona", "Test prompt"]
        ).expect("Failed to insert test persona");
        
        // Query the data back
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM personas WHERE id = ?", 
            ["test_id"], 
            |row| row.get(0)
        ).expect("Failed to query test persona");
        
        assert_eq!(count, 1, "Should have inserted one test persona");
    }
    
    // Acquire connection again and verify data persists
    {
        let conn = db.conn().lock().expect("Failed to reacquire connection");
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM personas WHERE id = ?", 
            ["test_id"], 
            |row| row.get(0)
        ).expect("Failed to query test persona after reacquiring connection");
        
        assert_eq!(count, 1, "Test persona should persist across connection acquisitions");
    }
}

#[tokio::test]
async fn test_multi_threaded_concurrent_access() {
    init_test_logging();
    let temp_dir = create_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();

    let db = Arc::new(Database::new(&db_path).expect("Failed to create database"));
    
    // Spawn multiple threads that concurrently access the database
    let mut handles = vec![];
    
    for i in 0..5 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let conn = db_clone.conn().lock().expect("Failed to acquire connection in thread");
            
            // Each thread inserts a unique persona
            let persona_id = format!("persona_{}", i);
            let result = conn.execute(
                "INSERT INTO personas (id, name, system_prompt) VALUES (?, ?, ?)",
                [persona_id.as_str(), "Concurrent Test", "Concurrent test prompt"]
            );
            
            if let Err(e) = result {
                eprintln!("Thread {} failed to insert: {}", i, e);
                return false;
            }
            
            drop(conn);
            true
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    let results: Vec<bool> = handles.into_iter()
        .map(|h| h.join().unwrap_or(false))
        .collect();
    
    // All threads should have succeeded
    assert!(results.iter().all(|&r| r), "All concurrent threads should succeed");
    
    // Verify total count
    let conn = db.conn().lock().expect("Failed to acquire final connection");
    let total_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM personas WHERE name = ?", 
        ["Concurrent Test"], 
        |row| row.get(0)
    ).expect("Failed to count concurrent personas");
    
    assert_eq!(total_count, 5, "Should have 5 concurrently inserted personas");
}

#[tokio::test]
async fn test_foreign_keys_enabled() {
    init_test_logging();
    let temp_dir = create_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();

    let db = Database::new(&db_path).expect("Failed to create database");
    let conn = db.conn().lock().expect("Failed to acquire connection");
    
    // Check that foreign keys are enabled
    let foreign_keys_enabled: i32 = conn.query_row(
        "PRAGMA foreign_keys;", 
        [], 
        |row| row.get(0)
    ).expect("Failed to query foreign_keys pragma");
    
    assert_eq!(foreign_keys_enabled, 1, "Foreign keys should be enabled");
    
    // Test foreign key constraint by trying to insert invalid room_persona
    let result = conn.execute(
        "INSERT INTO room_persona (room_id, persona_id) VALUES (?, ?)",
        ["test_room", "non_existent_persona"]
    );
    
    // This should fail due to foreign key constraint
    assert!(result.is_err(), "Foreign key constraint should prevent invalid insertion");
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(error) => {
            assert!(
                error.to_string().contains("FOREIGN KEY") ||
                error.to_string().contains("constraint failed"),
                "Error should indicate foreign key constraint violation: {}",
                error
            );
        }
    }
}

// Test that cloning the database shares the same underlying connection
#[tokio::test]
async fn test_database_clone_shares_connection() {
    init_test_logging();
    let temp_dir = create_temp_dir();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();

    let db1 = Database::new(&db_path).expect("Failed to create database");
    let db2 = db1.clone();
    
    // Insert into db1
    {
        let conn = db1.conn().lock().expect("Failed to acquire db1 connection");
        conn.execute(
            "INSERT INTO personas (id, name, system_prompt) VALUES (?, ?, ?)",
            ["shared_test", "Shared Test", "Shared test prompt"]
        ).expect("Failed to insert via db1");
    }
    
    // Query from db2 - should see the same data
    {
        let conn = db2.conn().lock().expect("Failed to acquire db2 connection");
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM personas WHERE id = ?", 
            ["shared_test"], 
            |row| row.get(0)
        ).expect("Failed to query via db2");
        
        assert_eq!(count, 1, "Cloned database should share the same connection and data");
    }
}

// Test error handling when database path contains invalid characters
#[tokio::test]
async fn test_invalid_characters_in_path_error_handling() {
    init_test_logging();
    
    // On Unix systems, paths with null bytes are invalid
    // On Windows, certain characters like < > : " | ? * are invalid
    let invalid_path = "/tmp/invalid\0path.db";
    
    let result = Database::new(invalid_path);
    assert!(result.is_err(), "Database creation should fail for path with invalid characters");
    
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(error) => {
            let error_str = error.to_string();
            assert!(
                error_str.contains("nul") ||
                error_str.contains("null") ||
                error_str.contains("invalid") ||
                error_str.contains("contain"),
                "Error should indicate invalid characters in path: {}",
                error_str
            );
        }
    }
}