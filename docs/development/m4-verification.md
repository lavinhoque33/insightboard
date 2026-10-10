# M4 verification record

Evidence for the [M4 contract](../architecture/m4-quality-gate-contract.md) acceptance drills (section 14). Main thread
only. One section per slice; M4 is accepted only after M4a, M4b, and M4c are recorded and the owner accepts.

## M4a — reproducibility, gates, and exposure

- Status: **locally verified** on this workstation, 2026-10-10 (UTC). Not pushed; no CI.
- Commits: `d4bdca9` (format only: `cargo fmt`, Prettier), `2780fc0` (M4a implementation). Session
  [2026-10-10-03-m4a](sessions/2026-10-10-03-m4a.md).
- Environment: Fedora 44 (kernel 7.2.8), rustc 1.99.0 (rustup, pinned by `rust-toolchain.toml`), Node 20.19.0
  (project `.tools/`), Docker Compose 5.5.1, Postgres 16 and Redis 7 from `docker-compose.yml`.
- Scope: D4, D12, D13, D14, D18, D26, D29 (compose); decisions 1, 2, 3, 7, 14, 16, 17. D3 (decision 4) landed earlier
  in `354e06c` and is rechecked by drill 2.
- Implemented by `implement-light` (reformat) and `implement-medium` (rest); adversarial review by `verify`;
  drills 1–4 run by the main thread.

### Drills

| # | Drill | Expected | Observed (main thread) | Result |
| --- | --- | --- | --- | --- |
| 1 | Fresh clone → `make setup verify` | Green with host Rust or the container fallback | `git clone` of the local repository at `2780fc0` into `/tmp`, project Node linked in, `make setup verify` exit 0: `.env` created with mode 600; `cargo fmt --check`, clippy `-D warnings`, 14 backend tests, build; ESLint and Prettier, Vitest 2 files and 3 tests, `vue-tsc` and `vite build`. Clone removed afterwards | Pass (host Rust) |
| 2 | Compose and backend up; `ss -ltn` | Postgres, Redis, backend, Vite on `127.0.0.1` only | After `make db-up` recreated the containers under project names (`insightboard-postgres-1`, `insightboard-redis-1`; volumes `insightboard_postgres_data` and `insightboard_redis_data` kept, 3 users and 2 dashboards still present), the M4a backend and Vite started: listeners `127.0.0.1:5432`, `127.0.0.1:6379`, `127.0.0.1:8080`, `127.0.0.1:5173`; none on `0.0.0.0`, `[::]`, or `*` | Pass |
| 3 | Preflight and simple request from an origin not in `CORS_ORIGINS` | No CORS grant; the Vite dev origin is granted | Default `CORS_ORIGINS`. `OPTIONS /api/dashboards` and `GET /healthz` with `Origin: http://evil.example`: no `access-control-allow-origin`. Same requests from `http://127.0.0.1:5173` and `http://localhost:5173`: the exact origin echoed; methods `GET,POST,PUT,DELETE`; headers `authorization,content-type`; no credentials header | Pass |
| 4 | `npm audit --omit=dev` | 0 high or critical, or owner-accepted residuals | `npm audit --omit=dev`: 0 vulnerabilities; full `npm audit`: 0 (baseline: 8 and 17) | Pass; no residuals |

### Other checks

