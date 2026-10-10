# Baseline verification (M0–M3)

Maintained by the **main thread** after reviewing and rerunning relevant work. Subagents may read this record and
return check results in their reports; only the main thread edits it. Reported findings remain unaccepted until
main-thread review; owner acceptance of M0–M3 is recorded in the [roadmap](../product/roadmap.md). The expected
behaviour and the drills are defined in the [M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md).

## Status

M0 — Foundation, M1 — Accounts and dashboards, M2 — Widgets and editor, and M3 — UI polish and contract alignment
are **implemented but unverified**:

- Delivered 2025-10-28 to 2026-03-03 under the legacy stop-and-confirm phase protocol, which treated phases as
  milestones and marked them "COMPLETED & VERIFIED".
- **Reported** verified on a previous workstation (`/home/gui-13/PersProj/...`) between 2025-10-31 and 2026-03-03;
  no evidence artifacts beyond prose.
- The 2026-03-03 audit (`f15c6a3`) found and fixed defects inside phases already marked completed.
- The 2026-10-10 automated baseline on this workstation passes builds and tests; `cargo fmt --check`,
  `cargo clippy -D warnings`, and `npm audit` fail (section "Automated baseline").
- Drills B0–B12 ran on this workstation on 2026-10-10 at `354e06c` (section "Drills"). Owner acceptance of M0–M3
  follows the owner's review of this record and the recorded defects.

## Environment

This workstation, observed 2026-10-10:

- Fedora 44, x86_64. Clone `/home/axeleratorx/PersProject/InsightBoard`; drills ran at `354e06c` (`f15c6a3` plus
  the onboarding docs and the decision 4 change). Origin `https://github.com/lavinhoque33/insightboard.git` is
  public.
- Docker 29.8.2, Compose v5.5.1; the user is in the `docker` group, so `sudo` is not needed.
- Node 20.19.0 from the gitignored `.tools/node-v20.19.0-linux-x64/bin` (matches `.nvmrc`); host Node 24.19.0 also
  satisfies Vite 7 (`^20.19 || >=22.12`).
- Rust: user-level rustup installed 2026-10-10 at the owner's request (stable, rustc 1.99.0, rustfmt, clippy;
  `~/.cargo/env` sourced from the shell profiles). The native build needed `openssl-devel`, installed with dnf
  (it also updated `openssl` and `openssl-libs` 3.5.8 → 3.5.9). The automated baseline below used the
  `rust:1-bookworm` container instead.
- Host firewall: firewalld zone `FedoraWorkstation` on `wlp1s0` opens `1025-65535/tcp` and `/udp`. Before the D3
  fix, compose published `5432` and `6379` on all interfaces and the backend bound `0.0.0.0:8080`. Since `354e06c`
  every listener is on `127.0.0.1` (B0).
- Compose volumes `insightboard_postgres_data` and `insightboard_redis_data` existed from 2026-10-01; the Postgres
  data directory was initialised with `postgres/postgres` and held an empty `insightboard` database (no tables).
  `.env` keeps `POSTGRES_USER=postgres`, and the role's password was rotated in place to the generated
  `POSTGRES_PASSWORD` (no volume reset).

## Legacy evidence (reported)

Claims from the legacy documents, written by agents on a previous workstation (`/home/gui-13/PersProj/...`). They
are **reported** evidence only: none was rerun here, none produced artifacts beyond prose, and several were later
contradicted. The documents are removed from the working tree in this session and cited at `f15c6a3`.

