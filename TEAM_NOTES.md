# Mixi Game — Team Notes

> Single source of truth for workflow, requirements, reminders, and known issues.
> **Update this file whenever something important changes.**

---

## 👥 Team Roles

| Person   | Responsibility            | Works in      | Branch prefix      |
|----------|---------------------------|---------------|--------------------|
| Backend  | Rust server, game logic   | `server/`     | `feat/backend-*`   |
| Frontend | Unity client, UI, visuals | `client/`     | `feat/unity-*`     |

---

## 🌿 Git Workflow

### Branch structure
```
main        ← stable, always runnable — NO direct commits
  └── dev   ← integration, merge all PRs here
        ├── feat/backend-*
        └── feat/unity-*
```

### Daily flow
```bash
# Start new work
git checkout dev
git pull origin dev
git checkout -b feat/backend-your-feature   # or feat/unity-...

# Save work
git add server/        # backend: only stage server/
git add client/        # unity: only stage client/
git commit -m "feat: short description"
git push -u origin feat/your-branch-name

# Open PR → dev, get review, merge, delete branch
```

### Commit message format
```
feat:  new feature
fix:   bug fix
chore: config, deps, tooling
docs:  documentation only
```

### SSH setup (required — GitHub no longer accepts passwords)
```bash
ssh-keygen -t ed25519 -C "your@email.com"   # skip if key exists
cat ~/.ssh/id_ed25519.pub                    # copy this
# GitHub → Settings → SSH keys → New SSH key → paste
git remote set-url origin git@github.com:username/Mixi_Game.git
```

---

## ⚙️ Backend (Rust) — Setup & Requirements

### Run the server
```bash
cd server
cargo run
# Listening on 0.0.0.0:8080

# Verbose logs (recommended khi test):
RUST_LOG=debug cargo run
```

### Before pushing
```bash
cargo check      # phải pass không có error
cargo clippy     # fix warnings
cargo test       # 5 unit tests phải pass
```

### Dependencies (pinned in Cargo.toml)
| Crate              | Purpose                        |
|--------------------|--------------------------------|
| axum 0.7.5         | HTTP + WebSocket server        |
| tokio 1.38         | Async runtime                  |
| serde / serde_json | Serialization                  |
| dashmap 5.5.3      | Concurrent in-memory state     |
| uuid 1.8           | Player/lobby IDs               |
| rand 0.8.5         | Shop roll RNG                  |
| tracing            | Structured logging             |

> ⚠️ Không upgrade major version mà không bàn với team — axum 0.7 → 0.8 là breaking change.

### Cấu trúc AppState (state dùng chung toàn server)
```
AppState
├── lobby_manager      — DashMap<Uuid, Lobby>
├── players            — DashMap<Uuid, Player>        (register → lưu vào đây)
├── boards             — DashMap<Uuid, Board>          (bench + placed units)
├── shop_offers        — DashMap<Uuid, Vec<String>>   (tên tướng đang hiện)
├── unit_pool          — Mutex<UnitPool>               (pool 42 tướng dùng chung)
├── streaks            — DashMap<Uuid, (win, loss)>
├── player_senders     — DashMap<Uuid, mpsc::Sender>  (push msg về client)
└── ready_players      — DashMap<Uuid, bool>
```

### Game flow (server side)
```
POST /player/register  → tạo Player, lưu vào AppState
POST /lobby/create     → tạo Lobby
WS  /ws/{lobby}/{player} → kết nối, nhận shop đầu tiên
POST /lobby/:id/start  → điền bots vào slot còn trống, spawn game loop

Game loop (tokio task):
  Round N:
    1. Grant income (base 5 + interest + streak)
    2. Roll shop mới cho tất cả
    3. Broadcast GameState(phase="planning")
    4. Bot tự planning ngay (mua + đặt tướng)
    5. Chờ 30 giây (human player hành động qua WS)
    6. Broadcast GameState(phase="combat")
    7. Ghép cặp round-robin, simulate combat
    8. Trừ máu người thua, cập nhật streak
    9. Broadcast CombatResult + PlayerUpdate
   10. Broadcast GameState(phase="results"), chờ 5 giây
   11. Lặp lại — kết thúc khi còn ≤ 1 người sống
```

