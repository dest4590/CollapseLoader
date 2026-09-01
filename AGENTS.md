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

Communication is exclusively via Tauri `invoke()` commands – there is no HTTP API between the two halves. The frontend API layer (`src/api/clients/internal.ts`) routes all HTTP calls through `invoke("api_request")`, so the Rust backend handles actual network requests.

### Frontend layout (`src/`)

- `main.ts` → `bootstrap/applicationBootstrap.ts` (mounts correct Vue component based on `?window=` URL param)
- Multiple window types: `main` (default), `network`, `customization` – each is a different root component
- `features/` – domain modules: `auth/`, `chat/`, `clients/`, `download/`, `friends/`, `marketplace/`, `presets/`, `social/`
- `api/` – Tauri invoke wrappers (`ApiClient` class, re-exports `apiGet`/`apiPost`/etc.)
- `services/` – business logic services (i18n, settings, theme, chat, updater, etc.)
- `shared/` – reusable components, composables, types, utils
- `components/` – top-level UI components (`core/`, `customization/`, `features/`, `modals/`, `presets/`, `settings/`)
- `composables/` – app-level composables (`useAppInit`, `useAppHandlers`, `useDownloadSpeedMonitor`, `useIrcChat`)
- `views/` – page-level view components (16 views mapped in `router/views.ts`)
- `windows/` – separate window root components (`CustomizationWindow.vue`, `NetworkWindow.vue`)
- `layouts/` – layout components (`Sidebar.vue`, `Titlebar.vue`, forms, modals)
- `utils/` – top-level utilities (`settings.ts`, `tabs.ts`)
- `assets/` – static assets (CSS, images, icons, videos)
- `config.ts` – API URL initialisation; must call `initializeApiUrl()` before using `getApiUrl()`

### Router

The app uses a **custom lightweight router** (not vue-router) implemented with Vue refs in `router/index.ts`. API: `push()`, `replace()`, `back()`, `canGoBack()`, `clearHistory()`. Route-to-component mapping is in `router/views.ts` with `tabOrder` for transition direction.

Supported routes: `home`, `news`, `settings`, `about`, `customization`, `custom_clients`, `app_logs`, `account`, `login`, `register`, `verify`, `friends`, `user-profile`, `marketplace`, `network_debug`, `chat`.

### State management

There are no Pinia/Vuex stores. State is managed through **services** (singletons) and **composables** (reactive state). The `@stores` alias is defined in config but `src/stores/` does not exist.

### Path aliases (configured in both `vite.config.ts` and `tsconfig.json`)

```
@          → src/
@shared    → src/shared/
@features  → src/features/
@api       → src/api/
@services  → src/services/
@layouts   → src/layouts/
@router    → src/router/index.ts  (bare alias)
@stores    → src/stores/  (alias defined but directory does not exist)
@components → src/components/
```

### Backend layout (`src-tauri/src/`)

- `main.rs` – binary entry point; parses CLI args, loads `.env`, calls `prepare_startup()` then `run()`
- `lib.rs` – library crate entry; Tauri builder, `invoke_handler!` macro (120+ commands), `setup` hook, `on_window_event`
- `commands/` – Tauri command handlers:
    - `clients.rs` – client launch, download, mods, logs, shortcuts
    - `irc.rs` – IRC connect/disconnect/send (`IrcState` managed by Tauri)
    - `mod_builds.rs` – mod build CRUD (create, update, delete, export, import)
    - `network.rs` – `api_request`, network history
    - `presets.rs` – preset CRUD
    - `report.rs` – network report generation/export
    - `settings.rs` – accounts, favorites, flags, settings, telemetry
    - `updater.rs` – check/download/install updates
    - `utils.rs` – data folder, base64, version, Discord RPC, tray, launch history
- `core/` – business logic
    - `clients/` – client manager, custom clients, log checker, agent overlay verification
    - `network/` – downloader, servers (health checks), API client, cache, server ads
    - `platform/` – OS-specific code (Windows WebView2, Linux WebKitGTK, message boxes, DPI)
    - `storage/` – persistent state (settings, accounts, favorites, flags, presets, custom clients, mod builds, launch history)
    - `utils/` – globals, logging macros, helpers, process, hashing, archive, Discord RPC, DPI, taskbar, CLI args, module tags
    - `state.rs` – `AppState`, `ClientState`, `CustomClientsState` (Tauri managed state)
    - `app_runtime.rs` – `StartupRuntime`, tray menu parsing, deep link parsing/deduplication
