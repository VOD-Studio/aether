//! # 命令处理器注册表
//!
//! 管理命令处理器的注册和查找。

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::command::context::CommandContext;
use crate::ui;

use super::permission::Permission;

/// 命令处理器 trait。
///
/// 所有命令处理器都必须实现此 trait，定义命令的基本信息和执行逻辑。
///
/// # Required Methods
///
/// - `name`: 命令名称（不含前缀）
/// - `execute`: 命令执行逻辑
///
/// # Optional Methods
///
/// - `description`: 命令描述，用于帮助信息
/// - `usage`: 使用说明，用于帮助信息
/// - `permission`: 所需权限级别，默认为 `Anyone`
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::{CommandHandler, CommandContext, Permission};
/// use async_trait::async_trait;
///
/// struct HelpHandler;
///
/// #[async_trait]
/// impl CommandHandler for HelpHandler {
///     fn name(&self) -> &str {
///         "help"
///     }
///
///     fn description(&self) -> &str {
///         "显示帮助信息"
///     }
///
///     async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
///         // 命令执行逻辑
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// 命令名称（不含前缀）。
    ///
    /// 例如命令 `!help` 的 name 为 `"help"`。
    fn name(&self) -> &str;

    /// 命令描述。
    ///
    /// 用于帮助信息，简要说明命令功能。
    fn description(&self) -> &str {
        "暂无描述"
    }

    /// 使用说明。
    ///
    /// 用于帮助信息，说明命令的参数和用法。
    fn usage(&self) -> &str {
        ""
    }

    /// 所需权限级别。
    ///
    /// 默认为 `Anyone`，任何房间成员都可执行。
    fn permission(&self) -> Permission {
        Permission::Anyone
    }

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
}

/// 命令注册表。
///
/// 管理所有已注册的命令处理器，支持：
/// - 注册和查找命令处理器
/// - 生成帮助信息（纯文本和 HTML 格式）
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::CommandRegistry;
/// use std::sync::Arc;
///
/// let mut registry = CommandRegistry::new();
///
/// // 注册命令
/// registry.register(Arc::new(HelpHandler));
///
/// // 查找命令
/// let handler = registry.get("help");
///
/// // 生成帮助信息
/// let help = registry.generate_help();
/// ```
#[derive(Clone)]
pub struct CommandRegistry {
    /// 命令处理器映射，key 为命令名称。
    handlers: HashMap<String, Arc<dyn CommandHandler>>,
}

impl CommandRegistry {
    /// 创建新的空注册表。
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// 注册命令处理器。
    ///
    /// 如果已存在同名命令，新处理器会覆盖旧的。
    ///
    /// # Arguments
    ///
    /// * `handler` - 命令处理器实例
    pub fn register(&mut self, handler: Arc<dyn CommandHandler>) {
        self.handlers.insert(handler.name().to_string(), handler);
    }

    /// 获取命令处理器。
    ///
    /// # Arguments
    ///
    /// * `name` - 命令名称
    ///
    /// # Returns
    ///
    /// 如果找到返回 `Some(handler)`，否则返回 `None`。
    pub fn get(&self, name: &str) -> Option<Arc<dyn CommandHandler>> {
        self.handlers.get(name).cloned()
    }

    /// 获取所有已注册的命令名称。
    ///
    /// # Returns
    ///
    /// 返回命令名称列表。
    #[allow(dead_code)]
    pub fn commands(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }

    /// 生成纯文本格式的帮助信息。
    ///
    /// 按命令名称排序，包含命令描述、用法和权限要求。
    ///
    /// # Returns
    ///
    /// 返回格式化的帮助文本。
    pub fn generate_help(&self) -> String {
        let mut help = String::from("可用命令:\n\n");

        let mut commands: Vec<_> = self.handlers.iter().collect();
        commands.sort_by_key(|(name, _)| *name);

        for (name, handler) in commands {
            help.push_str(&format!("**!{}** - {}\n", name, handler.description()));
            if !handler.usage().is_empty() {
                help.push_str(&format!("  用法: {}\n", handler.usage()));
            }
            if handler.permission() != Permission::Anyone {
                help.push_str(&format!(
                    "  权限: {}\n",
                    handler.permission().display_name()
                ));
            }
            help.push('\n');
        }

        help
    }

