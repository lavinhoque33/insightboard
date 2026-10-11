# M0–M3 baseline contract: InsightBoard as built at `f15c6a3`

- Status: **accepted** 2026-10-10 with M0–M3 — the as-built record of M0–M3 at `f15c6a3` (2026-03-03). It records
  behaviour as it is, defects included, not as it was intended. Since `f15c6a3`, D3 is fixed (`354e06c`, M4
  decision 4) and D30 and D31 were added from the drills recorded in the
  [baseline verification record](../development/baseline-verification.md). M4 changes this behaviour only through
  the [M4 contract](m4-quality-gate-contract.md).
- Sources: the code at `f15c6a3`; the three read-only audits of 2026-10-10 summarised in session
  [2026-10-10-01](../development/sessions/2026-10-10-01-workflow-onboarding.md);
  [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) sections 7–10; [roadmap](../product/roadmap.md) M0–M3;
  [system overview](system-overview.md); [ADR 0001](../decisions/ADR-0001-foundation-stack.md);
  `contracts/api-contract.v1.json`; the legacy
  [condensed spec](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/.github/instructions/Insightboard_Condensed_spec.instructions.md)
  and [testing audit](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/testing_audit.md) (both
  **reported** evidence only).

M0–M3 were delivered 2025-10-28 to 2026-03-03 under the retired phase protocol (plan section 1.3). Line references
are at `f15c6a3`. `[INFERENCE]` marks behaviour deduced from code or library defaults and not yet observed on this
workstation; the drills in section 13 turn those into observations.

## 1. Scope

| Milestone | Delivered behaviour covered here |
| --- | --- |
| M0 — Foundation | Compose Postgres and Redis, configuration, embedded migrations, `/healthz`, Vue shell (sections 4–6) |
| M1 — Accounts and dashboards | Register, login, `/me`, owner-scoped dashboard CRUD, frontend auth flow and list (sections 2, 3, 7, 8) |
| M2 — Widgets and editor | Five provider data endpoints with Redis caching; widget registry; GridStack editor with persisted layout (sections 2, 6, 9, 10) |
| M3 — UI polish and contract alignment | daisyUI styling, auth and editor fixes, the v1 payload contract, draft-on-add editor (sections 9–11) |

The canonical defect list (section 12) is part of this contract: a drill that exposes a listed defect records an
observed defect, not a pass.

## 2. Routes and status codes

Router: `backend/src/main.rs:74-87,104-122`. Every route except `/healthz`, register, and login requires
`Authorization: Bearer <jwt>`; the `UserCtx` extractor runs first, so a missing or bad token returns 401 before
query or body validation `[INFERENCE: axum runs extractors in order]`.

| # | Method and path | Auth | Input and validation | Success | Errors |
| --- | --- | --- | --- | --- | --- |
| 1 | `GET /healthz` | no | — | 200 `{status:"ok",service:"insightboard-backend",version}` (`backend/src/handlers/health.rs`); no dependency check | — |
| 2 | `POST /api/auth/register` | no | `{email,password}`; both non-empty; password at least 8 bytes; no email format check or normalisation | 201 `{token,user:{id,email,created_at}}` | 400 "Email and password are required", "Password must be at least 8 characters", "Email already registered"; 500 on a duplicate-email race (D17) |
| 3 | `POST /api/auth/login` | no | `{email,password}` non-empty; exact, case-sensitive email match | 200 `{token,user}` | 401 "Invalid credentials" (unknown email or wrong password) |
| 4 | `GET /api/me` | yes | — | 200 `{id,email,created_at}` | 401; 500 when the user no longer exists (D17) |
| 5 | `GET /api/dashboards` | yes | — | 200 array of the caller's dashboards, `updated_at DESC`, no paging | 401, 500 |
| 6 | `POST /api/dashboards` | yes | `{name, layout_json?, settings_json?}`; `name` trimmed and non-empty; JSON fields unvalidated; omitted fields stored as JSON `null` (D8) | 201 dashboard | 400 "Dashboard name is required"; 401; extractor rejections (section 3) |
| 7 | `GET /api/dashboards/:id` | yes | path UUID | 200 dashboard | 404 "Dashboard not found" when missing or not owned; 400 (text) for a non-UUID id |
| 8 | `PUT /api/dashboards/:id` | yes | `{name?,layout_json?,settings_json?}`; fields not sent keep their value; `name` not trimmed or validated (D8) | 200 dashboard | 404 "Dashboard not found"; 400; 401 |
| 9 | `DELETE /api/dashboards/:id` | yes | path UUID | 204 empty | 404 "Dashboard not found"; 401 |
| 10 | `GET /api/data/github?username=` | yes | `username` required, non-blank, not encoded (D2) | 200 array of `{id,type,repo:{name},created_at,payload}` | 400 "username is required"; 502 "External service error" |
| 11 | `GET /api/data/weather?city=` | yes | `city` required, not encoded (D7); key checked first | 200 `{city,country,temperature,feels_like,humidity,description,wind_speed,icon}` | 400 "OpenWeather API key not configured"; 502 |
| 12 | `GET /api/data/news?topic=` | yes | `topic` optional, default `technology`, not encoded (D7); key checked first | 200 array of `{title,description,url,source:{name},publishedAt,urlToImage}` | 400 "NewsAPI key not configured"; 502 |
| 13 | `GET /api/data/crypto?ids=` | yes | `ids` optional, default `bitcoin,ethereum`; split, trimmed, lower-cased, empties dropped | 200 array of `{id,symbol,name,current_price,price_change_percentage_24h,market_cap,total_volume}` | 400 "ids is required"; 502, including a null upstream number (D20) |
| 14 | `GET /api/data/status?urls=` | yes | `urls` required comma list; no scheme, host, or count limit (D1) | 200 array of `{url,status,response_time_ms,error}`; per-URL failure is `status: 0` with the raw error text | 400 "urls is required" |

