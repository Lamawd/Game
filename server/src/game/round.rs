use crate::models::player::Player;

/// How much gold a player earns at the start of each round
pub fn calculate_income(player: &Player, win_streak: u32, loss_streak: u32) -> u32 {
    let base = 5u32;

    // Interest: +1 per 10 gold, max +5
    let interest = (player.gold / 10).min(5);

    // Streak bonus
    let streak_bonus = if win_streak >= 5 || loss_streak >= 5 {
        3
    } else if win_streak >= 3 || loss_streak >= 3 {
        2
    } else if win_streak >= 2 || loss_streak >= 2 {
        1
    } else {
        0
    };

    base + interest + streak_bonus
}

/// Round phase order (mirrors TFT pacing)
#[derive(Debug, Clone, PartialEq)]
pub enum RoundPhase {
    Planning,  // Players buy/sell/move units
    Combat,    // Auto-battle resolves
    Results,   // Show outcome, hand out gold
}

/// Increment round and return its phase
pub fn advance_round(current_round: u32) -> (u32, RoundPhase) {
    let next = current_round + 1;
    // Rounds divisible by 7 are PvE carousel (simplified: still combat)
    (next, RoundPhase::Planning)
}
