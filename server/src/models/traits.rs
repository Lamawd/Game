use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Trait enum — gộp cả Tộc (Origins) và Hệ (Classes)
// ---------------------------------------------------------------------------

/// Mỗi tướng mang 1-2 trait. Dùng cho synergy tracking và combat bonuses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UnitTrait {
    // ── Tộc (Origins) ──────────────────────────────────────────────────────
    /// Tổ đội 97: Nhism, Levi, MeoU
    /// (2): Hồi 3 NL/giây  (3): Hồi 5 NL/giây + 15% hút máu
    ToDoi97,

    /// Bò "Húc": Fat, FunkyM, Rambo, Mimosa, Boongminz
    /// (2/4/6): Hất tung + hồi máu khi dưới 50%
    BoHuc,

    /// Tổ đội lừa đảo: Độ Mixi, Lê Khôi, QNT, Nhạc
    /// (2/3/4/6): Buff sát thương toàn đội + rớt vàng
    LuaDao,

    /// Nổ xịt: Jikey, MeoU, Pino
    /// (2/4): Bom nổ khi chết (hoặc khói làm mù)
    NoXit,

    /// Máy nói: Dev, Lê Khôi, Mimosa
    /// (2/3): Giảm Giáp/KP địch trong phạm vi 2 ô
    MayNoi,

    /// Bản ngã: Pino, Snake, Levi, FunkyM
    /// (2/4): Lá chắn đầu trận + tăng vĩnh viễn sát thương mỗi mạng
    BanNga,

    /// Nóc nhà: Trang Mixi, Thanh Chin, Hà Beso, Linda Tạ, Trân Mèo
    /// (2/4/6): Cấm đánh + giảm Giáp/KP địch đầu trận
    NocNha,

    /// 500 Bros: BOMMAN, Dũng Kajima, Hữu Nghĩa, Leopard, Thế Anh 96
    /// (2/4/6): Tốc độ đánh + hồi năng lượng
    Bros500,

    // ── Hệ (Classes) ───────────────────────────────────────────────────────
    /// Fap sư: tăng SMPT toàn đội; (6): chiêu có thể chí mạng
    FapSu,

    /// Chiến sĩ: tăng Giáp; (6): phản lại 15% SMVL gánh chịu
    ChienSi,

    /// Chất thép: tăng Máu tối đa + hồi máu thụ động
    ChatThep,

    /// Tay to: tăng SMCK; (6): đòn thứ 5 gây thêm sát thương chuẩn
    TayTo,

    /// Đá stream: tăng Tốc độ đánh; (6): đòn đánh không thể né tránh
    DaStream,
}

impl UnitTrait {
    /// Tên hiển thị tiếng Việt (cho client)
    pub fn display_name(&self) -> &'static str {
        match self {
            UnitTrait::ToDoi97   => "Tổ đội 97",
            UnitTrait::BoHuc     => "Bò Húc",
            UnitTrait::LuaDao    => "Tổ đội lừa đảo",
            UnitTrait::NoXit     => "Nổ xịt",
            UnitTrait::MayNoi    => "Máy nói",
            UnitTrait::BanNga    => "Bản ngã",
            UnitTrait::NocNha    => "Nóc nhà",
            UnitTrait::Bros500   => "500 Bros",
            UnitTrait::FapSu     => "Fap sư",
            UnitTrait::ChienSi   => "Chiến sĩ",
            UnitTrait::ChatThep  => "Chất thép",
            UnitTrait::TayTo     => "Tay to",
            UnitTrait::DaStream  => "Đá stream",
        }
    }

    /// Các ngưỡng kích hoạt synergy (breakpoints)
    pub fn breakpoints(&self) -> &'static [u8] {
        match self {
            UnitTrait::ToDoi97   => &[2, 3],
            UnitTrait::BoHuc     => &[2, 4, 6],
            UnitTrait::LuaDao    => &[2, 3, 4, 6],
            UnitTrait::NoXit     => &[2, 4],
            UnitTrait::MayNoi    => &[2, 3],
            UnitTrait::BanNga    => &[2, 4],
            UnitTrait::NocNha    => &[2, 4, 6],
            UnitTrait::Bros500   => &[2, 4, 6],
            UnitTrait::FapSu     => &[2, 4, 6],
            UnitTrait::ChienSi   => &[2, 4, 6],
            UnitTrait::ChatThep  => &[2, 4, 6],
            UnitTrait::TayTo     => &[2, 4, 6],
            UnitTrait::DaStream  => &[2, 4, 6],
        }
    }

    /// Trả về breakpoint đang active (cao nhất đạt được)
    pub fn active_breakpoint(&self, count: u8) -> Option<u8> {
        self.breakpoints().iter().rev().find(|&&bp| count >= bp).copied()
    }
}

