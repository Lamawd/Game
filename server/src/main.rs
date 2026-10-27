mod api;
mod game;
mod lobby;
mod models;
mod ws;

use std::sync::Arc;
use axum::{Router, routing::get};
use dashmap::DashMap;
use tokio::sync::{Mutex, mpsc};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::game::shop::UnitPool;
use crate::lobby::LobbyManager;
use crate::models::board::Board;
use crate::models::player::Player;
use crate::ws::messages::ServerMessage;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "mixi_server=debug,info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState::new());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(api::health::health_check))
        .nest("/api/v1", api::router(state.clone()))
        .route("/ws/:lobby_id/:player_id", get(ws::handler::ws_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:8080";
    tracing::info!("Mixi Game Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Shared state toàn server
#[derive(Clone)]
pub struct AppState {
    pub lobby_manager: LobbyManager,

    /// Player registry: player_id → Player
    pub players: Arc<DashMap<Uuid, Player>>,

    /// Board per player: player_id → Board
    pub boards: Arc<DashMap<Uuid, Board>>,

    /// Shop offer đang hiện cho mỗi player: player_id → Vec<tên tướng>
    pub shop_offers: Arc<DashMap<Uuid, Vec<String>>>,

    /// Shared unit pool cho toàn lobby
    pub unit_pool: Arc<Mutex<UnitPool>>,

    /// Win/loss streaks: player_id → (win_streak, loss_streak)
    pub streaks: Arc<DashMap<Uuid, (u32, u32)>>,

    /// Sender channel để push message đến mỗi connected player
    /// player_id → UnboundedSender<ServerMessage>
    pub player_senders: Arc<DashMap<Uuid, mpsc::UnboundedSender<ServerMessage>>>,

    /// Set player IDs đã nhấn Ready trong vòng hiện tại
    pub ready_players: Arc<DashMap<Uuid, bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            lobby_manager: LobbyManager::new(),
            players: Arc::new(DashMap::new()),
            boards: Arc::new(DashMap::new()),
            shop_offers: Arc::new(DashMap::new()),
            unit_pool: Arc::new(Mutex::new(UnitPool::new())),
            streaks: Arc::new(DashMap::new()),
            player_senders: Arc::new(DashMap::new()),
            ready_players: Arc::new(DashMap::new()),
        }
    }
}