- `build.rs` – build script; captures git hash/branch/body, sets `DEVELOPMENT` env var
- `tests/` – Rust unit tests (7 modules: clients_command, data, manager, runtime, settings, updater, utils_command)

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
- **ESLint** uses flat config (`eslint.config.cjs`) targeting `.ts`, `.tsx`, `.vue`, `.js` files. Unused vars require `_` prefix.
- **Prettier**: 4-space indent, trailing commas (`es5`). See `.prettierrc.yaml`.
- **Tailwind CSS v4** config lives in CSS via `@import "tailwindcss"` and `@plugin "daisyui"` – no `tailwind.config.js`.
- **daisyUI v5** is the component library (see `.github/instructions/daisyui.instructions.md` for full class reference). Use daisyUI semantic color names (`bg-primary`, `text-base-content`, etc.) instead of Tailwind hardcoded colors so themes work.
- **Monaco editor** loads from CDN (`cdn.jsdelivr.net/npm/monaco-editor@0.55/min/vs`) – not bundled locally.
- **`daisyui` is excluded from Vite `optimizeDeps`** – do not add it back.
- **Rust `profile.dev`**: incremental builds, `opt-level=0`, `debug=1`. Dependencies are compiled at `opt-level=2` for reasonable dev performance.
- **i18n**: 5 languages supported (English, Polish, Russian, Ukrainian, Chinese Simplified) via `vue-i18n` in `services/i18n/`.

---

## CI / Build

### Workflows

- **`build.yml`** – triggers on push to `main` or `dev` branches, or manual `workflow_dispatch`.
    - `check-flags` job: parses commit message for `[skip ci]`/`skip-ci`/`skip_ci` and `release`/`build macos` keywords.
    - `build` job (matrix: ubuntu-22.04 + windows-latest): installs deps, builds Tauri bundles, uploads artifacts.
    - `build-macos` job (conditional): builds universal binary (`universal-apple-darwin`) with aarch64 + x86_64 targets.
    - `create-release` job: creates GitHub prerelease tagged `build-<sha>` with all artifacts.
- **`cleanup-artifacts.yml`** – runs weekly (Sunday 2 AM UTC) or manually; keeps only the 5 newest build artifacts.

### Build details

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

| Script                             | Purpose                                                                  |
| ---------------------------------- | ------------------------------------------------------------------------ |
| `scripts/bump_version.py`          | Bump version across all config files                                     |
| `scripts/serve_mock_release.py`    | Serve mock update JSON on `localhost:8000` for updater testing           |
| `scripts/new_client.py`            | Add new client entry (interactive menu or CLI args)                      |
| `scripts/new_client.cjs`           | Node.js equivalent of new_client.py (CLI-only)                           |
| `scripts/md5.py`                   | Compute MD5 hash of a file (interactive or CLI)                          |
| `scripts/md5.cjs`                  | Node.js equivalent of md5.py (CLI-only)                                  |
| `scripts/scripts_gui.py`           | Web GUI with buttons – opens browser at localhost:8765                   |
| `scripts/remove_releases.py`       | Delete old GitHub releases (dry-run by default, needs `GITHUB_TOKEN`)    |
| `scripts/remove_unused_actions.py` | Delete unused GH Actions runs (dry-run by default, needs `GITHUB_TOKEN`) |

---

## Rust tests

Tests live in `src-tauri/src/tests/`. Run with `cargo test` from `src-tauri/`. Test files cover: clients commands, data, manager, runtime, settings, updater, utils commands.

---

## Deep links

The app registers the `collapseloader://` URI scheme. Supported actions: `verify-email` and `launch-client`. Single-instance enforcement is handled via `tauri-plugin-single-instance`.

---

## Tauri plugins

The app uses these Tauri plugins: `opener`, `notification`, `dialog`, `fs`, `deep-link`, `single-instance` (with deep-link feature).

---

## Windows-only behaviour

- Junction points (not symlinks) are used on Windows to share `resourcepacks` and `shaderpacks` across clients.
- `junction` crate is a Windows-only dependency.
- DPI scaling helper (`core/utils/dpi.rs`) runs a background process on Windows when configured.
- Taskbar progress indicator (`core/utils/taskbar.rs`) via Windows COM integration.
- WebView2 is required; the app prompts to install it if missing.

---

## Linux-only behaviour

- WebKitGTK dependency checks at startup (`core/platform/linux.rs`).
- `zbus` crate used for Linux-specific D-Bus integration.
- `mold` linker used in CI for faster builds.
