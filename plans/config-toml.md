# 配置系统重构方案：支持 TOML 配置文件

## 目标

实现配置系统支持 TOML 配置文件，同时保持环境变量配置兼容。

## 设计原则

1. **向后兼容**: 现有环境变量配置无需修改
2. **优先级**: 环境变量 > TOML 配置 > 代码默认值
3. **扁平访问**: 使用 `#[serde(flatten)]` 保持代码中扁平访问方式
4. **统一格式**: MCP 外部服务器使用 TOML 列表，保留 JSON 环境变量（deprecated）

---

## 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `Cargo.toml` | 修改 | 添加 `toml`, `clap` 依赖 |
| `src/config.rs` | 重构 | 嵌套结构体 + TOML 加载 + 环境变量覆盖 |
| `src/mcp/config.rs` | 修改 | 添加 `apply_env_overrides` 方法 |
| `src/main.rs` | 修改 | 添加 clap 命令行参数 `-c` |
| `config.example.toml` | 新建 | 示例配置文件 |

---

## 详细设计

### 1. 依赖添加 (`Cargo.toml`)

```toml
toml = "0.8"
clap = { version = "4", features = ["derive"] }
```

---

### 2. 结构体重构 (`src/config.rs`)

#### 2.1 结构设计

使用 `#[serde(flatten)]` 实现 TOML 嵌套、代码扁平访问：

```
Config (代码访问: config.matrix_homeserver)
├── [matrix] (TOML 嵌套)
│   ├── homeserver: String (必需)
│   ├── username: String (必需)
│   ├── password: String (必需)
│   ├── device_id: Option<String>
│   ├── device_display_name: String
│   └── store_path: String
├── [openai] (TOML 嵌套)
│   ├── api_key: String (必需)
│   ├── base_url: String
│   ├── model: String
│   └── system_prompt: Option<String>
├── [bot] (TOML 嵌套)
│   ├── command_prefix: String
│   ├── max_history: usize
│   ├── owners: Vec<String>
│   └── db_path: String
├── [streaming] (TOML 嵌套)
│   ├── enabled: bool
│   ├── min_interval_ms: u64
│   └── min_chars: usize
├── [vision] (TOML 嵌套)
│   ├── enabled: bool
│   ├── model: Option<String>
│   └── max_image_size: u32
├── [log] (TOML 嵌套)
│   └── level: String
├── proxy: Option<String>
└── mcp: McpConfig
```

