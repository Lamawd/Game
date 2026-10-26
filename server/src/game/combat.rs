use crate::models::unit::Unit;
use crate::models::board::Board;

/// Result of a single combat simulation
#[derive(Debug)]
pub struct CombatResult {
    pub winner_player_id: Option<uuid::Uuid>, // None = draw
    pub surviving_units: Vec<Unit>,
    pub damage_dealt: u32,
    pub rounds_taken: u32,
}

/// Simulate combat between two boards.
/// This is a simplified tick-based simulation — expand with proper
/// targeting, ability triggers, and trait bonuses later.
pub fn simulate(attacker_board: &mut Board, defender_board: &mut Board) -> CombatResult {
    let mut ticks = 0u32;
    const MAX_TICKS: u32 = 200; // safety cap

    while ticks < MAX_TICKS {
        ticks += 1;

        let attacker_alive = attacker_board.placed.values().filter(|u| u.is_alive()).count();
        let defender_alive = defender_board.placed.values().filter(|u| u.is_alive()).count();

        if attacker_alive == 0 || defender_alive == 0 {
            break;
        }

        // Very simplified: every attacker damages every defender each tick
        // TODO: replace with hex-distance targeting, attack speed, ability casts
        tick_combat(attacker_board, defender_board);
        tick_combat(defender_board, attacker_board);
    }

    let attacker_survivors: Vec<Unit> = attacker_board
        .placed
        .values()
        .filter(|u| u.is_alive())
        .cloned()
        .collect();

    let defender_survivors: Vec<Unit> = defender_board
        .placed
        .values()
        .filter(|u| u.is_alive())
        .cloned()
        .collect();

    let winner_player_id = if !attacker_survivors.is_empty() && defender_survivors.is_empty() {
        Some(attacker_board.owner_id)
    } else if !defender_survivors.is_empty() && attacker_survivors.is_empty() {
        Some(defender_board.owner_id)
    } else {
        None // draw or timeout
    };

    let damage = calculate_damage(&attacker_survivors);

    let surviving_units = if winner_player_id == Some(attacker_board.owner_id) {
        attacker_survivors
    } else {
        defender_survivors
    };

    CombatResult {
        winner_player_id,
        surviving_units,
        damage_dealt: damage,
        rounds_taken: ticks,
    }
}

fn tick_combat(attackers: &Board, defenders: &mut Board) {
    let attack_total: u32 = attackers
        .placed
        .values()
        .filter(|u| u.is_alive())
        .map(|u| u.attack_damage())
        .sum();

    if attack_total == 0 {
        return;
    }

    // Distribute damage evenly across defenders (placeholder logic)
    let defender_count = defenders.placed.values().filter(|u| u.is_alive()).count();
    if defender_count == 0 {
        return;
    }
    let dmg_per_target = (attack_total / defender_count as u32).max(1);

    for unit in defenders.placed.values_mut() {
        if unit.is_alive() {
            let mitigated = dmg_per_target.saturating_sub(unit.base_armor / 5);
            unit.current_hp = unit.current_hp.saturating_sub(mitigated);
        }
    }
}

/// TFT-style: base damage = player level + sum of surviving unit costs
fn calculate_damage(survivors: &[Unit]) -> u32 {
    let unit_cost_sum: u32 = survivors.iter().map(|u| u.cost as u32).sum();
    // Base 1 + 1 per surviving unit cost (rounded)
    1 + unit_cost_sum
}