### Known TODOs (backend)
- [ ] Synergy bonuses trong combat — `models/traits.rs` đã có hàm tính, cần apply vào `combat.rs`
- [ ] Hex-distance targeting — combat hiện phân damage đều, cần dùng `HexPosition` để target gần nhất
- [ ] Matchmaking thật — ghép cặp theo health/level thay vì round-robin
- [ ] Ability / kỹ năng tướng — chưa implement, chỉ đánh tay
- [ ] Persistent player data — restart wipes state (by design for now)
- [ ] Carousel rounds — PvE draft mỗi 7 vòng

---

## 🎮 Frontend (Unity) — Setup từ đầu

### Bước 1 — Mở project
1. Cài **Unity Hub** + Unity **2022.3 LTS** (hoặc mới hơn)
2. Unity Hub → **Open** → chọn thư mục `Mixi_Game/client/`

### Bước 2 — Cài packages (làm 1 lần sau khi clone)
Vào **Window → Package Manager**:

| Package          | Cách cài                                                          |
|------------------|-------------------------------------------------------------------|
| NativeWebSocket  | Add by git URL: `https://github.com/endel/NativeWebSocket.git#upm` |
| TextMeshPro      | Unity Registry → tìm "TextMeshPro" → Install → **Import TMP Essentials** khi được hỏi |

### Bước 3 — Project Settings (làm 1 lần)
**Edit → Project Settings → Editor:**
- Asset Serialization Mode: **Force Text** ← bắt buộc để git hoạt động
- Version Control Mode: **Visible Meta Files** ← bắt buộc

### Bước 4 — Tạo Scene để chạy thử

Tạo Scene mới (`File → New Scene`) hoặc dùng scene có sẵn, rồi tạo các GameObject sau:

| GameObject       | Script đính kèm         | Inspector cần set                        |
|------------------|-------------------------|------------------------------------------|
| `NetworkManager` | `NetworkManager.cs`     | serverHost=`localhost`, serverPort=`8080` |
| `RestClient`     | `RestClient.cs`         | serverHost=`localhost`, serverPort=`8080` |
| `PlayerState`    | `PlayerState.cs`        | —                                        |
| `BoardManager`   | `BoardManager.cs`       | hexTilePrefab, unitPrefab (xem bước 5)  |
| `ShopUI`         | `ShopUI.cs`             | rollButton, buyXpButton, goldLabel, levelLabel, slots[5] |

### Bước 5 — Tạo Prefabs tối thiểu để chạy thử

**HexTile prefab:**
1. Tạo `GameObject → 3D Object → Cylinder` (hoặc Hexagon mesh nếu có)
2. Scale về `(0.9, 0.1, 0.9)`
3. Thêm component `HexTile.cs`
4. Kéo vào `Assets/Prefabs/` → đặt tên `HexTile`
5. Gán vào `BoardManager.hexTilePrefab`

**Unit prefab:**
1. Tạo `GameObject → 3D Object → Capsule`
2. Thêm component `UnitView.cs`
3. Thêm `TextMeshPro - Text (3D)` con để hiện tên tướng
4. Kéo vào `Assets/Prefabs/` → đặt tên `Unit`
5. Gán vào `BoardManager.unitPrefab`

### Bước 6 — Chạy thử (test với bots)

Tạo một script `GameBootstrap.cs` tạm để tự động hóa flow test:

