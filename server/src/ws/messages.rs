use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::board::HexPosition;
use crate::models::unit::Unit;

// ---------------------------------------------------------------------------
// CLIENT → SERVER
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Player sẵn sàng bắt đầu vòng đánh
    Ready,
    /// Mua tướng từ shop
    BuyUnit { unit_name: String, cost: u8 },
    /// Bán tướng (bench hoặc board)
    SellUnit { unit_id: Uuid },
    /// Đặt tướng từ bench lên board
    PlaceUnit { unit_id: Uuid, position: HexPosition },
    /// Di chuyển tướng trên board
    MoveUnit { from: HexPosition, to: HexPosition },
    /// Đưa tướng từ board về bench
    BenchUnit { position: HexPosition },
    /// Roll shop (tốn 2 vàng)
    RollShop,
    /// Mua XP (tốn 4 vàng)
    BuyXp,
    /// Heartbeat
    Ping,
}

// ---------------------------------------------------------------------------
// SERVER → CLIENT
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Xác nhận kết nối
    Connected {
        player_id: Uuid,
        lobby_id: Uuid,
    },

    /// Snapshot toàn bộ game state (gửi khi phase đổi, khi connect)
    GameState {
        round: u32,
        phase: String,
        players: Vec<PlayerSnapshot>,
    },

    /// Shop update sau khi roll hoặc đầu vòng
    ShopUpdate {
        units: Vec<Unit>,
    },

    /// Board state đầy đủ sau khi có thay đổi (place/move/bench)
    BoardState {
        player_id: Uuid,
        /// Dạng array để Unity JsonUtility parse được (không dùng HashMap)
        placed: Vec<BoardCell>,
        bench: Vec<Unit>,
        placed_count: usize,
    },

    /// Kết quả combat
    CombatResult {
        winner_player_id: Option<Uuid>,
        damage_dealt: u32,
        surviving_unit_count: usize,
    },

    /// Gold/health/level của một player thay đổi
    PlayerUpdate {
        player_id: Uuid,
        gold: u32,
        health: u32,
        level: u8,
    },

    /// Error trả về client
    Error { message: String },

    /// Pong
    Pong,
}

// ---------------------------------------------------------------------------
// Shared DTOs
// ---------------------------------------------------------------------------

/// Một ô trên board (dùng thay HashMap để Unity parse được)
#[derive(Debug, Clone, Serialize)]
pub struct BoardCell {
    pub position: String, // "col,row"
    pub unit: Unit,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerSnapshot {
    pub player_id: Uuid,
    pub username: String,
    pub health: u32,
    pub gold: u32,
    pub level: u8,
}
