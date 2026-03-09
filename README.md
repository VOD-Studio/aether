# Aether

基于 Matrix 协议的 AI 助手机器人。支持多模型、流式输出、人设系统、权限控制。

## 功能特性

### 核心功能

- 流式输出 - 打字机效果，实时显示 AI 响应
- 多会话管理 - 私聊按用户隔离，群聊按房间隔离
- 图片理解 - Vision API 分析用户发送或回复的图片
- 兼容性强 - 支持 OpenAI 及其兼容 API（DeepSeek、通义千问等）

### 人设系统

- 内置人设 - 4 个预置人设（毒舌程序员、赛博禅师、维基百科娘、猫娘助手）
- 自定义人设 - 支持创建、删除自定义人设
- 房间级配置 - 每个房间可独立设置人设

### 命令系统

- 模块化架构 - 基于 Trait 的命令处理器，易于扩展
- 权限控制 - 三级权限模型（任何人/房间管理员/Bot所有者）
- 子命令支持 - 支持 `!bot info`、`!persona set` 等层级结构

### 运维功能

- Bot 管理 - 运行时修改名称、头像、加入房间
- 会话持久化 - SQLite 存储会话状态和人设配置
- 代理支持 - HTTP/SOCKS5 代理

## 快速开始

### 安装

```bash
git clone https://github.com/your-username/aether.git
cd aether
make build
```

### 配置

复制配置模板并填写：

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```env
# Matrix 配置（必需）
MATRIX_HOMESERVER=https://matrix.example.org
MATRIX_USERNAME=your_username
MATRIX_PASSWORD=your_password

# OpenAI API 配置（必需）
OPENAI_API_KEY=your_api_key

# 可选配置
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_MODEL=gpt-4o-mini
BOT_OWNERS=@user1:matrix.org,@user2:matrix.org
```

### 运行

```bash
make run
```

## 命令参考

### 聊天命令

| 命令 | 说明 | 权限 |
|------|------|------|
| `!<消息>` | 与 AI 对话 | 任何人 |
| `!reset` | 清除当前会话历史 | 任何人 |
| `!help` | 显示帮助菜单 | 任何人 |

### 人设命令

| 命令 | 说明 | 权限 |
|------|------|------|
| `!persona list` | 列出所有人设 | 任何人 |
| `!persona info <id>` | 查看人设详情 | 任何人 |
| `!persona set <id>` | 设置房间人设 | 房间管理员 |
| `!persona off` | 关闭房间人设 | 房间管理员 |
| `!persona create <id> "名称" "提示词"` | 创建自定义人设 | 房间管理员 |
| `!persona delete <id>` | 删除自定义人设 | 房间管理员 |

### 管理命令

| 命令 | 说明 | 权限 |
|------|------|------|
| `!bot info` | 查看 Bot 基本信息 | 任何人 |
| `!bot ping` | 测试响应延迟 | 任何人 |
| `!bot name <名称>` | 修改显示名称 | Bot 所有者 |
| `!bot avatar <url>` | 修改头像 | Bot 所有者 |
| `!bot join <房间ID>` | 加入指定房间 | Bot 所有者 |
| `!bot rooms` | 列出已加入房间 | Bot 所有者 |
| `!leave` | 离开当前房间 | 房间管理员 |

## 人设系统

### 内置人设

| ID | 名称 | 特点 |
|----|------|------|
| `sarcastic-dev` | 毒舌程序员 | 20年经验，对低质量代码愤怒，先吐槽再回答 |
| `cyber-zen` | 赛博禅师 | 用 TCP/IP 诠释佛法，简短深邃 |
| `wiki-chan` | 维基百科娘 | 知识渊博、严谨客观、标注来源 |
| `neko-chan` | 猫娘助手 | 语气活泼可爱，句末加「喵~」 |

### 自定义人设

```bash
# 创建自定义人设
!persona create my-assistant "我的助手" "你是一个专业的 Rust 开发顾问..."

# 设置到当前房间
!persona set my-assistant

# 查看效果
!persona info my-assistant

# 删除
!persona delete my-assistant
```

## 权限模型

| 级别 | 说明 | 适用命令 |
|------|------|----------|
| Anyone | 任何房间成员 | 聊天、查看人设、ping |
| RoomMod | 房间管理员或私聊用户 | 设置人设、离开房间 |
| BotOwner | Bot 所有者（配置 BOT_OWNERS） | 修改名称/头像、加入房间 |

私聊房间默认拥有 RoomMod 权限。

## 配置说明

### 必需配置

| 配置项 | 说明 |
|--------|------|
| `MATRIX_HOMESERVER` | Matrix 服务器地址 |
| `MATRIX_USERNAME` | Matrix 用户名 |
| `MATRIX_PASSWORD` | Matrix 密码 |
| `OPENAI_API_KEY` | API 密钥 |