    /// 生成 HTML 格式的帮助菜单。
    ///
    /// 使用 UI 模板的帮助菜单样式。
    ///
    /// # Returns
    ///
    /// 返回 HTML 格式的帮助菜单。
    pub fn generate_help_html(&self) -> String {
        let mut commands: Vec<_> = self.handlers.iter().collect();
        commands.sort_by_key(|(name, _)| *name);

        // 构建命令列表项 (name, description)
        let cmd_items: Vec<(String, String)> = commands
            .iter()
            .map(|(name, handler)| {
                let cmd_name = format!("!{}", name);
                (cmd_name, handler.description().to_string())
            })
            .collect();

        ui::help_menu(&cmd_items)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::context::CommandContext;

    struct MockHandler {
        name: &'static str,
        desc: &'static str,
        permission: Permission,
    }

    impl MockHandler {
        fn new(name: &'static str, desc: &'static str) -> Self {
            Self {
                name,
                desc,
                permission: Permission::Anyone,
            }
        }

        fn with_permission(name: &'static str, desc: &'static str, permission: Permission) -> Self {
            Self {
                name,
                desc,
                permission,
            }
        }
    }

    #[async_trait]
    impl CommandHandler for MockHandler {
        fn name(&self) -> &str {
            self.name
        }

        fn description(&self) -> &str {
            self.desc
        }

        fn permission(&self) -> Permission {
            self.permission
        }

        async fn execute(&self, _ctx: &CommandContext<'_>) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_new_registry_is_empty() {
        let registry = CommandRegistry::new();
        assert!(registry.commands().is_empty());
    }

    #[test]
    fn test_register_adds_handler() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("test", "Test command"));

        registry.register(handler);

        assert_eq!(registry.commands().len(), 1);
        assert!(registry.commands().contains(&"test"));
    }

    #[test]
    fn test_get_returns_registered_handler() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("test", "Test command"));

        registry.register(handler);

        let retrieved = registry.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test");
    }

    #[test]
    fn test_get_returns_none_for_unregistered() {
        let registry = CommandRegistry::new();
        let retrieved = registry.get("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_commands_returns_all_names() {
        let mut registry = CommandRegistry::new();
        let handler1 = Arc::new(MockHandler::new("cmd1", "Command 1"));
        let handler2 = Arc::new(MockHandler::new("cmd2", "Command 2"));

        registry.register(handler1);
        registry.register(handler2);

        let commands = registry.commands();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"cmd1"));
        assert!(commands.contains(&"cmd2"));
    }

    #[test]
    fn test_register_overwrites_duplicate() {
        let mut registry = CommandRegistry::new();
        let handler1 = Arc::new(MockHandler::new("test", "First"));
        let handler2 = Arc::new(MockHandler::new("test", "Second"));

        registry.register(handler1);
        registry.register(handler2);

        assert_eq!(registry.commands().len(), 1);
        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.description(), "Second");
    }

    #[test]
    fn test_generate_help_includes_command_name() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("help", "Show help"));

        registry.register(handler);

        let help = registry.generate_help();
        assert!(help.contains("!help"));
    }

    #[test]
    fn test_generate_help_includes_description() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("test", "This is a test command"));

        registry.register(handler);

        let help = registry.generate_help();
        assert!(help.contains("This is a test command"));
    }

    #[test]
    fn test_generate_help_includes_permission_when_not_anyone() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::with_permission(
            "admin",
            "Admin command",
            Permission::BotOwner,
        ));

        registry.register(handler);

        let help = registry.generate_help();
        assert!(help.contains("权限"));
    }

    #[test]
    fn test_generate_help_html_valid_html() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("test", "Test command"));

        registry.register(handler);

        let html = registry.generate_help_html();
        assert!(html.contains("<"));
        assert!(html.contains(">"));
        assert!(html.contains("!test"));
    }
}
