use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    // Matrix 配置
    pub matrix_homeserver: String,
    pub matrix_username: String,
    pub matrix_password: String,
    pub device_display_name: String,

    // AI API 配置
    pub openai_api_key: String,
    pub openai_base_url: String,
    pub openai_model: String,
    pub system_prompt: Option<String>,

    // 机器人配置
    pub command_prefix: String,
    pub max_history: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // 加载 .env 文件（如果存在）
        match dotenvy::dotenv() {
            Ok(path) => {
                tracing::debug!(".env 文件已加载: {:?}", path);
            }
            Err(e) => {
                tracing::warn!(
                    ".env 文件加载失败: {}。\n\
                     请检查：\n\
                     1. 文件是否存在于当前目录\n\
                     2. 文件格式是否正确（包含空格的值需要用引号包裹，如: NAME=\"value with spaces\"）\n\
                     将从环境变量读取配置。",
                    e
                );
            }
        }

        Ok(Self {
            matrix_homeserver: std::env::var("MATRIX_HOMESERVER").map_err(|_| {
                anyhow::anyhow!(
                    "MATRIX_HOMESERVER 未设置。\n\
                         请在 .env 文件或环境变量中配置 Matrix 服务器地址。\n\
                         示例: MATRIX_HOMESERVER=https://matrix.org"
                )
            })?,
            matrix_username: std::env::var("MATRIX_USERNAME").map_err(|_| {
                anyhow::anyhow!(
                    "MATRIX_USERNAME 未设置。\n\
                         请在 .env 文件或环境变量中配置 Matrix 用户名。\n\
                         示例: MATRIX_USERNAME=your_username"
                )
            })?,
            matrix_password: std::env::var("MATRIX_PASSWORD").map_err(|_| {
                anyhow::anyhow!(
                    "MATRIX_PASSWORD 未设置。\n\
                         请在 .env 文件或环境变量中配置 Matrix 密码。"
                )
            })?,
            device_display_name: std::env::var("DEVICE_DISPLAY_NAME")
                .unwrap_or_else(|_| "AI Bot".to_string()),
            openai_api_key: std::env::var("OPENAI_API_KEY").map_err(|_| {
                anyhow::anyhow!(
                    "OPENAI_API_KEY 未设置。\n\
                         请在 .env 文件或环境变量中配置 API 密钥。\n\
                         示例: OPENAI_API_KEY=sk-..."
                )
            })?,
            openai_base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            openai_model: std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            system_prompt: std::env::var("SYSTEM_PROMPT").ok(),
            command_prefix: std::env::var("BOT_COMMAND_PREFIX")
                .unwrap_or_else(|_| "!ai".to_string()),
            max_history: std::env::var("MAX_HISTORY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        })
    }
}