#### 2.2 结构体定义

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub matrix: MatrixConfig,
    
    #[serde(flatten)]
    pub openai: OpenAiConfig,
    
    #[serde(flatten)]
    pub bot: BotConfig,
    
    #[serde(flatten)]
    pub streaming: StreamingConfig,
    
    #[serde(flatten)]
    pub vision: VisionConfig,
    
    #[serde(flatten)]
    pub log: LogConfig,
    
    pub proxy: Option<String>,
    
    #[serde(default)]
    pub mcp: McpConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatrixConfig {
    #[serde(default)]
    pub matrix_homeserver: String,
    
    #[serde(default)]
    pub matrix_username: String,
    
    #[serde(default)]
    pub matrix_password: String,
    
    pub matrix_device_id: Option<String>,
    
    #[serde(default = "default_device_display_name")]
    pub device_display_name: String,
    
    #[serde(default = "default_store_path")]
    pub store_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiConfig {
    #[serde(default)]
    pub openai_api_key: String,
    
    #[serde(default = "default_openai_base_url")]
    pub openai_base_url: String,
    
    #[serde(default = "default_openai_model")]
    pub openai_model: String,
    
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    #[serde(default = "default_command_prefix")]
    pub command_prefix: String,
    
    #[serde(default = "default_max_history")]
    pub max_history: usize,
    
    #[serde(default)]
    pub bot_owners: Vec<String>,
    
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamingConfig {
    #[serde(default = "default_streaming_enabled")]
    pub streaming_enabled: bool,
    
    #[serde(default = "default_streaming_min_interval")]
    pub streaming_min_interval_ms: u64,
    
    #[serde(default = "default_streaming_min_chars")]
    pub streaming_min_chars: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VisionConfig {
    #[serde(default = "default_vision_enabled")]
    pub vision_enabled: bool,
    
    pub vision_model: Option<String>,
    
    #[serde(default = "default_vision_max_image_size")]
    pub vision_max_image_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
}
```

#### 2.3 默认值函数

```rust
fn default_device_display_name() -> String { "AI Bot".to_string() }
fn default_store_path() -> String { "./store".to_string() }
fn default_openai_base_url() -> String { "https://api.openai.com/v1".to_string() }
fn default_openai_model() -> String { "gpt-4o-mini".to_string() }
fn default_command_prefix() -> String { "!".to_string() }
fn default_max_history() -> usize { 10 }
fn default_db_path() -> String { "./data/aether.db".to_string() }
fn default_streaming_enabled() -> bool { true }
fn default_streaming_min_interval() -> u64 { 1000 }
fn default_streaming_min_chars() -> usize { 50 }
fn default_vision_enabled() -> bool { true }
fn default_vision_max_image_size() -> u32 { 1024 }
fn default_log_level() -> String { "info".to_string() }
```

#### 2.4 加载逻辑

```rust
impl Config {
    /// 主入口：加载 TOML + 环境变量覆盖
    pub fn load(path: &str) -> Result<Self> {
        // 1. 加载 .env 文件（兼容）
        #[cfg(not(test))]
        match dotenvy::dotenv() {
            Ok(p) => tracing::debug!(".env 文件已加载: {:?}", p),
            Err(e) => tracing::debug!(".env 文件加载失败: {}", e),
        }
        
        // 2. 加载 TOML 或警告 + 默认值
        let mut config = if std::path::Path::new(path).exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("无法读取配置文件: {}", path))?;
            toml::from_str(&content)
                .with_context(|| format!("配置文件格式错误: {}", path))?
        } else {
            tracing::warn!(
                "配置文件 {} 不存在，将使用默认值和环境变量",
                path
            );
            Self::default()
        };
        
        // 3. 环境变量覆盖
        config.apply_env_overrides();
        
        // 4. 验证必需字段
        config.validate()?;
        
        Ok(config)
    }
    
    /// 环境变量覆盖（保持原有变量名）
    fn apply_env_overrides(&mut self) {
        // Matrix 配置
        if let Ok(v) = std::env::var("MATRIX_HOMESERVER") {
            self.matrix_homeserver = v;
        }
        if let Ok(v) = std::env::var("MATRIX_USERNAME") {
            self.matrix_username = v;
        }
        if let Ok(v) = std::env::var("MATRIX_PASSWORD") {
            self.matrix_password = v;
        }
        if let Ok(v) = std::env::var("MATRIX_DEVICE_ID") {
            self.matrix_device_id = Some(v);
        }
        if let Ok(v) = std::env::var("DEVICE_DISPLAY_NAME") {
            self.device_display_name = v;
        }
        if let Ok(v) = std::env::var("STORE_PATH") {
            self.store_path = v;
        }
        
        // OpenAI 配置
        if let Ok(v) = std::env::var("OPENAI_API_KEY") {
            self.openai_api_key = v;
        }
        if let Ok(v) = std::env::var("OPENAI_BASE_URL") {
            self.openai_base_url = v;
        }
        if let Ok(v) = std::env::var("OPENAI_MODEL") {
            self.openai_model = v;
        }
        if let Ok(v) = std::env::var("SYSTEM_PROMPT") {
            self.system_prompt = Some(v);
        }
        
        // Bot 配置
        if let Ok(v) = std::env::var("BOT_COMMAND_PREFIX") {
            self.command_prefix = v;
        }
        if let Ok(v) = std::env::var("MAX_HISTORY") {
            if let Ok(n) = v.parse() {
                self.max_history = n;
            }
        }
        if let Ok(v) = std::env::var("BOT_OWNERS") {
            self.bot_owners = v.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(v) = std::env::var("DB_PATH") {
            self.db_path = v;
        }
        
        // 流式输出配置
        if let Ok(v) = std::env::var("STREAMING_ENABLED") {
            self.streaming_enabled = v.to_lowercase() != "false";
        }
        if let Ok(v) = std::env::var("STREAMING_MIN_INTERVAL_MS") {
            if let Ok(n) = v.parse() {
                self.streaming_min_interval_ms = n;
            }
        }
        if let Ok(v) = std::env::var("STREAMING_MIN_CHARS") {
            if let Ok(n) = v.parse() {
                self.streaming_min_chars = n;
            }
        }
        
        // Vision 配置
        if let Ok(v) = std::env::var("VISION_ENABLED") {
            self.vision_enabled = v.to_lowercase() != "false";
        }
        if let Ok(v) = std::env::var("VISION_MODEL") {
            self.vision_model = Some(v);
        }
        if let Ok(v) = std::env::var("VISION_MAX_IMAGE_SIZE") {
            if let Ok(n) = v.parse() {
                self.vision_max_image_size = n;
            }
        }
        
        // 日志配置
        if let Ok(v) = std::env::var("LOG_LEVEL") {
            self.log_level = v;
        }
        
        // 代理配置
        if let Ok(v) = std::env::var("PROXY") {
            self.proxy = Some(v);
        }
        
        // MCP 配置
        self.mcp.apply_env_overrides();
    }
    
    /// 验证必需字段
    fn validate(&self) -> Result<()> {
        if self.matrix_homeserver.is_empty() {
            anyhow::bail!(
                "MATRIX_HOMESERVER 未设置。\n\
                 请在 config.toml 的 [matrix] 部分或环境变量中配置 Matrix 服务器地址。\n\
                 示例: homeserver = \"https://matrix.org\""
            );
        }
        if self.matrix_username.is_empty() {
            anyhow::bail!(
                "MATRIX_USERNAME 未设置。\n\
                 请在 config.toml 的 [matrix] 部分或环境变量中配置 Matrix 用户名。\n\
                 示例: username = \"@bot:matrix.org\""
            );
        }
        if self.matrix_password.is_empty() {
            anyhow::bail!(
                "MATRIX_PASSWORD 未设置。\n\
                 请在 config.toml 的 [matrix] 部分或环境变量中配置 Matrix 密码。"
            );
        }
        if self.openai_api_key.is_empty() {
            anyhow::bail!(
                "OPENAI_API_KEY 未设置。\n\
                 请在 config.toml 的 [openai] 部分或环境变量中配置 API 密钥。\n\
                 示例: api_key = \"sk-...\""
            );
        }
        Ok(())
    }
    
    /// 向后兼容方法
    pub fn from_env() -> Result<Self> {
        Self::load("config.toml")
    }
}
```

---

### 3. MCP 配置修改 (`src/mcp/config.rs`)

#### 3.1 添加环境变量覆盖方法

```rust
impl McpConfig {
    /// 环境变量覆盖（由主 Config 调用）
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("MCP_ENABLED") {
            self.enabled = v.to_lowercase() != "false";
        }
        
        self.builtin_tools.apply_env_overrides();
        
        // 保留 MCP_EXTERNAL_SERVERS JSON 支持（向后兼容，标记为 deprecated）
        if let Ok(json) = std::env::var("MCP_EXTERNAL_SERVERS") {
            match serde_json::from_str(&json) {
                Ok(servers) => {
                    tracing::warn!(
                        "MCP_EXTERNAL_SERVERS 环境变量已弃用，请使用 config.toml 的 [[mcp.external_servers]] 配置外部服务器"
                    );
                    self.external_servers = servers;
                }
                Err(e) => {
                    tracing::warn!("MCP_EXTERNAL_SERVERS JSON 解析失败: {}", e);
                }
            }
        }
    }
}

impl BuiltinToolsConfig {
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("MCP_BUILTIN_TOOLS_ENABLED") {
            self.enabled = v.to_lowercase() != "false";
        }
        self.web_fetch.apply_env_overrides();
    }
}

