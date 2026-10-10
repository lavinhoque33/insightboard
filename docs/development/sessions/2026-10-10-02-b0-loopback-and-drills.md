# 2026-10-10-02-b0-loopback-and-drills

- Request / acceptance: the owner (2026-10-10) approved M4 decision 4 as the B0 mitigation for D3, chose a
  user-level rustup install for running the backend natively, and chose B5 with owner-supplied free-tier keys.
  Acceptance: Postgres, Redis, the backend, and Vite listen on loopback only (`ss -ltn`), Postgres and Redis
  credentials come from `.env`, docs match; Rust stable with rustfmt and clippy on the host; drills B1–B12 run and
  recorded in the [baseline verification record](../baseline-verification.md).
- Milestone / constraints: M0–M3 baseline drills; the decision 4 change is the first M4a item, landed early as B0.
  Scope is decision 4 only: CORS (decision 7, D4), compose cleanup (`version`, `container_name`, D29), and the
  remaining D18 configuration items stay in M4a proper. Loopback only; provider smoke at human scale within the
  approved targets; keys are entered by the owner into `.env`, never pasted or logged.
- Uncommitted changes at start: none (`main` = `origin/main` at `2855f4b`).
- Workspace observed at start: existing compose volumes `insightboard_postgres_data` and
  `insightboard_redis_data` created 2026-10-01 (Postgres 16 data directory, about 46 MB, initialised with
  `postgres/postgres`). No containers running; nothing listening on 5432, 6379, 8080, or 5173. No `~/.cargo` or
  `~/.rustup`; `openssl-devel` not installed.

## Report summary: 2026-10-10-02/decision-4

- Report source / role: `implement-medium` (LoopbackDecision4), implementation.
- Reported state: submitted.
- Ownership / changed paths: `backend/src/config.rs`, `backend/src/main.rs`, `docker-compose.yml`, `.env.example`,
  `frontend/vite.config.ts`, `docs/development/local-setup.md`, `docs/architecture/system-overview.md`,
  `README.md`.
- Findings / work performed: `APP_HOST` parsed as `IpAddr`, default `127.0.0.1`, startup error naming the variable
  for a non-IP value; bind `SocketAddr::new(app_host, app_port)`; two unit tests. Compose ports `127.0.0.1:`;
  `POSTGRES_USER`, `POSTGRES_PASSWORD`, `REDIS_PASSWORD` required with `${VAR:?}`; Redis `--requirepass`,
  `REDISCLI_AUTH` so `docker exec insightboard-redis redis-cli` still works; healthchecks fail on bad auth.
  `.env.example` with empty secrets and `<…>` URL placeholders. Vite `server.host` and proxy target `127.0.0.1`.
  Docs updated; setup uses a non-default user `insightboard_app` and documents in-place password rotation for an
  existing volume. Noted: the Redis password is visible in `docker inspect` (interpolated into `command`).
- Checks / evidence (reported): `docker compose --env-file <tmp> config` with throwaway values showed
  `host_ip: 127.0.0.1` and failed naming each missing secret; container fallback `cargo build`, `cargo test`
  (7 passed); clippy only the 2 known D13 errors; `npm run build` and `npm run test` (3) pass. Did not start the
  stack (barred by the brief).
- Remaining work / blockers: none reported; proposed wording for `CODING_PRACTICES.md` and `.omp/AGENTS.md` D3 lines.
- Proposed next action: main thread validates at runtime.

## Main-thread review: 2026-10-10-02/decision-4

- Decision: **accepted** after one main-thread fix. The new startup log line was not rustfmt-clean (`cargo fmt
  --check` went from 67 to 68 hunks); the main thread changed it to log the computed `addr`, restoring 67 hunks,
  and amended the unpublished commit. Committed locally as `354e06c`.
- Evidence reviewed / checks executed (main thread, observed): full diff inspected; `.env` created with
  generated secrets (Python `secrets`, never printed, mode 600, gitignored); `POSTGRES_USER=postgres` kept for the
  existing volume and the role's password rotated in place via `psql` over the container socket from stdin. The
  volume held an empty `insightboard` database. Runtime B0 checks in the baseline record: `ss -ltn` loopback only
  for all four services; LAN-address connections refused; old Postgres password rejected from the host; Redis
  `NOAUTH` without a password. Native `cargo build`, `cargo test` (7), fmt (67 hunks, unchanged from before),
  clippy (2 known errors).
- Gaps / follow-up owner: none for decision 4. CORS (D4) and compose hygiene (D29) remain M4a.

## Report summary: 2026-10-10-02/toolchain

- Report source / role: main thread.
- Work performed: rustup installed at user level (`rustup-init -y --profile minimal --component rustfmt,clippy`;
  stable rustc 1.99.0; shell profiles source `~/.cargo/env`). The first native build failed in `openssl-sys`
  (no OpenSSL headers); `openssl-devel` installed with `sudo dnf` as the prerequisite for the owner-chosen native
  build, which also updated `openssl` and `openssl-libs` 3.5.8 → 3.5.9. The build then passed.

## Report summary: 2026-10-10-02/drills

- Report source / role: main thread (curl-equivalent Python requests, the harness's headless Chromium limited to
  `127.0.0.1`).
- Results (details and evidence in the [baseline verification record](../baseline-verification.md)): B0, B1, B2,
  B3, B4, B6, B7, B9 pass; B8 observed D9; B10 pass with D23 observed; B11 as expected (D13, D14); B12
  observations (D25, D31). B5 without keys: weather and news 400 as expected, github and status 200, crypto 502.
