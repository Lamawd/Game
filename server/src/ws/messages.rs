use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::board::HexPosition;
use crate::models::unit::Unit;

/// Messages sent from the CLIENT (Unity) to the SERVER
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Player is ready to start the round
    Ready,

    /// Buy a unit from the shop
    BuyUnit {
        unit_name: String,
        cost: u8,
    },

    /// Sell a unit from bench or board
    SellUnit {
        unit_id: Uuid,
    },

    /// Move/place a unit on the board
    PlaceUnit {
        unit_id: Uuid,
        position: HexPosition,
    },

    /// Move unit from one board position to another
    MoveUnit {
        from: HexPosition,
        to: HexPosition,
    },

    /// Send unit back to bench
    BenchUnit {
        position: HexPosition,
    },

    /// Re-roll the shop for 2 gold
    RollShop,

    /// Buy XP for 4 gold
    BuyXp,

    /// Heartbeat / ping
    Ping,
}

/// Messages sent from the SERVER to the CLIENT (Unity)
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Connection acknowledged
    Connected {
        player_id: Uuid,
        lobby_id: Uuid,
    },

    /// Full game state snapshot (sent on connect and after big changes)
    GameState {
        round: u32,
        phase: String,
        players: Vec<PlayerSnapshot>,
    },

    /// Shop updated (after roll or round start)
    ShopUpdate {
        units: Vec<Unit>,
    },

    /// A unit was placed / moved on the board
    BoardUpdate {
        player_id: Uuid,
    },

    /// Combat result for this round
    CombatResult {
        winner_player_id: Option<Uuid>,
        damage_dealt: u32,
        surviving_unit_count: usize,
    },

    /// A player's gold / health / level changed
    PlayerUpdate {
        player_id: Uuid,
        gold: u32,
        health: u32,
        level: u8,
    },

    /// Error message back to the client
    Error {
        message: String,
    },

    /// Pong response
    Pong,
}

#[derive(Debug, Serialize)]
pub struct PlayerSnapshot {
    pub player_id: Uuid,
    pub username: String,
    pub health: u32,
    pub gold: u32,
    pub level: u8,
}