Any database failure on any route returns 500 "Database error occurred". Unknown routes return 404 and wrong
methods 405 with empty bodies `[INFERENCE: axum 0.7 defaults]`.

## 3. Error model

Application errors (`backend/src/error.rs:39-73`) return JSON `{"error": "<message>"}`:

| Variant | Status | Body | Logged |
| --- | --- | --- | --- |
| Database | 500 | "Database error occurred" | yes, with detail |
| Auth(msg) | 401 | the message, including "Invalid token: <kind>" (D17) | no |
| Unauthorized | 401 | "Unauthorized" | no |
| Validation(msg) | 400 | the message | no |
| NotFound(msg) | 404 | the message | no |
| ExternalApi(msg) | 502 | "External service error" | yes, with the provider error text (D7) |
| Internal(msg) | 500 | "Internal server error" | yes |
| Redis, Forbidden | 500, 403 | never constructed (D26) | — |

Axum extractor rejections are text/plain, not JSON: 400 malformed JSON or missing query parameter or bad path
UUID, 413 body over the default limit, 415 wrong content type, 422 missing field or wrong type
`[INFERENCE: axum 0.7 defaults]`. The frontend reads `data.message || data.error` and formats 422 `details`; that
branch is unreachable with this backend (D16). Network failures and axios timeouts surface as
"No response from server. Please check your connection." with status 0.

## 4. Configuration

Backend (`backend/src/config.rs:18-34`), loaded with `dotenvy` from the working directory or a parent after
tracing has started (`backend/src/main.rs:38-48`):

| Variable | Behaviour |
| --- | --- |
| `APP_PORT` | Optional, default 8080 |
| `DATABASE_URL`, `REDIS_URL`, `JWT_SECRET` | Required; a missing value panics at startup (D18); no secret strength check (D17) |
| `GITHUB_API_TOKEN`, `OPENWEATHER_API_KEY`, `NEWSAPI_API_KEY` | Optional; trimmed, blank treated as unset |
| `COINMARKETCAP_API_KEY` | Read, never used (D18, D26) |
| `RUST_LOG` | Honoured only from the process environment; default `insightboard_backend=debug,tower_http=debug`; the `.env` value is ignored (D18) |
| `CORS_ORIGINS`, `SENTRY_DSN`, `AWS_REGION` | Documented in `.env.example`, never read (D4, D18) |

Hard-coded: bind host `0.0.0.0` (D3), permissive CORS (D4), pool size 10, JWT TTL 7 days, cache TTLs, status
timeout 5 s, provider base URLs (D11).

Frontend: `VITE_API_URL` (default `/api`, `frontend/src/api/client.ts:15`); the Vite dev server uses its default
host and port 5173 and proxies `/api` to `http://localhost:8080` (`frontend/vite.config.ts`).

Compose (`docker-compose.yml`): `postgres:16-alpine` (`insightboard-postgres`, database `insightboard`,
`postgres/postgres`, `5432:5432`, volume `postgres_data`) and `redis:7-alpine` (`insightboard-redis`, AOF on, no
password, `6379:6379`, volume `redis_data`), both with healthchecks and `restart: unless-stopped`; published on all
interfaces (D3).

