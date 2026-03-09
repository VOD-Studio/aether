# Command Handler Documentation Template

This template provides a unified standard for documenting command handlers in the Aether Matrix Bot project, based on established patterns from `src/command/registry.rs` and `src/command/mod.rs`.

## Struct Documentation

```rust
/// <Brief one-line description of the command>.
///
/// <Detailed description of what the command does, its purpose,
/// and any important behavior notes.>
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::{CommandHandler, CommandContext, Permission};
/// use async_trait::async_trait;
///
/// /// <Brief description>.
/// pub struct <HandlerName>Handler;
///
/// #[async_trait]
/// impl CommandHandler for <HandlerName>Handler {
///     fn name(&self) -> &str {
///         "<command-name>"
///     }
///
///     fn description(&self) -> &str {
///         "<Command description for help text>"
///     }
///
///     fn usage(&self) -> &str {
///         "<command-name> [args] - <usage description>"
///     }
///
///     fn permission(&self) -> Permission {
///         Permission::<Anyone|RoomMod|BotOwner>
///     }
///
///     async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
///         // Command implementation
///         Ok(())
///     }
/// }
/// ```
pub struct <HandlerName>Handler;
```

## Required Method Documentation

### `name(&self) -> &str`

```rust
/// 命令名称（不含前缀）。
///
/// 例如命令 `!help` 的 name 为 `"help"`。
fn name(&self) -> &str;
```

### `description(&self) -> &str`

```rust
/// 命令描述。
///
/// 用于帮助信息，简要说明命令功能。
fn description(&self) -> &str {
    "暂无描述"
}
```

### `usage(&self) -> &str`

```rust
/// 使用说明。
///
/// 用于帮助信息，说明命令的参数和用法。
fn usage(&self) -> &str {
    ""
}
```

### `permission(&self) -> Permission`

```rust
/// 所需权限级别。
///
/// 默认为 `Anyone`，任何房间成员都可执行。
fn permission(&self) -> Permission {
    Permission::Anyone
}
```

### `execute(&self, ctx: &CommandContext<'_>) -> Result<()>`

```rust
/// 执行命令。
///
/// # Arguments
///
/// * `ctx` - 命令执行上下文，包含客户端、房间、发送者等信息
///
/// # Returns
///
/// 成功时返回 `Ok(())`，失败时返回错误。
async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()>;
```

## Complete Example

```rust
/// Bot 信息命令处理器。
///
/// 显示 Bot 的基本信息，包括：
/// - 显示名称
/// - 当前模型
/// - 命令前缀
/// - 已启用功能
///
/// # 权限
///
/// 任何房间成员都可以执行此命令。
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::{CommandHandler, CommandContext, Permission};
/// use async_trait::async_trait;
///
/// /// Bot 信息命令处理器。
/// pub struct BotInfoHandler;
///
/// #[async_trait]
/// impl CommandHandler for BotInfoHandler {
///     fn name(&self) -> &str {
///         "bot info"
///     }
///
///     fn description(&self) -> &str {
///         "查看 Bot 基本信息"
///     }
///
///     fn usage(&self) -> &str {
///         "bot info - 显示 Bot 的名称、模型、前缀等信息"
///     }
///
///     fn permission(&self) -> Permission {
///         Permission::Anyone
///     }
///
///     async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
///         // 获取 Bot 信息并发送响应
///         let info = ctx.get_bot_info().await?;
///         ctx.reply(&info).await?;
///         Ok(())
///     }
/// }
/// ```
pub struct BotInfoHandler;

#[async_trait]
impl CommandHandler for BotInfoHandler {
    fn name(&self) -> &str {
        "bot info"
    }

    fn description(&self) -> &str {
        "查看 Bot 基本信息"
    }

    fn usage(&self) -> &str {
        "bot info - 显示 Bot 的名称、模型、前缀等信息"
    }

    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // Implementation...
        Ok(())
    }
}
```

## Documentation Checklist

When documenting a command handler, ensure:

- [ ] Struct has a brief one-line description
- [ ] Struct has detailed description (if complex)
- [ ] Example section with complete implementation skeleton
- [ ] `name()` - Brief description with example
- [ ] `description()` - Brief description of purpose
- [ ] `usage()` - Brief description (if overridden)
- [ ] `permission()` - Brief description (if overridden)
- [ ] `execute()` - Arguments and Returns sections

## Common Patterns

### Simple Command (Anyone)

```rust
/// Ping 命令处理器。
///
/// 测试 Bot 响应延迟，返回 pong。
pub struct PingHandler;

#[async_trait]
impl CommandHandler for PingHandler {
    fn name(&self) -> &str {
        "ping"
    }

    fn description(&self) -> &str {
        "测试 Bot 响应"
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        ctx.reply("pong").await?;
        Ok(())
    }
}
```

### Admin Command (RoomMod)

```rust
/// 人设设置命令处理器。
///
/// 为当前房间设置指定的人设。
///
/// # 权限
///
/// 需要房间管理员权限。
pub struct PersonaSetHandler {
    store: Arc<PersonaStore>,
}

#[async_trait]
impl CommandHandler for PersonaSetHandler {
    fn name(&self) -> &str {
        "persona set"
    }

    fn description(&self) -> &str {
        "设置房间人设"
    }

    fn usage(&self) -> &str {
        "persona set <id> - 设置房间使用的 AI 人设"
    }

    fn permission(&self) -> Permission {
        Permission::RoomMod
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // Implementation...
        Ok(())
    }
}
```

### Owner Command (BotOwner)

```rust
/// Bot 名称修改命令处理器。
///
/// 修改 Bot 的显示名称。
///
/// # 权限
///
/// 仅 Bot 所有者可执行。
pub struct BotNameHandler;

#[async_trait]
impl CommandHandler for BotNameHandler {
    fn name(&self) -> &str {
        "bot name"
    }

    fn description(&self) -> &str {
        "修改 Bot 显示名称"
    }

    fn usage(&self) -> &str {
        "bot name <名称> - 修改 Bot 的显示名称"
    }

    fn permission(&self) -> Permission {
        Permission::BotOwner
    }

    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()> {
        // Implementation...
        Ok(())
    }
}
```

## Style Guidelines

1. **Use Chinese for user-facing descriptions** - `description()` and `usage()` should be in Chinese for consistency with the codebase.

2. **Keep method docs brief** - One-line description is sufficient for simple methods.

3. **Document behavior, not implementation** - Focus on what the command does, not how.

4. **Include permissions when relevant** - Add a `# 权限` section for non-Anyone commands.

5. **Use `///` for doc comments** - Not `//` or `//!` for struct/method docs.

6. **Example uses `ignore` attribute** - The example code is not meant to compile standalone.