use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A player in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,

    // In-game economy
    pub gold: u32,
    pub health: u32,
    pub level: u8,
    pub xp: u32,
}

impl Player {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            created_at: Utc::now(),
            gold: 2,       // TFT-like starting gold
            health: 100,
            level: 1,
            xp: 0,
        }
    }

    /// XP needed to reach the next level
    pub fn xp_to_next_level(&self) -> u32 {
        match self.level {
            1 => 2,
            2 => 6,
            3 => 10,
            4 => 20,
            5 => 36,
            6 => 56,
            7 => 80,
            8 => 100,
            _ => u32::MAX, // level 9 is max
        }
    }

    pub fn buy_xp(&mut self) -> bool {
        if self.gold >= 4 {
            self.gold -= 4;
            self.xp += 4;
            if self.xp >= self.xp_to_next_level() && self.level < 9 {
                self.level += 1;
                self.xp = 0;
            }
            true
        } else {
            false
        }
    }
}
