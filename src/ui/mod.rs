//! UI 模块
//!
//! 提供现代卡片风格的消息模板和统一的消息发送函数。

pub mod templates;

pub use templates::{error, help_menu, info, info_card, leaderboard, send_html, success, warning};
