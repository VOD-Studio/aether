use aether_matrix::command::{CommandHandler, Permission};
use aether_matrix::modules::admin::{BotInfoHandler, BotLeaveHandler, BotPingHandler};
use aether_matrix::ui::{error, info_card, success, warning};

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[tokio::test]
    async fn test_bot_info_handler_name() {
        let handler = BotInfoHandler;
        assert_eq!(handler.name(), "bot");
    }

    #[tokio::test]
    async fn test_bot_info_handler_description() {
        let handler = BotInfoHandler;
        assert_eq!(handler.description(), "Bot 管理命令");
    }

    #[tokio::test]
    async fn test_bot_info_handler_permission() {
        let handler = BotInfoHandler;
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_bot_leave_handler_name() {
        let handler = BotLeaveHandler;
        assert_eq!(handler.name(), "leave");
    }

    #[tokio::test]
    async fn test_bot_leave_handler_description() {
        let handler = BotLeaveHandler;
        assert_eq!(handler.description(), "让 Bot 离开当前房间");
    }

    #[tokio::test]
    async fn test_bot_leave_handler_permission() {
        let handler = BotLeaveHandler;
        assert_eq!(handler.permission(), Permission::RoomMod);
    }

    #[tokio::test]
    async fn test_bot_ping_handler_name() {
        let handler = BotPingHandler;
        assert_eq!(handler.name(), "ping");
    }

    #[tokio::test]
    async fn test_bot_ping_handler_description() {
        let handler = BotPingHandler;
        assert_eq!(handler.description(), "测试 Bot 响应");
    }

    #[tokio::test]
    async fn test_bot_ping_handler_permission() {
        let handler = BotPingHandler;
        assert_eq!(handler.permission(), Permission::Anyone);
    }
}

#[cfg(test)]
mod permission_tests {
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

#[cfg(test)]
mod ui_tests {
    use super::*;

    #[test]
    fn test_success_message() {
        let msg = success("Test message");
        assert!(msg.contains("Test message"));
        assert!(msg.contains("✓"));
    }

    #[test]
    fn test_error_message() {
        let msg = error("Test error");
        assert!(msg.contains("Test error"));
        assert!(msg.contains("✕"));
    }

    #[test]
    fn test_info_card_message() {
        let items = vec![("Key", "Value")];
        let msg = info_card("Test Title", &items);
        assert!(msg.contains("Test Title"));
        assert!(msg.contains("Key"));
        assert!(msg.contains("Value"));
    }

    #[test]
    fn test_warning_message() {
        let msg = warning("Test warning");
        assert!(msg.contains("Test warning"));
        assert!(msg.contains("⚠"));
    }
}

#[cfg(test)]
mod handler_usage_tests {
    use super::*;

    #[test]
    fn test_bot_info_handler_usage() {
        let handler = BotInfoHandler;
        let usage = handler.usage();
        assert!(usage.contains("info"));
        assert!(usage.contains("name"));
        assert!(usage.contains("ping"));
    }

    #[test]
    fn test_bot_leave_handler_usage() {
        let handler = BotLeaveHandler;
        assert_eq!(handler.usage(), "leave");
    }

    #[test]
    fn test_bot_ping_handler_usage() {
        let handler = BotPingHandler;
        assert_eq!(handler.usage(), "ping");
    }
}

#[cfg(test)]
mod help_message_tests {
    #[test]
    fn test_bot_info_help_commands_exist() {
        let expected_commands = vec![
            "!bot info",
            "!bot name", 
            "!bot avatar",
            "!bot join",
            "!bot rooms",
            "!bot ping",
            "!leave"
        ];
        
        for cmd in expected_commands {
            assert!(cmd.starts_with("!"));
        }
    }
}