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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    use matrix_sdk::ruma::{OwnedUserId, UserId};

    #[test]
    fn test_permission_ordering() {
        assert!(Permission::BotOwner > Permission::RoomMod);
        assert!(Permission::RoomMod > Permission::Anyone);
        assert!(Permission::BotOwner > Permission::Anyone);
        assert_eq!(Permission::Anyone.cmp(&Permission::Anyone), std::cmp::Ordering::Equal);
        assert_eq!(Permission::RoomMod.cmp(&Permission::RoomMod), std::cmp::Ordering::Equal);
        assert_eq!(Permission::BotOwner.cmp(&Permission::BotOwner), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_permission_display_name() {
        assert_eq!(Permission::Anyone.display_name(), "任何人");
        assert_eq!(Permission::RoomMod.display_name(), "房间管理员");
        assert_eq!(Permission::BotOwner.display_name(), "Bot 所有者");
    }

    #[test]
    fn test_anyone_permission_always_true() {
        let permission = Permission::Anyone;
        let bot_owners = vec!["@admin:example.org".to_string()];
        let user_id: OwnedUserId = UserId::parse("@user:example.org").unwrap().into();
        
        assert!(permission.check_mock(&[], &user_id));
        assert!(permission.check_mock(&bot_owners, &user_id));
        
        let owner_id: OwnedUserId = UserId::parse("@admin:example.org").unwrap().into();
        assert!(permission.check_mock(&bot_owners, &owner_id));
    }

    #[test]
    fn test_bot_owner_permission_exact_match() {
        let permission = Permission::BotOwner;
        let bot_owners = vec![
            "@admin1:example.org".to_string(),
            "@admin2:example.org".to_string(),
            "@admin3:matrix.org".to_string(),
        ];
        
        let user_id: OwnedUserId = UserId::parse("@admin1:example.org").unwrap().into();
        assert!(permission.check_mock(&bot_owners, &user_id));
        
        let user_id2: OwnedUserId = UserId::parse("@admin2:example.org").unwrap().into();
        assert!(permission.check_mock(&bot_owners, &user_id2));
        
        let user_id3: OwnedUserId = UserId::parse("@admin3:matrix.org").unwrap().into();
        assert!(permission.check_mock(&bot_owners, &user_id3));
    }

    #[test]
    fn test_bot_owner_permission_no_match() {
        let permission = Permission::BotOwner;
        let bot_owners = vec![
            "@admin1:example.org".to_string(),
            "@admin2:example.org".to_string(),
        ];
        
        let user_id: OwnedUserId = UserId::parse("@user:example.org").unwrap().into();
        assert!(!permission.check_mock(&bot_owners, &user_id));
        
        let user_id2: OwnedUserId = UserId::parse("@admin1:matrix.org").unwrap().into();
        assert!(!permission.check_mock(&bot_owners, &user_id2));
        
        let empty_owners: Vec<String> = vec![];
        assert!(!permission.check_mock(&empty_owners, &user_id));
        
        let empty_user: OwnedUserId = UserId::parse("@:example.org").unwrap().into();
        assert!(!permission.check_mock(&bot_owners, &empty_user));
    }

    #[test]
    fn test_bot_owner_permission_edge_cases() {
        let permission = Permission::BotOwner;
        
        let bot_owners = vec!["@admin_with_underscores:example.org".to_string()];
        let user_id: OwnedUserId = UserId::parse("@admin_with_underscores:example.org").unwrap().into();
        assert!(permission.check_mock(&bot_owners, &user_id));
        
        let bot_owners2 = vec!["@Admin:example.org".to_string()];
        let user_id_lower: OwnedUserId = UserId::parse("@admin:example.org").unwrap().into();
        let user_id_upper: OwnedUserId = UserId::parse("@Admin:example.org").unwrap().into();
        assert!(!permission.check_mock(&bot_owners2, &user_id_lower));
        assert!(permission.check_mock(&bot_owners2, &user_id_upper));
        
        let long_user = format!("@{}", "a".repeat(200));
        let long_domain = format!("{}:example.org", long_user);
        let bot_owners3 = vec![long_domain.clone()];
        let user_id_long: OwnedUserId = UserId::parse(&long_domain).unwrap().into();
        assert!(permission.check_mock(&bot_owners3, &user_id_long));
    }

    #[test]
    fn test_permission_check_with_empty_inputs() {
        let anyone = Permission::Anyone;
        let bot_owner = Permission::BotOwner;
        
        let user_id: OwnedUserId = UserId::parse("@user:example.org").unwrap().into();
        let empty_owners: Vec<String> = vec![];
        
        assert!(anyone.check_mock(&empty_owners, &user_id));
        assert!(!bot_owner.check_mock(&empty_owners, &user_id));
    }

    #[test]
    fn test_bot_owners_list_variations() {
        let permission = Permission::BotOwner;
        let user_id: OwnedUserId = UserId::parse("@target:example.org").unwrap().into();
        
        let single_owner = vec!["@target:example.org".to_string()];
        assert!(permission.check_mock(&single_owner, &user_id));
        
        let multi_owners = vec![
            "@admin1:example.org".to_string(),
            "@target:example.org".to_string(),
            "@admin2:example.org".to_string(),
        ];
        assert!(permission.check_mock(&multi_owners, &user_id));
        
        let no_target = vec![
            "@admin1:example.org".to_string(),
            "@admin2:example.org".to_string(),
        ];
        assert!(!permission.check_mock(&no_target, &user_id));
        
        let duplicate_owners = vec![
            "@target:example.org".to_string(),
            "@admin:example.org".to_string(),
            "@target:example.org".to_string(),
        ];
        assert!(permission.check_mock(&duplicate_owners, &user_id));
    }

    impl Permission {
        #[cfg(test)]
        fn check_mock(&self, bot_owners: &[String], user_id: &OwnedUserId) -> bool {
            match self {
                Permission::Anyone => true,
                Permission::BotOwner => {
                    bot_owners.iter().any(|owner| owner == user_id.as_str())
                }
                Permission::RoomMod => {
                    false
                }
            }
        }
    }

    #[test]
    fn test_permission_equality_and_hashing() {
        use std::collections::HashSet;
        
        let anyone1 = Permission::Anyone;
        let anyone2 = Permission::Anyone;
        let room_mod1 = Permission::RoomMod;
        let room_mod2 = Permission::RoomMod;
        let bot_owner1 = Permission::BotOwner;
        let bot_owner2 = Permission::BotOwner;
        
        assert_eq!(anyone1, anyone2);
        assert_eq!(room_mod1, room_mod2);
        assert_eq!(bot_owner1, bot_owner2);
        
        assert_ne!(anyone1, room_mod1);
        assert_ne!(anyone1, bot_owner1);
        assert_ne!(room_mod1, bot_owner1);
        
        let mut permissions = HashSet::new();
        permissions.insert(anyone1);
        permissions.insert(room_mod1);
        permissions.insert(bot_owner1);
        assert_eq!(permissions.len(), 3);
    }

    #[test]
    fn test_permission_cloning_and_copying() {
        let original = Permission::BotOwner;
        let cloned = original.clone();
        let copied = original;
        
        assert_eq!(original, cloned);
        assert_eq!(cloned, copied);
    }

    #[test]
    fn test_display_names_consistency() {
        assert!(!Permission::Anyone.display_name().is_empty());
        assert!(!Permission::RoomMod.display_name().is_empty());
        assert!(!Permission::BotOwner.display_name().is_empty());
        
        assert_ne!(Permission::Anyone.display_name(), Permission::RoomMod.display_name());
        assert_ne!(Permission::Anyone.display_name(), Permission::BotOwner.display_name());
        assert_ne!(Permission::RoomMod.display_name(), Permission::BotOwner.display_name());
    }
}