```csharp
// Gắn vào bất kỳ GameObject nào trong scene
// Khi nhấn Play, tự động: register → create lobby → connect WS → start với bots
using UnityEngine;

public class GameBootstrap : MonoBehaviour
{
    [SerializeField] private string testUsername = "TestPlayer";

    private void Start()
    {
        RestClient.Instance.RegisterPlayer(testUsername, OnRegistered, Debug.LogError);
    }

    private void OnRegistered(PlayerResponse resp)
    {
        Debug.Log($"[Boot] Player: {resp.player.id}");
        PlayerState.Instance.Init(resp.player.id, resp.player.username);

        RestClient.Instance.CreateLobby(resp.player.id, 8, OnLobbyCreated, Debug.LogError);
    }

    private void OnLobbyCreated(LobbyResponse resp)
    {
        Debug.Log($"[Boot] Lobby: {resp.lobby.id}");

        // Kết nối WebSocket
        NetworkManager.Instance.Connect(resp.lobby.id, PlayerState.Instance.PlayerId);
        NetworkManager.Instance.OnConnected += () =>
        {
            // Sau khi WS connected, bắt đầu game với bots
            RestClient.Instance.StartLobby(resp.lobby.id, OnStarted, Debug.LogError);
        };
    }

    private void OnStarted(StartLobbyResponse resp)
    {
        Debug.Log($"[Boot] Game started! {resp.human_count} human + {resp.bot_count} bots");
    }
}
```

### Bước 7 — Xử lý messages từ server

Trong `NetworkManager`, subscribe `OnMessageReceived` từ một script trung tâm:

```csharp
NetworkManager.Instance.OnMessageReceived += OnServerMessage;

void OnServerMessage(ServerMessage msg)
{
    switch (msg)
    {
        case GameStateMessage gs:
            Debug.Log($"Round {gs.round} — {gs.phase}");
            // TODO: cập nhật UI phase
            break;
        case ShopUpdateMessage su:
            ShopUI.Instance.Refresh(su.units);
            break;
        case BoardStateMessage bs:
            BoardManager.Instance.ApplyBoardState(bs);
            break;
        case PlayerUpdateMessage pu when pu.player_id == PlayerState.Instance.PlayerId:
            PlayerState.Instance.UpdateStats(pu.gold, pu.health, pu.level);
            break;
        case CombatResultMessage cr:
            Debug.Log($"Combat: winner={cr.winner_player_id}, dmg={cr.damage_dealt}");
            break;
        case ErrorMessage err:
            Debug.LogWarning($"[Server Error] {err.message}");
            break;
    }
}
```

### Git rules cho Unity (CRITICAL)
- ✅ Luôn commit file `.meta` — mỗi asset đều có một file meta, Unity vỡ nếu thiếu
- ✅ Chỉ stage `client/` — `git add client/`
- ❌ Không `git add .` trừ khi chắc chắn không có file server bẩn
- ❌ Không xóa file `.meta` thủ công
- ⚠️ Báo team trước khi sửa cùng một `.unity` scene hoặc `.prefab`

### Known TODOs (Unity)
- [ ] Tạo HexTile prefab và gán vào `BoardManager.hexTilePrefab`
- [ ] Tạo Unit prefab và gán vào `BoardManager.unitPrefab`
- [ ] Set up shop panel UI với 5 `ShopSlotUI` slots
- [ ] Kết nối `rollButton`, `buyXpButton` trong `ShopUI`
- [ ] Thêm `goldLabel`, `levelLabel` TMP_Text vào `ShopUI`
- [ ] Implement `UnitView.SnapBack()` — animate về chỗ cũ nếu thả vào ô không hợp lệ
- [ ] Handle `BoardStateMessage` trong `BoardManager.ApplyBoardState()`
- [ ] Handle `CombatResultMessage` — hiện winner/damage trên UI
- [ ] Handle `ErrorMessage` — toast/popup

---

## 🔌 API Contract (Client ↔ Server)

> Nếu một bên thay đổi shape message, update **cả hai** `Messages.cs` (Unity) **và** `ws/messages.rs` (Rust) trong cùng một PR.

### WebSocket endpoint
```
ws://localhost:8080/ws/{lobby_id}/{player_id}
```

### Client → Server
| type         | fields                                   |
|--------------|------------------------------------------|
| `ping`       | —                                        |
| `ready`      | —                                        |
| `buy_unit`   | `unit_name: str`, `cost: int`            |
| `sell_unit`  | `unit_id: uuid`                          |
| `place_unit` | `unit_id: uuid`, `position: {col, row}`  |
| `move_unit`  | `from: {col, row}`, `to: {col, row}`     |
| `bench_unit` | `position: {col, row}`                   |
| `roll_shop`  | —                                        |
| `buy_xp`     | —                                        |

