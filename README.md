# InsightBoard

InsightBoard is a personal dashboard builder. You register, create dashboards, and arrange widgets on a
drag-and-drop grid; each widget shows data from a public provider, fetched and cached by the backend. It is a
Rust/Axum API backed by Postgres and Redis, with a Vue 3 and TypeScript client. Everything runs locally; nothing
is deployed.

- **$0 of spending.** Free provider tiers only, with keys you supply; no cloud resources.
- **Local only.** Intended to bind to loopback; the current code does not yet (D3, below).

**Status:** M0–M3 are **implemented but unverified**. They were delivered between 2025-10-28 and 2026-03-03
under a legacy phase protocol and **reported** verified on a previous workstation, with no evidence beyond
prose; the 2026-03-03 audit (`f15c6a3`) still found and fixed defects in phases marked complete. On this
workstation, the 2026-10-10 automated baseline **observed** passing builds and tests, and failing
`cargo fmt --check`, `cargo clippy -D warnings`, and `npm audit`. User journeys have not been re-exercised here;
owner acceptance follows baseline drills B1–B12 in the
[baseline verification record](docs/development/baseline-verification.md). M4 — Quality gate and hardening is
next: **planned**, with its [contract](docs/architecture/m4-quality-gate-contract.md) **proposed** and owner
decisions pending. See the [roadmap](docs/product/roadmap.md).

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

The [defect catalogue](docs/architecture/m0-m3-baseline-contract.md) lists D1–D29 with references and the slice
that fixes each. The ones to know first:

- **D3 — network exposure.** The backend binds `0.0.0.0:8080`; compose publishes Postgres (`postgres/postgres`)
  and Redis (no password) on all interfaces. On this workstation the host firewall opens ports 1025–65535, so
  these are reachable from the local network while running `[INFERENCE: not probed]`. Read the
  [local setup warning](docs/development/local-setup.md) before starting anything.
- **D1 — status widget SSRF.** Any signed-in user can make the backend request any URL, including loopback,
  private, and cloud-metadata addresses, and read back status and error text.
- **D9 — widget configuration never pre-fills.** Editing an existing widget opens an empty form; saving replaces
  its configuration.
- **D11 — thin automated tests.** Provider base URLs are hard-coded, so nothing can be mocked yet.
- **D14 — vulnerable npm dependencies.** `npm audit --omit=dev` **observed** 8 findings (7 high, 1 moderate) on
  2026-10-10.

## Run it locally

Prerequisites: Docker with Compose, Node 20.19.0 (`.nvmrc`), and Rust stable. Steps, configuration, and
troubleshooting are in [local setup](docs/development/local-setup.md). In short, after resolving D3 exposure as
described there:

```sh
cp .env.example .env                    # then replace JWT_SECRET with a locally generated value
docker compose up -d                    # Postgres 16 and Redis 7
cd backend && cargo run                 # terminal 1: applies migrations, serves port 8080
cd frontend && npm ci && npm run dev    # terminal 2, from the repo root: http://localhost:5173
```

Provider keys are optional; without them Weather and News show their "not configured" errors.

## Checks

Commands, current results, and known failing gates (D13, D14) are in [testing](docs/development/testing.md).
Automated tests never call live providers.

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
```

There is no `Makefile`, Dockerfile, or CI workflow yet. M4 proposes the `Makefile` and CI; container images
belong to M5.

License: MIT intended; no LICENSE file committed. Repository:
[github.com/lavinhoque33/insightboard](https://github.com/lavinhoque33/insightboard) (public). Contact:
`ihoque33@yahoo.com`.
