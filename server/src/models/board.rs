use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::models::unit::Unit;

/// A hex grid position on the board (col, row)
/// Board is 7 wide x 4 deep (player side) like TFT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexPosition {
    pub col: u8,
    pub row: u8,
}

impl HexPosition {
    pub fn new(col: u8, row: u8) -> Self {
        Self { col, row }
    }

    pub fn is_valid(&self) -> bool {
        self.col < 7 && self.row < 4
    }
}

/// The player's board — placement grid + bench
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub owner_id: Uuid,
    /// Units placed on the hex grid (position -> unit)
    pub placed: HashMap<String, Unit>, // key = "col,row"
    /// Units on the bench (up to 9)
    pub bench: Vec<Unit>,
}

impl Board {
    pub fn new(owner_id: Uuid) -> Self {
        Self {
            owner_id,
            placed: HashMap::new(),
            bench: Vec::new(),
        }
    }

    fn pos_key(pos: HexPosition) -> String {
        format!("{},{}", pos.col, pos.row)
    }

    /// Place a unit from bench onto the board grid
    pub fn place_unit(&mut self, bench_index: usize, pos: HexPosition) -> bool {
        if !pos.is_valid() || bench_index >= self.bench.len() {
            return false;
        }
        if self.placed.contains_key(&Self::pos_key(pos)) {
            return false; // position occupied
        }
        let unit = self.bench.remove(bench_index);
        self.placed.insert(Self::pos_key(pos), unit);
        true
    }

    /// Move a placed unit to a new position
    pub fn move_unit(&mut self, from: HexPosition, to: HexPosition) -> bool {
        if !to.is_valid() {
            return false;
        }
        let from_key = Self::pos_key(from);
        let to_key = Self::pos_key(to);
        if let Some(unit) = self.placed.remove(&from_key) {
            // Swap if target is occupied
            if let Some(other) = self.placed.remove(&to_key) {
                self.placed.insert(from_key, other);
            }
            self.placed.insert(to_key, unit);
            true
        } else {
            false
        }
    }

    /// Send a placed unit back to the bench
    pub fn bench_unit(&mut self, pos: HexPosition) -> bool {
        if let Some(unit) = self.placed.remove(&Self::pos_key(pos)) {
            if self.bench.len() < 9 {
                self.bench.push(unit);
                return true;
            } else {
                // Put back if bench is full
                self.placed.insert(Self::pos_key(pos), unit);
            }
        }
        false
    }

    /// How many units are currently placed (counts toward level cap)
    pub fn placed_count(&self) -> usize {
        self.placed.len()
    }

    /// Add a unit directly to bench (e.g. after buying from shop)
    pub fn add_to_bench(&mut self, unit: Unit) -> bool {
        if self.bench.len() < 9 {
            self.bench.push(unit);
            true
        } else {
            false
        }
    }
}