// ---------------------------------------------------------------------------
// Synergy bonuses — giá trị buff cụ thể để combat simulator tra cứu
// ---------------------------------------------------------------------------

/// Tính bonus Giáp từ hệ Chiến sĩ
pub fn warrior_armor_bonus(bp: u8) -> u32 {
    match bp { 2 => 20, 4 => 45, 6 => 80, _ => 0 }
}

/// Tính bonus Máu tối đa từ hệ Chất thép
pub fn ironbody_hp_bonus(bp: u8) -> u32 {
    match bp { 2 => 100, 4 => 250, 6 => 500, _ => 0 }
}

/// Hồi máu thụ động mỗi giây từ hệ Chất thép (phần trăm máu tối đa)
pub fn ironbody_regen_pct(bp: u8) -> f32 {
    match bp { 4 => 0.01, 6 => 0.025, _ => 0.0 }
}

/// Bonus SMCK từ hệ Tay to (nhân số)
pub fn bighand_dmg_bonus(bp: u8) -> f32 {
    match bp { 2 => 0.10, 4 => 0.15, 6 => 0.30, _ => 0.0 }
}

/// Bonus Tốc độ đánh từ hệ Đá stream (nhân số)
pub fn dastream_aspd_bonus(bp: u8) -> f32 {
    match bp { 2 => 0.05, 4 => 0.15, 6 => 0.25, _ => 0.0 }
}

/// Bonus SMPT toàn đội từ hệ Fap sư
pub fn fapsu_ap_bonus(bp: u8) -> u32 {
    match bp { 2 => 10, 4 => 25, 6 => 45, _ => 0 }
}

/// Năng lượng hồi mỗi giây từ tộc Tổ đội 97
pub fn todoi97_energy_regen(bp: u8) -> u32 {
    match bp { 2 => 3, 3 => 5, _ => 0 }
}

/// Bonus Tốc độ đánh từ tộc 500 Bros (nhân số)
pub fn bros500_aspd_bonus(bp: u8) -> f32 {
    match bp { 2 => 0.04, 4 => 0.08, 6 => 0.12, _ => 0.0 }
}

/// Năng lượng hồi mỗi giây từ tộc 500 Bros
pub fn bros500_energy_regen(bp: u8) -> u32 {
    match bp { 2 => 2, 4 => 4, 6 => 8, _ => 0 }
}

/// Bonus sát thương % toàn đội từ tộc Lừa đảo
pub fn luadao_dmg_pct(bp: u8) -> f32 {
    match bp { 2 => 0.05, 3 => 0.10, 4 => 0.15, 6 => 0.40, _ => 0.0 }
}

/// Tỉ lệ rớt vàng từ tộc Lừa đảo
pub fn luadao_gold_drop_chance(bp: u8) -> f32 {
    match bp { 2 => 0.10, 3 => 0.15, 4 => 0.25, 6 => 0.40, _ => 0.0 }
}

// ---------------------------------------------------------------------------
// Combat role — loại vai trò chiến đấu (dùng nội bộ cho targeting logic)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CombatRole {
    /// Xạ thủ — xa, vật lý, tốc đánh cao
    Marksman,
    /// Pháp sư — xa, phép, tung chiêu diện rộng
    Mage,
    /// Xạ thủ phép / Hỗ trợ — xa, hỗn hợp
    MagicMarksman,
    /// Đấu sĩ vật lý (Bruiser) — gần, máu + vật lý
    Bruiser,
    /// Đấu sĩ phép — gần, phép + máu (hút máu)
    MagicBruiser,
    /// Siêu Tanker (Warden) — gần, Giáp/KP rất cao, phản sát thương
    Warden,
    /// Tiên phong phép — gần, tank + nổ phép diện rộng
    VanguardMage,
    /// Tiên phong vật lý — gần, tank + phá giáp
    VanguardAD,
    /// Sát thủ vật lý — gần, crit, burst nhanh
    Assassin,
    /// Sát thủ phép — gần, lao vào dồn phép cực mạnh rồi lùi
    MagicAssassin,
}
