use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;

use crate::models::lobby::Lobby;

/// Thread-safe in-memory lobby store
#[derive(Clone)]
pub struct LobbyManager {
    lobbies: Arc<DashMap<Uuid, Lobby>>,
}

impl LobbyManager {
    pub fn new() -> Self {
        Self {
            lobbies: Arc::new(DashMap::new()),
        }
    }

    pub fn create(&self, host_player_id: Uuid, max_players: u8) -> Lobby {
        let lobby = Lobby::new(host_player_id, max_players);
        self.lobbies.insert(lobby.id, lobby.clone());
        lobby
    }

    pub fn get(&self, id: Uuid) -> Option<Lobby> {
        self.lobbies.get(&id).map(|l| l.clone())
    }

    pub fn join(&self, lobby_id: Uuid, player_id: Uuid) -> Option<Lobby> {
        self.lobbies.get_mut(&lobby_id).and_then(|mut lobby| {
            if lobby.add_player(player_id) {
                Some(lobby.clone())
            } else {
                None
            }
        })
    }

    pub fn leave(&self, lobby_id: Uuid, player_id: Uuid) -> Option<Lobby> {
        self.lobbies.get_mut(&lobby_id).and_then(|mut lobby| {
            if lobby.remove_player(player_id) {
                Some(lobby.clone())
            } else {
                None
            }
        })
    }

    pub fn remove(&self, lobby_id: Uuid) {
        self.lobbies.remove(&lobby_id);
    }

    pub fn set_status(&self, lobby_id: Uuid, status: crate::models::lobby::LobbyStatus) {
        if let Some(mut lobby) = self.lobbies.get_mut(&lobby_id) {
            lobby.status = status;
        }
    }
}
