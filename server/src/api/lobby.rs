use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::game::bot;
use crate::game::game_loop;
use crate::models::board::Board;
use crate::models::lobby::{Lobby, LobbyStatus};
use crate::models::player::Player;

#[derive(Deserialize)]
pub struct CreateLobbyRequest {
    pub host_player_id: Uuid,
    pub max_players: Option<u8>,
}

#[derive(Serialize)]
pub struct LobbyResponse {
    pub lobby: Lobby,
}

pub async fn create_lobby(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateLobbyRequest>,
) -> Result<Json<LobbyResponse>, StatusCode> {
    let max = req.max_players.unwrap_or(8).min(8);
    // Kiểm tra host đã register chưa
    if !state.players.contains_key(&req.host_player_id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let lobby = state.lobby_manager.create(req.host_player_id, max);
    Ok(Json(LobbyResponse { lobby }))
}

pub async fn get_lobby(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<LobbyResponse>, StatusCode> {
    state.lobby_manager.get(id)
        .map(|lobby| Json(LobbyResponse { lobby }))
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
pub struct JoinLobbyRequest {
    pub player_id: Uuid,
}

pub async fn join_lobby(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<JoinLobbyRequest>,
) -> Result<Json<LobbyResponse>, StatusCode> {
    if !state.players.contains_key(&req.player_id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.lobby_manager.join(id, req.player_id)
        .map(|lobby| Json(LobbyResponse { lobby }))
        .ok_or(StatusCode::BAD_REQUEST)
}

pub async fn leave_lobby(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<JoinLobbyRequest>,
) -> Result<StatusCode, StatusCode> {
    state.lobby_manager.leave(id, req.player_id)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or(StatusCode::NOT_FOUND)
}

// ---------------------------------------------------------------------------
// POST /lobby/:id/start
// Điền bots vào các slot còn trống rồi bắt đầu game loop.
// Unity gọi endpoint này khi muốn chơi test (1 người + N bots).
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct StartResponse {
    pub lobby_id: Uuid,
    pub human_count: usize,
    pub bot_count: usize,
    pub bot_names: Vec<String>,
}

pub async fn start_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
) -> Result<Json<StartResponse>, StatusCode> {
    let lobby = state.lobby_manager.get(lobby_id).ok_or(StatusCode::NOT_FOUND)?;

    if lobby.status != LobbyStatus::Waiting {
        return Err(StatusCode::CONFLICT); // đã bắt đầu rồi
    }

    let human_ids: Vec<Uuid> = lobby.player_ids.clone();
    let slots_to_fill = (lobby.max_players as usize).saturating_sub(human_ids.len());

    let mut bot_ids: Vec<Uuid> = Vec::new();
    let mut bot_names: Vec<String> = Vec::new();

    // Tạo bot players và đăng ký vào state
    for i in 0..slots_to_fill {
        let name = bot::create_bot_name(i as u8);
        let bot_player = Player::new(name.clone());
        let bot_id = bot_player.id;

        state.players.insert(bot_id, bot_player);
        state.boards.insert(bot_id, Board::new(bot_id));
        state.streaks.insert(bot_id, (0, 0));

        // Thêm bot vào lobby
        state.lobby_manager.join(lobby_id, bot_id);

        bot_ids.push(bot_id);
        bot_names.push(name);
    }

    // Đánh dấu lobby đang chạy
    state.lobby_manager.set_status(lobby_id, LobbyStatus::InProgress);

    let human_count = human_ids.len();
    let bot_count = bot_ids.len();

    // Spawn game loop
    let state_clone = Arc::clone(&state);
    tokio::spawn(game_loop::run(state_clone, lobby_id, human_ids, bot_ids));

    tracing::info!(
        "Lobby {} started: {} humans + {} bots",
        lobby_id, human_count, bot_count
    );

    Ok(Json(StartResponse {
        lobby_id,
        human_count,
        bot_count,
        bot_names,
    }))
}