Superseded in M4a (`2780fc0`, 2026-10-10): `.env` loads before tracing, so `RUST_LOG` from `.env` works, and a missing
or blank `DATABASE_URL`, `REDIS_URL`, or `JWT_SECRET` fails startup with a named error; `CORS_ORIGINS` is read (an
exact-origin allowlist, default the two Vite dev origins); `COINMARKETCAP_API_KEY`, `SENTRY_DSN`, and `AWS_REGION`
are gone. Compose has no `version`, `container_name`, or fixed network name, so containers are named by the compose
project (for example `insightboard-redis-1`); use `docker compose exec redis|postgres …`. The current reference is
[local setup](../development/local-setup.md) and the [M4 contract](m4-quality-gate-contract.md).

Superseded in M4b (`b20986e`, 2026-10-10), across sections 2–7 and 9: errors follow the M4 error model (422 with
`details`, 409 duplicates and the 100-dashboard cap, 429 rate limits, 503 for a missing key, generic 401 and 502, JSON
for every rejection); tokens last 24 hours and `JWT_SECRET` must be at least 32 bytes; emails are normalised;
dashboards default to `{"widgets":[]}` and `{}` (migration 003); the status widget refuses non-public addresses;
providers share one client with timeouts and a `User-Agent`; cache keys are normalised and status results are per
user; `/readyz` reports Postgres and Redis. New variables (provider base URLs, timeouts, rate limits,
`STATUS_ALLOWED_FIXTURES`) are listed in [local setup](../development/local-setup.md).

Superseded in M4c (`5caff27`, `13e9018`, 2026-10-10) for the frontend: the editor loads one dashboard by id and saves
through a serialised queue; the client checks token expiry and keeps the destination on 401; titles come from `name`
only; daisyUI is stable 5 with custom light and dark themes that follow the OS, with a toggle.

## 5. Persistence

- `users(id uuid pk default gen_random_uuid(), email text unique not null, password_hash text not null,
  created_at timestamptz not null default now())`, plus a redundant `idx_users_email` (D29).
- `dashboards(id uuid pk, user_id uuid not null references users on delete cascade, name text not null,
  layout_json jsonb not null default '[]', settings_json jsonb not null default '{}', created_at, updated_at)`;
  indexes on `user_id` and `updated_at DESC`. `NOT NULL` does not prevent a JSON `null` value (D8).
- Migrations `backend/migrations/001_create_users.sql` and `002_create_dashboards.sql` are embedded with
  `sqlx::migrate!` (`backend/src/db.rs:27`) and applied at startup (`backend/src/main.rs:59`); `sqlx-cli` is not
  needed. No down migrations.
- Queries are runtime `sqlx::query_as` strings with bound parameters; compiling needs no database. Pool maximum 10
  connections; a database that is down at startup aborts the process.
- Ownership: every dashboard read and write filters by the token's user id; another user's id returns 404.

## 6. Cache

| Endpoint | Key | TTL |
| --- | --- | --- |
| github | `github:{username}` | 300 s |
| weather | `weather:{city}` | 600 s |
| news | `news:{topic}` | 900 s |
| crypto | `crypto:{normalised ids}` | 300 s |
| status | `status:{raw urls}` | 120 s |

Values are JSON strings written with `SET … EX`. Keys are raw user input, unbounded, and shared across users
(D29). A Redis failure at startup aborts the process; per request, read errors count as a miss and write errors are
ignored (D19). Each cache operation opens a new connection (D19). Upstream errors are not cached.

## 7. Auth

- Tokens: HS256 JWT signed with `JWT_SECRET`; claims `sub` (user id), `email`, `exp`, `iat`; lifetime 7 days
  (`backend/src/auth.rs:63`). No refresh, revocation, or logout endpoint.
- Passwords: `Argon2::default()` with a random salt, run on the async worker threads (D6).
- The extractor validates the token and parses `sub`; it does not check that the user still exists (D17).
- Login returns the same 401 body for an unknown email and a wrong password.

## 8. Frontend routes and session

| Path | View | Guard |
| --- | --- | --- |
| `/` | redirect to `/dashboards` when authenticated, else `/login` | — |
| `/login`, `/register` | LoginView, RegisterView | guest only; authenticated users go to `/dashboards` |
| `/dashboards` | DashboardListView | auth; otherwise `/login?redirect=<path>` |
| `/dashboards/:id` | DashboardEditorView (lazy chunk) | auth |
| anything else | redirect to `/` | — |

