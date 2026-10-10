# ADR 0001: Foundation stack

- Status: **proposed** — retroactive record, written 2026-10-10, of decisions made 2025-10-28 to 2026-03-03 under
  the legacy spec; owner confirmation pending with the baseline review.
- Milestone: M0–M3
- Sources: the legacy
  [condensed spec](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/.github/instructions/Insightboard_Condensed_spec.instructions.md)
  ("Tech Stack", "Authentication", "Architectural decisions" under Phase 6), the legacy
  [README](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/README.md), the commit history to `f15c6a3`,
  `backend/Cargo.toml`, `frontend/package.json` and `frontend/package-lock.json`, and the
  [M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md).

## Context

InsightBoard was specified on 2025-10-28 (`c4b31da`) as a portfolio project: a Rust backend and a Vue 3 frontend
aggregating third-party data into a customisable dashboard, "AWS-ready" for a later free-tier serverless
deployment. The legacy spec fixed the main technologies and left versions and layout to the implementing agent,
which worked phase by phase under a stop-and-confirm protocol (plan section 1.3) and recorded decisions as checklist
prose rather than decision records.

This ADR records the stack as it exists at `f15c6a3`, so later ADRs (starting with ADR 0002 for M4) have a baseline
to amend. It does not re-decide anything; where the 2026-10-10 audits showed a consequence the legacy docs did not
state, it is listed under Consequences with its defect ID.

## Decisions

### Backend

| Area | Choice at `f15c6a3` | Notes |
| --- | --- | --- |
| Language and edition | Rust, edition 2021 | No `rust-toolchain.toml` and no `rust-version`; built with rustc 1.91.0 on a previous workstation (reported) and 1.99.0 on this workstation (observed 2026-10-10) |
| HTTP | axum 0.7 (`macros`), axum-extra 0.9 (`typed-header`), tower-http 0.5 (cors, trace, compression-gzip), tokio 1 (`full`) | Resolved without a lockfile on 2026-10-10: axum 0.7.9, tower-http 0.5.2, tokio 1.53.2 |
| Persistence | Postgres via sqlx 0.8 with **runtime** `query_as` strings; migrations embedded with `sqlx::migrate!` and applied at startup | No compile-time query checks and no offline query data; compiling needs no database. Resolved: sqlx 0.8.6 |
| Cache | Redis via `redis` 0.27, JSON values with a TTL per provider | Resolved: redis 0.27.6 |
| Auth | Local accounts; JWT HS256 via `jsonwebtoken` 9 with a 7-day lifetime; passwords hashed with `argon2` 0.5 defaults | Cognito was named as a later production option in the legacy spec and never built. Resolved: jsonwebtoken 9.3.1 |
| Outbound calls | `reqwest` 0.12 (`json`) from the backend: the browser never calls a provider directly; responses cached in Redis with a TTL per provider | Resolved: reqwest 0.12.28 |
| Errors and logging | `thiserror`/`anyhow`; `tracing` with `tracing-subscriber` (`env-filter`) | — |

### Local infrastructure

`docker-compose.yml` runs `postgres:16-alpine` and `redis:7-alpine` (AOF on) with named volumes and healthchecks.
The backend and frontend run natively. There is no Dockerfile, no CI, and no infrastructure code.

### Frontend

| Area | Choice at `f15c6a3` |
| --- | --- |
| Framework | Vue 3 with the Composition API (`<script setup>`), TypeScript, Vite 7 |
| State and routing | Pinia, Vue Router (history mode) |
| HTTP | axios with request and response interceptors |
| Layout editor | GridStack 10 (resolved 10.3.1) |
| Styling | Tailwind CSS 4 through the PostCSS plugin (`@tailwindcss/postcss`), daisyUI 5.0.0-beta.9 through `@plugin "daisyui"` |
| Tests | Vitest 4 with `@vue/test-utils` and jsdom |
| Package manager and runtime | npm (`package-lock.json`, lockfileVersion 3); Node 20.19.0 in `.nvmrc` (added in `14d7f3e`, 2025-11-01) |

### Cross-cutting

