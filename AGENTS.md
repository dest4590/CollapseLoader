# AGENTS.md – CollapseLoader

Tauri v2 desktop app: Vue 3 + TypeScript frontend, Rust backend. Minecraft cheat-client launcher for Windows, Linux, macOS.

---

## Developer commands

```sh
npm install          # install JS deps (use npm ci in CI)
npm run tauri dev    # dev mode – starts Vite (port 1420) AND Rust backend together
npm run tauri build  # production build (runs vue-tsc + vite build + cargo release)

npm run check        # vue-tsc typecheck only (no emit)
npm run lint         # eslint on src/ (.ts, .tsx, .vue)
npm run lint:fix     # eslint --fix
npm run format       # prettier --write "src/**/*"

# Run Rust unit tests (from src-tauri/)
cargo test
```

`tauri dev` / `tauri build` must be run from the **repo root** (not `src-tauri/`). Tauri's `beforeDevCommand` automatically runs `npm run dev` (Vite), so don't start Vite separately.

---

## Architecture

### Two-part project

| Layer    | Location         | Language           |
| -------- | ---------------- | ------------------ |
| Frontend | `src/`           | Vue 3 + TypeScript |
| Backend  | `src-tauri/src/` | Rust (Tauri v2)    |

Communication is exclusively via Tauri `invoke()` commands – there is no HTTP API between the two halves.

### Frontend layout (`src/`)

- `main.ts` → `bootstrap/applicationBootstrap.ts` (mounts correct Vue component based on `?window=` URL param)
- Multiple window types: `main` (default), `network`, `customization` – each is a different root component
- `features/` – domain folders: `auth/`, `clients/`, `download/`, `friends/`, `marketplace/`, `presets/`, `social/`
- `api/` – Tauri invoke wrappers
- `services/` – shared services (i18n, etc.)
- `shared/` – reusable components/composables
- `config.ts` – API URL initialisation; must call `initializeApiUrl()` before using `getApiUrl()`

### Path aliases (configured in both `vite.config.ts` and `tsconfig.json`)

```
@          → src/
@shared    → src/shared/
@features  → src/features/
@api       → src/api/
@services  → src/services/
@layouts   → src/layouts/
@router    → src/router/index.ts  (bare alias)
@stores    → src/stores/
```

### Backend layout (`src-tauri/src/`)

- `lib.rs` – app entry, Tauri builder, all `invoke_handler` registrations
- `commands/` – Tauri command handlers (clients, irc, presets, settings, updater, utils, network, report)
- `core/` – business logic
    - `clients/` – client manager
    - `network/` – downloader, servers, analytics, API
    - `platform/` – OS-specific code (Windows message boxes, DPI)
    - `storage/` – persistent state (settings, accounts, favorites, presets, custom clients, flags)
    - `utils/` – globals, logging macros, helpers, process, hashing, archive, Discord RPC

**Adding a new Tauri command**: implement in `commands/<module>.rs`, then register it in the `invoke_handler!` macro in `lib.rs`.

### Data directory (runtime)

- Windows: `%APPDATA%\CollapseLoader\`
- Linux: `$XDG_DATA_HOME/CollapseLoader/` (falls back to `~/CollapseLoader/` for legacy)
- macOS: `~/Library/Application Support/CollapseLoader/`
- Can be overridden via a `CollapseLoaderRoot.txt` file in the base dir.

---

## Environment / `.env`

Copy `.env.example` to `.env` before working locally. Key vars read by the Rust backend at startup:

| Variable                               | Effect                                                                  |
| -------------------------------------- | ----------------------------------------------------------------------- |
| `DEVELOPMENT=true`                     | Enables dev-mode features                                               |
| `MOCK_CLIENTS=true`                    | Use fake client list (offline dev)                                      |
| `FORCE_CDN=<url>`                      | Override CDN server URL                                                 |
| `FORCE_API=<url>`                      | Override API server URL                                                 |
| `LOCAL_UPDATER_URL=true`               | Point updater at `localhost:8000` (use `scripts/serve_mock_release.py`) |
| `SKIP_AGENT_OVERLAY_VERIFICATION=true` | Skip overlay file checks                                                |

Env vars are parsed at Rust startup via `parse_env_bool()` in `core/utils/globals.rs`.

---

## Toolchain quirks

- **TypeScript is strict**: `strict`, `noUnusedLocals`, `noUnusedParameters` are all enabled. Unused vars cause build failures unless prefixed with `_`.
- **ESLint targets `.ts`, `.tsx`, `.vue`** – `.js` files in `src/` are also linted.
- **Prettier**: 4-space indent, trailing commas (`es5`). No `tailwind.config.js` – Tailwind v4 config lives in CSS via `@import "tailwindcss"` and `@plugin "daisyui"`.
- **daisyUI v5** is the component library (see `.github/instructions/daisyui.instructions.md` for full class reference). Use daisyUI semantic color names (`bg-primary`, `text-base-content`, etc.) instead of Tailwind hardcoded colors so themes work.
- **Monaco editor** loads from CDN (`cdn.jsdelivr.net/npm/monaco-editor@0.55/min/vs`) – not bundled locally.
- **`daisyui` is excluded from Vite `optimizeDeps`** – do not add it back.
- **Rust `profile.dev`**: incremental builds, `opt-level=0`, `debug=1`. Dependencies are compiled at `opt-level=2` for reasonable dev performance.

---

## CI / Build

- CI triggers on push to `main` or `dev` branches.
- Commit messages containing `[skip ci]`, `skip-ci`, or `skip_ci` skip the build job.
- Commit messages containing `release` or `build macos` trigger the macOS universal binary build (otherwise macOS is skipped).
- Artifacts: `.msi`, NSIS `.exe`, portable `.exe` (Windows); `.AppImage`, `.deb`, `.rpm` (Linux); `.dmg` (macOS, conditional).
- CI uses `npm ci` (not `npm install`) – keep `package-lock.json` committed.

---

## Versioning

Version appears in **three files** simultaneously – use the script to keep them in sync:

```sh
python scripts/bump_version.py <new-version> [--codename NEW_CODENAME]
```

Files updated: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `CODENAME` in `src-tauri/src/core/utils/globals.rs`.

---

## Scripts

| Script                             | Purpose                                                        |
| ---------------------------------- | -------------------------------------------------------------- |
| `scripts/bump_version.py`          | Bump version across all config files                           |
| `scripts/serve_mock_release.py`    | Serve mock update JSON on `localhost:8000` for updater testing |
| `scripts/new_client.cjs`           | Scaffold a new client entry                                    |
| `scripts/remove_releases.py`       | Delete old GitHub releases (dry-run by default)                |
| `scripts/remove_unused_actions.py` | Delete unused GH Actions runs                                  |

---

## Rust tests

Tests live in `src-tauri/src/tests/`. Run with `cargo test` from `src-tauri/`. Test files cover: clients commands, data, manager, runtime, settings, updater, utils commands.

---

## Deep links

The app registers the `collapseloader://` URI scheme. Supported actions: `verify-email` and `launch-client`. Single-instance enforcement is handled via `tauri-plugin-single-instance`.

---

## Windows-only behaviour

- Junction points (not symlinks) are used on Windows to share `resourcepacks` and `shaderpacks` across clients.
- `junction` crate is a Windows-only dependency.
- DPI scaling helper (`core/utils/dpi.rs`) runs a background process on Windows when configured.
- WebView2 is required; the app prompts to install it if missing.
