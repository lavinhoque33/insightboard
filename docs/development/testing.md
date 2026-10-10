# Testing and verification

## Evidence before completion

Tests demonstrate behaviour and failure paths; a clean build, formatting, and a green `/healthz` are narrower
evidence than a working journey. Report exact commands and outcomes, where they ran, and when; never imply an
unexecuted check passed.

| Label | Meaning |
| --- | --- |
| **observed** | Run in the current session, in a named environment, with the outcome recorded |
| **reported** | Claimed by someone else (a subagent, or the legacy docs on a previous workstation) and not rerun |
| `[INFERENCE]` | Deduced from code or library behaviour, not exercised |

Milestone status uses **planned**, **in progress**, **implemented but unverified**, **locally verified**,
**accepted**, **blocked**, **deferred**; only reviewed evidence moves a status. M0–M3 evidence lives in the
[baseline verification record](baseline-verification.md).

**Automated tests never call live providers** (GitHub, OpenWeather, NewsAPI, CoinGecko) or arbitrary status
URLs, locally or in CI. Provider behaviour is tested against local `mockito` servers and silent loopback listeners
through the configurable base URLs (`GITHUB_API_BASE_URL` and the others, M4b); status-policy tests use IP
literals, `localhost`, and allowlisted loopback fixtures only, and blocked targets are rejected before any
connection.

## Commands

The root `Makefile` is the gate (M4 decision 2). Run from the repository root; `make doctor` shows which tools it
found. **`make backend-check` needs the compose Postgres and Redis running** (`make db-up`): it stops with a clear
message otherwise, and never starts, stops, or recreates containers. `cargo test` reads `DATABASE_URL` and
`REDIS_URL` from the root `.env` (exported to the test process without printing). Database tests use
`#[sqlx::test]`, which creates and drops an isolated database per test (the compose user owns the server, so it may
create databases); Redis tests write only under unique `ib-test:<uuid>:` key prefixes with short TTLs and never
flush. The Vitest suites mock the API and need neither.

| Target | What it runs |
| --- | --- |
| `make backend-check` | Checks Postgres and Redis are running, then in `backend/`: `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` (environment from `.env`), `cargo build --locked` (`--locked` fails instead of silently changing the committed `Cargo.lock`) |
| `make web-check` | In `frontend/`: `npm run lint` (ESLint with `--max-warnings 0`, then `prettier --check .`), `npm test` (`vitest run`), `npm run build` (`vue-tsc -b && vite build`) |
| `make verify` | `backend-check` then `web-check`; works from a fresh clone after `make setup` and `make db-up` |
| `make format` | `cargo fmt` and `npm run format` (Prettier write); needs host Rust |
| `make setup` | `npm ci` (and a generated `.env` if absent); run once per clone before `web-check` |

The chain stops at the first failing step. Frontend commands use the project Node in
`.tools/node-v20.19.0-linux-x64/bin` when present, else host Node 20.19+ (or 22.12+).

**Single runs and extras** (from `frontend/` with Node on `PATH`, or from `backend/`):

| Command | What it runs |
| --- | --- |
| `npm ci` | Clean install from `package-lock.json` |
| `npm run lint` | ESLint (flat config, `eslint-plugin-vue`, `typescript-eslint`, `eslint-config-prettier`; `@typescript-eslint/no-explicit-any` is an error) and the Prettier check |
| `npm run test` | `vitest run` (jsdom, setup `src/test/setup.ts`) |
| `npm run build` | `vue-tsc -b && vite build`: type-checks app, spec files, and `vite.config.ts`/`vitest.config.ts` (`tsconfig.node.json`), then bundles |
| `npm audit --omit=dev` | Advisories in production dependencies |
| `npx vitest run src/components/__tests__/BaseWidget.spec.ts` | One test file |
| `cargo test jwt_roundtrip` | One backend test by name filter (from `backend/`; export `.env` first for database and Redis tests: `set -a; . ../.env; set +a`) |

There are no `test:watch`, `test:coverage`, or `test:e2e` scripts; Playwright is not installed (M4c).

**Backend toolchain.** `rust-toolchain.toml` pins Rust 1.99.0 with `rustfmt` and `clippy`; `backend/Cargo.lock` is
committed, so builds use the locked dependency versions. With host rustup, `cargo` fetches the pinned toolchain on
first use inside the checkout.

