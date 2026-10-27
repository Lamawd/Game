use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::AppState;
use crate::game::game_loop;
use crate::models::board::Board;
use crate::ws::messages::{ClientMessage, ServerMessage};

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path((lobby_id, player_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, lobby_id, player_id, state))
}

// ---------------------------------------------------------------------------
// Socket lifecycle
// ---------------------------------------------------------------------------

async fn handle_socket(
    socket: WebSocket,
    lobby_id: Uuid,
    player_id: Uuid,
    state: Arc<AppState>,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Kiểm tra lobby
    if state.lobby_manager.get(lobby_id).is_none() {
        send_ws(&mut ws_tx, ServerMessage::Error { message: "Lobby not found".into() }).await;
        return;
    }

    // Kiểm tra player đã register
    if !state.players.contains_key(&player_id) {
        send_ws(&mut ws_tx, ServerMessage::Error {
            message: "Player not registered. POST /api/v1/player/register first.".into(),
        }).await;
        return;
    }

    // Đảm bảo board tồn tại
    state.boards.entry(player_id).or_insert_with(|| Board::new(player_id));

    // Tạo mpsc channel — server push messages về client qua channel này
    let (server_tx, mut server_rx) = mpsc::unbounded_channel::<ServerMessage>();
    state.player_senders.insert(player_id, server_tx);

    // Spawn task: đọc từ server_rx → ghi ra websocket
    let mut forward_task = tokio::spawn(async move {
        while let Some(msg) = server_rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap_or_default();
            if ws_tx.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Gửi Connected ack
    if let Some(tx) = state.player_senders.get(&player_id) {
        let _ = tx.send(ServerMessage::Connected { player_id, lobby_id });
    }

    // Gửi shop đầu tiên
    {
        let level = state.players.get(&player_id).map(|p| p.level).unwrap_or(1);
        let offer = {
            let pool = state.unit_pool.lock().await;
            pool.roll_shop(level)
        };
        let names: Vec<String> = offer.iter().map(|u| u.name.clone()).collect();
        state.shop_offers.insert(player_id, names);
        if let Some(tx) = state.player_senders.get(&player_id) {
            let _ = tx.send(ServerMessage::ShopUpdate { units: offer });
        }
    }

    tracing::info!("Player {} connected to lobby {}", player_id, lobby_id);

    // Main receive loop
    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                handle_client_message(client_msg, player_id, lobby_id, &state).await;
                            }
                            Err(e) => {
                                if let Some(tx) = state.player_senders.get(&player_id) {
                                    let _ = tx.send(ServerMessage::Error {
                                        message: format!("Invalid message: {}", e),
                                    });
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        // axum handles pong automatically, but we log it
                        tracing::trace!("Ping from {}", player_id);
                        let _ = data; // suppress unused warning
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            // Forward task ended (websocket send error) → disconnect
            _ = &mut forward_task => break,
        }
    }

    // Cleanup khi disconnect
    state.player_senders.remove(&player_id);
    state.ready_players.remove(&player_id);
    forward_task.abort();
    tracing::info!("Player {} disconnected from lobby {}", player_id, lobby_id);
}

// ---------------------------------------------------------------------------
// Message dispatch
// ---------------------------------------------------------------------------

async fn handle_client_message(
    msg: ClientMessage,
    player_id: Uuid,
    lobby_id: Uuid,
    state: &Arc<AppState>,
) {
    match msg {
        // ── Ping ────────────────────────────────────────────────────────────
        ClientMessage::Ping => {
            send_back(state, player_id, ServerMessage::Pong).await;
        }

        // ── Ready ───────────────────────────────────────────────────────────
        ClientMessage::Ready => {
            state.ready_players.insert(player_id, true);
            tracing::debug!("Player {} ready in lobby {}", player_id, lobby_id);

            // Kiểm tra xem tất cả player trong lobby đã ready chưa
            maybe_start_game(state, lobby_id).await;
        }

        // ── BuyUnit ─────────────────────────────────────────────────────────
        ClientMessage::BuyUnit { unit_name, cost } => {
            let gold = match state.players.get(&player_id) {
                Some(p) => p.gold,
                None => return,
            };

            if gold < cost as u32 {
                send_back(state, player_id, ServerMessage::Error {
                    message: format!("Not enough gold ({}/{})", gold, cost),
                }).await;
                return;
            }

            let unit = {
                let mut pool = state.unit_pool.lock().await;
                pool.remove(&unit_name)
            };

            let unit = match unit {
                Some(u) => u,
                None => {
                    send_back(state, player_id, ServerMessage::Error {
                        message: format!("{} is sold out", unit_name),
                    }).await;
                    return;
                }
            };

            let added = {
                let mut board = state.boards.entry(player_id)
                    .or_insert_with(|| Board::new(player_id));
                board.add_to_bench(unit.clone())
            };

            if !added {
                let mut pool = state.unit_pool.lock().await;
                pool.return_unit(unit);
                send_back(state, player_id, ServerMessage::Error {
                    message: "Bench full (9/9)".into(),
                }).await;
                return;
            }

            {
                let mut p = state.players.get_mut(&player_id).unwrap();
                p.gold -= cost as u32;
            }

            // Star merge check
            check_star_merge(player_id, &unit_name, state).await;

            send_player_update(state, player_id).await;
            send_board_state(state, player_id).await;
        }

        // ── SellUnit ────────────────────────────────────────────────────────
        ClientMessage::SellUnit { unit_id } => {
            let sold = {
                let mut board = state.boards.entry(player_id)
                    .or_insert_with(|| Board::new(player_id));
                let bench_pos = board.bench.iter().position(|u| u.id == unit_id);
                if let Some(i) = bench_pos {
                    Some(board.bench.remove(i))
                } else {
                    let key = board.placed.iter()
                        .find(|(_, u)| u.id == unit_id)
                        .map(|(k, _)| k.clone());
                    key.and_then(|k| board.placed.remove(&k))
                }
            };

            match sold {
                None => {
                    send_back(state, player_id, ServerMessage::Error {
                        message: format!("Unit {} not found", unit_id),
                    }).await;
                }
                Some(unit) => {
                    let refund = match unit.tier {
                        1 => unit.cost as u32,
                        2 => unit.cost as u32 * 3,
                        _ => unit.cost as u32 * 9,
                    };
                    {
                        let mut pool = state.unit_pool.lock().await;
                        pool.return_unit(unit);
                    }
                    if let Some(mut p) = state.players.get_mut(&player_id) {
                        p.gold += refund;
                    }
                    send_player_update(state, player_id).await;
                    send_board_state(state, player_id).await;
                }
            }
        }

        // ── PlaceUnit ───────────────────────────────────────────────────────
        ClientMessage::PlaceUnit { unit_id, position } => {
            let ok = {
                let mut board = state.boards.entry(player_id)
                    .or_insert_with(|| Board::new(player_id));
                let idx = board.bench.iter().position(|u| u.id == unit_id);
                idx.map_or(false, |i| board.place_unit(i, position))
            };
            if !ok {
                send_back(state, player_id, ServerMessage::Error {
                    message: "Cannot place unit".into(),
                }).await;
            } else {
                send_board_state(state, player_id).await;
            }
        }

        // ── MoveUnit ────────────────────────────────────────────────────────
        ClientMessage::MoveUnit { from, to } => {
            let ok = {
                let mut board = state.boards.entry(player_id)
                    .or_insert_with(|| Board::new(player_id));
                board.move_unit(from, to)
            };
            if !ok {
                send_back(state, player_id, ServerMessage::Error {
                    message: "Cannot move unit".into(),
                }).await;
            } else {
                send_board_state(state, player_id).await;
            }
        }

        // ── BenchUnit ───────────────────────────────────────────────────────
        ClientMessage::BenchUnit { position } => {
            let ok = {
                let mut board = state.boards.entry(player_id)
                    .or_insert_with(|| Board::new(player_id));
                board.bench_unit(position)
            };
            if !ok {
                send_back(state, player_id, ServerMessage::Error {
                    message: "Cannot bench unit".into(),
                }).await;
            } else {
                send_board_state(state, player_id).await;
            }
        }

        // ── RollShop ────────────────────────────────────────────────────────
        ClientMessage::RollShop => {
            let gold = state.players.get(&player_id).map(|p| p.gold).unwrap_or(0);
            if gold < 2 {
                send_back(state, player_id, ServerMessage::Error {
                    message: "Need 2 gold to roll".into(),
                }).await;
                return;
            }

            let level = {
                let mut p = state.players.get_mut(&player_id).unwrap();
                p.gold -= 2;
                p.level
            };

            let offer = {
                let pool = state.unit_pool.lock().await;
                pool.roll_shop(level)
            };
            let names: Vec<String> = offer.iter().map(|u| u.name.clone()).collect();
            state.shop_offers.insert(player_id, names);

            send_back(state, player_id, ServerMessage::ShopUpdate { units: offer }).await;
            send_player_update(state, player_id).await;
        }

        // ── BuyXp ───────────────────────────────────────────────────────────
        ClientMessage::BuyXp => {
            let ok = {
                let mut p = match state.players.get_mut(&player_id) {
                    Some(p) => p,
                    None => return,
                };
                p.buy_xp()
            };
            if !ok {
                send_back(state, player_id, ServerMessage::Error {
                    message: "Need 4 gold to buy XP".into(),
                }).await;
            } else {
                send_player_update(state, player_id).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Game start trigger
// ---------------------------------------------------------------------------

async fn maybe_start_game(state: &Arc<AppState>, lobby_id: Uuid) {
    let lobby = match state.lobby_manager.get(lobby_id) {
        Some(l) => l,
        None => return,
    };

    // Đếm số player đã ready
    let ready_count = lobby.player_ids.iter()
        .filter(|id| state.ready_players.get(id).map_or(false, |v| *v))
        .count();

    // Bắt đầu khi tất cả đã ready (ít nhất 1 người)
    if ready_count == lobby.player_ids.len() && ready_count >= 1 {
        tracing::info!("Lobby {} — all {} players ready, starting game", lobby_id, ready_count);
        state.ready_players.clear();

        // Phân biệt human và bot (bot ID có trong AppState.players nhưng không có sender)
        let human_ids: Vec<Uuid> = lobby.player_ids.iter()
            .filter(|id| state.player_senders.contains_key(id))
            .copied()
            .collect();
        let bot_ids: Vec<Uuid> = lobby.player_ids.iter()
            .filter(|id| !state.player_senders.contains_key(id))
            .copied()
            .collect();

        let state_clone = Arc::clone(state);
        tokio::spawn(game_loop::run(state_clone, lobby_id, human_ids, bot_ids));
    }
}

// ---------------------------------------------------------------------------
// Star merge
// ---------------------------------------------------------------------------

async fn check_star_merge(player_id: Uuid, unit_name: &str, state: &Arc<AppState>) {
    loop {
        let ones: Vec<usize> = {
            let board = match state.boards.get(&player_id) { Some(b) => b, None => break };
            board.bench.iter().enumerate()
                .filter(|(_, u)| u.name == unit_name && u.tier == 1)
                .map(|(i, _)| i)
                .collect()
        };
        if ones.len() >= 3 {
            let mut indices = ones;
            indices.sort_unstable_by(|a, b| b.cmp(a));
            let removed: Vec<_> = {
                let mut board = state.boards.get_mut(&player_id).unwrap();
                indices[..3].iter().map(|&i| board.bench.remove(i)).collect()
            };
            let mut merged = removed.into_iter().next().unwrap();
            merged.upgrade();
            merged.id = Uuid::new_v4();
            state.boards.get_mut(&player_id).unwrap().bench.push(merged);
            tracing::info!("{} merged to 2★ for player {}", unit_name, player_id);
            continue;
        }

        let twos: Vec<usize> = {
            let board = match state.boards.get(&player_id) { Some(b) => b, None => break };
            board.bench.iter().enumerate()
                .filter(|(_, u)| u.name == unit_name && u.tier == 2)
                .map(|(i, _)| i)
                .collect()
        };
        if twos.len() >= 3 {
            let mut indices = twos;
            indices.sort_unstable_by(|a, b| b.cmp(a));
            let removed: Vec<_> = {
                let mut board = state.boards.get_mut(&player_id).unwrap();
                indices[..3].iter().map(|&i| board.bench.remove(i)).collect()
            };
            let mut merged = removed.into_iter().next().unwrap();
            merged.upgrade();
            merged.id = Uuid::new_v4();
            state.boards.get_mut(&player_id).unwrap().bench.push(merged);
            tracing::info!("{} merged to 3★ for player {}", unit_name, player_id);
        }
        break;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn send_back(state: &AppState, player_id: Uuid, msg: ServerMessage) {
    if let Some(tx) = state.player_senders.get(&player_id) {
        let _ = tx.send(msg);
    }
}

async fn send_player_update(state: &AppState, player_id: Uuid) {
    if let Some(p) = state.players.get(&player_id) {
        send_back(state, player_id, ServerMessage::PlayerUpdate {
            player_id: p.id,
            gold: p.gold,
            health: p.health,
            level: p.level,
        }).await;
    }
}

async fn send_board_state(state: &AppState, player_id: Uuid) {
    if let Some(board) = state.boards.get(&player_id) {
        let placed: Vec<crate::ws::messages::BoardCell> = board.placed.iter()
            .map(|(k, u)| crate::ws::messages::BoardCell {
                position: k.clone(),
                unit: u.clone(),
            })
            .collect();
        let msg = ServerMessage::BoardState {
            player_id,
            placed,
            bench: board.bench.clone(),
            placed_count: board.placed_count(),
        };
        send_back(state, player_id, msg).await;
    }
}

async fn send_ws(
    tx: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: ServerMessage,
) {
    let json = serde_json::to_string(&msg).unwrap_or_default();
    let _ = tx.send(Message::Text(json.into())).await;
}