### Server → Client
| type            | fields                                                              |
|-----------------|---------------------------------------------------------------------|
| `connected`     | `player_id`, `lobby_id`                                             |
| `game_state`    | `round`, `phase` (`planning`/`combat`/`results`), `players[]`       |
| `shop_update`   | `units[]` — array UnitDto                                           |
| `board_state`   | `player_id`, `placed[]` (BoardCell array), `bench[]`, `placed_count`|
| `combat_result` | `winner_player_id?`, `damage_dealt`, `surviving_unit_count`         |
| `player_update` | `player_id`, `gold`, `health`, `level`                              |
| `error`         | `message`                                                           |
| `pong`          | —                                                                   |

### REST endpoints
```
GET  /health
POST /api/v1/player/register        body: { username }
POST /api/v1/lobby/create           body: { host_player_id, max_players? }
GET  /api/v1/lobby/:id
POST /api/v1/lobby/:id/join         body: { player_id }
POST /api/v1/lobby/:id/leave        body: { player_id }
POST /api/v1/lobby/:id/start        body: {} — điền bots + bắt đầu game loop
```

---

## 🎲 Game Rules Reference

| Rule            | Value                                             |
|-----------------|---------------------------------------------------|
| Players/lobby   | 8 (có thể ít hơn, slot còn lại tự động điền bot) |
| Board size      | 7 cols × 4 rows (phía player)                    |
| Bench size      | 9 units max                                       |
| Shop size       | 5 units                                           |
| Roll cost       | 2 gold                                            |
| XP cost         | 4 gold (+4 XP)                                    |
| Base income     | 5 gold/round (từ vòng 2 trở đi)                  |
| Interest        | +1 per 10g held, max +5                           |
| Streak bonus    | +1 (2 streak), +2 (3–4), +3 (5+)                 |
| Star merge      | 3× 1★ → 1× 2★   /   3× 2★ → 1× 3★               |
| Tier scaling    | 2★ = 1.8×   /   3★ = 3.24× (HP + AD + AP)        |
| Sell refund     | 1★ = cost×1   /   2★ = cost×3   /   3★ = cost×9  |
| Level cap       | 9                                                 |
| Starting health | 100                                               |
| Planning time   | 30 giây                                           |
| Results time    | 5 giây                                            |

---

## 🧪 Testing & Balancing

### Chạy test nhanh bằng curl (không cần Unity)

```bash
# 1. Kiểm tra server sống
curl http://localhost:8080/health

# 2. Đăng ký player
curl -X POST http://localhost:8080/api/v1/player/register \
  -H "Content-Type: application/json" \
  -d '{"username": "TestPlayer"}'
# → lưu lại "id" trong response

# 3. Tạo lobby
curl -X POST http://localhost:8080/api/v1/lobby/create \
  -H "Content-Type: application/json" \
  -d '{"host_player_id": "<PLAYER_ID>", "max_players": 2}'
# → lưu lại "id" của lobby

# 4. Bắt đầu với bots (lobby 2 người → 1 bot)
curl -X POST http://localhost:8080/api/v1/lobby/<LOBBY_ID>/start \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Dùng wscat để test WebSocket thủ công

```bash
# Cài wscat
npm install -g wscat

# Kết nối
wscat -c ws://localhost:8080/ws/<LOBBY_ID>/<PLAYER_ID>

