/// Game loop: Planning (30s) → Combat → Results (5s) → lặp lại.
/// Spawn bằng tokio::spawn khi lobby bắt đầu.
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::AppState;
use crate::game::{bot, combat, round};
use crate::models::board::Board;
use crate::ws::messages::{PlayerSnapshot, ServerMessage};

const PLANNING_SECS: u64 = 30;
const RESULTS_SECS: u64 = 5;

pub async fn run(
    state: Arc<AppState>,
    lobby_id: Uuid,
    human_ids: Vec<Uuid>,
    bot_ids: Vec<Uuid>,
) {
    let all_ids: Vec<Uuid> = human_ids.iter().chain(bot_ids.iter()).copied().collect();
    let mut current_round = 0u32;

    loop {
        current_round += 1;
        tracing::info!("[Loop] Lobby {} Round {} — Planning", lobby_id, current_round);

        // ── Planning ────────────────────────────────────────────────────────

        // Phát income (bỏ qua vòng 1)
        if current_round > 1 {
            grant_income(&state, &all_ids).await;
        }

        // Roll shop mới cho tất cả
        refresh_shops(&state, &all_ids).await;

        // Broadcast phase planning
        push_game_state(&state, lobby_id, current_round, "planning").await;

        // Bot tự động planning ngay
        run_bot_turn(&state, &bot_ids).await;

        // Chờ hết giờ planning
        sleep(Duration::from_secs(PLANNING_SECS)).await;

        // ── Combat ──────────────────────────────────────────────────────────
        tracing::info!("[Loop] Lobby {} Round {} — Combat", lobby_id, current_round);
        push_game_state(&state, lobby_id, current_round, "combat").await;

        let pairs = pair_players(&all_ids, current_round);
        do_combat(&state, &pairs).await;

        // ── Results ─────────────────────────────────────────────────────────
        push_game_state(&state, lobby_id, current_round, "results").await;

        // Kiểm tra kết thúc
        let alive: Vec<Uuid> = all_ids.iter()
            .filter(|&&id| state.players.get(&id).map_or(false, |p| p.health > 0))
            .copied()
            .collect();

        if alive.len() <= 1 {
            if let Some(&winner) = alive.first() {
                let name = state.players.get(&winner)
                    .map(|p| p.username.clone())
                    .unwrap_or_default();
                tracing::info!("[Loop] Lobby {} — Winner: {} ({})", lobby_id, name, winner);
            }
            break;
        }

        sleep(Duration::from_secs(RESULTS_SECS)).await;
    }
}

// ---------------------------------------------------------------------------

async fn grant_income(state: &AppState, ids: &[Uuid]) {
    for &pid in ids {
        let (ws, ls) = state.streaks.get(&pid).map(|s| *s).unwrap_or((0, 0));
        let income = {
            match state.players.get(&pid) {
                Some(p) => round::calculate_income(&p, ws, ls),
                None => continue,
            }
        };
        if let Some(mut p) = state.players.get_mut(&pid) {
            p.gold += income;
        }
        push_player_update(state, pid).await;
    }
}

async fn refresh_shops(state: &AppState, ids: &[Uuid]) {
    for &pid in ids {
        let level = state.players.get(&pid).map(|p| p.level).unwrap_or(1);
        let offer = {
            let pool = state.unit_pool.lock().await;
            pool.roll_shop(level)
        };
        let names: Vec<String> = offer.iter().map(|u| u.name.clone()).collect();
        state.shop_offers.insert(pid, names);
        push_to(state, pid, ServerMessage::ShopUpdate { units: offer }).await;
    }
}

async fn run_bot_turn(state: &AppState, bot_ids: &[Uuid]) {
    for &bot_id in bot_ids {
        let player = match state.players.get(&bot_id) { Some(p) => p.clone(), None => continue };
        let board = match state.boards.get(&bot_id) { Some(b) => b.clone(), None => continue };

        // Bot quyết định mua
        let actions = {
            let pool = state.unit_pool.lock().await;
            bot::plan(&player, &board, &pool)
        };

        // Thực thi mua
        for name in &actions.bought_unit_names {
            let unit_opt = {
                let mut pool = state.unit_pool.lock().await;
                pool.remove(name)
            };
            if let Some(unit) = unit_opt {
                let cost = unit.cost as u32;
                if let Some(mut b) = state.boards.get_mut(&bot_id) {
                    b.add_to_bench(unit);
                }
                if let Some(mut p) = state.players.get_mut(&bot_id) {
                    p.gold = p.gold.saturating_sub(cost);
                }
            }
        }
        // Trừ thêm phí roll
        if actions.rolled_count > 0 {
            if let Some(mut p) = state.players.get_mut(&bot_id) {
                p.gold = p.gold.saturating_sub(actions.rolled_count * 2);
            }
        }

        // Bot đặt tướng lên board
        let placements = {
            let board = state.boards.get(&bot_id)
                .map(|b| b.clone())
                .unwrap_or_else(|| Board::new(bot_id));
            bot::decide_placement(&board)
        };
        for (unit_id, pos) in placements {
            if let Some(mut b) = state.boards.get_mut(&bot_id) {
                if let Some(idx) = b.bench.iter().position(|u| u.id == unit_id) {
                    b.place_unit(idx, pos);
                }
            }
        }
    }
}