1. **Backend proxy for providers.** GitHub, OpenWeather, NewsAPI, CoinGecko, and arbitrary status URLs are fetched
   by the backend, which holds any provider keys and caches results in Redis.
2. **Session in localStorage.** The JWT and the user record are stored in localStorage and sent as a Bearer header.
3. **Dashboards as JSONB.** Layout and settings are JSONB columns on `dashboards` (`layout_json`, `settings_json`),
   not normalised tables.
4. **Machine-readable payload contract.** `contracts/api-contract.v1.json` (JSON Schema 2020-12), mirrored by hand
   in `frontend/src/contracts/apiContract.ts`, added on 2026-03-03 (`f15c6a3`).
5. **AWS serverless as a later target.** Lambda behind API Gateway with hosted Postgres and Redis, S3 and
   CloudFront for the frontend, and EventBridge for background refresh were the stated target; none of it exists.
   In this workflow it becomes M6 (readiness, no deployment) and M7 (deferred experiment).
6. **No new dependencies without approval.** Carried over from the legacy agent guide ("avoid introducing
   alternative frameworks, databases, or state systems unless explicitly requested"); every new dependency now
   needs owner approval.

## Alternatives considered

Only alternatives recorded in the legacy documents and history are listed; nothing else is reconstructed.

- **Vuex** — the legacy spec chose Pinia "over Vuex" for official status and TypeScript support.
- **fetch** — the legacy spec chose axios "over Fetch" for interceptors that centralise auth and error handling.
- **Cognito JWT validation** — listed in the legacy spec as the production option beside local JWT; not built,
  now an unscheduled backlog item unless M6 takes it.
- **Hosted Postgres and Redis (Neon or Supabase, Upstash)** — the legacy spec's deployment targets; local compose is
  used instead, and `.env.example` keeps commented production-style URLs (see D21).
- **daisyUI 4** — the legacy spec recorded 4.12.10; `162cf60` (2025-11-04) moved to the 5.0.0 beta for
  compatibility with Tailwind 4.
- **Tailwind Vite plugin** — added in `9a7740c` (2025-11-01); the build uses the PostCSS plugin and the Vite plugin
  remains installed but unused (D24).
- **pnpm** — named in the legacy spec and README; the repository has used npm with `package-lock.json`.
- **CoinMarketCap** — `.env.example` keeps a key "if you switch"; the crypto widget uses CoinGecko without a key.

## Consequences

- The proxy keeps keys off the client and gives one place for caching, and it also concentrates outbound risk in the
  backend: the status widget is an open fetcher (D1), user input reaches provider URLs unencoded (D2, D7), and only
  the status client has a timeout (D5).
- Runtime queries keep compilation independent of a database; the legacy claim of compile-time query checking was
  false.
- Gitignoring `Cargo.lock`, the floating daisyUI beta range, and missing toolchain pins make builds
  non-reproducible (D12); the daisyUI beta with v4-era class names is a styling liability (D24).
- localStorage keeps the session simple and exposes the token to any script on the page; the client never checks
  expiry (D23). Changing storage is owner call 8 in M4.
- JSONB keeps the editor flexible and leaves shape enforcement to code that does not exist yet (D8, D15).
- The development defaults (permissive CORS, `0.0.0.0` bind, compose ports on all interfaces with default
  credentials) were never tightened (D3, D4).
- sqlx and redis lack TLS features, so the hosted target cannot connect as configured (D21, M6).
- Unused declared dependencies (`governor`, `config`, `mockito`, `tokio-test`, `chart.js`, `@tailwindcss/forms`,
  `@tailwindcss/vite`) suggest intended features that were not built (D6, D11, D26).

## Follow-ups

The [M4 contract](../architecture/m4-quality-gate-contract.md) decisions amend this record: 1 (lockfile and
toolchain), 3 (gates), 4 (loopback defaults), 5 (outbound policy), 6 (rate limiting), 8 (auth and session storage),
9 (dashboard JSON), 11 (contract tests), 14 (dependencies, daisyUI stable). ADR 0002 records the owner's choices
when the M4 contract is accepted. Cloud seams (Lambda adapter, IaC tool, TLS features) are decided in M6.
