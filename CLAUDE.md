# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Matrix AI 机器人 - 一个基于 Matrix 协议的 AI 助手机器人，使用 OpenAI 兼容 API 提供聊天功能。支持流式输出（打字机效果）、多会话管理、会话持久化、图片理解（Vision API）。

## 常用命令

```bash
make build    # 编译项目 (release)
make run      # 运行项目
make test     # 运行测试
make check    # 快速检查（不生成二进制文件）
make fmt      # 格式化代码
make lint     # 运行 clippy lint
make fix      # 自动修复代码问题并格式化
```

## 架构

```
src/
├── main.rs           # 入口点：初始化日志和 Bot
├── lib.rs            # 库入口：模块导出和文档
├── bot.rs            # Bot 结构体：初始化 Matrix 客户端和事件处理器
├── config.rs         # 配置管理：从环境变量加载配置
├── ai_service.rs     # AI 服务：封装 OpenAI API，管理会话历史
├── conversation.rs   # 会话管理：多用户/多房间的会话历史管理
├── traits.rs         # Trait 抽象：AiServiceTrait 支持 mock 测试
├── media.rs          # 媒体处理：图片下载、缩放、base64 编码
└── event_handler/    # Matrix 事件处理模块
    ├── mod.rs        # 主处理器：消息路由和响应逻辑
    ├── invite.rs     # 邀请处理：自动接受房间邀请
    ├── streaming.rs  # 流式处理：打字机效果的节流逻辑
    └── extract.rs    # 消息提取：文本和引用图片提取
```

### 核心数据流

1. `main.rs` 加载配置并初始化日志
2. `Bot::new()` 创建 Matrix 客户端并登录
3. 注册两类事件处理器：邀请事件（自动加入房间）和消息事件
4. 消息到达时 `EventHandler` 判断是否需要响应：
   - 私聊：总是响应
   - 群聊：需要命令前缀（默认 `!ai`）或 @提及
5. 根据消息类型调用 `AiService`：
   - 文本消息：普通聊天
   - 图片消息：Vision API 分析
   - 回复图片：分析引用的图片
6. `ConversationManager` 按 session_id（用户ID或房间ID）隔离会话

### 关键类型

- `Config`: 所有配置项，从环境变量加载
- `Bot`: 机器人主结构，封装 Matrix 客户端初始化和事件处理器注册
- `AiService`: Clone 封装，内部使用 `Arc<AiServiceInner>` 共享状态，实现 `AiServiceTrait`
- `AiServiceTrait`: AI 服务 trait 抽象，支持依赖注入和 mock 测试
- `EventHandler<T>`: 泛型事件处理器，支持任意实现 `AiServiceTrait` 的服务
- `StreamingHandler`: 流式响应处理，混合节流策略
- `ConversationManager`: 管理多会话历史，支持历史长度限制

### 流式输出机制

当 `STREAMING_ENABLED=true` 时，机器人使用流式响应：
1. `AiService::chat_stream()` 返回共享状态 `StreamingState` 和 Stream
2. `StreamingHandler` 消费 Stream，使用混合节流策略更新消息：
   - 时间触发：超过 `STREAMING_MIN_INTERVAL_MS`（默认 1000ms）
   - 字符触发：累积超过 `STREAMING_MIN_CHARS`（默认 50 字符）
3. 首次发送新消息，后续使用 Matrix 消息编辑 API 更新内容

### Vision API 支持

当 `VISION_ENABLED=true` 时，机器人支持图片理解：
1. 用户发送图片消息时，机器人下载并分析图片
2. 用户回复图片消息时，机器人分析引用的图片
3. 图片自动缩放至 `VISION_MAX_IMAGE_SIZE` 以下（保持宽高比）
4. 使用配置的 `VISION_MODEL` 或默认模型进行分析

## 配置

复制 `.env.example` 为 `.env` 并填写配置：

**必需配置：**
- `MATRIX_HOMESERVER` - Matrix 服务器地址
- `MATRIX_USERNAME` / `MATRIX_PASSWORD` - 登录凭据
- `OPENAI_API_KEY` - API 密钥

**可选配置：**
- `MATRIX_DEVICE_ID` - 设备ID（持久化设备，避免重复登录创建新设备）
- `DEVICE_DISPLAY_NAME` - 设备显示名称（默认 AI Bot）
- `STORE_PATH` - Matrix SDK 存储路径（默认 ./store）
- `OPENAI_BASE_URL` - API 地址（默认 OpenAI，支持兼容接口）
- `OPENAI_MODEL` - 模型名称（默认 gpt-4o-mini）
- `SYSTEM_PROMPT` - 系统提示词
- `BOT_COMMAND_PREFIX` - 命令前缀（默认 !ai）
- `MAX_HISTORY` - 最大历史轮数（默认 10）
- `STREAMING_ENABLED` - 启用流式输出（默认 true）
- `STREAMING_MIN_INTERVAL_MS` - 流式更新最小间隔（默认 1000）
- `STREAMING_MIN_CHARS` - 流式更新最小字符数（默认 50）
- `LOG_LEVEL` - 日志级别（默认 info）
- `VISION_ENABLED` - 启用图片理解（默认 true）
- `VISION_MODEL` - Vision 模型（默认使用 OPENAI_MODEL）
- `VISION_MAX_IMAGE_SIZE` - 图片最大尺寸（默认 1024）

## 机器人命令

- `!ai <消息>` - 与 AI 对话（前缀可配置）
- `!reset` - 清除当前会话历史
- `!help` - 显示帮助
- 发送图片 - 分析图片内容
- 回复图片 - 分析引用的图片

## 代码风格

- 使用空格缩进（4 空格），Makefile 使用 tab
- 使用 `anyhow::Result` 进行错误处理
- 使用 `tracing` 进行日志记录
- 使用 `Arc` + `RwLock`/`Mutex` 模式共享可变状态
- Rust edition 2024
- 使用 trait 抽象（`AiServiceTrait`）支持测试

## 运行时数据

- `./store/` - Matrix SDK 的 SQLite 存储目录，保存会话和同步状态