| Check | Who | Outcome |
| --- | --- | --- |
| `make verify` on the working tree | `verify` (reported) | Exit 0; same counts as drill 1 |
| Container fallback (`make backend-check` without host Rust, `rust:1.99.0-bookworm`) | `implement-medium` (reported) | Exit 0 in a copy of the tree; no root-owned files left. Not rerun by the main thread or `verify` (host Rust present) |
| `cargo metadata --locked`; `.gitignore` still ignores `.env`, `target/`, `node_modules/`, `dist/`, `.tools/` | `verify` (reported) | Lockfile consistent; ignores intact. Backend checks now run with `--locked` (main-thread change) |
| Startup errors without `.env` or with blank `DATABASE_URL`/`REDIS_URL`/`JWT_SECRET`, bad `APP_HOST`, `APP_PORT`, `CORS_ORIGINS` | `verify` (reported) | Exit 1 with a message naming the variable; no panic |
| `RUST_LOG` from `.env` | `verify` (reported) | Honoured (`trace` and `off` observed); the process environment still wins |
| `make setup` with an existing `.env`; generated `.env` | `implement-medium`, `verify` (reported) | Existing file untouched (byte-identical); new file mode 600, placeholders replaced, nothing printed |
| `make db-reset` without `CONFIRM=yes` | `implement-medium` (reported) | Refuses (exit 2); never run with `CONFIRM=yes` |
| Backend restart with retained data and rotated credentials | Main thread | Backend connected to Postgres and Redis after the recreate and after the credential rotation below |

### Incident: local credentials exposed in an agent transcript

During review, a `verify` probe ran `docker compose config` with the repository `.env` in scope and printed part of
the rendered file, so the local `POSTGRES_PASSWORD` and `REDIS_PASSWORD` reached that agent's session transcript.
They are loopback-only development credentials; no provider key or `JWT_SECRET` is referenced by the compose file. The
main thread rotated both the same day without printing them: `ALTER ROLE` with the new Postgres password, `.env`
rewritten (mode 600) with the new values and matching URLs, containers recreated, backend restarted and connected.
Temporary files from the probe were deleted.

### Carried forward (not M4a defects)

- `frontend/src/api/client.ts` formats 422 `details` with `join('; ')`, which renders `[object Object]` for the
  contract's `[{field, message}]` shape; the `errors`/`validation` fallbacks are unreachable (D16, M4b). **Resolved
  in M4b** (`b20986e`).
- `WidgetConfigModal.vue` casts form values to `string` before calling a field validator; safe only while validators
  sit on text fields (M4c).
- Two checkouts still share loopback ports 5432 and 6379 (documented in local setup).
- The workstation `.env` keeps the old `RUST_LOG` filter (`insightboard=debug`); `.env.example` now uses
  `insightboard_backend`. Cosmetic; owner's file.

## M4b — outbound safety, API correctness, and contract

- Status: **locally verified** on this workstation, 2026-10-10 (UTC). Not pushed.
- Commit: `b20986e`. Session [2026-10-10-04-m4b](sessions/2026-10-10-04-m4b.md).
- Environment: as M4a; backend debug build from `b20986e` on `127.0.0.1:8080` with migration 003 applied to the
  workstation database (no rows needed backfill: 0 null or `[]` layouts, 0 null settings).
- Scope: D1, D2, D5, D6, D7, D8, D15, D16, D17, D19, D20, D29 (index, cache keys, list bound), D30; decisions 5, 6,
  8, 9, 10, 11; the M4a carry-forward on 422 formatting.
- Implemented by `implement-heavy` (backend, migration, contract pair, feature docs) and `implement-medium`
  (frontend); `ajv-formats`, proposed by the frontend agent, was rejected by the main thread (not approved in ADR
  0002) and replaced by local format validators. Verified by `verify` and `verify-browser`; drills by the main thread.

### Drills

