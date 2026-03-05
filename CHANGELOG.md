# Changelog

本文档记录项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.1.1] - 2026-03-05

### 新增

- Vision 多模态支持
  - 支持图片理解功能，用户发送图片时自动分析图片内容
  - 支持回复图片消息进行 Vision 分析
  - 图片自动缩放至配置的最大尺寸（保持宽高比）
  - 可配置的 Vision 专用模型和最大图片尺寸

- 流式输出功能
  - 支持流式响应，实现打字机效果
  - 混合节流策略：时间触发 + 字符触发
  - 首次发送新消息，后续使用 Matrix 消息编辑 API 更新
  - 可配置的最小更新间隔和最小字符数

- 配置系统
  - 添加 .env 文件支持，从环境变量加载配置
  - 添加设备ID配置支持（MATRIX_DEVICE_ID），避免重复登录创建新设备
  - 添加可配置的设备显示名称（DEVICE_DISPLAY_NAME）
  - 添加可配置的存储路径支持（STORE_PATH）
  - 添加日志级别配置（LOG_LEVEL）

- 会话管理
  - 支持提取引用消息内容作为上下文
  - 多用户/多房间的会话历史隔离
  - 可配置的最大历史轮数限制

- MSC 3456 提及检测
  - 支持检测 Matrix 房间提及事件
  - 群聊中 @提及时自动响应

- 测试基础设施
  - 提取 AiServiceTrait trait 抽象，支持依赖注入和 mock 测试
  - 添加 Config::from_env 单元测试
  - 添加 RoomClient trait 和集成测试基础设施

### 改进

- 错误处理与同步稳定性优化
- Release 构建优化配置
  - 最高优化级别（opt-level = 3）
  - 链接时优化（LTO fat）
  - 单个代码生成单元，优化更彻底
  - panic = abort 减小二进制大小
  - 剥离符号信息

### 重构

- 提取 Bot 结构体，改进代码组织
- 移除 async-trait 依赖，Rust 2024 原生支持 trait async fn
- 拆分 event_handler 为多模块结构
  - mod.rs: 主处理器，消息路由和响应逻辑
  - invite.rs: 邀请处理，自动接受房间邀请
  - streaming.rs: 流式处理，打字机效果的节流逻辑
  - extract.rs: 消息提取，文本和引用图片提取
- 使用 impl Future 语法优化 trait 定义
- 使用 watch 通道替代 broadcast 处理关闭信号
- 移除未使用的 RoomClient trait 和 SendMessageResult
- 移除测试模块中未使用的 MockAiService 代码
- 统一代码格式化风格

### 文档

- 添加项目 README 文档
- 添加 CLAUDE.md 开发指南
- 为所有公开 API 添加详细文档注释
- 完善 CLAUDE.md 流式输出和配置说明
- 更新 README 反映 Vision 支持和最新架构
- 更新 CLAUDE.md 反映 Vision API 支持和最新代码结构
- 修复模块文档链接语法

### 依赖

- 升级 async-openai 至 0.33 并适配 API 变更

### 样式

- 统一代码格式化风格

## [0.1.0] - 2026-02-XX

### 新增

- 初始化 matrix-xfy 项目
- 实现 Matrix AI 机器人核心功能
  - 基于 Matrix 协议的 AI 助手机器人
  - 使用 OpenAI 兼容 API 提供聊天功能
  - 支持私聊和群聊（命令前缀或 @提及）
  - 会话持久化