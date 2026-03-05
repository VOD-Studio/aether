//! 房间邀请处理。

use anyhow::Result;
use matrix_sdk::ruma::events::room::member::{MembershipState, StrippedRoomMemberEvent};
use matrix_sdk::{Client, Room};
use tracing::{info, warn};

/// 处理房间邀请事件。
///
/// 当机器人收到加入房间的邀请时自动加入。这是独立函数而非方法，
/// 以便在事件处理器注册时直接使用。
///
/// # Arguments
///
/// * `ev` - 房间成员事件（邀请）
/// * `client` - Matrix 客户端实例
/// * `room` - 发送邀请的房间
///
/// # Returns
///
/// 成功时返回 `Ok(())`，失败时返回错误。
///
/// # Example
///
/// ```ignore
/// client.add_event_handler(
///     |ev: StrippedRoomMemberEvent, client: Client, room: Room| async move {
///         if let Err(e) = handle_invite(ev, client, room).await {
///             tracing::error!("处理邀请失败: {}", e);
///         }
///     }
/// );
/// ```
pub async fn handle_invite(ev: StrippedRoomMemberEvent, client: Client, room: Room) -> Result<()> {
    // 只处理邀请事件，忽略其他成员状态变更
    if ev.content.membership != MembershipState::Invite {
        return Ok(());
    }

    let user_id = &ev.state_key;
    let my_user_id = client.user_id().expect("user_id should be available");

    // 只处理邀请自己的事件
    if user_id != my_user_id {
        return Ok(()); // 不是邀请机器人
    }

    let room_id = room.room_id();
    info!("收到房间邀请: {}", room_id);

    match client.join_room_by_id(room_id).await {
        Ok(_) => info!("成功加入房间: {}", room_id),
        Err(e) => warn!("加入房间失败: {}", e),
    }

    Ok(())
}
