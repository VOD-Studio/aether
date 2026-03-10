//! Meme 梗图模块。
//!
//! 提供 `!meme` 命令，使用 Tenor GIF API 搜索并发送梗图。

mod handlers;
mod tenor;

pub use handlers::MemeHandler;
pub use tenor::TenorClient;