### 可选配置

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `MATRIX_DEVICE_ID` | 设备 ID（避免重复登录） | 随机生成 |
| `DEVICE_DISPLAY_NAME` | 设备显示名称 | AI Bot |
| `STORE_PATH` | Matrix SDK 存储路径 | ./store |
| `OPENAI_BASE_URL` | API 地址 | https://api.openai.com/v1 |
| `OPENAI_MODEL` | 模型名称 | gpt-4o-mini |
| `SYSTEM_PROMPT` | 系统提示词 | - |
| `BOT_COMMAND_PREFIX` | 命令前缀 | ! |
| `BOT_OWNERS` | Bot 所有者列表（逗号分隔） | - |
| `DB_PATH` | 数据库路径（人设存储） | ./data/aether.db |
| `MAX_HISTORY` | 最大历史轮数 | 10 |
| `STREAMING_ENABLED` | 启用流式输出 | true |
| `STREAMING_MIN_INTERVAL_MS` | 流式更新最小间隔（毫秒） | 1000 |
| `STREAMING_MIN_CHARS` | 流式更新最小字符数 | 50 |
| `VISION_ENABLED` | 启用图片理解 | true |
| `VISION_MODEL` | Vision 模型 | 使用 OPENAI_MODEL |
| `VISION_MAX_IMAGE_SIZE` | 图片最大尺寸（像素） | 1024 |
| `PROXY` | HTTP/SOCKS5 代理 URL | - |
| `LOG_LEVEL` | 日志级别 | info |

## 项目结构

```
src/
├── main.rs              # 入口点
├── config.rs            # 配置管理
├── bot.rs               # Bot 初始化
├── ai_service.rs        # AI 服务封装
├── conversation.rs      # 会话管理
├── traits.rs            # Trait 抽象
├── media.rs             # 媒体处理
├── event_handler.rs     # Matrix 事件处理
├── command/             # 命令系统
│   ├── mod.rs
│   ├── context.rs       # 命令上下文
│   ├── gateway.rs       # 命令路由网关
│   ├── parser.rs        # 命令解析
│   ├── permission.rs    # 权限模型
│   └── registry.rs      # 命令注册表
├── modules/             # 功能模块
│   ├── admin/           # 管理命令
│   └── persona/         # 人设管理
├── store/               # 数据存储
│   ├── database.rs      # 数据库连接
│   └── persona_store.rs # 人设持久化
└── ui/                  # UI 模板
    └── templates.rs     # Matrix HTML 模板

tests/                   # 集成测试
migrations/              # 数据库迁移
```

## 架构设计

### 模块化命令系统

命令系统基于 `CommandHandler` trait 实现，支持灵活扩展：

```rust
#[async_trait]
pub trait CommandHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn permission(&self) -> Permission;
    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()>;
}
```

**核心组件：**

- `CommandRegistry` - 命令注册表，管理所有命令处理器
- `CommandGateway` - 路由网关，解析消息并分发到对应处理器
- `CommandContext` - 封装房间、发送者、参数等上下文信息
- `Permission` - 权限检查，支持 Anyone/RoomMod/BotOwner 三级

### 数据流

```
Matrix 消息
    │
    ▼
EventHandler
    │ 判断是否需要响应（私聊/前缀/@提及）
    ▼
CommandGateway
    │ 解析命令名和参数
    ▼
CommandRegistry.get(name)
    │ 获取命令处理器
    ▼
Permission.check()
    │ 权限验证
    ▼
CommandHandler.execute(ctx)
    │ 执行命令逻辑
    ▼
AiService / PersonaStore / Bot API
    │ 业务处理
    ▼
Room.send() - 返回响应
```

### 流式输出机制

启用流式输出时，使用混合节流策略：

1. **时间触发** - 超过 `STREAMING_MIN_INTERVAL_MS`（默认 1000ms）
2. **字符触发** - 累积超过 `STREAMING_MIN_CHARS`（默认 50 字符）

首次发送新消息，后续使用 Matrix 消息编辑 API 更新内容。

### 人设系统设计

- `PersonaStore` - 基于 SQLite 的人设持久化
- 内置人设通过 UPSERT 初始化，保证幂等性
- 房间与人设关联存储在 `room_persona` 表
- 设置人设时自动更新 Bot 显示名称（添加人设后缀）

## 开发

```bash
make build    # 编译项目（release）
make run      # 运行项目
make test     # 运行测试
make check    # 快速检查（不生成二进制文件）
make fmt      # 格式化代码
make lint     # 运行 clippy lint
make fix      # 自动修复代码问题并格式化
make clean    # 清理构建产物
```

## 技术栈

- [matrix-sdk](https://github.com/matrix-org/matrix-rust-sdk) - Matrix 客户端 SDK
- [async-openai](https://github.com/64bit/async-openai) - OpenAI 异步客户端
- [tokio](https://tokio.rs/) - 异步运行时
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite 绑定
- [async-trait](https://github.com/dtolnay/async-trait) - 异步 trait 支持
- [anyhow](https://github.com/dtolnay/anyhow) - 错误处理
- [image](https://github.com/image-rs/image) - 图片处理

## License

MIT