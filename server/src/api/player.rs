use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::player::Player;

#[derive(Deserialize)]
pub struct RegisterPlayerRequest {
    pub username: String,
}

#[derive(Serialize)]
pub struct PlayerResponse {
    pub player: Player,
}

pub async fn register_player(
    Json(req): Json<RegisterPlayerRequest>,
) -> Json<PlayerResponse> {
    let player = Player::new(req.username);
    Json(PlayerResponse { player })
}

pub async fn get_player(
    Path(_id): Path<Uuid>,
) -> StatusCode {
    // TODO: hook up to a persistent store
    StatusCode::NOT_IMPLEMENTED
}
