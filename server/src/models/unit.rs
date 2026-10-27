use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::traits::{CombatRole, UnitTrait};

// ---------------------------------------------------------------------------
// Unit struct
// ---------------------------------------------------------------------------

/// Một tướng / champion trên bàn cờ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub id: Uuid,
    pub name: String,
    pub cost: u8, // Bậc giá 1–5

    /// Sao hiện tại: 1, 2, hoặc 3
    pub tier: u8,

    /// Tộc và Hệ mà tướng thuộc về
    pub traits: Vec<UnitTrait>,

    /// Vai trò chiến đấu (dùng cho targeting)
    pub role: CombatRole,

    // ── Chỉ số cơ bản (1 sao) ────────────────────────────────────────────
    pub base_hp: u32,
    pub base_attack: u32,        // Sát thương vật lý cơ bản
    pub base_armor: u32,         // Giáp
    pub base_magic_resist: u32,  // Kháng phép
    pub base_attack_speed: f32,  // Số đòn đánh/giây (1 sao)
    pub base_ability_power: u32, // Sức mạnh phép thuật (SMPT)
    pub base_crit_chance: f32,   // Tỉ lệ chí mạng (0.0 – 1.0)
    pub base_mana: u32,          // Năng lượng khởi điểm
    pub max_mana: u32,           // Năng lượng tối đa để tung chiêu
    pub range: u8,               // 1 = cận chiến, 2+ = tầm xa

    // ── Chỉ số chiến đấu hiện tại (được tính lại mỗi trận) ───────────────
    pub current_hp: u32,
    pub current_mana: u32,
}

impl Unit {
    /// Tạo tướng mới với chỉ số đầy đủ
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        cost: u8,
        traits: Vec<UnitTrait>,
        role: CombatRole,
        base_hp: u32,
        base_attack: u32,
        base_armor: u32,
        base_magic_resist: u32,
        base_attack_speed: f32,
        base_ability_power: u32,
        base_crit_chance: f32,
        base_mana: u32,
        max_mana: u32,
        range: u8,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            cost,
            tier: 1,
            traits,
            role,
            base_hp,
            base_attack,
            base_armor,
            base_magic_resist,
            base_attack_speed,
            base_ability_power,
            base_crit_chance,
            base_mana,
            max_mana,
            range,
            current_hp: base_hp,
            current_mana: base_mana,
        }
    }

    // ── Scaling theo sao ──────────────────────────────────────────────────

    fn tier_mult(&self) -> f32 {
        match self.tier {
            1 => 1.0,
            2 => 1.8,
            3 => 3.24,
            _ => 1.0,
        }
    }

    pub fn max_hp(&self) -> u32 {
        (self.base_hp as f32 * self.tier_mult()) as u32
    }

    pub fn attack_damage(&self) -> u32 {
        (self.base_attack as f32 * self.tier_mult()) as u32
    }

    pub fn attack_speed(&self) -> f32 {
        // Tốc độ đánh không scale theo sao (chỉ scale qua đồ/synergy)
        self.base_attack_speed
    }

    pub fn ability_power(&self) -> u32 {
        (self.base_ability_power as f32 * self.tier_mult()) as u32
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    /// Nâng sao (gộp 3 bản)
    pub fn upgrade(&mut self) {
        if self.tier < 3 {
            self.tier += 1;
            self.current_hp = self.max_hp();
        }
    }

    /// Reset về trạng thái đầu trận
    pub fn reset_for_combat(&mut self) {
        self.current_hp = self.max_hp();
        self.current_mana = self.base_mana;
    }
}

// ---------------------------------------------------------------------------
// Champion roster — 42 tướng mùa 1
// Các hàm trả về Unit với chỉ số đã cân bằng theo tier giá và role.
//
// Quy ước chỉ số theo bậc giá (1 sao):
//   1 vàng: HP ~500-650,  AD ~40-55,  Armor ~20, MR ~20
//   2 vàng: HP ~650-850,  AD ~55-75,  Armor ~25, MR ~25
//   3 vàng: HP ~800-1050, AD ~70-100, Armor ~30, MR ~30
//   4 vàng: HP ~950-1200, AD ~90-130, Armor ~35, MR ~35
//   5 vàng: HP ~1100-1400,AD ~120-170,Armor ~40, MR ~40
// ---------------------------------------------------------------------------

