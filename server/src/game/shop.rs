use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::models::unit::{Unit, all_champions, pool_size_for_cost};

// ---------------------------------------------------------------------------
// UnitPool — pool tướng dùng chung cho toàn bộ lobby (giống TFT)
// ---------------------------------------------------------------------------

pub struct UnitPool {
    /// name → Vec<Unit> (các bản copy còn lại trong pool)
    pool: HashMap<String, Vec<Unit>>,
}

impl UnitPool {
    /// Khởi tạo pool với 42 tướng thật, số bản copy theo chuẩn TFT:
    /// 1 vàng = 29 bản, 2 vàng = 22, 3 vàng = 18, 4 vàng = 12, 5 vàng = 10
    pub fn new() -> Self {
        let templates = all_champions();
        let mut pool: HashMap<String, Vec<Unit>> = HashMap::new();

        for template in &templates {
            let copies = pool_size_for_cost(template.cost);
            let stack: Vec<Unit> = (0..copies).map(|_| {
                let mut unit = template.clone();
                unit.id = uuid::Uuid::new_v4(); // mỗi bản copy có ID riêng
                unit
            }).collect();
            pool.insert(template.name.clone(), stack);
        }

        Self { pool }
    }

    /// Lấy 5 tướng cho shop của player (không xóa khỏi pool — chỉ preview).
    /// Gọi `remove` khi player thực sự mua.
    pub fn roll_shop(&self, player_level: u8) -> Vec<Unit> {
        let weights = tier_weights(player_level);
        let mut rng = rand::thread_rng();
        let mut shop: Vec<Unit> = Vec::with_capacity(5);

        // Danh sách tên tướng theo từng bậc giá còn trong pool
        let mut by_cost: [Vec<&str>; 5] = Default::default();
        for (name, stack) in &self.pool {
            if !stack.is_empty() {
                if let Some(first) = stack.first() {
                    let idx = (first.cost as usize).saturating_sub(1).min(4);
                    by_cost[idx].push(name.as_str());
                }
            }
        }

        for _ in 0..5 {
            let cost = weighted_cost_pick(&weights, &mut rng);
            let idx = (cost as usize).saturating_sub(1).min(4);
            if let Some(&name) = by_cost[idx].choose(&mut rng) {
                if let Some(stack) = self.pool.get(name) {
                    if let Some(unit) = stack.first() {
                        shop.push(unit.clone());
                    }
                }
            }
        }

        shop
    }

    /// Xóa 1 bản copy khỏi pool khi player mua.
    /// Trả về Unit nếu thành công, None nếu đã hết.
    pub fn remove(&mut self, unit_name: &str) -> Option<Unit> {
        self.pool.get_mut(unit_name)?.pop()
    }

    /// Trả 1 bản copy vào pool khi player bán tướng.
    pub fn return_unit(&mut self, unit: Unit) {
        // Reset về 1 sao trước khi trả lại pool
        let mut base = unit;
        base.tier = 1;
        base.current_hp = base.base_hp;
        base.current_mana = base.base_mana;
        self.pool.entry(base.name.clone()).or_default().push(base);
    }

    /// Số bản còn lại của một tướng (hữu ích để hiện UI "còn X bản")
    pub fn remaining(&self, unit_name: &str) -> usize {
        self.pool.get(unit_name).map_or(0, |v| v.len())
    }
}

// ---------------------------------------------------------------------------
// Tỉ lệ roll theo level — giống TFT (5 bậc giá)
// [1-cost%, 2-cost%, 3-cost%, 4-cost%, 5-cost%]
// ---------------------------------------------------------------------------

fn tier_weights(level: u8) -> [u8; 5] {
    match level {
        1 | 2 => [100,  0,  0,  0,  0],
        3     => [ 75, 25,  0,  0,  0],
        4     => [ 55, 30, 15,  0,  0],
        5     => [ 45, 33, 20,  2,  0],
        6     => [ 25, 40, 30,  5,  0],
        7     => [ 19, 30, 35, 15,  1],
        8     => [ 15, 20, 35, 25,  5],
        _     => [ 10, 15, 25, 35, 15], // level 9
    }
}

fn weighted_cost_pick(weights: &[u8; 5], rng: &mut impl Rng) -> u8 {
    let roll: u8 = rng.gen_range(0..100);
    let mut cumulative = 0u8;
    for (i, &w) in weights.iter().enumerate() {
        cumulative = cumulative.saturating_add(w);
        if roll < cumulative {
            return (i + 1) as u8;
        }
    }
    1
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_has_all_champions() {
        let pool = UnitPool::new();
        // 9 + 12 + 9 + 6 + 6 = 42 tướng duy nhất
        assert_eq!(pool.pool.len(), 42);
    }

    #[test]
    fn pool_copy_counts_correct() {
        let pool = UnitPool::new();
        // Kiểm tra một đại diện mỗi bậc
        assert_eq!(pool.remaining("Jikey"), 29);
        assert_eq!(pool.remaining("MeoU"), 22);
        assert_eq!(pool.remaining("Pino"), 18);
        assert_eq!(pool.remaining("DjChip"), 12);
        assert_eq!(pool.remaining("Độ Mixi"), 10);
    }

    #[test]
    fn roll_returns_5_units() {
        let pool = UnitPool::new();
        let shop = pool.roll_shop(5);
        assert_eq!(shop.len(), 5);
    }

    #[test]
    fn remove_decrements_pool() {
        let mut pool = UnitPool::new();
        let before = pool.remaining("Jikey");
        pool.remove("Jikey");
        assert_eq!(pool.remaining("Jikey"), before - 1);
    }

    #[test]
    fn return_unit_increments_pool() {
        let mut pool = UnitPool::new();
        let unit = pool.remove("Jikey").unwrap();
        let before = pool.remaining("Jikey");
        pool.return_unit(unit);
        assert_eq!(pool.remaining("Jikey"), before + 1);
    }
}
