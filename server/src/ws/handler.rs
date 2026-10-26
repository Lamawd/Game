use std::sync::Arc;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use uuid::Uuid;

use crate::AppState;
use crate::ws::messages::{ClientMessage, ServerMessage};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path((lobby_id, player_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, lobby_id, player_id, state))
}

async fn handle_socket(socket: WebSocket, lobby_id: Uuid, player_id: Uuid, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Verify the lobby exists
    if state.lobby_manager.get(lobby_id).is_none() {
        let msg = serde_json::to_string(&ServerMessage::Error {
            message: "Lobby not found".into(),
        })
        .unwrap_or_default();
        let _ = sender.send(Message::Text(msg.into())).await;
        return;
    }

    // Send connection acknowledgement
    let ack = serde_json::to_string(&ServerMessage::Connected { player_id, lobby_id })
        .unwrap_or_default();
    if sender.send(Message::Text(ack.into())).await.is_err() {
        return;
    }

    tracing::info!("Player {} connected to lobby {}", player_id, lobby_id);

    // Main message loop
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        let response = process_message(client_msg, player_id, lobby_id, &state).await;
                        if let Some(resp) = response {
                            let json = serde_json::to_string(&resp).unwrap_or_default();
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Bad message from player {}: {}", player_id, e);
                        let err = serde_json::to_string(&ServerMessage::Error {
                            message: format!("Invalid message: {}", e),
                        })
                        .unwrap_or_default();
                        let _ = sender.send(Message::Text(err.into())).await;
                    }
                }
            }
            Message::Close(_) => break,
            Message::Ping(data) => {
                let _ = sender.send(Message::Pong(data)).await;
            }
            _ => {}
        }
    }

    tracing::info!("Player {} disconnected from lobby {}", player_id, lobby_id);
}

async fn process_message(
    msg: ClientMessage,
    player_id: Uuid,
    lobby_id: Uuid,
    _state: &Arc<AppState>,
) -> Option<ServerMessage> {
    match msg {
        ClientMessage::Ping => Some(ServerMessage::Pong),
        ClientMessage::Ready => {
            tracing::debug!("Player {} is ready in lobby {}", player_id, lobby_id);
            // TODO: track readiness, trigger combat when all ready
            None
        }
        ClientMessage::BuyUnit { unit_name, cost } => {
            tracing::debug!("Player {} buying {} (cost {})", player_id, unit_name, cost);
            // TODO: deduct gold, pull from shared pool, add to bench
            None
        }
        ClientMessage::SellUnit { unit_id } => {
            tracing::debug!("Player {} selling unit {}", player_id, unit_id);
            // TODO: return unit to pool, refund partial gold
            None
        }
        ClientMessage::PlaceUnit { unit_id, position } => {
            tracing::debug!("Player {} placing unit {} at {:?}", player_id, unit_id, position);
            // TODO: update board state
            None
        }
        ClientMessage::MoveUnit { from, to } => {
            tracing::debug!("Player {} moving unit from {:?} to {:?}", player_id, from, to);
            None
        }
        ClientMessage::BenchUnit { position } => {
            tracing::debug!("Player {} benching unit at {:?}", player_id, position);
            None
        }
        ClientMessage::RollShop => {
            tracing::debug!("Player {} rolling shop", player_id);
            // TODO: deduct 2 gold, return new shop units
            None
        }
        ClientMessage::BuyXp => {
            tracing::debug!("Player {} buying XP", player_id);
            // TODO: deduct 4 gold, increase XP/level
            None
        }
    }
}
