use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::models::lobby::Lobby;

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
    let lobby = state.lobby_manager.create(req.host_player_id, max);
    Ok(Json(LobbyResponse { lobby }))
}

pub async fn get_lobby(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<LobbyResponse>, StatusCode> {
    state
        .lobby_manager
        .get(id)
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
    state
        .lobby_manager
        .join(id, req.player_id)
        .map(|lobby| Json(LobbyResponse { lobby }))
        .ok_or(StatusCode::BAD_REQUEST)
}

pub async fn leave_lobby(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<JoinLobbyRequest>,
) -> Result<StatusCode, StatusCode> {
    state
        .lobby_manager
        .leave(id, req.player_id)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or(StatusCode::NOT_FOUND)
}
