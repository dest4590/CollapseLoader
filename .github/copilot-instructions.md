## CollapseLoader – Copilot Instructions

Purpose: GUI (Vue 3 + Tauri 2) launcher for secure Minecraft clients. Paired Django backend (`CollapseAuth`) provides auth, friends, admin + client distribution endpoints. Focus on fast startup, cached batched API calls, minimal network chatter, desktop-native UX via Tauri plugins.

### Tech & Build

-   Frontend: Vue 3 (script setup + composition API), TypeScript, Vite, Tailwind (via `@tailwindcss/vite`) + DaisyUI.
-   Desktop wrapper: Tauri 2 (`src-tauri/tauri.conf.json`); backend commands defined there (`beforeDevCommand`, `beforeBuildCommand`).
-   Build scripts: `npm run tauri dev` (Vite + Tauri dev auto-run), `npm run build` (type check + build), `npm run tauri dev`, `npm run tauri build`.
-   Type safety: Always run `yarn check` before large refactors; keep zero vue-tsc errors.

### Runtime Initialization Flow

1. Tauri starts, invokes Rust command `get_auth_url` -> `initializeAuthUrl()` stores base auth URL (`src/config.ts`).
2. i18n loads saved language via Tauri `get_settings` (`src/i18n/index.ts`).
3. Critical data preloading may call `/auth/init/` and `/auth/friends/batch/` (see `apiClient.preloadCriticalData`).
4. Friends subsystem sets up status polling (`useFriends.ts`).

### API Layer Pattern (`src/services/apiClient.ts`)

-   Single exported instance (`apiClient`) with helper bindings (`apiGet`, `apiPost`, …); use these—not raw axios.
-   Automatic base URL injection: prefix relative (`/...`) paths with `getAuthUrl()` result.
-   Headers auto-set: `Authorization: Token <token>` (localStorage key: `authToken`), `Accept-Language` from i18n.
-   Request de-duplication: concurrent identical GETs share a Promise (`pendingRequests` map).
-   Smart caching (GET only) for whitelisted endpoints (friends, profile, init, admin). TTL varies by endpoint; DO NOT add new cache use without updating `shouldCache` + `getCacheTTL`.
-   Cache invalidation on mutating calls uses substring/relationship heuristics (`invalidateCache` + `isRelatedEndpoint`). If adding new friend/admin endpoints ensure they land in relationship groups.
-   Batch logic: `batchGet` tries aggregated friends endpoint (`/auth/friends/batch/`) else falls back to individual calls.
-   Metrics available via `apiMetrics` & `apiCacheStats` (avoid expensive computations inside UI reactivity; snapshot when needed).

### State & Composables

-   Global singleton state patterns: modules like `useFriends.ts` declare a reactive object outside the composable, then return computed wrappers. Re-importing keeps a shared instance—avoid manual new state copies.
-   Friends module strategies:
    -   Primary load via `/auth/friends/batch/`; fallback to separate `/auth/friends/` + `/auth/friends/requests/`.
    -   Adaptive polling: interval increases with consecutive “no-change” status updates; resets on change.
    -   When mutating (send / respond / remove) rely on server response + local surgical updates instead of full reload where possible.
    -   Guard against redundant reload if data is <30s old unless `forceRefresh` param.
-   Introduce new global reactive modules following: (1) external `reactive` store, (2) composable that only exposes computed & action functions, (3) defensive early returns if unauthenticated.

### UI / Component Conventions

-   Modals: Controlled via `modalService` (`showModal(id, component, options, props, listeners)`); components stored with `markRaw` to avoid unnecessary reactivity. Provide a stable `id`; closing triggers delayed removal (300ms) for transition.
-   Notifications: Use `sendNativeNotification` (permission negotiation included); keep failures silent beyond console.
-   Avoid directly manipulating DOM (use Vue reactivity). For animations prefer GSAP if sequence logic is complex.

### i18n

-   Add translations in `src/i18n/locales/<code>.json`; register new locale in `getAvailableLanguages()` and backend `LANGUAGES` (Django).
-   Changing language updates localStorage and persists via `save_settings` Tauri command. Always use `changeLanguage(locale)`.

### Auth & Tokens

-   `authToken` stored in localStorage. On logout-like flows ensure: remove token, clear cached API (`apiClient.clearCache()`), reset composable states (e.g. call `clearFriendsData()`).
-   Any feature needing early API calls must await `initializeAuthUrl()` before first request or use existing helpers that already run after init.

### Adding New API Endpoints

-   Extend backend path under `/auth/...` for authenticated resources or `/api/...` for public/asset/client operations (see Django `Core/urls.py`). Align naming with existing verbs: list: `/resource/`, single action: `/resource/<id>/verb/`.
-   Frontend: add wrapper in a dedicated service file under `src/services/` if logically grouped (avoid bloating `apiClient`). Keep raw endpoint strings central: if adding several related endpoints consider an enum or object literal export.
-   If GET endpoint benefits from caching: add to `cacheableEndpoints` and determine TTL in `getCacheTTL`.

### Performance & Network Hygiene

-   Prefer batch endpoints for related entities (pattern shown with friends). If building a batch, also supply graceful fallback.
-   Logging: Keep `console.log` for performance metrics brief; follow existing descriptive style (action + duration + counts). Avoid noisy per-item logs.

### Tauri Integration

-   Use `@tauri-apps/api/*` for core & plugin interactions; wrap new invoke calls in a small helper (like `initializeAuthUrl`) that validates types.
-   Security: CSP disabled (`csp: null`)—do not inject remote scripts or eval; keep dependencies vetted.

### Styling / Tailwind

-   Use semantic utility class groupings; DaisyUI components allowed. Prefer consistent sizing tokens already present—inspect existing components before new custom CSS.

### Versioning & Release Helpers

-   Version lives both in `package.json` and `src-tauri/tauri.conf.json`; scripts in `scripts/` (e.g. `bump_version.py`) manage sync. When bumping manually, update both or run helper script.

### When Contributing Code

-   Use existing helper bindings (`apiGet` etc.) instead of recreating axios logic.
-   Maintain singleton composable pattern; never export raw reactive state without encapsulation.
-   Update relationships in `apiClient.isRelatedEndpoint` when new endpoints should invalidate caches.
-   Keep mutation methods optimistic only if server returns minimal data; otherwise fetch and rely on batch loader for coherence.

### Quick Examples

Fetch + cache profile:

```ts
import { apiGet } from '@/services/apiClient';
const profile = await apiGet('/auth/profile/');
```

Invalidate via mutation:

```ts
import { apiPost } from '@/services/apiClient';
await apiPost('/auth/friends/send/', { username }); // related friends caches auto-invalidated
```

### Avoid / Anti-Patterns

-   Do NOT call axios directly; breaks caching & metrics.
-   Do NOT duplicate polling timers; reuse existing adaptive logic or centralize new pollers similarly.
-   Avoid refetch loops: always check freshness timestamps like friends module (<30s shortcut) before heavy reloads.
-   Do NOT run `yarn check` after changes, i will check it myself, and say if there are any issues.

Questions / gaps: If adding (1) new batch patterns, (2) new Tauri commands, or (3) complex cache invalidation groups—highlight them for doc update.
