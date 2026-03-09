# Rust Struct Documentation Template

This template establishes the standard for documenting data structures in the Aether project. Follow the exact pattern from `src/config.rs` for all public structs.

## Template Structure

```rust
/// <BRIEF SUMMARY LINE>.
///
/// <DETAILED DESCRIPTION explaining the struct's purpose and role in the system.
/// This can span multiple lines and should give developers a clear understanding
/// of when and how to use this struct.>
///
/// # <SECTION NAME> (e.g., 必需字段, 可选字段, Example)
///
/// <Section content - use bullet points or numbered lists as appropriate>
///
/// # Example
///
/// ```no_run
/// use crate::module::StructName;
///
/// // Show typical usage
/// let instance = StructName::new(/* params */);
/// // Demonstrate key operations
/// ```
#[derive(Debug, Clone, /* other traits */)]
pub struct StructName {
    /// <FIELD SUMMARY LINE>.
    ///
    /// <Optional detailed description explaining:
    /// - Purpose and usage
    /// - Constraints or validation rules
    /// - Default value if applicable
    /// - Examples of valid values>
    ///
    /// <Optional: Example value or format>
    #[serde(default = "default_function_name")]
    pub field_name: FieldType,

    /// <NEXT FIELD SUMMARY LINE>.
    ///
    /// <Detailed description...>
    pub another_field: AnotherType,
}

impl StructName {
    /// <METHOD SUMMARY LINE>.
    ///
    /// <Detailed description of what the method does, when to use it,
    /// and any important behavior notes.>
    ///
    /// # Arguments
    ///
    /// * `param1` - <Description of parameter 1>
    /// * `param2` - <Description of parameter 2>
    ///
    /// # Returns
    ///
    /// <Description of the return value>
    ///
    /// # Errors
    ///
    /// <Description of error conditions, if any>
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crate::module::StructName;
    ///
    /// let result = StructName::method_name(/* params */);
    /// ```
    pub fn method_name(&self, param1: Type1, param2: Type2) -> Result<ReturnType> {
        // implementation
    }
}
```

## Documentation Patterns

### 1. Struct-Level Documentation

**Required Elements:**
- Summary line (ending with `.`)
- Detailed description of purpose
- Sections for required/optional fields or configuration
- Example section with runnable code

**Example from config.rs:**
```rust
/// Matrix AI 机器人的配置结构体。
///
/// 包含连接 Matrix 服务器、调用 AI API 以及控制机器人行为所需的全部配置项。
/// 配置可通过 TOML 文件或环境变量加载，详见 [`Config::load`]。
///
/// # 必需配置
///
/// - `matrix.homeserver`: Matrix 服务器地址
/// - `matrix.username`: Matrix 用户名
/// - `matrix.password`: Matrix 密码
/// - `openai.api_key`: OpenAI API 密钥
///
/// # 可选配置
///
/// 所有可选配置都有合理的默认值，详见各字段文档。
///
/// # Example
///
/// ```no_run
/// use aether_matrix::config::Config;
///
/// let config = Config::load("config.toml").expect("配置加载失败");
/// assert!(!config.matrix.homeserver.is_empty());
/// ```
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
```

### 2. Field-Level Documentation

**Pattern for required fields:**
```rust
/// Matrix 服务器地址（必需）。
///
/// 示例: `https://matrix.org`
#[serde(default)]
pub homeserver: String,
```

**Pattern for optional fields:**
```rust
/// Matrix 设备 ID（可选）。
///
/// 设置固定的设备 ID 可以避免重复登录创建新设备。
/// 建议使用一个有意义的标识符，如 `AETHER_BOT_001`。
pub device_id: Option<String>,
```

**Pattern for fields with defaults:**
```rust
/// 命令前缀。
///
/// 在群聊中触发 AI 响应的前缀，默认为 `!`。
#[serde(default = "default_command_prefix")]
pub command_prefix: String,

