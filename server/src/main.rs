mod api;
mod game;
mod lobby;
mod models;
mod ws;

use std::sync::Arc;
use axum::{Router, routing::get};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::lobby::LobbyManager;

#[tokio::main]
async fn main() {
    // Init tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "mixi_server=debug,info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState {
        lobby_manager: LobbyManager::new(),
    });

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

/// Shared application state passed to all handlers
#[derive(Clone)]
pub struct AppState {
    pub lobby_manager: LobbyManager,
}
