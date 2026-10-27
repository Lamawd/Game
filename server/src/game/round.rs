use crate::models::player::Player;

/// Tính vàng nhận được đầu vòng
pub fn calculate_income(player: &Player, win_streak: u32, loss_streak: u32) -> u32 {
    let base = 5u32;
    let interest = (player.gold / 10).min(5);
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

/// Phase của một vòng đấu
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoundPhase {
    Planning,
    Combat,
    Results,
}

use serde::{Deserialize, Serialize};
