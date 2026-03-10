//! Test utilities for integration tests.
//!
//! This module provides shared utility functions for setting up and running tests.

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test logging.
///
/// This function sets up tracing for tests with a reasonable default configuration.
/// It uses `Once` to ensure initialization happens only once per test process.
///
/// # Example
///
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_something() {
///     init_test_logging();
///     // ... test code
/// }
/// ```
pub fn init_test_logging() {
    INIT.call_once(|| {
        // Use RUST_LOG environment variable if set, otherwise default to "info"
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_test_writer()
            .init();
    });
}

/// Create a temporary directory for testing.
///
/// Returns the path to the temporary directory.
/// The directory will be automatically cleaned up when the returned TempDir is dropped.
///
/// # Example
///
/// ```rust,ignore
/// #[tokio::test]
/// async fn test_with_temp_dir() {
///     let temp_dir = create_temp_dir();
///     let db_path = temp_dir.path().join("test.db");
///     // ... use db_path
/// }
/// ```
pub fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}