| Date (UTC) | Source | Reported claim | Notes |
| --- | --- | --- | --- |
| 2025-10-31 (completed 20:08) | [`Documentation/BACKEND_VERIFICATION.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/Documentation/BACKEND_VERIFICATION.md) | Postgres 16 and Redis 7 healthy in compose; migrations applied; register, login, protected routes, and dashboard CRUD exercised with curl; all five widget endpoints answered; cache improvement for the status widget (`status:https://example.com`, TTL 120 s) from 266 ms to 11 ms total time; built with rustc 1.91.0 after three build fixes; status "ALL TESTS PASSING" | No automated tests existed on that date. The latency figures are **reported**, not re-measured. The report also states "checked concurrently" (the code is sequential, D20) and compile-time SQL checks (queries are runtime). It recorded `settings_json: null` on create without flagging it (D8). |
| 2025-10-31 to 2025-11-03 | [condensed spec](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/.github/instructions/Insightboard_Condensed_spec.instructions.md) status section | Phases 1–7 "COMPLETED & VERIFIED", phases 8–9 merged into 7, phase 10 "COMPLETED" (2025-11-03 21:30); dev server verified on port 5173; "KNOWN ISSUES: None"; "all 5 widgets fully functional"; daisyUI 4.12.10 | Contradicted by the 2026-03-03 audit (build broken, `/me` route mismatch, contract drift, no tests) and by the lockfile (daisyUI 5.0.0-beta.9). |
| 2026-03-02 21:20 | [`project_progress.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/project_progress.md) | Phases 1–10 `COMPLETED`; phase 11 (Testing and CI/CD) and phase 12 (Deployment Preparation) `NOT STARTED` | Phase 11 was stale a day later: `f15c6a3` added 5 backend and 3 frontend tests. |
| 2026-03-03 | [`testing_audit.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/testing_audit.md) | `cargo test` 5 passed; `cargo check` pass; `npm run build` pass; `npm audit` 0 vulnerabilities; runtime smoke of `/healthz`, auth, dashboard CRUD, status shape, and 400 for missing weather and news keys; compose smoke via `sudo` with cache key `status:https://example.com`; nine findings resolved; "Open bugs: 0"; contract "actively used by implementation" | `npm audit` observed 2026-10-10 reports 8 production findings (D14) `[INFERENCE: the advisories may postdate 2026-03-03]`. The audit did not assess outbound safety, timeouts, or exposure (D1–D7). The contract is used only as TypeScript types (D15). |
| 2026-03-03 16:59–17:19 | [`manual_audit.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/manual_audit.md) | Add Widget now opens configuration; the widget menu Configure works; header contrast fixed; draft-on-add (a widget exists only after Save); `npm run test` 2 then 3 tests passed; `npm run build` passed; ad hoc Playwright smoke with widget count 2 → 2 → 2 → 3 | Times converted from CST. No Playwright script was committed. The configure flow opens with an empty form (D9), which this audit did not detect. |

Run instructions from the same period
([`manual run instructions.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/manual%20run%20instructions.md))
are condensed into [local setup](local-setup.md) and [testing](testing.md); they named `httpstat.us`, which is not an
approved target.

## Automated baseline — 2026-10-10 (observed)

Run by the main thread on this workstation at about 03:36 UTC at `f15c6a3`. Frontend in `frontend/` with Node
20.19.0 from `.tools/`: `npm ci`, `npm run test`, `npm run build`, `npm audit --omit=dev`, `npm audit`. Backend in
`rust:1-bookworm` (rustc 1.99.0) with a read-only mount and no lockfile:

```
docker run --rm -v "$PWD/backend":/src:ro rust:1-bookworm bash -c 'rustup component add rustfmt clippy >/dev/null 2>&1; cp -r /src /work && cd /work && cargo build && cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings'
```

| Check | Result |
| --- | --- |
| `npm ci` | 246 packages; deprecation warning for `glob@10.5.0` |
| `npm run test` (Vitest 4.0.18) | 2 files, 3 tests, all pass |
| `npm run build` (`vue-tsc -b && vite build`) | Pass; CSS warning `Unknown at rule: @property` (daisyUI radialprogress); index JS 195.60 kB (68.29 kB gzip), editor chunk 93.47 kB, CSS 80.99 kB |
| `npm audit --omit=dev` | 8 findings: 7 high, 1 moderate (axios direct: NO_PROXY SSRF, prototype pollution; vue/@vue/server-renderer XSS; postcss; source-map-js; nanoid; form-data; follow-redirects moderate) — D14 |
| `npm audit` (all) | 17 findings: 1 critical, 13 high, 3 moderate (includes Vite dev-server advisories) — D14 |
| `cargo build` | Pass; 2 dead-code warnings (`UserCtx.email` `src/auth.rs:34`, `AppError::Forbidden` `src/error.rs:30`) |
| `cargo test` | 5 passed: `config::tests::normalize_optional_strips_blank_values`, `config::tests::normalize_optional_trims_and_keeps_non_empty`, `auth::tests::jwt_roundtrip`, `auth::tests::password_hash_rejects_wrong_password`, `auth::tests::password_hash_roundtrip` |
| `cargo fmt --check` | Fail: 67 diff hunks across all 15 source files — D13 |
| `cargo clippy --all-targets -- -D warnings` | Fail: only the 2 dead-code warnings — D13 |

Because there is no lockfile, the backend resolved its dependencies fresh (axum 0.7.9, sqlx 0.8.6, redis 0.27.6,
reqwest 0.12.28, jsonwebtoken 9.3.1, tokio 1.53.2, tower-http 0.5.2); a later run may resolve differently (D12).

Not run this session: compose, backend runtime, migrations, any HTTP request, browser journeys, provider calls.
Those are drills B1–B12.

## Drills

Defined in the [baseline contract](../architecture/m0-m3-baseline-contract.md) section 13. Provider drills stay
within the approved external targets in the [roadmap](../product/roadmap.md).