// ════════════════════════════════════════════════════════════════════════════
// 1 VÀNG (9 tướng): Jikey, Boongminz, Dev, Linda Tạ, Lil Chi,
//                   Lee Harri, Hữu Nghĩa, Vũ Mino, Leopard
// ════════════════════════════════════════════════════════════════════════════

pub fn jikey() -> Unit {
    Unit::new("Jikey", 1,
        vec![UnitTrait::NoXit, UnitTrait::TayTo],
        CombatRole::Bruiser,
        600, 50, 25, 20, 0.70, 100, 0.05, 0, 80, 1)
}

pub fn boongminz() -> Unit {
    Unit::new("Boongminz", 1,
        vec![UnitTrait::BoHuc, UnitTrait::DaStream],
        CombatRole::Marksman,
        500, 55, 20, 20, 0.90, 100, 0.05, 0, 80, 4)
}

pub fn dev() -> Unit {
    Unit::new("Dev", 1,
        vec![UnitTrait::MayNoi, UnitTrait::FapSu],
        CombatRole::Mage,
        550, 40, 20, 25, 0.65, 110, 0.05, 20, 80, 4)
}

pub fn linda_ta() -> Unit {
    Unit::new("Linda Tạ", 1,
        vec![UnitTrait::NocNha, UnitTrait::FapSu],
        CombatRole::Mage,
        500, 38, 20, 25, 0.65, 115, 0.05, 20, 80, 4)
}

pub fn lil_chi() -> Unit {
    Unit::new("Lil Chi", 1,
        vec![UnitTrait::NocNha, UnitTrait::DaStream],
        CombatRole::Marksman,
        480, 52, 20, 20, 0.85, 100, 0.08, 0, 75, 4)
}

pub fn lee_harri() -> Unit {
    Unit::new("Lee Harri", 1,
        vec![UnitTrait::ChienSi, UnitTrait::DaStream],
        CombatRole::Assassin,
        520, 58, 25, 20, 0.80, 100, 0.15, 0, 75, 1)
}

pub fn huu_nghia() -> Unit {
    Unit::new("Hữu Nghĩa", 1,
        vec![UnitTrait::Bros500, UnitTrait::ChatThep],
        CombatRole::Bruiser,
        650, 45, 30, 25, 0.65, 100, 0.05, 0, 80, 1)
}

pub fn vu_mino() -> Unit {
    Unit::new("Vũ Mino", 1,
        vec![UnitTrait::DaStream, UnitTrait::TayTo],
        CombatRole::Marksman,
        480, 55, 20, 20, 0.90, 100, 0.05, 0, 80, 4)
}

pub fn leopard() -> Unit {
    Unit::new("Leopard", 1,
        vec![UnitTrait::Bros500, UnitTrait::ChienSi],
        CombatRole::VanguardAD,
        640, 48, 35, 25, 0.65, 100, 0.05, 0, 80, 1)
}

// ════════════════════════════════════════════════════════════════════════════
// 2 VÀNG (12 tướng): MeoU, FunkyM, Quang Volka, FAT, Rambo, Thanh Chin,
//                    Trân Mèo, Huy HD, Tuyền Văn Hóa, Bụt (Duy Nhỏ),
//                    ĐạtKa, Huy Huế
// ════════════════════════════════════════════════════════════════════════════

pub fn meou() -> Unit {
    Unit::new("MeoU", 2,
        vec![UnitTrait::ToDoi97, UnitTrait::NoXit],
        CombatRole::VanguardAD,
        750, 70, 35, 30, 0.70, 100, 0.05, 0, 85, 1)
}