**Backend, container fallback.** When `cargo` is not on `PATH`, `make backend-check` (and `make verify`) runs the
same four steps in a throwaway `rust:<version>-bookworm` container, where `<version>` is read from
`rust-toolchain.toml`. It mounts `backend/` read-only and builds in a copy inside the container, so nothing is
written to the checkout and no root-owned files appear. It runs with `--network host` and `--env-file .env` so the
tests reach the compose Postgres and Redis on `127.0.0.1` (Linux Docker; Docker Desktop's host networking differs and
is not supported here). It needs Docker and internet access, and downloads the locked crates on every run (no cache
volume). `make backend-dev` and `make format` need host Rust.

**No external network in `cargo test`.** Every provider base URL in the test config is a loopback mock or the
closed loopback port 9; the status widget only sees IP literals (blocked before connecting), `localhost`, and
allowlisted loopback fixtures; `jsonschema` is built without its HTTP/file `$ref` resolvers. **Observed** on this
workstation on 2026-10-10 by running the test binary under `strace -f -e trace=connect`: every `connect` went to
`127.0.0.1` (plus resolver address-selection probes to `::1` port 0 when resolving `localhost`).

## What exists

Backend (`cargo test`, one binary; unit tests sit beside the code, router tests in `backend/src/tests/`):

| Area | Location | Covers |
| --- | --- | --- |
| Config | `config::tests` | Loopback default; required variables name themselves; CORS parsing; `JWT_SECRET` < 32 bytes rejected; base URL, timeout, and fixture-allowlist parsing (loopback `ip:port` only) |
| Auth helpers | `auth::tests` | Argon2 round trip and rejection; 24 h JWT lifetime; every invalid token is the generic `Unauthorized` |
| Request validation | `models::user::tests`, `models::dashboard::tests`, `widgets::*::tests` | Email normalisation and validation; password length; dashboard defaults, dotted layout/settings fields, blank names; GitHub username rules; crypto id normalisation; free-text normalisation |
| Outbound policy | `widgets::net_policy::tests`, `widgets::status::tests`, `widgets::http::tests` | Every blocked address class (loopback, unspecified, private, CGNAT, link-local and metadata, ULA, multicast, broadcast/reserved, documentation, benchmarking, IPv4-mapped/compatible, NAT64, 6to4, IETF-protocol `2001::/23`); the policy resolver rejects `localhost`; scheme/credential/length checks; IP-literal blocking incl. the hex form `0x7f000001`; exact fixture allowlist; path segments percent-encoded |
| Cache | `cache::tests`, `tests::cache` | Key bounding (hash of long or special parts); unreachable Redis fails open within 3 s; compose Redis round trip with TTL; undecodable entries are misses; provider data shared under normalised keys (one upstream call for two users) |
| Rate limits | `ratelimit::tests`, `tests::rate_limit` | Burst then 429 per key; cleanup keeps recent keys; auth limited per client IP (other IPs unaffected); data limited per user after authentication (401 before 429) |
| Middleware | `app::tests` | CORS allowlist (preflight and simple requests); server timeout → JSON 504; unknown route 404 and wrong method 405 as JSON |
| Auth routes | `tests::auth` (`#[sqlx::test]`) | Register/login/me match the contract; email normalised; duplicate (case-insensitive) → 409; three concurrent duplicate registrations → one 201 and two 409; invalid input → 422 details; wrong password and unknown email → same generic 401; `/me` for a deleted user → 401 |
| Dashboard routes | `tests::dashboards` (`#[sqlx::test]`) | Create without JSON fields stores `{"widgets":[]}` and `{}`; CRUD responses match the contract; blank name and invalid shapes → 422 with dotted fields; foreign dashboards 404 for GET/PUT/DELETE and invisible in the list; unparsable id 404; 101st dashboard → 409; create for a deleted user → 401; migration 003 backfills legacy `null`/`[]` values and normalises emails |
| Providers | `tests::providers` (`mockito`) | Each widget's success output matches the contract; the shared `User-Agent` is sent; GitHub token as bearer; NewsAPI key in `X-Api-Key` and absent from the URL; city and topic query-encoded, path-like GitHub usernames rejected without an upstream call; crypto null numbers kept as `null`; upstream 500 and undecodable bodies → generic 502; slow upstream → 502 at the configured timeout for all four providers; missing keys → 503; missing/blank query values → 422 naming the parameter |
| Status widget | `tests::status` | Blocked and invalid targets get generic classes and never reach a listening fixture; 11 URLs → 422; allowlisted fixture reached; a 302 is reported, not followed; timeout and refused connection classified; checks run five at a time; results cached per user |
| Errors and health | `tests::errors`, `tests::health` | Malformed JSON 400, wrong content type 415, non-object/mistyped bodies 422 with details, all as `application/json`; database down → generic 500; `/healthz` without dependencies; `/readyz` 200 with both up, 503 with either down |
| Contract | `tests::contract` | Handler output above is validated with `jsonschema` (formats on) against `contracts/api-contract.v1.json`; the contract compiles; a deliberately broken fixture set (pre-M4b `[]` layout, `w: 0`, null settings, bad uuid, blank name, missing field, leaked error fields, raw library error text as a status class) must fail validation (drill 11) |