fn pair_players(ids: &[Uuid], round: u32) -> Vec<(Uuid, Uuid)> {
    if ids.len() < 2 { return vec![]; }
    let mut v = ids.to_vec();
    let len = v.len();
    v.rotate_left((round as usize) % len);
    let mut pairs = vec![];
    let mut i = 0;
    while i + 1 < v.len() {
        pairs.push((v[i], v[i + 1]));
        i += 2;
    }
    // Số lẻ → người cuối đánh người đầu
    if v.len() % 2 == 1 {
        let last = *v.last().unwrap();
        pairs.push((last, v[0]));
    }
    pairs
}

async fn do_combat(state: &AppState, pairs: &[(Uuid, Uuid)]) {
    for &(a, b) in pairs {
        let (mut ba, mut bb) = match (state.boards.get(&a), state.boards.get(&b)) {
            (Some(ba), Some(bb)) => (ba.clone(), bb.clone()),
            _ => continue,
        };

        // Reset HP
        for u in ba.placed.values_mut() { u.reset_for_combat(); }
        for u in bb.placed.values_mut() { u.reset_for_combat(); }

        let result = combat::simulate(&mut ba, &mut bb);

        // Streak + damage
        let loser = match result.winner_player_id {
            Some(winner) if winner == a => {
                update_streak(state, a, true);
                update_streak(state, b, false);
                Some(b)
            }
            Some(winner) if winner == b => {
                update_streak(state, a, false);
                update_streak(state, b, true);
                Some(a)
            }
            _ => None,
        };

        if let Some(loser_id) = loser {
            if let Some(mut p) = state.players.get_mut(&loser_id) {
                p.health = p.health.saturating_sub(result.damage_dealt);
            }
            push_player_update(state, loser_id).await;
        }

        let msg = ServerMessage::CombatResult {
            winner_player_id: result.winner_player_id,
            damage_dealt: result.damage_dealt,
            surviving_unit_count: result.surviving_units.len(),
        };
        push_to(state, a, msg.clone()).await;
        push_to(state, b, msg).await;
    }
}

fn update_streak(state: &AppState, pid: Uuid, won: bool) {
    if let Some(mut s) = state.streaks.get_mut(&pid) {
        if won { s.0 += 1; s.1 = 0; } else { s.1 += 1; s.0 = 0; }
    }
}

// ---------------------------------------------------------------------------
// Push helpers
// ---------------------------------------------------------------------------

async fn push_to(state: &AppState, pid: Uuid, msg: ServerMessage) {
    if let Some(tx) = state.player_senders.get(&pid) {
        let _ = tx.send(msg);
    }
}

async fn push_player_update(state: &AppState, pid: Uuid) {
    if let Some(p) = state.players.get(&pid) {
        push_to(state, pid, ServerMessage::PlayerUpdate {
            player_id: p.id,
            gold: p.gold,
            health: p.health,
            level: p.level,
        }).await;
    }
}

async fn push_game_state(state: &AppState, lobby_id: Uuid, round: u32, phase: &str) {
    let snapshots: Vec<PlayerSnapshot> = state.players.iter().map(|e| {
        let p = e.value();
        PlayerSnapshot { player_id: p.id, username: p.username.clone(),
                         health: p.health, gold: p.gold, level: p.level }
    }).collect();

    let msg = ServerMessage::GameState {
        round, phase: phase.to_string(), players: snapshots,
    };

    // Gửi đến tất cả player trong lobby
    if let Some(lobby) = state.lobby_manager.get(lobby_id) {
        for pid in &lobby.player_ids {
            push_to(state, *pid, msg.clone()).await;
        }
    }
}
