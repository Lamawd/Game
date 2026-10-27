/// Broadcast helpers — gửi ServerMessage đến player cụ thể hoặc toàn lobby.
///
/// Mỗi WebSocket connection khi kết nối sẽ đăng ký một
/// `tokio::sync::mpsc::UnboundedSender<ServerMessage>` vào AppState.
/// Game loop và các handler dùng các hàm này để push message về client.
use uuid::Uuid;

use crate::AppState;
use crate::ws::messages::ServerMessage;

/// Gửi message đến một player cụ thể.
pub async fn send_to_player(state: &AppState, player_id: Uuid, msg: ServerMessage) {
    if let Some(tx) = state.player_senders.get(&player_id) {
        // Nếu channel đã đóng (player disconnect) thì ignore
        let _ = tx.send(msg);
    }
}

/// Gửi message đến tất cả player trong một lobby.
pub async fn broadcast_to_lobby(state: &AppState, lobby_id: Uuid, msg: ServerMessage) {
    let lobby = match state.lobby_manager.get(lobby_id) {
        Some(l) => l,
        None => return,
    };
    for pid in &lobby.player_ids {
        send_to_player(state, *pid, msg.clone()).await;
    }
}