/// 最大历史轮数。
///
/// 每个会话保留的最大对话轮数（一轮 = 一问一答）。
/// 超出限制时会自动丢弃最早的历史。
#[serde(default = "default_max_history")]
pub max_history: usize,
```

### 3. Method Documentation

**Constructor methods:**
```rust
/// 从配置文件和环境变量加载配置。
///
/// 加载顺序（优先级从高到低）：
/// 1. 环境变量
/// 2. TOML 配置文件
/// 3. `.env` 文件
/// 4. 代码默认值
///
/// # Arguments
///
/// * `path` - 配置文件路径
///
/// # Returns
///
/// 成功时返回填充好的 `Config` 实例。
///
/// # Errors
///
/// 当以下必需字段未设置时返回错误：
/// - `matrix.homeserver` / `MATRIX_HOMESERVER`
/// - `matrix.username` / `MATRIX_USERNAME`
/// - `matrix.password` / `MATRIX_PASSWORD`
/// - `openai.api_key` / `OPENAI_API_KEY`
///
/// # Example
///
/// ```no_run
/// use aether_matrix::config::Config;
///
/// let config = Config::load("config.toml").expect("配置加载失败");
/// ```
pub fn load(path: &str) -> Result<Self> {
```

**Simple getter/query methods:**
```rust
/// 根据 ID 获取人设。
///
/// 查询数据库中指定 ID 的人设记录。
///
/// # Arguments
///
/// * `id` - 人设唯一标识符
///
/// # Returns
///
/// - `Some(Persona)` - 找到匹配的人设
/// - `None` - 未找到匹配的人设
pub fn get_by_id(&self, id: &str) -> Result<Option<Persona>> {
```

**Mutation methods:**
```rust
/// 设置房间人设。
///
/// 将指定人设绑定到房间，影响该房间的 AI 响应风格。
/// 如果房间已有设置的人设，将更新为新人设。
///
/// # Arguments
///
/// * `room_id` - Matrix 房间 ID
/// * `persona_id` - 人设唯一标识符
/// * `set_by` - 执行设置的用户 ID
///
/// # Errors
///
/// 当数据库操作失败时返回错误。
pub fn set_room_persona(&self, room_id: &str, persona_id: &str, set_by: &str) -> Result<()> {
```

### 4. Cross-References

Use `[`Item`]` syntax to create links to other items:

```rust
/// 配置可通过 TOML 文件或环境变量加载，详见 [`Config::load`]。
///
/// 人设存储由 [`PersonaStore`] 管理，支持内置和自定义人设。
/// 详见 [`Persona`] 结构体文档。
```

### 5. Serde Attributes Documentation

Document `#[serde(...)]` attributes inline with the field:

```rust
/// 流式输出配置。
#[serde(default)]
pub streaming: StreamingConfig,

/// HTTP 代理 URL（可选）。
pub proxy: Option<String>,
```

## Style Guidelines

### Language
- Use Chinese for documentation in this project
- Summary lines should be concise (one sentence)
- Detailed descriptions can span multiple paragraphs

### Formatting
- Use `///` for doc comments (not `//!` except for module docs)
- Blank line after summary
- Indent example code properly
- Use `no_run` for examples that require external dependencies

### Sections
Common sections in order:
1. Summary line
2. Detailed description
3. `# 必需字段/Required Fields`
4. `# 可选字段/Optional Fields`
5. `# Example`
6. `# Arguments` (for methods)
7. `# Returns` (for methods)
8. `# Errors` (for methods that can fail)
9. `# Panics` (for methods that can panic, rarely used)

### Field Documentation Checklist
- [ ] Summary line with period
- [ ] `(必需)` or `(可选)` marker in summary
- [ ] Default value mentioned if applicable
- [ ] Constraints or validation rules
- [ ] Example values for complex types

### Method Documentation Checklist
- [ ] Summary line with period
- [ ] Detailed behavior description
- [ ] Arguments section with each parameter documented
- [ ] Returns section describing return value
- [ ] Errors section listing error conditions
- [ ] Example section with runnable code

## Complete Example

```rust
/// 人设定义，包含 AI 助手的性格和行为配置。
///
/// 人设用于自定义 AI 的响应风格和系统提示词。
/// 每个房间可以设置独立的人设，影响该房间的对话体验。
///
/// # 内置人设
///
/// 项目预置了 4 个内置人设：
/// - `sarcastic-dev`: 毒舌程序员
/// - `cyber-zen`: 赛博禅师
/// - `wiki-chan`: 维基百科娘
/// - `neko-chan`: 猫娘助手
///
/// # 自定义人设
///
/// 用户可以通过 `!persona create` 命令创建自定义人设，
/// 自定义人设存储在数据库中，可以被删除。
///
/// # Example
///
/// ```no_run
/// use aether_matrix::store::Persona;
///
/// let persona = Persona {
///     id: "my-assistant".to_string(),
///     name: "我的助手".to_string(),
///     system_prompt: "你是一个专业的 Rust 开发顾问...".to_string(),
///     avatar_emoji: Some("🦀".to_string()),
///     is_builtin: false,
///     created_by: Some("@user:matrix.org".to_string()),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Persona {
    /// 人设唯一标识符。
    ///
    /// 用于在命令中引用人设，如 `!persona set sarcastic-dev`。
    /// 内置人设使用连字符分隔的小写字母，如 `sarcastic-dev`。
    /// 自定义人设建议使用相同的命名风格。
    pub id: String,

    /// 人设显示名称。
    ///
    /// 在人设列表和状态中显示的友好名称。
    /// 示例: "毒舌程序员", "赛博禅师"
    pub name: String,

    /// 系统提示词。
    ///
    /// 发送给 AI 模型的系统提示，定义 AI 的行为和响应风格。
    /// 应该详细描述人设的性格、语气和专业领域。
    pub system_prompt: String,

    /// 头像 Emoji。
    ///
    /// 用于在人设列表中显示的可选表情符号。
    /// 示例: Some("💻"), Some("🐱"), None
    pub avatar_emoji: Option<String>,

    /// 是否为内置人设。
    ///
    /// 内置人设由系统预置，不可删除。
    /// 自定义人设由用户创建，可以删除。
    pub is_builtin: bool,

    /// 创建者用户 ID。
    ///
    /// 对于内置人设，此值为 `None`。
    /// 对于自定义人设，记录创建者的 Matrix 用户 ID。
    pub created_by: Option<String>,
}
```

## When to Use This Template

Apply this documentation pattern to:
- All public structs
- All public struct fields
- All public methods and functions
- All public enums and variants

**Exceptions:**
- Trivial getter/setter methods may omit detailed docs
- Private items only need documentation if complex

## Verification

Run `cargo doc --open` to verify documentation renders correctly.
Check for:
- All public items have documentation
- Links resolve correctly
- Code examples compile (use `cargo test --doc`)