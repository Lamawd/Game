use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::models::board::Board;
use crate::models::player::Player;

#[derive(Deserialize)]
pub struct RegisterPlayerRequest {
    pub username: String,
}

#[derive(Serialize)]
pub struct PlayerResponse {
    pub player: Player,
}

/// POST /api/v1/player/register
/// Tạo player mới và lưu vào AppState để ws/handler có thể tìm thấy.
pub async fn register_player(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterPlayerRequest>,
) -> Json<PlayerResponse> {
    let player = Player::new(req.username);
    // Khởi tạo board trống cho player luôn
    state.boards.insert(player.id, Board::new(player.id));
    state.streaks.insert(player.id, (0, 0));
    state.players.insert(player.id, player.clone());
    Json(PlayerResponse { player })
}

/// GET /api/v1/player/:id
pub async fn get_player(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    state
        .players
        .get(&id)
        .map(|p| Json(PlayerResponse { player: p.clone() }))
        .ok_or(StatusCode::NOT_FOUND)
}
