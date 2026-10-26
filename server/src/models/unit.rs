use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trait classifications (synergies) for units
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Trait {
    Warrior,
    Mage,
    Assassin,
    Tank,
    Ranger,
    Support,
}

/// A single unit / champion on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub id: Uuid,
    pub name: String,
    pub cost: u8,          // 1–5 gold cost tier
    pub tier: u8,          // 1, 2, or 3 stars
    pub traits: Vec<Trait>,

    // Base stats (scaled by tier)
    pub base_hp: u32,
    pub base_attack: u32,
    pub base_armor: u32,
    pub base_magic_resist: u32,
    pub attack_speed: f32,
    pub ability_power: u32,
    pub range: u8,         // 1 = melee, 2+ = ranged

    // Current combat stats (derived at fight start)
    pub current_hp: u32,
    pub current_mana: u32,
    pub max_mana: u32,
}

impl Unit {
    pub fn new(name: String, cost: u8, traits: Vec<Trait>, base_hp: u32, base_attack: u32, range: u8) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            cost,
            tier: 1,
            traits,
            base_hp,
            base_attack,
            base_armor: 20,
            base_magic_resist: 20,
            attack_speed: 0.75,
            ability_power: 100,
            range,
            current_hp: base_hp,
            current_mana: 0,
            max_mana: 100,
        }
    }

    /// Returns hp scaled by star tier (tier 2 = 1.8x, tier 3 = 3.24x)
    pub fn max_hp(&self) -> u32 {
        let multiplier = match self.tier {
            1 => 1.0,
            2 => 1.8,
            3 => 3.24,
            _ => 1.0,
        };
        (self.base_hp as f32 * multiplier) as u32
    }

    /// Returns attack scaled by tier
    pub fn attack_damage(&self) -> u32 {
        let multiplier = match self.tier {
            1 => 1.0,
            2 => 1.8,
            3 => 3.24,
            _ => 1.0,
        };
        (self.base_attack as f32 * multiplier) as u32
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    /// Upgrade to next star tier (merge 3 copies)
    pub fn upgrade(&mut self) {
        if self.tier < 3 {
            self.tier += 1;
            self.current_hp = self.max_hp();
        }
    }
}