| # | Drill | Expected | Observed | Result |
| --- | --- | --- | --- | --- |
| 5 | Status widget with loopback, metadata, private, non-http, over the cap | Generic error class each; nothing leaves the machine | Live (main thread): `http://127.0.0.1:8080/healthz`, `http://169.254.169.254/…`, `http://10.0.0.1/`, `http://[::1]/`, `http://0x7f000001/`, `http://100.64.0.1/` → `blocked_address`; `ftp://example.com/` → `invalid_url`; 11 URLs → 422 `urls: must list at most 10 URLs`; `https://example.com` → 200. Tests: `tests::status::rejected_targets_get_generic_classes_and_no_request_leaves` asserts a listening fixture is never contacted | Pass |
| 6 | GitHub username with `/` or `..` | Rejected; no other path reached | Live: `a/b`, `..`, `octo%2F..%2Frepos` → 422 `username`. Test with a zero-call mock proves no upstream request | Pass |
| 7 | Each provider against a slow local mock | Configured timeout, generic error | Live: second backend on `127.0.0.1:8081` with all four base URLs at a 12 s loopback mock: github, weather, news, crypto → 502 `External service error` at 8.0 s; status via `STATUS_ALLOWED_FIXTURES=127.0.0.1:18081` → `timeout` at 4.0 s. Logs show provider name and timeout only, no URL or key | Pass |
| 8 | Exceed auth and data rate limits | 429 `{error}` | Live: login burst → 429 `Too many requests; try again later`; 35 data requests → 422 then 429 after the burst. Browser: visible 429 banners on register and login | Pass |
| 9 | Create without JSON fields; update with blank name | Defaults stored; blank rejected | Live: stored `{"widgets": []}` and `{}` (read back from Postgres); blank name → 422 `name`; bad layout → 422 `layout_json.widgets.0.w` | Pass |
| 10 | Duplicate email, including two concurrent registrations | 409 | Live: case-different duplicate → 409; concurrent pair → 201 and 409, one row stored | Pass |
| 11 | Contract tests with a broken fixture | Fail; restored passes | Main thread changed `ErrorResponse.error` to `integer` in `contracts/api-contract.v1.json`: backend `tests::dashboards::crud_responses_match_the_contract` failed; all 7 frontend contract tests failed. Restored: `make verify` exit 0. Both suites also keep permanent broken-fixture tests | Pass |

### Other checks

| Check | Who | Outcome |
| --- | --- | --- |
| `make verify` | Main thread (after drill 11 restore); `verify` (reported) | Exit 0: 87 backend tests, 28 frontend tests (6 files), fmt, clippy, ESLint, Prettier, build |
| No external network in tests | `implement-heavy`, `verify` (reported) | `strace -f -e trace=connect` over `cargo test`: every connect to loopback |
| `/healthz`, `/readyz`; 400, 415, 401 bodies | Main thread | 200; `{"status":"ready"}`; `Malformed JSON body` 400; content-type 415; `{"error":"Unauthorized"}` 401 |
| D30 smoke (approved targets, human scale: 1 CoinGecko, 1 NewsAPI, 1 example.com request) | Main thread | Crypto 200 with prices; news 200 with articles; status 200 |
| SSRF review (IPv4-mapped/compat IPv6, numeric host forms, redirects, proxy, second-client search) | `verify` (reported) | No bypass found |
| Key leakage review (21 log sites) | `verify` (reported) | No URL, body, or key logged; NewsAPI key in header; GitHub token only to the GitHub base URL |
| Migration 003 on a throwaway database with colliding emails | `verify` (reported) | Collisions left unchanged, backfill applied; probe database dropped |
| Browser: register/login 422 field errors with ARIA, 409, 429, 401; status classes; crypto nulls; 502/503 widget states (stubbed); keyboard order; 1440 and 390 px | `verify-browser` (reported) | All pass; no uncaught errors |

### Main-thread fixes during review

- `ErrorResponse` description in both contract files now lists 405 and 413 (`verify` F1).
- `local-setup.md` notes that all clients behind the Vite proxy share one auth rate-limit bucket (`verify` F2).

### Carried forward (not M4b defects)

- Auth inputs are disabled while a request runs, so keyboard focus drops to `<body>` after each submit
  (`verify-browser` O2; D25, M4c).
- Auth forms show the backend's 429 text, widgets the friendlier `errorMessage.ts` text (consistent with section 7;
  wording choice for M4c).
- Users whose email normalisation would collide keep their stored casing (migration 003 design).
- `dns_error` status class is not covered by an automated test (would need external DNS).