impl WebFetchConfig {
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("MCP_BUILTIN_WEB_FETCH_ENABLED") {
            self.enabled = v.to_lowercase() != "false";
        }
        if let Ok(v) = std::env::var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH") {
            if let Ok(n) = v.parse() {
                self.max_length = n;
            }
        }
        if let Ok(v) = std::env::var("MCP_BUILTIN_WEB_FETCH_TIMEOUT") {
            if let Ok(n) = v.parse() {
                self.timeout = n;
            }
        }
    }
}
```

---

### 4. 入口点修改 (`src/main.rs`)

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "aether-matrix", about = "Matrix AI Bot")]
struct Args {
    /// 配置文件路径
    #[arg(short = 'c', long = "config", default_value = "./config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load(&args.config)?;
    
    // 使用 EnvFilter 支持通过环境变量动态调整日志级别
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new(&config.log_level).expect("Invalid log level"),
        )
        .init();
    
    info!("配置加载完成");
    
    // 创建并运行 Bot
    Bot::new(config).await?.run().await
}
```

---

### 5. 示例配置文件 (`config.example.toml`)

```toml
# ===========================================
# Aether Matrix Bot 配置文件示例
# ===========================================
# 复制此文件为 config.toml 并填写配置
# 
# 配置优先级：环境变量 > TOML 配置 > 默认值
# 即环境变量可以覆盖 TOML 中的配置

# Matrix 连接配置
[matrix]
homeserver = "https://matrix.org"
username = "@your-bot:matrix.org"
password = "your-password"
# device_id = "AETHER_BOT_001"  # 可选，持久化设备 ID，避免重复登录创建新设备
device_display_name = "AI Bot"
store_path = "./store"

# OpenAI API 配置
[openai]
api_key = "sk-..."
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
# system_prompt = "你是一个有帮助的 AI 助手"

# 机器人行为配置
[bot]
command_prefix = "!"
max_history = 10
# owners = ["@admin:matrix.org"]  # Bot 管理员列表，逗号分隔
db_path = "./data/aether.db"

# 流式输出配置
[streaming]
enabled = true
min_interval_ms = 1000
min_chars = 50

# Vision 配置
[vision]
enabled = true
# model = "gpt-4o-mini"  # 可选，默认使用 openai.model
max_image_size = 1024

# 日志配置
[log]
level = "info"

# 代理配置（可选）
# proxy = "http://127.0.0.1:7890"

# MCP (Model Context Protocol) 配置
[mcp]
enabled = true

[mcp.builtin_tools]
enabled = true

[mcp.builtin_tools.web_fetch]
enabled = true
max_length = 10000
timeout = 10

# 外部 MCP 服务器示例
# [[mcp.external_servers]]
# name = "filesystem"
# transport = "stdio"
# command = "mcp-server-filesystem"
# args = ["/home/user/documents"]
# enabled = true

# [[mcp.external_servers]]
# name = "database"
# transport = "http"
# url = "http://localhost:3000/mcp"
# enabled = true
```