pub fn funky_m() -> Unit {
    Unit::new("FunkyM", 2,
        vec![UnitTrait::BoHuc, UnitTrait::BanNga, UnitTrait::ChienSi],
        CombatRole::Warden,
        900, 60, 45, 40, 0.60, 100, 0.05, 0, 85, 1)
}

pub fn quang_volka() -> Unit {
    Unit::new("Quang Volka", 2,
        vec![UnitTrait::ChatThep, UnitTrait::TayTo],
        CombatRole::Bruiser,
        850, 72, 30, 25, 0.65, 100, 0.05, 0, 85, 1)
}

pub fn fat() -> Unit {
    Unit::new("FAT", 2,
        vec![UnitTrait::BoHuc, UnitTrait::ChatThep],
        CombatRole::Bruiser,
        950, 65, 35, 30, 0.60, 100, 0.05, 0, 85, 1)
}

pub fn rambo() -> Unit {
    Unit::new("Rambo", 2,
        vec![UnitTrait::BoHuc, UnitTrait::ChienSi],
        CombatRole::VanguardAD,
        800, 70, 45, 30, 0.65, 100, 0.05, 0, 85, 1)
}

pub fn thanh_chin() -> Unit {
    Unit::new("Thanh Chin", 2,
        vec![UnitTrait::NocNha, UnitTrait::FapSu],
        CombatRole::Mage,
        680, 55, 25, 35, 0.65, 125, 0.05, 25, 85, 4)
}

pub fn tran_meo() -> Unit {
    Unit::new("Trân Mèo", 2,
        vec![UnitTrait::NocNha, UnitTrait::ChatThep],
        CombatRole::VanguardMage,
        820, 58, 38, 38, 0.60, 115, 0.05, 15, 85, 1)
}

pub fn huy_hd() -> Unit {
    Unit::new("Huy HD", 2,
        vec![UnitTrait::ChatThep, UnitTrait::ChienSi],
        CombatRole::Warden,
        950, 55, 50, 45, 0.55, 100, 0.05, 0, 85, 1)
}

pub fn tuyen_van_hoa() -> Unit {
    Unit::new("Tuyền Văn Hóa", 2,
        vec![UnitTrait::DaStream, UnitTrait::FapSu],
        CombatRole::MagicMarksman,
        620, 62, 25, 30, 0.85, 115, 0.05, 10, 85, 4)
}

pub fn but_duy_nho() -> Unit {
    Unit::new("Bụt (Duy Nhỏ)", 2,
        vec![UnitTrait::TayTo, UnitTrait::ChatThep],
        CombatRole::Bruiser,
        880, 72, 30, 25, 0.65, 100, 0.05, 0, 85, 1)
}

pub fn datka() -> Unit {
    Unit::new("ĐạtKa", 2,
        vec![UnitTrait::FapSu, UnitTrait::DaStream],
        CombatRole::Mage,
        640, 52, 25, 30, 0.70, 120, 0.05, 20, 85, 4)
}

pub fn huy_hue() -> Unit {
    Unit::new("Huy Huế", 2,
        vec![UnitTrait::DaStream, UnitTrait::TayTo],
        CombatRole::Marksman,
        640, 74, 25, 25, 0.90, 100, 0.10, 0, 85, 4)
}

// ════════════════════════════════════════════════════════════════════════════
// 3 VÀNG (9 tướng): Pino, Nhism, QNT, Nhạc Nguyễn, Snake,
//                   Lê Khôi, Hà Beso, Masew, Thế Anh 96
// ════════════════════════════════════════════════════════════════════════════

pub fn pino() -> Unit {
    // Mặc định AD — có thể đổi role sang AP (Thầy Onip) bằng trang bị
    Unit::new("Pino", 3,
        vec![UnitTrait::BanNga, UnitTrait::NoXit, UnitTrait::DaStream],
        CombatRole::Marksman,
        850, 95, 30, 30, 0.85, 100, 0.10, 0, 90, 4)
}

pub fn nhism() -> Unit {
    Unit::new("Nhism", 3,
        vec![UnitTrait::ToDoi97, UnitTrait::DaStream],
        CombatRole::Marksman,
        820, 100, 28, 28, 0.95, 100, 0.10, 0, 90, 4)
}

