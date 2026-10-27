pub mod health;
pub mod lobby;
pub mod player;

use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use crate::AppState;

pub fn router(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // Lobby
        .route("/lobby/create",        post(lobby::create_lobby))
        .route("/lobby/:id",           get(lobby::get_lobby))
        .route("/lobby/:id/join",      post(lobby::join_lobby))
        .route("/lobby/:id/leave",     post(lobby::leave_lobby))
        .route("/lobby/:id/start",     post(lobby::start_lobby))  // bắt đầu với bots
        // Player
        .route("/player/register",     post(player::register_player))
        .route("/player/:id",          get(player::get_player))
}