- Session: localStorage keys `auth_token` (raw JWT) and `user` (JSON). "Authenticated" means both are present;
  expiry is never checked and `/me` is never called (D23).
- Requests carry `Authorization: Bearer <token>`; axios timeout 10 s.
- A 401 outside login and register clears both keys and sets `window.location.href = '/login'`, a full reload that
  loses the intended destination (D23). Login honours `?redirect=`.
- Logout clears the auth store and navigates to `/login`; the dashboard store is not reset (D23).

## 9. Widgets

Registry: `frontend/src/composables/widgetRegistry.ts`, registered at startup (`frontend/src/main.ts`).

| Type | Config field and client validation | Default config | Client refresh | Default size | Backend endpoint |
| --- | --- | --- | --- | --- | --- |
| `github` | `username`, required, GitHub username pattern | `{username:''}` | 300 s | 4×3 | row 10 |
| `weather` | `city`, required | `{city:''}` | 600 s | 3×3 | row 11 |
| `news` | `topic`, select of technology, business, science, health, sports, entertainment | `{topic:'technology'}` | 900 s | 4×4 | row 12 |
| `crypto` | `cryptoIds`, required, non-blank | `{cryptoIds:'bitcoin,ethereum,cardano'}` | 60 s | 3×4 | row 13 |
| `status` | `urls`, textarea, comma-separated, each must parse with `new URL()` | `{urls:''}` | 120 s | 4×3 | row 14 |

Each widget loads on mount, refreshes on its interval, reloads when its key config field changes, and clears its
timer on unmount. `minSize` is declared but not applied (D22). The frame (`BaseWidget.vue`) shows a spinner while
loading, an alert with "Try Again" on error, and a refresh, configure, and remove menu; the drag handle is the
header. The configuration modal opens empty for existing widgets and does not show defaults (D9).

## 10. Editor and layout persistence

- Load: the editor fetches the whole dashboard list and picks the id (D22); an unknown or foreign id shows the empty
  state and later saves are skipped silently (D10).
- Grid: GridStack 10.3.1, 12 columns, `cellHeight` 80, margin 10, drag handle `.widget-drag-handle`, resize
  handles `se, sw`.
- Add (draft-on-add): choosing a widget type opens the configuration modal with a draft; **Cancel** creates
  nothing; **Save Changes** with valid configuration adds the widget to the grid.
- Configure and remove act on the live widget; remove has no confirmation.
- Auto-save: 2 s after the last grid change or widget edit, `PUT /api/dashboards/:id` with only `layout_json`.
  Save errors reach only `console.error` (D10). Leaving the editor after any edit sends one more PUT (D22).
- Persisted layout:
  `layout_json = {"widgets":[{"id":"widget-<ms>-<rand>","type":"github|weather|news|crypto|status","x":<int>,"y":<int>,"w":<int>,"h":<int>,"config":{…}}]}`.
- Creating a dashboard from the list sends `name` = `settings_json.title` = `"New Dashboard"`,
  `layout_json = {"widgets":[]}`, `settings_json = {"title":"New Dashboard","theme":"light"}`. The client maps
  `layout_json`/`settings_json` to `layout`/`settings`, drops `name`, and displays `settings.title` (D28). There is
  no rename UI.

## 11. Machine-readable contract

`contracts/api-contract.v1.json` (JSON Schema 2020-12) defines payload shapes for auth, dashboards, widget
responses, and `ErrorResponse`. `frontend/src/contracts/apiContract.ts` mirrors it by hand and types the API layer.
The schema has no paths, methods, query parameters, status codes, or auth requirements, and nothing validates
either file against the running code (D15). Known divergences: dashboard JSON fields may be `null` or `[]` (D8),
news `url`/`publishedAt` may be empty strings, register accepts any email string and unknown fields. Both files
stay in place and change together under one writer.

## 12. Known defects

Canonical catalogue. IDs are stable; other documents reference them and never renumber. Severity follows the
audits; refs are at `f15c6a3`; slice is the proposed [M4](m4-quality-gate-contract.md) slice or later milestone.