pub fn qnt() -> Unit {
    Unit::new("QNT", 3,
        vec![UnitTrait::LuaDao, UnitTrait::FapSu],
        CombatRole::Mage,
        800, 72, 30, 38, 0.68, 140, 0.05, 30, 90, 4)
}

pub fn nhac_nguyen() -> Unit {
    Unit::new("Nhạc Nguyễn", 3,
        vec![UnitTrait::LuaDao, UnitTrait::FapSu],
        CombatRole::Mage,
        790, 70, 30, 38, 0.68, 140, 0.05, 30, 90, 4)
}

pub fn snake() -> Unit {
    // Mặc định AD Sát thủ — có thể đổi sang AP (Aben) bằng trang bị
    Unit::new("Snake", 3,
        vec![UnitTrait::BanNga, UnitTrait::ChienSi],
        CombatRole::Assassin,
        820, 105, 30, 30, 0.80, 100, 0.25, 0, 90, 1)
}

pub fn le_khoi() -> Unit {
    Unit::new("Lê Khôi", 3,
        vec![UnitTrait::LuaDao, UnitTrait::MayNoi, UnitTrait::FapSu],
        CombatRole::Mage,
        830, 68, 30, 40, 0.65, 150, 0.05, 35, 90, 4)
}

pub fn ha_beso() -> Unit {
    Unit::new("Hà Beso", 3,
        vec![UnitTrait::NocNha, UnitTrait::TayTo],
        CombatRole::Bruiser,
        1000, 90, 38, 35, 0.65, 100, 0.05, 0, 90, 1)
}

pub fn masew() -> Unit {
    Unit::new("Masew", 3,
        vec![UnitTrait::FapSu, UnitTrait::ChatThep],
        CombatRole::Mage,
        900, 65, 30, 40, 0.65, 155, 0.05, 30, 90, 4)
}

pub fn the_anh_96() -> Unit {
    Unit::new("Thế Anh 96", 3,
        vec![UnitTrait::Bros500, UnitTrait::DaStream],
        CombatRole::Marksman,
        840, 100, 28, 28, 0.95, 100, 0.12, 0, 90, 4)
}

// ════════════════════════════════════════════════════════════════════════════
// 4 VÀNG (6 tướng): DjChip, Trang Mixi, Thầy Giáo Ba,
//                   Cris Devil Gamer, Dũng Kajima, Mimosa
// ════════════════════════════════════════════════════════════════════════════

pub fn djchip() -> Unit {
    Unit::new("DjChip", 4,
        vec![UnitTrait::ChatThep, UnitTrait::FapSu],
        CombatRole::VanguardMage,
        1100, 90, 45, 45, 0.65, 160, 0.05, 25, 95, 1)
}

pub fn trang_mixi() -> Unit {
    Unit::new("Trang Mixi", 4,
        vec![UnitTrait::NocNha, UnitTrait::FapSu],
        CombatRole::MagicMarksman,
        950, 95, 35, 40, 0.75, 150, 0.08, 20, 95, 4)
}

pub fn thay_giao_ba() -> Unit {
    Unit::new("Thầy Giáo Ba", 4,
        vec![UnitTrait::TayTo, UnitTrait::DaStream],
        CombatRole::Marksman,
        950, 130, 32, 30, 1.00, 100, 0.15, 0, 95, 4)
}

pub fn cris_devil_gamer() -> Unit {
    Unit::new("Cris Devil Gamer", 4,
        vec![UnitTrait::DaStream, UnitTrait::FapSu],
        CombatRole::MagicMarksman,
        980, 110, 32, 38, 0.85, 130, 0.10, 15, 95, 4)
}

pub fn dung_kajima() -> Unit {
    Unit::new("Dũng Kajima", 4,
        vec![UnitTrait::Bros500, UnitTrait::ChatThep],
        CombatRole::VanguardMage,
        1150, 85, 48, 48, 0.60, 155, 0.05, 20, 95, 1)
}