Run by the main thread on 2026-10-10 between about 05:10 and 05:19 UTC at `354e06c`. The backend ran natively
from `backend/` (debug build), Vite from `frontend/` (Node 20.19.0), and browser drills in the harness's headless
Chromium restricted to `127.0.0.1`. Test accounts used `@example.test` addresses and remain in the local database.

| # | Drill | Result | Evidence |
| --- | --- | --- | --- |
| B0 | Precondition: D3 resolved by owner-approved loopback-only binds; `.env` from `.env.example` with a locally generated `JWT_SECRET`; no provider keys unless the owner supplies free-tier keys | **Pass** | Decision 4 approved 2026-10-10, landed in `354e06c`. `.env` created with generated `JWT_SECRET`, `POSTGRES_PASSWORD`, `REDIS_PASSWORD` (mode 600, gitignored, never printed). `ss -ltn`: `127.0.0.1:5173`, `:5432`, `:6379`, `:8080` only. Connections to the `wlp1s0` address on all four ports refused (from the workstation itself; no other host probed). Postgres from the host: old password `postgres` rejected, new password accepted. Redis without a password: `NOAUTH`. |
| B1 | Compose up, containers healthy, backend starts, migrations apply, `GET /healthz` | **Pass** | Both containers healthy, published as `127.0.0.1:5432->5432` and `127.0.0.1:6379->6379`. Backend logged `Listening on 127.0.0.1:8080`; `_sqlx_migrations` holds 1 `create users` and 2 `create dashboards`, both successful. `/healthz` 200 `{"service":"insightboard-backend","status":"ok","version":"0.1.0"}`; also 401 for `/api/me` through the Vite proxy. |
| B2 | Auth: register, login, `/me`, duplicate, 7-byte password, wrong password, no token | **Pass** | 201 `{token,user:{id,email,created_at}}`; 200 same shape; 200 `{id,email,created_at}`; 400 "Email already registered"; 400 "Password must be at least 8 characters"; 401 "Invalid credentials"; 401 `{"error":"Unauthorized"}`. Login took about 1 s (debug build; Argon2 on async workers, D6). |
| B3 | Dashboard CRUD and ownership | **Pass** | Create 201, list 200 (1), get 200, update 200 (`name` changed), delete 204, get after delete 404. A second user got 404 "Dashboard not found" for GET, PUT, and DELETE on the first user's dashboard and an empty list. |
| B4 | Persistence across `docker compose restart` and a backend restart | **Pass** | A dashboard with one widget was created; `docker compose restart` (both healthy again) and backend restart; login 200, the dashboard and its `layout_json` returned unchanged; a token issued before the restarts still valid. |
| B5 | Providers at human scale (weather and news without keys, one github username, crypto defaults, status with `https://example.com` only); with the owner's free-tier keys, one weather and one news request | **Defect observed: D30** (crypto and news always 502); every other part as expected | Keyless, about 05:13 UTC: weather 400 "OpenWeather API key not configured" and news 400 "NewsAPI key not configured" (no upstream call). GitHub `octocat`: 200, empty array (no recent public events). Status: 200 `[{url:"https://example.com",status:200,response_time_ms:47,error:null}]`. Crypto defaults: 502 "External service error"; the backend log shows CoinGecko 403; one manual request each with and without a `User-Agent` got 200 and 403 (D30). With keys, about 05:57 UTC after a backend restart: news `technology` 502, log "NewsAPI returned status: 400"; one manual request without a `User-Agent` got `code: userAgentMissing`, one with a `User-Agent` got `status: ok` and 10 articles, so the key works and the cause is D30. GitHub `octocat` with the token: 200, empty array. Weather `London` 502 at 05:57 and 06:01 UTC, OpenWeather 401 "Invalid API key" also on one manual request each time (the new key was not yet active; OpenWeather's FAQ allows up to 2 hours). At about 16:22 UTC, same key, no restart: weather 200 `{city:"London",country:"GB",temperature,feels_like,humidity,description:"light rain",wind_speed,icon:"10d"}`, matching the contract; the repeat was a cache hit in 3 ms and `weather:London` had TTL 595 of 600 s. Keys were never printed. Upstream requests: GitHub 2, CoinGecko 4, NewsAPI 3, OpenWeather 5, example.com 2. |
| B6 | Cache hit and key TTL in Redis | **Pass** | Repeats answered in 2–5 ms with identical bodies and `Cache hit` log lines for status and github. `redis-cli --scan`: `github:octocat` (TTL 264 of 300 s), `status:https://example.com` (TTL 85 of 120 s). The failed crypto response was not cached (TTL −2). |
| B7 | Browser editor journey with draft-on-add, drag, resize, reload | **Pass** (desktop 1280 px) | Registered in the UI; New Dashboard opened the editor. Add Widget → Status Monitor opened the configuration modal with no grid item; Cancel left 0 items and the server layout `{"widgets":[]}`. Save with `https://example.com` added 1 item, auto-saved within 3.5 s (`x0 y0 w4 h3`). Mouse drag and SE resize gave `x4 y0 w6 h5`, auto-saved; after reload the same position, size, and config, widget showing 200 and 47 ms. No page errors. Observed: Status Monitor is the fifth menu item and is visible only after scrolling inside the menu; at 4×3 the widget's summary row is clipped (D31). |
| B8 | Configure an existing widget (expected to expose D9) | **Defect observed: D9** | Widget options → Configure on the widget saved with `https://example.com` opened the modal with an empty textarea. Cancelled without saving. |
| B9 | Remove a widget and delete a dashboard | **Pass** | Remove (no confirmation) left 0 items; after auto-save and reload still 0, server layout `{"widgets":[]}`. Delete on the list raised a native confirm ("Are you sure you want to delete this dashboard? This action cannot be undone."); after accepting, the empty state showed and `GET /api/dashboards` returned 0 after reload. |
| B10 | Invalid `auth_token` in localStorage | **Pass with D23 observed** | `auth_token` set to an invalid value, then `/dashboards`: landed on `/login` with `auth_token` and `user` cleared. The URL had no `redirect=` (D23). The login page shows "Test with any email and password. Development mode enabled!" (false hint, D27). |
| B11 | Automated gate rerun | **As expected**: builds and tests pass; fmt, clippy, audit fail (D13, D14) | At `354e06c`, natively: `npm ci` 246 packages; `npm run test` 2 files, 3 tests pass; `npm run build` pass with the `@property` warning, bundle sizes unchanged from the baseline; `npm audit --omit=dev` 8 (7 high, 1 moderate); `npm audit` 17 (1 critical, 13 high, 3 moderate). `cargo build` pass with the 2 dead-code warnings; `cargo test` 7 passed; `cargo fmt --check` 67 hunks across 16 files (same as before the change; the earlier "15 files" was a miscount); clippy fails on the same 2 dead-code errors. |
| B12 | 390 px width and keyboard-only pass (observations, not pass/fail) | **Observed** | Login: no horizontal overflow; logical Tab order (brand, Login, Sign Up, email, password, Sign In, Create one now); visible focus; inputs are named by placeholder, not by their labels (D25); signing in by keyboard and Enter works. List: page 412 px wide (horizontal scroll, D31); the user menu is a focusable `div` that opens on focus and overlaps the header with the email clipped (D31); Logout is an `<a>` without `href`, unreachable by keyboard (D25). Editor: title wraps under the toolbar; Add Widget opens a menu whose five types are reachable by Tab, and the five buttons stay in the Tab order while the menu is closed; Enter on a type opens the modal, but focus stays behind it, there is no `role="dialog"`/`aria-modal`, Escape does not close it, and Tab reaches background buttons first (D25); Save by keyboard works. The grid keeps 12 columns, so the widget is about 120 px wide with overlapping controls and clipped content (D31). Drag and resize have no keyboard path (D25). |