- New defects, IDs assigned by the main thread in the
  [baseline contract](../../architecture/m0-m3-baseline-contract.md): **D30** (High) — CoinGecko returns 403 to
  requests without a `User-Agent` and `reqwest::get` sends none, so the crypto widget always fails; confirmed with
  one manual request with and one without a `User-Agent` (200 and 403); widened to news after the keyed run below.
  **D31** (Medium) — small-width layout and clipped menu and widget content.
- Upstream requests this session: GitHub 1, CoinGecko 4, example.com 2; nothing else external besides crates.io,
  the npm registry, rustup, and the Fedora mirrors.
- Incidental: another local client polled `/api/notifications/attention` on port 8080 (404) while the backend ran
  `[INFERENCE: another project's browser tab]`.

## Report summary: 2026-10-10-02/b5-keys

- Report source / role: main thread. The owner entered free-tier keys into `.env` (the owner chose "Later" when
  first asked, then reported the keys added). Presence checked by length only; values never printed.
- Work performed, about 05:57 UTC: backend restarted to read `.env`; one weather (`London`), one news
  (`technology`), and one github (`octocat`, now with the token) request.
- Results: news 502 (NewsAPI 400). One manual request without a `User-Agent` got `userAgentMissing` and one with a
  `User-Agent` got `ok` and 10 articles, so the key works and D30 covers news as well; D30 widened in the
  catalogue. Weather 502 (OpenWeather 401); one manual request with a `User-Agent` got "Invalid API key", so the
  key is rejected regardless of headers (new keys may need time to activate `[INFERENCE]`). GitHub 200, empty
  array.
- Recheck at the owner's request, about 06:01 UTC: `.env` had been edited at 06:00 UTC; backend restarted; the
  weather request and one manual request still got 401 "Invalid API key" (key format: 32 hex characters, as
  expected). OpenWeather's FAQ: keys activate up to 2 hours after registration.
- Second recheck at the owner's request, about 16:22 UTC: `.env` unchanged since 06:00 UTC (same key), no
  restart. Weather `London` 200 with the contract shape (`London`, `GB`, `light rain`); the repeat was a cache hit
  in 3 ms; `weather:London` TTL 595 of 600 s. The key had activated. The backend's `reqwest::get` sends no
  `User-Agent`, so OpenWeather is not affected by D30; the D30 entry was corrected.
- Upstream totals for the session: GitHub 2, CoinGecko 4, NewsAPI 3, OpenWeather 5, example.com 2.

## Owner decision: M0–M3 acceptance

- Presented about 16:25 UTC with every drill outcome and the defects observed (B5 D30, B8 D9, B10 D23, B11
  D13/D14, B12 D25/D31) and the plan's rule that acceptance records the product as built, defects included.
  Options offered: accept; accept but fix D30 first; not yet.
- Owner choice: **accept M0–M3** (2026-10-10). Recorded as **accepted** in the roadmap, plan, README, index,
  baseline record, and checkpoint; the baseline contract, the as-built description of M0–M3, is marked
  **accepted** with them. D30 stays scheduled in M4b.

## Owner decisions: M4 preparation

- Presented decisions 1–3 and 5–17 from the [M4 contract](../../architecture/m4-quality-gate-contract.md) with ★
  recommendations and main-thread recommendations for the owner calls.
- Owner choices (2026-10-10): every ★ decision approved as written; 9 and 14 approved as written; 8 — token 24
  hours, localStorage; 10 — 422 with details, 503 for a missing key; 15 — title from `name`, auto-save with visible
  error and retry, keyboard layout editing to the backlog.
- Main-thread preparation values (owner may change before the slice): status widget no redirects, 10 URLs, 5 at a
  time, 2 s connect and 4 s total timeout, per-user cache; provider client 3 s and 8 s with a `User-Agent`; rate
  limits auth 10/min per IP (burst 5) and data 60/min per user (burst 30); 100 dashboards per user, 409 beyond;
  422 body `{error, details:[{field, message}]}`; `/readyz` for Postgres and Redis.
- Recorded: contract marked **accepted**; [ADR 0002](../../decisions/ADR-0002-m4-quality-gate.md) written
  (decisions, approved dependencies, preparation values); roadmap decision table and agents per slice; plan
  (status, open decisions, backlog); README, index, testing, `.omp/AGENTS.md`. ADR 0001 left **proposed**: it asks
  for owner confirmation, which was not asked explicitly.

## Close

- Milestone and slice: M0–M3 baseline drills; M4a decision 4 landed early.
- What changed: `354e06c` (decision 4 code and feature docs); tracking and governance docs (roadmap, baseline
  record, baseline contract D3 status and D30/D31, M4 contract slices, plan, `CODING_PRACTICES.md`,
  `.omp/AGENTS.md`, testing, index, terminology, system overview gaps) in the session commit.
- What was verified, and in which environment: on this workstation, 2026-10-10 at `354e06c`: B0–B4, B6–B12 and the
  keyless half of B5 about 05:00–05:20 UTC; the keyed news and github requests about 05:57 UTC; keyed weather
  about 16:22 UTC. All drills B0–B12 are recorded.
- What remains unverified or blocked: nothing in the drills. Not pushed.
- Decisions and permissions still needed: owner confirmation of ADR 0001 (retroactive stack record); any push.
- Runtime state left: compose running (loopback); backend running from `backend/target/debug` on
  `127.0.0.1:8080` with the keys loaded; Vite stopped.
- Next recommended action: M4a — brief `implement-medium` (contract sections 1, 2, 11 audit part, 13; M4a drills),
  then `verify`, and record evidence in `docs/development/m4-verification.md`.
