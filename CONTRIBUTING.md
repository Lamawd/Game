# Team Git Workflow

## Branch Structure

```
main          ← stable, always runnable
  └── dev     ← integration branch, merge PRs here
        ├── feat/backend-combat    ← your Rust feature branches
        ├── feat/backend-shop
        ├── feat/unity-board-ui    ← teammate's Unity feature branches
        └── feat/unity-shop-ui
```

**Rule: never commit directly to `main`. Always branch from `dev`.**

---

## Your Roles

| Person        | Works in         | Branch prefix         |
|---------------|------------------|-----------------------|
| You (backend) | `server/`        | `feat/backend-*`      |
| Teammate      | `client/`        | `feat/unity-*`        |

Since your work is in separate folders, merge conflicts will be rare.

---

## Day-to-Day Workflow

### Starting a new feature
```bash
git checkout dev
git pull origin dev
git checkout -b feat/backend-your-feature
```

### Saving work
```bash
git add server/               # backend dev: only stage server/
git commit -m "feat: describe what you did"
git push -u origin feat/backend-your-feature
```

### Teammate (Unity)
```bash
git add client/               # only stage client/
git commit -m "feat: describe what you did"
```

### Merging back
1. Open a PR: `feat/...` → `dev`
2. The other person reviews
3. Merge when `cargo check` passes (backend) or tested in Play Mode (Unity)
4. Delete the feature branch after merge

---

## Unity-Specific Rules (Important)

1. **Always commit `.meta` files** — Unity generates a `.meta` for every asset.  
   If you commit an asset without its `.meta`, your teammate's project will break.

2. **Use Unity's YAML Smart Merge** for `.unity` / `.prefab` conflicts.  
   Set it up once:
   ```
   # In your global ~/.gitconfig:
   [merge "unityyamlmerge"]
       name = Unity SmartMerge
       driver = 'C:/Program Files/Unity/Hub/Editor/<version>/Editor/Data/Tools/UnityYAMLMerge.exe' merge -p %O %B %A %P
   ```
   Path varies by OS and Unity version — check your Unity Hub install.

3. **Avoid editing the same `.unity` scene or `.prefab` at the same time.**  
   Coordinate in chat: "I'm editing MainScene" before you open it.

4. **Set Unity → Edit → Project Settings → Editor:**
   - Asset Serialization Mode: **Force Text**
   - Version Control Mode: **Visible Meta Files**

---

## Commit Message Format

```
feat: add combat simulation loop
fix: correct gold income calculation
chore: update Cargo dependencies
docs: update README setup steps
```

---

## First-Time Setup (clone + run)

### Backend
```bash
git clone <repo-url>
cd Mixi_Game/server
cargo run
```

### Unity
1. Clone the repo
2. Unity Hub → Open → select `Mixi_Game/client/`
3. Package Manager → add by git URL:
   - `https://github.com/endel/NativeWebSocket.git#upm`
4. Install TextMeshPro from Package Manager
5. Press Play — the scene should load without errors
