//! # 权限模型
//!
//! 定义命令执行的权限级别和检查逻辑。

use matrix_sdk::Room;
use matrix_sdk::ruma::OwnedUserId;

/// 权限级别枚举。
///
/// 定义了三个权限级别，从小到大依次为：
/// - `Anyone`: 任何房间成员
/// - `RoomMod`: 房间管理员（power_level >= 50）
/// - `BotOwner`: Bot 所有者
///
/// # 权限继承
///
/// 权限级别支持比较，高级别自动拥有低级别的权限：
///
/// ```ignore
/// assert!(Permission::BotOwner > Permission::RoomMod);
/// assert!(Permission::RoomMod > Permission::Anyone);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    /// 任何房间成员都可以执行。
    ///
    /// 适用于公开命令，如 `!help`。
    Anyone,
    /// 仅房间管理员可以执行（power_level >= 50）。
    ///
    /// 在私聊房间中，所有用户都视为有此权限。
    /// 适用于房间管理命令，如 `!leave`。
    RoomMod,
    /// 仅 Bot 所有者可以执行。
    ///
    /// 适用于敏感操作，如配置修改、系统命令。
    BotOwner,
}

impl Permission {
    /// 检查用户是否具有此权限。
    ///
    /// # Arguments
    ///
    /// * `room` - 消息来源房间，用于判断私聊和房间成员状态
    /// * `user_id` - 要检查的用户 ID
    /// * `bot_owners` - Bot 所有者列表
    ///
    /// # Returns
    ///
    /// 如果用户具有此权限返回 `true`，否则返回 `false`。
    ///
    /// # 规则
    ///
    /// - `Anyone`: 总是返回 `true`
    /// - `RoomMod`: 私聊房间返回 `true`，群聊房间检查用户是否是房间成员
    /// - `BotOwner`: 检查用户是否在 `bot_owners` 列表中
    ///
    /// # Example
    ///
    /// ```ignore
    /// let permission = Permission::RoomMod;
    /// let has_permission = permission.check(&room, &user_id, &bot_owners).await;
    /// ```
    pub async fn check(&self, room: &Room, user_id: &OwnedUserId, bot_owners: &[String]) -> bool {
        match self {
            Permission::Anyone => true,
            Permission::RoomMod => {
                // 私聊房间允许所有操作
                if room.is_direct().await.unwrap_or(false) {
                    return true;
                }
                // 检查用户是否是房间成员
                // get_member 返回 Option<RoomMember>，如果用户是成员则返回 Some
                room.get_member(user_id.as_ref())
                    .await
                    .ok()
                    .flatten()
                    .is_some()
            }
            Permission::BotOwner => {
                // 检查用户是否是 Bot 所有者
                bot_owners.iter().any(|owner| owner == user_id.as_str())
            }
        }
    }

    /// 获取权限级别的显示名称。
    ///
    /// 用于错误消息和帮助信息。
    ///
    /// # Returns
    ///
    /// 返回权限级别的中文显示名称。
    ///
    /// # Example
    ///
    /// ```
    /// use aether_matrix::command::Permission;
    ///
    /// assert_eq!(Permission::Anyone.display_name(), "任何人");
    /// assert_eq!(Permission::RoomMod.display_name(), "房间管理员");
    /// assert_eq!(Permission::BotOwner.display_name(), "Bot 所有者");
    /// ```
    pub fn display_name(&self) -> &'static str {
        match self {
            Permission::Anyone => "任何人",
            Permission::RoomMod => "房间管理员",
            Permission::BotOwner => "Bot 所有者",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_ordering() {
        assert!(Permission::BotOwner > Permission::RoomMod);
        assert!(Permission::RoomMod > Permission::Anyone);
    }

    #[test]
    fn test_permission_display_name() {
        assert_eq!(Permission::Anyone.display_name(), "任何人");
        assert_eq!(Permission::RoomMod.display_name(), "房间管理员");
        assert_eq!(Permission::BotOwner.display_name(), "Bot 所有者");
    }
}