## Defects found

The canonical list is section 12 of the [baseline contract](../architecture/m0-m3-baseline-contract.md); this
record does not duplicate it.

- **Observed 2026-10-10 on this workstation:** D13 (fmt and clippy failures), D14 (npm audit findings), D12 (fresh
  dependency resolution without a lockfile), D24 in part (the `@property` build warning), D6 in part (about 1 s
  logins in a debug build), D9 (B8), D23 (B10, lost redirect), D25 (B12), D27 in part (login hint), and the
  firewall half of D3. D3 itself is fixed (B0).
- **New, observed in the drills, IDs assigned by the main thread:** D30 (crypto and news always 502: CoinGecko and
  NewsAPI reject requests without a `User-Agent`; OpenWeather accepts them) and D31 (layout at 390 px and clipped
  menu and widget content).
- **From code reading (2026-10-10 audits), not yet observed at runtime:** the remaining IDs. B5 and B6 touched the
  paths of D1, D5, D7, D19, and D29 but stayed within approved targets and did not try to exploit them.
- **Incidental:** while the backend ran on `127.0.0.1:8080`, some other local client sent
  `GET /api/notifications/attention?limit=200` (404), probably another project's browser tab expecting a
  different service on port 8080 `[INFERENCE]`. It does not affect the results.
- Defects found during the drills that are not in the catalogue are recorded here by description and proposed to
  the main thread for an ID; IDs are never assigned in this file.

## Limits of this record

Local verification only, on one workstation. No CI exists. The legacy rows are reported claims from another
workstation and do not count as verification here. Package counts, sizes, and test counts above are from the
2026-10-10 run and will change; the automated gate after M4a is `make verify`, not these numbers.