---

### 6. 测试更新

需要修改 `src/config.rs` 中的测试以适应新结构：

1. `setup_env` 函数保持不变
2. 修改测试断言以适应 `#[serde(flatten)]` 的扁平结构
3. 添加 TOML 解析测试
4. 添加环境变量覆盖测试
5. 添加配置文件不存在警告测试

---

## 环境变量映射表

| 环境变量 | TOML 路径 | 代码访问 |
|---------|----------|---------|
| `MATRIX_HOMESERVER` | `matrix.homeserver` | `config.matrix_homeserver` |
| `MATRIX_USERNAME` | `matrix.username` | `config.matrix_username` |
| `MATRIX_PASSWORD` | `matrix.password` | `config.matrix_password` |
| `MATRIX_DEVICE_ID` | `matrix.device_id` | `config.matrix_device_id` |
| `DEVICE_DISPLAY_NAME` | `matrix.device_display_name` | `config.device_display_name` |
| `STORE_PATH` | `matrix.store_path` | `config.store_path` |
| `OPENAI_API_KEY` | `openai.api_key` | `config.openai_api_key` |
| `OPENAI_BASE_URL` | `openai.base_url` | `config.openai_base_url` |
| `OPENAI_MODEL` | `openai.model` | `config.openai_model` |
| `SYSTEM_PROMPT` | `openai.system_prompt` | `config.system_prompt` |
| `BOT_COMMAND_PREFIX` | `bot.command_prefix` | `config.command_prefix` |
| `MAX_HISTORY` | `bot.max_history` | `config.max_history` |
| `BOT_OWNERS` | `bot.owners` | `config.bot_owners` |
| `DB_PATH` | `bot.db_path` | `config.db_path` |
| `STREAMING_ENABLED` | `streaming.enabled` | `config.streaming_enabled` |
| `STREAMING_MIN_INTERVAL_MS` | `streaming.min_interval_ms` | `config.streaming_min_interval_ms` |
| `STREAMING_MIN_CHARS` | `streaming.min_chars` | `config.streaming_min_chars` |
| `VISION_ENABLED` | `vision.enabled` | `config.vision_enabled` |
| `VISION_MODEL` | `vision.model` | `config.vision_model` |
| `VISION_MAX_IMAGE_SIZE` | `vision.max_image_size` | `config.vision_max_image_size` |
| `LOG_LEVEL` | `log.level` | `config.log_level` |
| `PROXY` | `proxy` | `config.proxy` |
| `MCP_ENABLED` | `mcp.enabled` | `config.mcp.enabled` |
| `MCP_BUILTIN_TOOLS_ENABLED` | `mcp.builtin_tools.enabled` | `config.mcp.builtin_tools.enabled` |
| `MCP_BUILTIN_WEB_FETCH_ENABLED` | `mcp.builtin_tools.web_fetch.enabled` | `config.mcp.builtin_tools.web_fetch.enabled` |
| `MCP_BUILTIN_WEB_FETCH_MAX_LENGTH` | `mcp.builtin_tools.web_fetch.max_length` | `config.mcp.builtin_tools.web_fetch.max_length` |
| `MCP_BUILTIN_WEB_FETCH_TIMEOUT` | `mcp.builtin_tools.web_fetch.timeout` | `config.mcp.builtin_tools.web_fetch.timeout` |
| `MCP_EXTERNAL_SERVERS` | (deprecated) JSON | `config.mcp.external_servers` |

---

## 实现步骤

1. **添加依赖** - 修改 `Cargo.toml`
2. **重构 Config** - 修改 `src/config.rs`
3. **修改 MCP 配置** - 修改 `src/mcp/config.rs`
4. **修改入口点** - 修改 `src/main.rs`
5. **创建示例文件** - 新建 `config.example.toml`
6. **运行测试** - 确保所有测试通过

---

## 注意事项

1. **向后兼容**: `Config::from_env()` 方法保留，内部调用 `load("config.toml")`
2. **deprecated 警告**: `MCP_EXTERNAL_SERVERS` 环境变量使用时会输出警告
3. **配置文件不存在警告**: 输出警告日志，但继续使用默认值和环境变量
4. **必需字段验证**: 缺失时提供友好的错误提示，包含 TOML 和环境变量两种配置方式示例