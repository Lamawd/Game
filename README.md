# Mixi Game

A tactical auto-battler (TFT-style) built with **Unity (C#)** as the client and **Rust (Axum)** as the backend.

---

## Project Structure

```
Mixi_Game/
├── client/                          # Unity project
│   └── Assets/Scripts/
│       ├── Network/
│       │   ├── NetworkManager.cs    # WebSocket connection singleton
│       │   ├── RestClient.cs        # HTTP REST calls (lobby, player)
│       │   ├── Messages.cs          # All client↔server message types
│       │   └── ServerMessageParser.cs
│       ├── Game/
│       │   ├── BoardManager.cs      # Hex grid + drag-and-drop placement
│       │   ├── HexTile.cs           # Single hex tile behaviour
│       │   ├── UnitView.cs          # Unit visual + drag logic
│       │   └── PlayerState.cs       # Local player gold/health/level
│       └── UI/
│           ├── ShopUI.cs            # Shop panel (roll, buy XP)
│           └── ShopSlotUI.cs        # Individual shop unit slot
│
└── server/                          # Rust backend
    └── src/
        ├── main.rs                  # Entry point, router, AppState
        ├── api/                     # HTTP REST handlers
        │   ├── health.rs
        │   ├── lobby.rs
        │   └── player.rs
        ├── ws/                      # WebSocket handlers + message types
        │   ├── handler.rs
        │   └── messages.rs
        ├── lobby/mod.rs             # In-memory lobby manager (DashMap)
        ├── game/
        │   ├── combat.rs            # Auto-battle simulation
        │   ├── shop.rs              # Unit pool + shop rolls
        │   └── round.rs             # Round pacing, gold income
        └── models/
            ├── player.rs            # Player state + XP/level logic
            ├── lobby.rs             # Lobby model
            ├── unit.rs              # Unit/champion with star tiers
            └── board.rs             # Hex board + bench placement
```

---

## Tech Stack

| Layer       | Technology          | Purpose                              |
|-------------|---------------------|--------------------------------------|
| Frontend    | Unity 2022+ / C#    | Rendering, UI, input, drag-and-drop  |
| Transport   | WebSocket (axum ws) | Real-time game state sync            |
| REST API    | axum HTTP           | Lobby creation, player registration  |
| Backend     | Rust / Tokio        | Game logic, combat sim, shop rolls   |
| Concurrency | DashMap + Arc       | Shared state across async tasks      |

---

## Communication Protocol

### REST (HTTP)
```
POST /api/v1/player/register   → Create a player session
POST /api/v1/lobby/create      → Create a lobby
GET  /api/v1/lobby/:id         → Get lobby info
POST /api/v1/lobby/:id/join    → Join a lobby
POST /api/v1/lobby/:id/leave   → Leave a lobby
GET  /health                   → Health check
```

### WebSocket
Connect: `ws://host:8080/ws/{lobby_id}/{player_id}`

**Client → Server messages** (`type` field discriminates):
| type         | payload                        |
|--------------|-------------------------------|
| `ping`       | —                             |
| `ready`      | —                             |
| `buy_unit`   | `unit_name`, `cost`           |
| `sell_unit`  | `unit_id`                     |
| `place_unit` | `unit_id`, `position`         |
| `move_unit`  | `from`, `to`                  |
| `bench_unit` | `position`                    |
| `roll_shop`  | —                             |
| `buy_xp`     | —                             |

**Server → Client messages**:
| type            | payload                                        |
|-----------------|------------------------------------------------|
| `connected`     | `player_id`, `lobby_id`                        |
| `game_state`    | `round`, `phase`, `players[]`                  |
| `shop_update`   | `units[]`                                      |
| `combat_result` | `winner_player_id`, `damage_dealt`             |
| `player_update` | `player_id`, `gold`, `health`, `level`         |
| `error`         | `message`                                      |
| `pong`          | —                                              |

---

## Running the Server

```bash
cd server
cargo run
# Server starts on 0.0.0.0:8080
# Set RUST_LOG=debug for verbose logs
```

## Setting Up the Unity Client

1. Open Unity Hub → **Open project** → select `client/`
2. Install via Package Manager:
   - [NativeWebSocket](https://github.com/endel/NativeWebSocket) (add via git URL)
   - TextMeshPro (built-in, install via Unity Package Manager)
3. In the scene, create a `NetworkManager` GameObject and attach `NetworkManager.cs`
4. Create a `RestClient` GameObject and attach `RestClient.cs`
5. Create a `PlayerState` GameObject and attach `PlayerState.cs`
6. Set `serverHost` and `serverPort` on the components to match your server

---

## Game Design (TFT-like)

- **8 players** per lobby
- **Hex board** — 7 columns × 4 rows (your side), mirrored for the opponent
- **5-unit shop** — rolls from a shared pool; cost tiers 1–5
- **Star system** — buy 3 copies → auto-merge to 2★; 3× 2★ → 3★
- **Economy** — 5 base gold/round + interest (1 per 10g, max 5) + streak bonus
- **Combat** — server simulates auto-battle; sends result back to all clients
- **Level cap** — 9, set by XP; level gates how many units you can field

---

## Roadmap

- [ ] Full combat targeting (hex distance, aggro, abilities)
- [ ] Trait/synergy system (active bonuses for matching traits)
- [ ] Carousel rounds (shared draft)
- [ ] Persistent player data (PostgreSQL / SQLite)
- [ ] Matchmaking (pair players for combat each round)
- [ ] Spectator mode
- [ ] Unit animations and ability VFX
