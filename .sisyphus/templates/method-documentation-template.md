# Method Documentation Template

This template follows the documentation patterns established in `src/ai_service.rs`, `src/traits.rs`, `src/media.rs`, and `src/bot.rs`.

## Basic Template

```rust
/// [One-line summary of what the method does.]
///
/// [Optional: More detailed description of behavior, use cases, or important notes.
/// Can span multiple lines. Include any important behavioral details here.]
///
/// # Arguments
///
/// * `param1` - [Type and purpose of parameter]
/// * `param2` - [Type and purpose of parameter, e.g., `"user id"` for string params]
/// * `param3` - [Type and purpose, note if optional with `None` default behavior]
///
/// # Returns
///
/// [Description of return value. Specify type and meaning.]
/// [If the return type is complex, explain each component.]
///
/// # Errors
///
/// [List all error conditions. Use bullet points if multiple.]
/// - [Error condition 1]
/// - [Error condition 2]
///
/// # Example
///
/// ```
/// use your_crate::module::method_name;
///
/// // Basic usage
/// let result = method_name(arg1, arg2).await?;
///
/// // Or for methods that need setup:
/// // let data = your_function(&client, "mxc://example.org/abc", None, 1024).await?;
/// ```
pub async fn your_method(&self, param1: &str, param2: Option<u32>) -> Result<String> {
    // implementation
}
```

## Real Examples from the Codebase

### Example 1: Simple Method (from traits.rs)

```rust
/// 追加文本到累积内容。
///
/// # Arguments
///
/// * `delta` - 要追加的文本片段
///
/// # Example
///
/// ```
/// use aether_matrix::traits::StreamingState;
///
/// let mut state = StreamingState::new();
/// state.append("Hello");
/// state.append(" World");
/// assert_eq!(state.content(), "Hello World");
/// ```
pub fn append(&mut self, delta: &str) {
    self.accumulated.push_str(delta);
}
```

### Example 2: Method with All Sections (from media.rs)

```rust
/// 下载 Matrix 媒体并转换为 base64 data URL。
///
/// 从 Matrix 服务器下载图片，按需缩放，然后编码为 base64 data URL。
/// 支持 Matrix Content (MXC) URI 格式。
///
/// # Arguments
///
/// * `client` - Matrix 客户端引用
/// * `mxc_uri` - Matrix 内容 URI，格式如 `mxc://server/media_id`
/// * `_expected_media_type` - 预期的媒体类型（可选，用于验证）
/// * `max_size` - 图片最大边长（像素），超过时会自动缩放
///
/// # Returns
///
/// 成功时返回 base64 编码的 data URL，格式为 `data:image/png;base64,{data}`
/// 缩放后的图片统一输出为 PNG 格式。
///
/// # Errors
///
/// 当以下情况发生时返回错误：
/// - MXC URI 无效
/// - 图片下载失败
/// - 图片解析或缩放失败
///
/// # Example
///
/// ```
/// use aether_matrix::media::download_image_as_base64;
///
/// // let data_url = download_image_as_base64(
/// //     &client,
/// //     "mxc://matrix.org/abc123",
/// //     Some("image/png"),
/// //     1024,  // 最大 1024 像素
/// // ).await?;
/// ```
pub async fn download_image_as_base64(
    client: &Client,
    mxc_uri: &MxcUri,
    _expected_media_type: Option<&str>,
    max_size: u32,
) -> Result<String> {
    // implementation
}
```

### Example 3: Async Method with Trait Bound (from traits.rs)

```rust
/// 执行普通（非流式）聊天。
///
/// 发送用户消息并返回 AI 的完整回复。
///
/// # Arguments
///
/// * `session_id` - 会话标识符，用于隔离不同用户/房间的对话
/// * `prompt` - 用户输入的消息内容
///
/// # Returns
///
/// 成功时返回 AI 的完整回复文本。
///
/// # Errors
///
/// 当 API 调用失败时返回错误。
fn chat(&self, session_id: &str, prompt: &str) -> impl Future<Output = Result<String>> + Send;
```

### Example 4: Constructor with Complex Logic (from bot.rs)

```rust
/// 创建并初始化 Bot 实例。
///
/// 初始化流程：
/// 1. 创建 Matrix 客户端
/// 2. 检查是否存在已保存的会话
/// 3. 如无会话则执行登录
/// 4. 创建 AI 服务和事件处理器
///
/// # Arguments
///
/// * `config` - 机器人配置
///
/// # Returns
///
/// 成功时返回初始化完成的 `Bot` 实例。
///
/// # Errors
///
/// 当以下情况发生时返回错误：
/// - Matrix 客户端构建失败
/// - 登录失败
/// - 获取用户 ID 失败
pub async fn new(config: Config) -> Result<Self> {
    // implementation
}
```

### Example 5: Method with Extra Section (from media.rs)

```rust
/// 缩放图片（如果超过最大尺寸）。
///
/// 保持宽高比，将图片缩放到最大边不超过 `max_size`。
/// 如果图片已经足够小，则不做任何处理。
///
/// # Arguments
///
/// * `image_data` - 原始图片数据
/// * `max_size` - 最大边长（像素）
///
/// # Returns
///
/// 成功时返回缩放后的 PNG 格式图片数据。
///
/// # Errors
///
/// 当图片数据无效时返回错误。
///
/// # Example
///
/// ```
/// use aether_matrix::media::resize_image_if_needed;
///
/// // 假设 image_data 是有效的图片数据
/// // let resized = resize_image_if_needed(&image_data, 1024)?;
/// ```
///
/// # Algorithm
///
/// 使用 Lanczos3 算法进行高质量缩放，适合照片和复杂图像。
/// 对于简单的图标或线条图，可能产生轻微的模糊，但整体效果优于其他算法。
pub fn resize_image_if_needed(image_data: &[u8], max_size: u32) -> Result<Vec<u8>> {
    // implementation
}
```

## Section Guidelines

### Summary Line
- Start with a verb (创建, 执行, 获取, 检查, etc.)
- One line, no period at the end
- Be concise but informative

### Detailed Description (Optional)
- Explain behavior, use cases, side effects
- Mention important implementation details
- Can use numbered lists for multi-step processes

### Arguments
- Use `* \`param_name\` - description` format
- Include type hints in description when helpful: `"session_id" - 会话标识符`
- Note default behavior for `Option` parameters
- Omit this section for methods with no parameters

### Returns
- Describe what the method returns
- For complex types, explain each component
- Omit this section if return type is self-explanatory (e.g., `()`)

### Errors
- List all possible error conditions
- Use bullet points for multiple conditions
- Start conditions with "当" or just list them
- Omit this section for infallible methods

### Example
- Include runnable doctest when possible
- Use `//` comments for setup code that can't compile in isolation
- Show basic usage first, then optional advanced usage
- Omit this section for trivial methods (getters/setters)

### Extra Sections
Common additional sections:
- `# Algorithm` - For methods with notable algorithms
- `# Panics` - For methods that can panic
- `# Safety` - For unsafe methods
- `# Performance` - For performance-critical methods

## Writing Tips

1. **Be consistent** - Follow the same pattern across similar methods
2. **Be precise** - Use specific types and values in descriptions
3. **Be helpful** - Include edge cases and common pitfalls
4. **Be concise** - Avoid redundancy between sections
5. **Use Chinese** - The codebase uses Chinese documentation; maintain consistency