Totals: 87 backend tests (**observed** 2026-10-10, `make backend-check` on this workstation). Not covered by
automated tests: the `dns_error` status class (it would need a non-resolving name, which could query external DNS)
and live provider behaviour (manual, human-scale smoke only).

Frontend (Vitest): component tests under `frontend/src/**/__tests__/` and the M4b client and contract (ajv) tests;
run `npm test` for the current list and count. Store, guard, modal, editor, and Playwright suites and CI are
**pending** (M4c).

## Gate status

Before M4a (**observed** 2026-10-10 on this workstation at `354e06c`; details in the
[baseline verification record](baseline-verification.md)): `npm audit --omit=dev` found 8 issues (7 high, 1
moderate; full audit 17), `cargo fmt --check` failed on 67 hunks (D13, reformatted in `d4bdca9`), and clippy failed
on 2 dead-code errors (D13). M4a removed the dead code, added ESLint, bumped dependencies to clear the audits
(D14), and committed `Cargo.lock` (D12); the M4a results are recorded in `m4-verification.md` once the main thread
writes it. Still open in the gates: the daisyUI beta dependency (M4c, D24), CI (M4c), and the test suites in
"Target test architecture". The legacy 2026-03-03 audit **reported** `npm audit` with 0 findings on a previous
workstation.

## Manual drills

- Baseline drills B0–B12 are defined in the [M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md)
  and their results recorded in the [baseline verification record](baseline-verification.md). Later milestones
  define their drills in their own contract and record them in their own `*-verification.md`.
- **B0 is a precondition:** loopback binds and `.env` credentials ([local setup](local-setup.md)); met on this
  workstation since 2026-10-10 (`354e06c`).
- Drills record the command or UI steps, the expected result from the contract, the **observed** result, date
  and time in UTC, the environment, and the commit. A drill that exposes a known defect (for example B8 and D9)
  is recorded as an observed defect, not a pass.
- Provider drills stay at human scale. Status-widget drills use only `https://example.com` and loopback
  fixtures; never `httpstat.us`; never load-test anything.
- Browser drills cover desktop and 390 px widths with keyboard-only use; record observations, since automated
  checks are not an accessibility evaluation.
- Never paste secrets, tokens, or full logs into a record; reference them by name.

## Target test architecture (M4)

From the [M4 contract](../architecture/m4-quality-gate-contract.md), **accepted** 2026-10-10:

- **Contract tests (decision 11):** Rust `jsonschema` against handler output; ajv in Vitest against fixtures and
  `apiContract.ts`-typed payloads; a deliberately broken fixture must fail.
- **Backend (decision 12):** configurable provider base URLs with local mock servers; router tests through
  `tower::ServiceExt::oneshot`; `#[sqlx::test]` against compose Postgres; Redis cache tests.
- **Frontend (decision 12):** API client, stores, router guards, configuration modal, and editor tests; Playwright
  journeys against a provider fixture.
- **CI (decision 13):** GitHub Actions backend and web jobs with Postgres and Redis services, no secrets, no live
  providers. Enabling CI is an owner action.
- **Gates (decision 3), in place since M4a:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  ESLint, Prettier, `vue-tsc`, Vitest, and `cargo test`, run by `make verify`.

The gates above exist. Since M4b the backend contract, provider-mock, `#[sqlx::test]`, and Redis tests exist, and
the frontend has ajv contract tests; the frontend store/guard/modal/editor tests, Playwright, and CI items are
**pending** (M4c).

## What a verification record contains

- Scope: milestone or slice, commit, environment (machine, OS, toolchain versions), date in UTC.
- Every command run, with its outcome and the relevant counts; skipped checks with the reason.
- Each drill from the contract: expected, observed, evidence reference, and pass, fail, or observed defect.
- Defects found, referenced by D-ID or listed for the main thread to catalogue.
- Known limits and what was not exercised.
- Which results the main thread ran itself and which are **reported** by an agent.

## What not to test

Wording, wiring, forwarding, or incidental defaults. Do not pin log text or error prose beyond the contract. Do
not add tests for prose-only edits; check links and command consistency instead.
