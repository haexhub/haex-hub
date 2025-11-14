pub mod database;
pub mod filesystem;
pub mod helpers;
pub mod manager;
pub mod web;

// Re-export manager types
pub use manager::ExtensionWebviewManager;
