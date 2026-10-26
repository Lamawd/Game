# Mixi Game — Team Notes

> Single source of truth for workflow, requirements, reminders, and known issues.
> **Update this file whenever something important changes.**

---

## 👥 Team Roles

| Person   | Responsibility        | Works in       | Branch prefix      |
|----------|-----------------------|----------------|--------------------|
| Backend  | Rust server, game logic | `server/`    | `feat/backend-*`   |
| Frontend | Unity client, UI, visuals | `client/` | `feat/unity-*`     |

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
# For verbose logs:
RUST_LOG=debug cargo run
```

### Before pushing
```bash
cargo check      # must pass with no errors
cargo clippy     # fix any warnings flagged
```

### Dependencies (pinned in Cargo.toml)
| Crate            | Purpose                        |
|------------------|--------------------------------|
| axum 0.7.5       | HTTP + WebSocket server        |
| tokio 1.38       | Async runtime                  |
| serde / serde_json | Serialization               |
| dashmap 5.5.3    | Concurrent in-memory state     |
| uuid 1.8         | Player/lobby IDs               |
| rand 0.8.5       | Shop roll RNG                  |
| tracing          | Structured logging             |

> ⚠️ Do NOT upgrade major versions without discussing with the team — axum 0.7 → 0.8 is a breaking change.

### Known TODOs (backend)
- [ ] Wire up `BuyUnit` in `ws/handler.rs` — deduct gold, pull from `UnitPool`, add to bench
- [ ] Wire up `RollShop` — deduct 2 gold, return new shop offer
- [ ] Wire up `BuyXp` — deduct 4 gold, call `player.buy_xp()`
- [ ] Wire up `SellUnit` — return unit to pool, refund gold
- [ ] Track player readiness → trigger combat when all players ready
- [ ] Broadcast `game_state` to all players in lobby after each action
- [ ] Star merge logic — auto-upgrade unit to 2★ when 3 copies owned
- [ ] Matchmaking — pair players for combat each round
- [ ] Combat: replace placeholder tick logic with hex-distance targeting

---

## 🎮 Frontend (Unity) — Setup & Requirements

### First-time project setup
1. Unity Hub → Open → select `Mixi_Game/client/`
2. **Package Manager → add by git URL:**
   ```
   https://github.com/endel/NativeWebSocket.git#upm
   ```
3. **Package Manager → install TextMeshPro** (built-in, accept when prompted)
4. Edit → Project Settings → Editor:
   - Asset Serialization Mode: **Force Text** ← required for git to work
   - Version Control Mode: **Visible Meta Files** ← required for git to work

### Scene setup (do once after cloning)
Create these GameObjects in the scene and attach the scripts:

| GameObject       | Script                  |
|------------------|-------------------------|
| `NetworkManager` | `NetworkManager.cs`     |
| `RestClient`     | `RestClient.cs`         |
| `PlayerState`    | `PlayerState.cs`        |
| `BoardManager`   | `BoardManager.cs`       |
| `ShopUI`         | `ShopUI.cs`             |

Set `serverHost` and `serverPort` on `NetworkManager` and `RestClient` to match the backend (`localhost`, `8080`).

### Git rules for Unity (CRITICAL)
- ✅ Always commit `.meta` files — every asset has one, Unity breaks without them
- ✅ Only stage `client/` — `git add client/`
- ❌ Never `git add .` unless you're sure no server files are dirty
- ❌ Never delete a `.meta` file manually
- ⚠️ Coordinate before editing the same `.unity` scene or `.prefab` — tell the team in chat first

### Known TODOs (Unity)
- [ ] Create hex tile prefab and assign to `BoardManager.hexTilePrefab`
- [ ] Create unit prefab and assign to `BoardManager.unitPrefab`
- [ ] Set up shop panel UI with 5 `ShopSlotUI` slots wired to `ShopUI.slots`
- [ ] Connect `rollButton` and `buyXpButton` in `ShopUI`
- [ ] Add `goldLabel` and `levelLabel` TMP_Text references in `ShopUI`
- [ ] Implement `UnitView.SnapBack()` — animate unit back if dropped on invalid tile
- [ ] Handle `CombatResultMessage` — show winner/damage in UI
- [ ] Handle `ErrorMessage` — show toast/popup to player

---

## 🔌 API Contract (Client ↔ Server)

> If either side changes a message shape, update `Messages.cs` (Unity) AND `ws/messages.rs` (Rust) together in the same PR.

### WebSocket endpoint
```
ws://localhost:8080/ws/{lobby_id}/{player_id}
```

### Client → Server (`type` field discriminates)
| type         | fields                          |
|--------------|---------------------------------|
| `ping`       | —                               |
| `ready`      | —                               |
| `buy_unit`   | `unit_name: str`, `cost: int`   |
| `sell_unit`  | `unit_id: uuid`                 |
| `place_unit` | `unit_id: uuid`, `position: {col, row}` |
| `move_unit`  | `from: {col, row}`, `to: {col, row}` |
| `bench_unit` | `position: {col, row}`          |
| `roll_shop`  | —                               |
| `buy_xp`     | —                               |

### Server → Client
| type            | fields                                              |
|-----------------|-----------------------------------------------------|
| `connected`     | `player_id`, `lobby_id`                             |
| `game_state`    | `round`, `phase`, `players[]`                       |
| `shop_update`   | `units[]`                                           |
| `board_update`  | `player_id`                                         |
| `combat_result` | `winner_player_id?`, `damage_dealt`, `surviving_unit_count` |
| `player_update` | `player_id`, `gold`, `health`, `level`              |
| `error`         | `message`                                           |
| `pong`          | —                                                   |

### REST endpoints
```
GET  /health
POST /api/v1/player/register   body: { username }
POST /api/v1/lobby/create      body: { host_player_id, max_players? }
GET  /api/v1/lobby/:id
POST /api/v1/lobby/:id/join    body: { player_id }
POST /api/v1/lobby/:id/leave   body: { player_id }
```

---

## 🎲 Game Rules Reference

| Rule            | Value                                          |
|-----------------|------------------------------------------------|
| Players/lobby   | 8                                              |
| Board size      | 7 cols × 4 rows (player side)                  |
| Bench size      | 9 units max                                    |
| Shop size       | 5 units                                        |
| Roll cost       | 2 gold                                         |
| XP cost         | 4 gold (+4 XP)                                 |
| Base income     | 5 gold/round                                   |
| Interest        | +1 per 10g held, max +5                        |
| Streak bonus    | +1 (2), +2 (3-4), +3 (5+) wins or losses      |
| Star merge      | 3× 1★ → 1× 2★ &nbsp;&nbsp; 3× 2★ → 1× 3★    |
| Level cap       | 9                                              |
| Starting health | 100                                            |

---

## 🐛 Known Issues & Blockers

| # | Description | Owner | Status |
|---|-------------|-------|--------|
| 1 | WebSocket handler stubs return `None` — no game state is broadcast yet | Backend | Open |
| 2 | Unity scene not created yet — scripts exist but no scene/prefabs | Unity | Open |
| 3 | No player persistence — restart wipes all state | Backend | Open (by design for now) |

> Add new issues here before opening a GitHub Issue, so we don't duplicate.

---

## 📅 Update Log

| Date       | Who      | What changed |
|------------|----------|--------------|
| 2026-10-26 | Backend  | Initial scaffold — server + client base code committed |