| ID | Sev | Defect | Refs | Slice |
| --- | --- | --- | --- | --- |
| D1 | Critical | Status widget SSRF: any scheme and host (loopback, private, link-local, cloud metadata); redirects followed `[INFERENCE: reqwest default policy]`; unbounded URL count; raw reqwest error text returned; results cached globally by raw query | `backend/src/widgets/status.rs:29-75` (error text `:68`) | M4b; fixed in `b20986e` |
| D2 | Critical | GitHub `username` interpolated unencoded into the upstream path while the server token is attached (path injection with the owner's token) `[INFERENCE: exploitability not exercised]` | `backend/src/widgets/github.rs:47,50-52` | M4b; fixed in `b20986e` |
| D3 | High | Network exposure: backend binds `0.0.0.0` (`main.rs:90`); compose publishes Postgres and Redis on all interfaces with `postgres/postgres` and no Redis password; host firewall zone opens 1025–65535/tcp (observed) | `backend/src/main.rs:90`, `docker-compose.yml` | M4a; fixed early in `354e06c` (M4 decision 4, 2026-10-10; B0 observed in the [baseline record](../development/baseline-verification.md)) |
| D4 | High | `CorsLayer::permissive()`; `CORS_ORIGINS` documented but never read | `backend/src/main.rs:82`, `.env.example` | M4a; fixed in `2780fc0` (allowlist from `CORS_ORIGINS`) |
| D5 | High | No outbound timeout on github, weather, news, crypto clients; no server-side timeout layer; a new reqwest client per request | `github.rs:45`, `weather.rs:45`, `news.rs:55`, `crypto.rs:58` | M4b; fixed in `b20986e` |
| D6 | High | No rate limiting (`governor` declared, unused); Argon2 hashing runs on async workers | `backend/Cargo.toml:58`, `backend/src/auth.rs` | M4b; fixed in `b20986e` |
| D7 | High | User input unencoded in provider URLs (`city`, `topic`, `ids`); provider API keys travel in query strings and may reach logs via the external-error path | `weather.rs:41`, `news.rs:51`, `crypto.rs:54`, `error.rs:56` | M4b; fixed in `b20986e` (OpenWeather's key stays in the query because the provider has no header option; URLs are never logged) |
| D8 | High | Dashboard JSON integrity: create without `layout_json`/`settings_json` stores JSON `null`; DB default `'[]'` disagrees with the contract's `{"widgets":[]}`; no shape validation; update accepts an empty name | `backend/src/models/dashboard.rs:22-25`, `migrations/002_create_dashboards.sql`, `handlers/dashboard.rs:109-110` | M4b; fixed in `b20986e` (migration 003) |
| D9 | High | Widget config modal never pre-fills: the watch on `show` lacks `immediate` and the modal mounts with `show` already true, so editing wipes existing config, defaults never appear, and News fails validation until re-picked | `frontend/src/components/WidgetConfigModal.vue:26-34`, `frontend/src/views/DashboardEditorView.vue:100-106,116-122,164-167,458-460` | M4c; fixed in `5caff27` (drill 12) |
| D10 | High | Silent editor failures: save errors only reach `console.error`; a failed load or unknown/foreign dashboard id shows the empty state and later saves return early, dropping edits | `DashboardEditorView.vue:38-53,196,231-233` | M4c; fixed in `5caff27` (drill 13) |
| D11 | High | Thin automated tests: 5 backend unit tests, 3 frontend component tests; no integration, contract, E2E, or CI; hard-coded provider base URLs block mocking (`mockito` unused) | `backend/src/{auth,config}.rs`, `frontend/src/**/__tests__` | M4b (backend) and M4c; fixed in `b20986e`, `e20a0bc`, `5caff27`, `13e9018` (87 backend, 91 Vitest, 2 Playwright journeys, CI) |
| D12 | High | Non-reproducible builds: `Cargo.lock` gitignored; no `rust-toolchain.toml` or `rust-version`; no `engines` field; `daisyui ^5.0.0-beta.9` floats | `.gitignore:16`, `backend/Cargo.toml`, `frontend/package.json` | M4a; fixed in `2780fc0`; daisyUI stable `^5.7.47` in `5caff27` |
| D13 | High | Failing and incomplete quality gates: `cargo fmt --check` (67 diff hunks across all 15 source files); `cargo clippy --all-targets -- -D warnings` (2 dead-code warnings); no ESLint or Prettier; `vitest.config.ts` is outside every `tsconfig` and never type-checked; `any` remains in `client.ts`, widget `catch` blocks, the widget store, and the modal | `backend/src/auth.rs:34`, `backend/src/error.rs:30`, `frontend/tsconfig.node.json`, `frontend/src/api/client.ts` | M4a; fixed in `d4bdca9` (reformat) and `2780fc0` |
| D14 | High | Vulnerable npm dependencies (observed 2026-10-10): `npm audit --omit=dev` 8 findings (7 high, 1 moderate; axios direct); full audit 17 (1 critical, 13 high, 3 moderate). The 2026-03-03 audit reported 0 | `frontend/package-lock.json` | M4a; fixed in `2780fc0` (both audits 0) |
| D15 | High | API contract partial and unenforced: v1 JSON Schema covers payload shapes only (no paths, methods, query params, statuses, auth, errors); `apiContract.ts` is a hand mirror; nothing validates either side; backend can emit violating payloads (D8, empty news `url`/`publishedAt`) | `contracts/api-contract.v1.json`, `frontend/src/contracts/apiContract.ts`, `backend/src/widgets/news.rs:72-78` | M4b; fixed in `b20986e` for shapes and enforcement (error bodies added; `jsonschema` and ajv tests on both sides); paths and methods stay out of the schema until OpenAPI (after M4) |
| D16 | Medium | Error model mismatch: client 422 "details" formatting is unreachable (backend sends 400 `{error}`); axum extractor rejections are text/plain; missing provider key returns 400 | `frontend/src/api/client.ts:46-77`, `backend/src/error.rs:51` | M4b; fixed in `b20986e` |
| D17 | Medium | Auth correctness: duplicate-email race returns 500 and a duplicate returns 400 not 409; `/me` for a deleted user returns 500; email neither normalised nor validated; JWT error kind leaked in 401 text; no JWT secret strength check; login skips Argon2 for an unknown email, a timing difference `[INFERENCE: not measured]` (register already reveals existing emails) | `backend/src/handlers/auth.rs:15-46,78-84,101-103`, `backend/src/auth.rs` | M4b; fixed in `b20986e` |
| D18 | Medium | Config and startup: `RUST_LOG` in `.env` ignored (tracing initialises before dotenv), and the `.env.example` filter targets `insightboard` rather than the crate target `insightboard_backend`; missing required vars panic; `SENTRY_DSN`, `AWS_REGION` documented but unread; `COINMARKETCAP_API_KEY` read but unused | `backend/src/main.rs:38-48`, `backend/src/config.rs:24-28`, `.env.example` | M4a; fixed in `2780fc0` |
| D19 | Medium | Redis and health: every cache operation opens a new connection with no connect timeout; Redis failure is fatal at startup yet fail-open per request; `/healthz` checks no dependency | `backend/src/cache.rs:23-25`, `backend/src/handlers/health.rs` | M4b; fixed in `b20986e` (`/readyz`; Redis optional at startup, fail-open) |
| D20 | Medium | Provider robustness: a null CoinGecko value fails the whole crypto response with 502 `[INFERENCE]`; status checks run sequentially (up to N×5 s) against the frontend's 10 s axios timeout | `backend/src/widgets/crypto.rs:70-71`, `status.rs:45`, `frontend/src/api/client.ts:16` | M4b; fixed in `b20986e` |
| D21 | Medium | sqlx and redis built without TLS features; the production-style URLs in `.env.example:19,28` would fail | `backend/Cargo.toml`, `.env.example` | M6a; fixed in `e84ed43` (drill 4) |
| D22 | Medium | Editor grid: registry `minSize` ignored; an extra PUT on every editor exit after any edit (`autoSaveTimer` never reset) issued after `grid.destroy`; saves not serialised; Vue and GridStack both remove the DOM node; legacy `addWidget(existingEl)`; editor loads the whole list instead of `GET /dashboards/:id` | `DashboardEditorView.vue:38-53,149,171-177,189,254-262`, `frontend/src/composables/widgetRegistry.ts` | M4c; fixed in `5caff27` (drill 14) |
| D23 | Medium | Session handling: 401 forces a hard reload and loses the redirect; token expiry never checked; `fetchCurrentUser` never called; logout does not reset the dashboard store | `frontend/src/api/client.ts:85-103`, `frontend/src/stores/auth.ts`, `frontend/src/App.vue:10-13` | M4c; fixed in `5caff27` |
| D24 | Medium | Styling drift: daisyUI 5.0.0-beta.9 with v4-era class names (`card-bordered`, `input-bordered`, `form-control`, `label-text`, `btn loading`); invalid `hsl(var(--b2))`; unlayered overrides of daisyUI classes; inert `tailwind.config.js` and forms plugin; unused `@tailwindcss/vite`; nonstandard `@theme extends`; build warns `Unknown at rule: @property` | `frontend/src/App.vue:173`, `frontend/src/style.css:5,120-300` | M4c; fixed in `5caff27`; restyled in `13e9018` (light and dark themes) |
| D25 | Medium | Accessibility: login/register labels lack `for`; logout is `<a @click>` without href; modal lacks dialog semantics, focus trap, Escape; icon-only buttons without `aria-label`; menus lack Escape/`aria-expanded`; layout editing is mouse-only; no live regions for saving/loading | `frontend/src/views/LoginView.vue:28,54`, `App.vue:89`, `WidgetConfigModal.vue`, `BaseWidget.vue` | M4c; fixed in `5caff27` and `13e9018` (dialog initial focus); keyboard layout editing in the backlog (owner decision) |
| D26 | Low | Dead code and unused dependencies. Rust: `Cache::delete`/`exists`, `AppError::Redis`/`Forbidden`, `UserCtx.email`, `coinmarketcap_api_key`; crates `config`, `tokio-test`. TS: `types/index.ts` request types with wrong keys and `ApiResponse`; store members `validateWidgetConfig`, `getDashboardById`, `setCurrentDashboard`, `reset`; BaseWidget `refreshInterval` prop; `src/assets/vue.svg`. npm: `chart.js` (never imported), `@tailwindcss/forms`, `@tailwindcss/vite` | `backend/src/{cache,error,auth,config}.rs`, `backend/Cargo.toml`, `frontend/src/types/index.ts`, `frontend/src/stores/{widgets,dashboard}.ts`, `frontend/src/components/BaseWidget.vue`, `frontend/package.json` | M4a; fixed in `2780fc0` |
| D27 | Low | Display and copy: "Just now" badge never ticks; status "Errors" count excludes status 0; crypto refreshes every 60 s against a 300 s cache; login hint "Test with any email and password" is false; hard-coded 70 px offsets and a double scroll in the editor | `BaseWidget.vue:32-43`, `StatusWidget.vue:296`, `LoginView.vue:169`, `DashboardEditorView.vue:266,269` | M4c; fixed in `5caff27` |
| D28 | Low | Naming and settings: `name` vs `settings.title` duplication; `mapServerToClient` drops `name`; no rename UI; `settings.theme` and `refresh_interval` unused | `frontend/src/api/dashboard.ts:14-24` | M4c (owner decision 15); fixed in `5caff27` (`name` only, rename UI; `settings_json` unused by the client, per-dashboard refresh settings in the backlog) |
| D29 | Low | Schema, cache, compose hygiene: redundant `idx_users_email`; raw, unbounded cache keys shared across users; dashboard list unpaged with no per-user cap; obsolete compose `version: '3.8'`; fixed `container_name` values prevent a second checkout's stack | `migrations/001_create_users.sql`, `backend/src/widgets/*.rs`, `backend/src/handlers/dashboard.rs`, `docker-compose.yml` | M4a (compose; fixed in `2780fc0`) / M4b (index, keys, list bounds; fixed in `b20986e`) |
| D30 | High | Crypto and news widgets never work: the providers reject requests without a `User-Agent`, and `reqwest::get` sends none. CoinGecko answers 403 and NewsAPI 400 `userAgentMissing`, so every crypto and news request returns 502 "External service error" (observed 2026-10-10 in B5; manual requests with a `User-Agent` got 200 and `ok`). OpenWeather accepts requests without one (B5 weather 200 through the same `reqwest::get`) | `backend/src/widgets/crypto.rs:58`, `news.rs:55` | M4b; fixed in `b20986e` (live crypto and news 200 on 2026-10-10) |
| D31 | Medium | Layout at small widths and overflow (observed 2026-10-10 in B7 and B12): at 390 px the dashboard list overflows horizontally (412 px) and the open user menu overlaps the header with the email clipped; the editor keeps 12 grid columns, so a default 4-wide widget is about 120 px wide with overlapping header controls and clipped content, and the title wraps under the toolbar. At desktop width the Add Widget menu shows four types and Status Monitor only after scrolling inside the menu, and the status widget's summary row is clipped at its default 4×3 size | `frontend/src/views/DashboardListView.vue`, `DashboardEditorView.vue`, `App.vue`, `frontend/src/components/widgets/StatusWidget.vue` | M4c (decision 15); fixed in `5caff27` and `13e9018` (1-column static grid below 768 px; no overflow at 390 px) |

## 13. Acceptance drills

Precondition **B0**: D3 exposure resolved (owner-approved loopback-only binds for compose ports and the backend) or
another owner-approved mitigation; `.env` copied from `.env.example` with a locally generated `JWT_SECRET` (never
committed or pasted); no provider keys unless the owner supplies free-tier keys for B5.

| # | Drill | Expected (as built at `f15c6a3`) |
| --- | --- | --- |
| B1 | `docker compose up -d`; both containers healthy; start backend; migrations apply; `GET /healthz` | 200 `{status:"ok",service:"insightboard-backend",version}` |
| B2 | Auth: register, login, `/me`; duplicate register; password of 7 bytes; wrong password; `/me` without token | 201, 200, 200; 400 "Email already registered"; 400; 401 "Invalid credentials"; 401 |
| B3 | Dashboard CRUD and ownership: create, list, get, update, delete; second user GETs the first user's dashboard | 201/200/200/200/204; 404 "Dashboard not found" |
| B4 | Persistence: create data, `docker compose restart` (never `down -v`), restart backend | Users and dashboards still present |
| B5 | Providers at human scale: weather and news without keys; one github username; crypto default ids; status with `https://example.com` only | 400 with the two "not configured" messages; github events array; crypto array; status entry with HTTP status and time |
| B6 | Cache: repeat a B5 request; inspect key and TTL with `docker exec insightboard-redis redis-cli TTL <key>` (since M4a: `docker compose exec redis redis-cli TTL <key>`) | Second response served from cache; key present with TTL ≤ the documented value |
| B7 | Browser editor journey: create dashboard; Add Widget opens the config modal first; Cancel creates nothing; Save creates the widget; drag and resize; reload | Layout and config persist after reload |
| B8 | Configure an existing widget | Form opens empty — exposes D9 (record as observed defect, not a pass) |
| B9 | Remove a widget; wait for auto-save; reload. Delete a dashboard from the list | Removal persists; dashboard gone |
| B10 | Replace `auth_token` in localStorage with an invalid value and navigate | 401 → storage cleared, hard redirect to `/login` (D23 notes the lost redirect) |
| B11 | Automated gate: the commands in [testing](../development/testing.md) (frontend `npm ci`, `npm run test`, `npm run build`, `npm audit --omit=dev`; backend build, test, fmt, clippy) | Builds and tests pass; fmt, clippy, audit fail until M4a (D13, D14) |
| B12 | 390 px width and keyboard-only pass over login, list, editor | Observations recorded (D25), not pass/fail |

Provider drills follow the approved external targets in the [roadmap](../product/roadmap.md). Results go to the
[baseline verification record](../development/baseline-verification.md) (main thread).

## 14. Out of scope

Fixing any defect in section 12 (M4 and later); new widgets or providers; production builds, images, or release
material (M5); cloud adapters, TLS features, or IaC (M6); any deployment (M7); the unscheduled backlog in the
[project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) section 5.

## 15. Package layout

Backend crate `insightboard-backend` (`backend/`, edition 2021):

| Module | Responsibility |
| --- | --- |
| `src/main.rs` | Tracing, configuration, database and Redis start-up, migrations, router, CORS/compression/trace layers, bind, graceful shutdown |
| `src/config.rs` | Environment variables into `Config` |
| `src/db.rs` | Postgres pool and embedded migrations |
| `src/cache.rs` | Redis JSON get and set with TTL |
| `src/auth.rs` | Argon2 hashing, JWT issue and validation, `UserCtx` extractor |
| `src/error.rs` | `AppError` and its HTTP mapping |
| `src/handlers/{health,auth,dashboard}.rs` | Health, auth, and dashboard handlers |
| `src/models/{user,dashboard}.rs` | Rows and request/response types |
| `src/widgets/{github,weather,news,crypto,status}.rs` | Provider proxies with caching |
| `migrations/` | SQL migrations 001 and 002 |

Frontend (`frontend/`, Vue 3 + Vite + TypeScript):

| Path | Responsibility |
| --- | --- |
| `src/main.ts`, `src/App.vue` | Bootstrap (Pinia, router, widget registry) and app shell |
| `src/router/index.ts` | Routes and guards |
| `src/api/` | Axios client and interceptors; auth, dashboard, and widget calls |
| `src/stores/{auth,dashboard,widgets}.ts` | Pinia stores |
| `src/composables/widgetRegistry.ts` | Widget type registration |
| `src/views/` | Login, Register, DashboardList, DashboardEditor |
| `src/components/` | `BaseWidget`, `WidgetConfigModal`, `widgets/*Widget.vue` |
| `src/contracts/apiContract.ts` | Hand mirror of the v1 contract |
| `src/style.css` | Tailwind 4 and daisyUI 5 beta entry and global overrides |
| `src/**/__tests__/` | Vitest component tests |
