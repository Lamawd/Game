/// Bot AI đơn giản dùng để test khi chưa đủ 8 player thật.
///
/// Chiến lược: mỗi vòng Planning bot sẽ:
///   1. Roll shop cho đến khi có tướng mua được (tối đa 3 lần)
///   2. Mua tướng rẻ nhất có thể
///   3. Đặt tất cả tướng bench lên board (các ô đầu tiên còn trống)
///   4. Nhấn Ready
use rand::seq::SliceRandom;
use uuid::Uuid;

use crate::models::board::{Board, HexPosition};
use crate::models::player::Player;
use crate::game::shop::UnitPool;

/// Kết quả mỗi lần bot "suy nghĩ"
#[derive(Debug)]
pub struct BotActions {
    pub bought_unit_names: Vec<String>,  // tên tướng đã mua
    pub gold_spent: u32,
    pub rolled_count: u32,
}

/// Chạy một bước planning cho bot.
/// Trả về list tên tướng bot muốn mua (caller tự deduct gold và update pool).
pub fn plan(
    player: &Player,
    board: &Board,
    pool: &UnitPool,
) -> BotActions {
    let mut actions = BotActions {
        bought_unit_names: Vec::new(),
        gold_spent: 0,
        rolled_count: 0,
    };

    let mut remaining_gold = player.gold;
    let bench_space = 9_usize.saturating_sub(board.bench.len());

    if bench_space == 0 {
        return actions; // bench đầy, không mua thêm
    }

    // Roll tối đa 3 lần để tìm tướng mua được
    let max_rolls = 3u32;
    for _ in 0..max_rolls {
        if remaining_gold < 2 {
            break;
        }

        let offer = pool.roll_shop(player.level);
        actions.rolled_count += 1;
        remaining_gold = remaining_gold.saturating_sub(2);

        // Chọn tướng rẻ nhất trong offer mà còn đủ tiền mua
        let affordable: Vec<&crate::models::unit::Unit> = offer.iter()
            .filter(|u| u.cost as u32 <= remaining_gold && pool.remaining(&u.name) > 0)
            .collect();

        if let Some(unit) = affordable.iter().min_by_key(|u| u.cost) {
            let name = unit.name.clone();
            let cost = unit.cost as u32;
            actions.bought_unit_names.push(name);
            actions.gold_spent += cost + 2; // cost + roll cost
            remaining_gold = remaining_gold.saturating_sub(cost);
            break; // mua 1 tướng mỗi vòng là đủ cho test
        }
    }

    actions
}

/// Quyết định đặt tướng nào lên ô nào trên board.
/// Trả về Vec<(unit_id, position)>.
pub fn decide_placement(board: &Board) -> Vec<(Uuid, HexPosition)> {
    let mut placements = Vec::new();
    let mut rng = rand::thread_rng();

    // Tất cả ô board hợp lệ (col 0-6, row 0-3)
    let mut free_positions: Vec<HexPosition> = (0u8..7)
        .flat_map(|c| (0u8..4).map(move |r| HexPosition::new(c, r)))
        .filter(|pos| !board.placed.contains_key(&format!("{},{}", pos.col, pos.row)))
        .collect();

    free_positions.shuffle(&mut rng);

    // Đặt hết bench lên board (không vượt số ô còn trống)
    let to_place = board.bench.len().min(free_positions.len());
    for i in 0..to_place {
        placements.push((board.bench[i].id, free_positions[i]));
    }

    placements
}

/// Tạo một Player bot với tên ngẫu nhiên và đăng ký vào state.
pub fn create_bot_name(index: u8) -> String {
    let names = [
        "Bot_Nhism", "Bot_FunkyM", "Bot_FAT", "Bot_Rambo",
        "Bot_Snake", "Bot_Pino", "Bot_MeoU", "Bot_Levi",
    ];
    names
        .get(index as usize)
        .copied()
        .unwrap_or("Bot_Unknown")
        .to_string()
}