pub fn mimosa() -> Unit {
    Unit::new("Mimosa", 4,
        vec![UnitTrait::BoHuc, UnitTrait::MayNoi, UnitTrait::FapSu],
        CombatRole::MagicBruiser,
        1050, 80, 38, 45, 0.65, 160, 0.05, 30, 95, 1)
}

// ════════════════════════════════════════════════════════════════════════════
// 5 VÀNG (6 tướng): Độ Mixi, PewPew, Xemesis, Bomman, SOFM, Levi
// ════════════════════════════════════════════════════════════════════════════

pub fn do_mixi() -> Unit {
    // Carry chính của tộc Lừa đảo — xạ thủ vật lý
    Unit::new("Độ Mixi", 5,
        vec![UnitTrait::LuaDao, UnitTrait::DaStream],
        CombatRole::Marksman,
        1150, 160, 35, 35, 1.05, 100, 0.20, 0, 100, 4)
}

pub fn pewpew() -> Unit {
    Unit::new("PewPew", 5,
        vec![UnitTrait::ChienSi, UnitTrait::TayTo],
        CombatRole::VanguardAD,
        1300, 140, 60, 50, 0.70, 100, 0.10, 0, 100, 1)
}

pub fn xemesis() -> Unit {
    Unit::new("Xemesis", 5,
        vec![UnitTrait::ChatThep, UnitTrait::ChienSi],
        CombatRole::Warden,
        1500, 110, 75, 70, 0.60, 100, 0.05, 0, 100, 1)
}

pub fn bomman() -> Unit {
    Unit::new("BOMMAN", 5,
        vec![UnitTrait::Bros500, UnitTrait::ChienSi],
        CombatRole::VanguardAD,
        1350, 145, 65, 55, 0.70, 100, 0.10, 0, 100, 1)
}

pub fn sofm() -> Unit {
    Unit::new("SOFM", 5,
        vec![UnitTrait::ChienSi, UnitTrait::DaStream],
        CombatRole::Assassin,
        1100, 170, 38, 38, 0.90, 100, 0.35, 0, 100, 1)
}

pub fn levi() -> Unit {
    // Mặc định AD — có thể đổi sang AP (Khó Chánh) bằng trang bị
    Unit::new("Levi", 5,
        vec![UnitTrait::ToDoi97, UnitTrait::BanNga, UnitTrait::ChienSi],
        CombatRole::Assassin,
        1100, 165, 40, 40, 0.85, 100, 0.30, 0, 100, 1)
}

// ---------------------------------------------------------------------------
// Hàm tiện ích: tạo tất cả tướng cho UnitPool
// ---------------------------------------------------------------------------

/// Trả về danh sách tất cả 42 champion templates (1 sao).
/// UnitPool sẽ nhân bản theo số lượng pool chuẩn TFT.
pub fn all_champions() -> Vec<Unit> {
    vec![
        // 1 vàng
        jikey(), boongminz(), dev(), linda_ta(), lil_chi(),
        lee_harri(), huu_nghia(), vu_mino(), leopard(),
        // 2 vàng
        meou(), funky_m(), quang_volka(), fat(), rambo(),
        thanh_chin(), tran_meo(), huy_hd(), tuyen_van_hoa(),
        but_duy_nho(), datka(), huy_hue(),
        // 3 vàng
        pino(), nhism(), qnt(), nhac_nguyen(), snake(),
        le_khoi(), ha_beso(), masew(), the_anh_96(),
        // 4 vàng
        djchip(), trang_mixi(), thay_giao_ba(), cris_devil_gamer(),
        dung_kajima(), mimosa(),
        // 5 vàng
        do_mixi(), pewpew(), xemesis(), bomman(), sofm(), levi(),
    ]
}

/// Số bản copy trong pool theo bậc giá (TFT standard)
pub fn pool_size_for_cost(cost: u8) -> usize {
    match cost {
        1 => 29,
        2 => 22,
        3 => 18,
        4 => 12,
        5 => 10,
        _ => 0,
    }
}