# Gửi messages (paste vào terminal sau khi connect):
{"type":"ping"}
{"type":"roll_shop"}
{"type":"buy_unit","unit_name":"Jikey","cost":1}
{"type":"place_unit","unit_id":"<UNIT_UUID>","position":{"col":0,"row":0}}
{"type":"ready"}
```

### Những gì cần log/quan sát để phát hiện lỗi balance

**Server log cần bật:**
```bash
RUST_LOG=debug cargo run
```
Log sẽ hiện:
- Mỗi action của player và bot
- Star merge events
- Kết quả combat mỗi vòng (winner, damage, ticks)
- Income mỗi vòng

**Dấu hiệu tướng quá mạnh / quá yếu:**
| Triệu chứng | Nguyên nhân có thể |
|-------------|-------------------|
| Combat kết thúc sau 1–2 tick | AD quá cao so với HP |
| Combat đạt MAX_TICKS (200) không ai thắng | HP quá cao hoặc AD quá thấp |
| Bot luôn thắng human từ vòng 1 | Bot mua tướng 1 vàng nhiều hơn, cần check income |
| Một tướng cụ thể thắng 100% | Base stats quá cao — xem `models/unit.rs` |
| Damage dealt = 1 mọi vòng | `calculate_damage` trả về 1 (survivors rỗng) |

**Cách đọc combat log:**
```
[Loop] Lobby X Round 3 — Combat
→ tìm CombatResult log: winner, damage_dealt, rounds_taken
→ nếu rounds_taken = 200 → combat timeout, cần giảm HP hoặc tăng AD
→ nếu damage_dealt quá cao (> 20) → tướng 3★ cost cao còn sống nhiều
```

### Checklist balance sau mỗi lần test

```
[ ] Combat có kết thúc trước 200 ticks không?
[ ] Trận trung bình kéo dài bao nhiêu ticks? (target: 30–80)
[ ] Player mất bao nhiêu máu trung bình mỗi vòng? (target: 5–15)
[ ] Bot có hoạt động không (mua tướng, đặt lên board)?
[ ] Shop có ra đúng tướng theo level không? (level 1-2 chỉ ra 1-cost)
[ ] Star merge có xảy ra tự động không?
[ ] Streak bonus có tính đúng không? (2+ wins liên tiếp → bonus vàng)
```

### File cần sửa khi điều chỉnh balance

| Muốn thay đổi            | File                         | Thứ cần sửa                          |
|--------------------------|------------------------------|--------------------------------------|
| Chỉ số tướng (HP/AD/...) | `server/src/models/unit.rs`  | Hàm từng tướng: `jikey()`, `levi()` v.v. |
| Tỉ lệ roll theo level    | `server/src/game/shop.rs`    | Hàm `tier_weights()`                 |
| Income / lãi suất        | `server/src/game/round.rs`   | Hàm `calculate_income()`             |
| Damage trong combat      | `server/src/game/combat.rs`  | Hàm `tick_combat()`, `calculate_damage()` |
| Thời gian Planning       | `server/src/game/game_loop.rs` | `PLANNING_SECS`                    |
| Scaling 2★ / 3★          | `server/src/models/unit.rs`  | Hàm `tier_mult()`                    |
| Synergy bonuses          | `server/src/models/traits.rs`| Các hàm `*_bonus()`                  |

---

## 🐛 Known Issues & Blockers

| # | Description | Owner | Status |
|---|-------------|-------|--------|
| 1 | Unity scene chưa được tạo — scripts có nhưng chưa có scene/prefab | Unity | Open |
| 2 | Combat dùng damage đều cho tất cả target — chưa có hex-distance targeting | Backend | Open |
| 3 | Synergy bonuses đã có hàm tính nhưng chưa apply vào combat simulator | Backend | Open |
| 4 | Ability tướng chưa implement — chỉ đánh tay | Backend | Open |
| 5 | No player persistence — restart wipes all state | Backend | Open (by design) |

> Thêm issue mới vào đây trước khi mở GitHub Issue.

---

## 📅 Update Log

| Date       | Who      | What changed |
|------------|----------|--------------|
| 2026-10-26 | Backend  | Initial scaffold — server + client base code committed |
| 2026-10-27 | Backend  | Data layer hoàn chỉnh: 42 tướng thật, 8 tộc, 5 hệ, synergy bonuses, unit pool chuẩn TFT |
| 2026-10-27 | Backend  | Game flow hoàn chỉnh: register→lobby→WS→start wires hết; BuyUnit/Sell/Place/Move/Roll/BuyXp/StarMerge đều hoạt động; mpsc push channel; game loop với bot AI và matchmaking round-robin |
| 2026-10-27 | Backend  | Thêm POST /lobby/:id/start — tự điền bots vào slot còn trống, spawn game loop |
| 2026-10-27 | Unity    | Messages.cs: thêm BoardStateMessage, BoardCell, StartLobbyResponse; ServerMessageParser: thêm board_state; RestClient: thêm StartLobby() |
