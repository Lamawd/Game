use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LobbyStatus {
    Waiting,
    InProgress,
    Finished,
}

/// A game lobby that holds players before and during a match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub id: Uuid,
    pub host_player_id: Uuid,
    pub player_ids: Vec<Uuid>,
    pub max_players: u8,
    pub status: LobbyStatus,
    pub created_at: DateTime<Utc>,
    pub round: u32,
}

impl Lobby {
    pub fn new(host_player_id: Uuid, max_players: u8) -> Self {
        Self {
            id: Uuid::new_v4(),
            host_player_id,
            player_ids: vec![host_player_id],
            max_players,
            status: LobbyStatus::Waiting,
            created_at: Utc::now(),
            round: 0,
        }
    }

    pub fn is_full(&self) -> bool {
        self.player_ids.len() >= self.max_players as usize
    }

    pub fn add_player(&mut self, player_id: Uuid) -> bool {
        if self.is_full() || self.player_ids.contains(&player_id) {
            return false;
        }
        self.player_ids.push(player_id);
        true
    }

    pub fn remove_player(&mut self, player_id: Uuid) -> bool {
        let before = self.player_ids.len();
        self.player_ids.retain(|id| *id != player_id);
        self.player_ids.len() < before
    }
}
