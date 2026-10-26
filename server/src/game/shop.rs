use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::models::unit::{Unit, Trait};

/// Pool of available units, shared across all players like TFT
/// Each cost tier has a fixed pool count that depletes as players buy
pub struct UnitPool {
    pool: Vec<Unit>,
}

impl UnitPool {
    /// Initialize the shared unit pool with all available champions
    pub fn new() -> Self {
        let mut pool = Vec::new();

        // Cost 1 — 29 copies each
        for _ in 0..29 {
            pool.push(Unit::new("Soldier".into(), 1, vec![Trait::Warrior], 500, 50, 1));
            pool.push(Unit::new("Scout".into(), 1, vec![Trait::Ranger], 400, 60, 3));
            pool.push(Unit::new("Acolyte".into(), 1, vec![Trait::Mage], 350, 45, 4));
        }
        // Cost 2 — 22 copies each
        for _ in 0..22 {
            pool.push(Unit::new("Knight".into(), 2, vec![Trait::Tank, Trait::Warrior], 750, 70, 1));
            pool.push(Unit::new("Archer".into(), 2, vec![Trait::Ranger], 550, 80, 4));
        }
        // Cost 3 — 18 copies each
        for _ in 0..18 {
            pool.push(Unit::new("Duelist".into(), 3, vec![Trait::Assassin, Trait::Warrior], 700, 120, 1));
            pool.push(Unit::new("Sorcerer".into(), 3, vec![Trait::Mage], 600, 100, 5));
        }
        // Cost 4 — 12 copies each
        for _ in 0..12 {
            pool.push(Unit::new("Paladin".into(), 4, vec![Trait::Tank, Trait::Support], 1100, 90, 1));
            pool.push(Unit::new("Sniper".into(), 4, vec![Trait::Ranger, Trait::Assassin], 750, 150, 5));
        }
        // Cost 5 — 10 copies each
        for _ in 0..10 {
            pool.push(Unit::new("Warlord".into(), 5, vec![Trait::Warrior, Trait::Tank], 1400, 200, 1));
        }

        Self { pool }
    }

    /// Draw `count` units for a player's shop offer (without removing from pool yet)
    pub fn roll_shop(&self, player_level: u8, count: usize) -> Vec<Unit> {
        let weights = tier_weights(player_level);
        let mut rng = thread_rng();
        let mut shop = Vec::with_capacity(count);

        let available: Vec<&Unit> = self.pool.iter().collect();
        if available.is_empty() {
            return shop;
        }

        for _ in 0..count {
            // Pick a cost tier first, then a random unit of that tier
            let cost = weighted_cost_pick(&weights, &mut rng);
            let candidates: Vec<&Unit> = available.iter()
                .filter(|u| u.cost == cost)
                .copied()
                .collect();
            if let Some(unit) = candidates.choose(&mut rng) {
                shop.push((*unit).clone());
            }
        }
        shop
    }

    /// Remove a unit from the pool (called when a player buys it)
    pub fn remove(&mut self, unit_name: &str, cost: u8) -> Option<Unit> {
        if let Some(idx) = self.pool.iter().position(|u| u.name == unit_name && u.cost == cost) {
            Some(self.pool.remove(idx))
        } else {
            None
        }
    }

    /// Return a unit to the pool (called on sell)
    pub fn return_unit(&mut self, unit: Unit) {
        self.pool.push(unit);
    }
}

/// TFT-style odds per level: [1-cost%, 2-cost%, 3-cost%, 4-cost%, 5-cost%]
fn tier_weights(level: u8) -> [u8; 5] {
    match level {
        1 => [100, 0, 0, 0, 0],
        2 => [100, 0, 0, 0, 0],
        3 => [75, 25, 0, 0, 0],
        4 => [55, 30, 15, 0, 0],
        5 => [45, 33, 20, 2, 0],
        6 => [25, 40, 30, 5, 0],
        7 => [19, 30, 35, 15, 1],
        8 => [15, 20, 35, 25, 5],
        _ => [10, 15, 25, 35, 15], // level 9
    }
}

fn weighted_cost_pick(weights: &[u8; 5], rng: &mut impl rand::Rng) -> u8 {
    let roll: u8 = rng.gen_range(0..100);
    let mut cumulative = 0u8;
    for (i, &w) in weights.iter().enumerate() {
        cumulative += w;
        if roll < cumulative {
            return (i + 1) as u8;
        }
    }
    1 // fallback
}
