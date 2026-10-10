# InsightBoard

InsightBoard is a personal dashboard builder. You register, create dashboards, and arrange widgets on a
drag-and-drop grid; each widget shows data from a public provider, fetched and cached by the backend. It is a
Rust/Axum API backed by Postgres and Redis, with a Vue 3 and TypeScript client. Everything runs locally; nothing
is deployed.

- **$0 of spending.** Free provider tiers only, with keys you supply; no cloud resources.
- **Local only.** Binds loopback by default (M4 decision 4, below).

**Status:** M0–M3 are **accepted** (2026-10-10), defects included. They were delivered between 2025-10-28 and
2026-03-03 under a legacy phase protocol and **reported** verified on a previous workstation, with no evidence
beyond prose. On this workstation, the 2026-10-10 baseline drills B0–B12 **observed** passing builds, tests, and
user journeys, failing `cargo fmt --check`, `cargo clippy -D warnings`, and `npm audit`, and defects including
D9, D23, D25, D30 (crypto and news widgets always fail), and D31 (small-width layout); see the
[baseline verification record](docs/development/baseline-verification.md). M4 — Quality gate and hardening is
**in progress** under its [accepted contract](docs/architecture/m4-quality-gate-contract.md)
([ADR 0002](docs/decisions/ADR-0002-m4-quality-gate.md)): slices M4a and M4b are **locally verified** (2026-10-10;
[M4 verification record](docs/development/m4-verification.md)); M4c is next. See the [roadmap](docs/product/roadmap.md).

## What works (as built at `f15c6a3`)

- **Accounts:** register and log in with email and password (Argon2 hashing, 7-day HS256 JWT kept in
  `localStorage`); `GET /api/me` returns the current user.
- **Dashboards:** owner-scoped create, list, open, update, and delete. Another user's dashboard is a 404.
- **Widgets:** five types, each proxied and cached in Redis by the backend:

  | Widget | Input | Provider | Cache TTL |
  | --- | --- | --- | --- |
  | GitHub | `username` | GitHub public events (optional token) | 300 s |
  | Weather | `city` | OpenWeather current weather (key required) | 600 s |
  | News | `topic` | NewsAPI everything (key required) | 900 s |
  | Crypto | `ids` (CoinGecko ids) | CoinGecko markets, no key | 300 s |
  | Status | `urls` (comma list) | a GET to each URL, 5 s each | 120 s |

  Without a key, Weather and News return 400 "OpenWeather API key not configured" and "NewsAPI key not
  configured".
- **Editor:** GridStack grid (12 columns). Adding a widget opens its configuration first and creates nothing
  until you save; drag and resize; layout and configuration auto-save 2 s after the last change. Widgets
  refresh on their own interval.
- **Contract:** payload shapes are described in [`contracts/api-contract.v1.json`](contracts/api-contract.v1.json)
  (JSON Schema) and mirrored by hand in
  [`frontend/src/contracts/apiContract.ts`](frontend/src/contracts/apiContract.ts). Nothing enforces either yet.
- **Tests:** 5 backend unit tests and 3 frontend component tests; no integration, end-to-end, or CI.

The full as-built reference, including every route and status code, is the
[M0–M3 baseline contract](docs/architecture/m0-m3-baseline-contract.md).

## Limits

The [defect catalogue](docs/architecture/m0-m3-baseline-contract.md) lists D1–D31 with references and the slice
that fixes each. The ones to know first:

- **D3 — network exposure (mitigated).** Per M4 decision 4 (owner-approved 2026-10-10) the backend binds
  `127.0.0.1` by default (`APP_HOST`), compose publishes Postgres and Redis on `127.0.0.1` only, and their
  credentials come from `.env`; **locally verified** on this workstation 2026-10-10 (precondition B0 in the
  [baseline verification record](docs/development/baseline-verification.md)). The host firewall on this
  workstation opens ports 1025–65535; the services listen on `127.0.0.1` only.
  Read [local setup](docs/development/local-setup.md) before starting anything.
- **D1 — status widget SSRF (fixed in M4b).** The status widget now accepts only `http`/`https` URLs that resolve
  to public addresses, follows no redirects, and checks at most 10 URLs with short timeouts.
- **D9 — widget configuration never pre-fills.** Editing an existing widget opens an empty form; saving replaces
  its configuration (M4c).
- **D11 — automated tests (partly fixed).** Backend tests use local mocks, Postgres, and Redis; frontend journeys,
  Playwright, and CI come in M4c.
- **D14 — vulnerable npm dependencies (fixed in M4a).** `npm audit` **observed** 8 production findings on
  2026-10-10; after the M4a bumps both `npm audit --omit=dev` and the full audit report 0.

## Run it locally

Prerequisites: Docker with Compose, Node 20.19.0 (`.nvmrc`), `make`, `openssl`, and Rust (pinned by
`rust-toolchain.toml`; `make backend-check` falls back to a container without it). Steps, configuration, and
troubleshooting are in [local setup](docs/development/local-setup.md). In short:

```sh
make setup        # npm ci; creates .env with generated secrets if it is absent
make db-up        # Postgres 16 and Redis 7 on 127.0.0.1
make backend-dev  # terminal 1: applies migrations, serves 127.0.0.1:8080
make web-dev      # terminal 2: http://127.0.0.1:5173
```

Provider keys are optional; without them Weather and News return 503 "not configured" errors.

## Checks

`make verify` runs every gate (fmt, clippy, cargo test, ESLint, Prettier, Vitest, type-check and build). Backend
tests use the compose Postgres and Redis, so run `make db-up` first. Commands and current results are in
[testing](docs/development/testing.md). Automated tests never call live providers.

## Where to start

1. The [documentation index](docs/index.md).
2. [.omp/AGENTS.md](.omp/AGENTS.md) and [CODING_PRACTICES.md](CODING_PRACTICES.md): how work is done here.
3. The [session handoff](docs/development/session-handoff.md) for the current checkpoint, then the
   [roadmap](docs/product/roadmap.md) and the [project plan](INSIGHTBOARD_PROJECT_PLAN.md).

## Layout

```text
backend/            Rust crate insightboard-backend: src/{handlers,models,widgets}, auth, cache, config, db, error
backend/migrations/ SQL migrations, embedded and applied at startup
frontend/           Vue 3 + TypeScript + Vite client: src/{api,stores,router,views,components,composables,contracts}
contracts/          api-contract.v1.json (JSON Schema, payload shapes)
docs/               product, architecture, decisions, development, guides (non-authoritative walkthroughs)
.omp/AGENTS.md      agent operating guide
docker-compose.yml  Postgres and Redis only
Makefile            setup, database, dev servers, checks (make help)
```

There is no Dockerfile or CI workflow yet: CI belongs to M4c and container images to M5.

License: [MIT](LICENSE). Repository:
[github.com/lavinhoque33/insightboard](https://github.com/lavinhoque33/insightboard) (public). Contact:
`ihoque33@yahoo.com`.
