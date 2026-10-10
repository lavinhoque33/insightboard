# Delivery roadmap

This document turns the [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) milestones (sections 12 and 13) into
observable delivery gates without replacing the plan's product direction. The main thread alone updates it after
reviewing evidence; subagents return findings for the main thread to record in the
[session handoff](../development/session-handoff.md). A completed session task does not complete a milestone.

Status words: **planned**, **in progress**, **implemented but unverified**, **locally verified**, **accepted**,
**blocked**, **deferred**; contracts and ADRs also use **proposed**. Environment qualifications are stated where
they matter.

## Milestone tracker

| Milestone | Outcome | Status |
| --- | --- | --- |
| M0 — Foundation | Toolchain, compose Postgres and Redis, Axum skeleton with config, migrations, and health; Vue shell | **Implemented but unverified**; see the qualification below and the [baseline verification record](../development/baseline-verification.md) |
| M1 — Accounts and dashboards | Register and login with JWT sessions; owner-scoped dashboard CRUD; frontend auth flow and dashboard list | **Implemented but unverified**; see the qualification below and the [baseline verification record](../development/baseline-verification.md) |
| M2 — Widgets and editor | Five provider widgets through a cached backend proxy; GridStack editor with persisted layout | **Implemented but unverified**; see the qualification below and the [baseline verification record](../development/baseline-verification.md) |
| M3 — UI polish and contract alignment | daisyUI styling, auth and editor fixes, v1 payload contract, draft-on-add editor | **Implemented but unverified**; see the qualification below and the [baseline verification record](../development/baseline-verification.md) |
| M4 — Quality gate and hardening | Reproducible builds, passing gates, loopback defaults, safe outbound calls, real test suites, CI | **Planned**; [contract](../architecture/m4-quality-gate-contract.md) **proposed**, owner decisions pending |
| M5 — Local release | A reproducible, documented, independently runnable local product | **Planned** |
| M6 — AWS readiness | Cloud adapters and an inspectable, synth-only experiment proposal, without deployment | **Planned** |
| M7 — AWS experiment | A validated and cleaned-up cloud execution path | **Deferred**; owner approval and a $0 path required |

## Qualification for M0–M3

- Delivered 2025-10-28 to 2026-03-03 under the legacy stop-and-confirm phase protocol, which treated phases as
  milestones and marked them "COMPLETED & VERIFIED".
- **Reported** verified on a previous workstation (`/home/gui-13/PersProj/...`) between 2025-10-31 and 2026-03-03;
  no evidence artifacts beyond prose.
- The 2026-03-03 audit (`f15c6a3`) found and fixed defects inside phases already marked completed.
- The 2026-10-10 automated baseline on this workstation passes builds and tests; `cargo fmt --check`,
  `cargo clippy -D warnings`, and `npm audit` fail (D13, D14).
- Drills B0–B4 and B6–B12 ran on this workstation on 2026-10-10 at `354e06c` after the decision 4 change (B0);
  they observed D9, D23, D25, and two new defects, D30 (crypto widget always 502) and D31 (small-width layout).
  B5's free-tier-key half is pending owner keys. Owner acceptance of M0–M3 follows that and a review of the
  [baseline verification record](../development/baseline-verification.md).

The binding as-built behaviour, the defect catalogue D1–D31, and drills B0–B12 are in the
[M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md) (**proposed**). Acceptance of M0–M3 records
the product as it is, defects included; fixing defects is M4 work.

## M0 — Foundation (implemented but unverified)

**Objective (plan 12/M0):** a local development baseline that starts, persists data, and builds.

**Delivered (as built; legacy phases 1 and 2, 2025-10-28 to 2025-10-31):**

- `docker-compose.yml`: Postgres 16 and Redis 7 (AOF) with named volumes and healthchecks.
- `backend/`: Rust crate `insightboard-backend` on axum 0.7 with environment configuration, a Postgres pool,
  embedded migrations applied at startup, a Redis client, `GET /healthz`, CORS, compression, request tracing, and
  graceful shutdown.
- `frontend/`: Vue 3 + Vite + TypeScript shell with Pinia, Vue Router, and an axios client.
- `.env.example`, `frontend/.env.example`, `.nvmrc` (Node 20.19.0), ignore rules.

**Status:** **implemented but unverified** (qualification above); owner acceptance pending drills B1–B12.

**Exit evidence and where it is recorded:**

| Gate | Evidence |
| --- | --- |
| Services start; migrations apply; health answers | Drill B1 in the [baseline verification record](../development/baseline-verification.md) |
| Data survives a restart without data loss | Drill B4 |
| Builds and tests pass | Drill B11 (2026-10-10 automated baseline as reference: builds and tests pass; fmt, clippy, audit fail) |
| No unintended network exposure | Precondition B0; not met at `f15c6a3` (D3); met at `354e06c` (observed 2026-10-10) |

**Keep out:** cloud resources, public access, production builds.

## M1 — Accounts and dashboards (implemented but unverified)

**Objective (plan 12/M1):** a user can create an account, sign in, and manage their own dashboards.

**Delivered (as built; legacy phases 3, 4, and 6, 2025-10-31 to 2025-11-01, with 2025-11-04 and 2026-03-03 fixes):**

- `POST /api/auth/register`, `POST /api/auth/login`, `GET /api/me`; Argon2 hashing; HS256 JWT valid for 7 days.
- `GET|POST /api/dashboards`, `GET|PUT|DELETE /api/dashboards/:id`, all scoped to the token's user.
- Login and register views, guarded routes, session in localStorage, dashboard list with create, open, delete.

**Status:** **implemented but unverified** (qualification above); owner acceptance pending drills B1–B12.

| Gate | Evidence |
| --- | --- |
| Register, login, `/me`, and their error cases | Drill B2 |
| Dashboard CRUD and ownership isolation | Drill B3 |
| Dashboard list create and delete in the browser | Drills B7 and B9 |
| Invalid session handling | Drill B10 (D23 expected) |

**Keep out:** refresh tokens, Cognito, roles, public dashboards.

## M2 — Widgets and editor (implemented but unverified)

**Objective (plan 12/M2):** five useful widgets on a dashboard the user can arrange, with provider data fetched
and cached by the backend.

**Delivered (as built; legacy phases 5, 7, 8, and 9, 2025-10-31 to 2025-11-01):**

- `/api/data/{github,weather,news,crypto,status}` with per-provider Redis TTLs.
- Widget registry, `BaseWidget`, five widget components with interval refresh, the configuration modal.
- GridStack editor (12 columns) with drag, resize, add, configure, remove, and 2 s auto-save of `layout_json`.

**Status:** **implemented but unverified** (qualification above); owner acceptance pending drills B1–B12.

| Gate | Evidence |
| --- | --- |
| Providers answer at human scale within approved targets | Drill B5 |
| Responses are cached with the documented TTLs | Drill B6 |
| Layout and configuration persist across reload | Drill B7 |
| Existing widgets can be reconfigured | Drill B8 (D9 expected: recorded as an observed defect, not a pass) |
| Removal and dashboard deletion persist | Drill B9 |

**Keep out:** new providers, background refresh, charts.

## M3 — UI polish and contract alignment (implemented but unverified)

**Objective (plan 12/M3):** a consistent interface and one agreed payload shape across backend and frontend.

**Delivered (as built; legacy phase 10 on 2025-11-03, fixes 2025-11-04 to 2025-11-05, remediation 2026-03-03 in
`f15c6a3`):**

- daisyUI 5.0.0-beta.9 on Tailwind 4 across all views and widgets.
- Auth and editor fixes: login error display, `/me` route alignment, dashboard field mapping, widget menu click
  handling, interval cleanup.
- `contracts/api-contract.v1.json` and `frontend/src/contracts/apiContract.ts`; backend payloads aligned to them.
- Draft-on-add: a new widget exists only after **Save Changes**.
- 5 backend unit tests and 3 frontend component tests.

**Status:** **implemented but unverified** (qualification above); owner acceptance pending drills B1–B12.

| Gate | Evidence |
| --- | --- |
| Draft-on-add: Cancel creates nothing, Save creates the widget | Drill B7 |
| Payloads match the v1 contract | Responses from B2, B3, and B5 compared with `contracts/api-contract.v1.json` by inspection (no automated check, D15) |
| Layout at 390 px and keyboard use | Drill B12 (observations, D25) |
| Automated tests pass | Drill B11 |

**Keep out:** dark theme, new widgets, OpenAPI.

## M4 — Quality gate and hardening (planned)

**Objective (plan 12/M4):** reproducible builds, passing gates, loopback defaults, safe outbound calls, real test
suites, and CI, fixing the defects the baseline recorded. Legacy phase 11 (Testing and CI/CD) folds in here.

The binding details will be the [M4 contract](../architecture/m4-quality-gate-contract.md) (**proposed**) and
ADR 0002 (written at acceptance).

**Slices, in order:**

- **M4a — Reproducibility, gates, and exposure:** D3 (landed early in `354e06c`, 2026-10-10), D4, D12, D13, D14,
  D18, D26, D29 (compose). Decisions 1, 2, 3, 4, 7, 14, 16, 17.
- **M4b — Outbound safety, API correctness, and contract:** D1, D2, D5, D6, D7, D8, D15, D16, D17, D19, D20,
  D29 (cache keys), D30. Decisions 5, 6, 8, 9, 10, 11.
- **M4c — Frontend correctness, test suites, and CI:** D9, D10, D11, D22, D23, D24, D25, D27, D28, D31. Decisions
  12, 13, 15 (and the daisyUI part of 14).

**Decisions pending (owner):** full text and ★ recommendations in the contract.

| # | Decision | Recommendation | Owner choice | Date |
| --- | --- | --- | --- | --- |
| 1 | Commit `Cargo.lock`; pin the toolchain; `rust-version` | ★ as proposed | pending | |
| 2 | Root `Makefile` mirroring StatusForge, host Rust or container fallback | ★ as proposed | pending | |
| 3 | fmt and clippy gates; ESLint and Prettier; `vue-tsc` in `build` | ★ as proposed | pending | |
| 4 | Loopback defaults, compose on `127.0.0.1`, credentials from `.env` (before drills, B0) | ★ as proposed | **approved** ★; landed in `354e06c` as the B0 mitigation | 2026-10-10 |
| 5 | Outbound policy for the status widget and all providers | ★ as proposed | pending | |
| 6 | `governor` rate limiting; Argon2 in `spawn_blocking` | ★ as proposed | pending | |
| 7 | CORS allowlist from `CORS_ORIGINS` | ★ as proposed | pending | |
| 8 | Auth hardening; owner calls: token TTL, localStorage vs httpOnly cookie | no ★ | pending | |
| 9 | Dashboard JSON defaults, migration and backfill, shape validation | no ★ | pending | |
| 10 | Error model; owner calls: 400 vs 422 with details, 400 vs 503 for a missing key | no ★ | pending | |
| 11 | Contract tests on both sides; OpenAPI later | ★ as proposed | pending | |
| 12 | Test suites with local mocks, no live providers | ★ as proposed | pending | |
| 13 | GitHub Actions CI without secrets or live providers | ★ as proposed | pending | |
| 14 | Dependency fixes, daisyUI stable, unused removals, `engines` | no ★ | pending | |
| 15 | Frontend defects; owner calls: title source (`name` ★), auto-save with visible errors ★, keyboard layout editing | partial ★ | pending | |
| 16 | MIT `LICENSE` | ★ as proposed | pending | |
| 17 | Keep `docs/guides/` non-authoritative | ★ as proposed | pending | |

**Agents:** to be chosen in M4 preparation and recorded here; tiers follow [`.omp/AGENTS.md`](../../.omp/AGENTS.md).

**Next slice:** M4 preparation (present decisions 1–3 and 5–17, record the owner's choices and dates above, write
ADR 0002, accept the contract), then M4a. Decision 4 (D3) already landed in `354e06c` as the B0 mitigation.

**Keep out:** cloud work, app container images (M5), OpenAPI generation, new widgets, dark theme, public dashboards.

## M5 — Local release (planned)

Scope, exit evidence, and keep-out follow plan section 12/M5: production builds, loopback-bound app containers or a
compose profile, release notes and a runbook, README screenshots, a version tag only with owner approval. Legacy
phase 12 (local half). Contract written after M4 is accepted.

## M6 — AWS readiness (planned)

Scope, exit evidence, and keep-out follow plan section 12/M6: a Lambda-compatible packaging seam, synth-only IaC
(tool chosen by the owner), a Cognito-compatible auth seam unless deferred, TLS features for sqlx and redis (D21),
and a cost and cleanup proposal. No AWS credentials, no deploy. Legacy phase 12 (cloud half).

## M7 — AWS experiment (deferred)

Deferred. Needs explicit owner approval, a $0 path, and a cleanup plan; its contract is written only after M6 is
accepted. Do not pre-build its infrastructure.

## Approved external targets

| Target | Purpose | Limits | Approval |
| --- | --- | --- | --- |
| GitHub REST API (`api.github.com`) | Manual smoke of the github widget (B5, B6) | Human scale: a handful of requests; optional token supplied by the owner | Plan section 4 (2026-10-10; owner confirmation pending) |
| OpenWeather (`api.openweathermap.org`) | Manual smoke of the weather widget | Human scale; only with a free-tier key the owner supplies | Same |
| NewsAPI (`newsapi.org`) | Manual smoke of the news widget | Human scale; only with a free-tier key the owner supplies | Same |
| CoinGecko (`api.coingecko.com`) | Manual smoke of the crypto widget | Human scale; no key | Same |
| `https://example.com` | Status-widget drills | A handful of requests | Same |
| Loopback fixtures | Status-widget drills and tests | Loopback only | Same |

Nothing on this list is approved for automated tests or CI; those use local mocks only. `httpstat.us` (from the old
run instructions) is not approved. Never load-test any target. Add a row (target, purpose, limits, approval date)
before any other external request.

## Next action

1. Owner enters free-tier `OPENWEATHER_API_KEY` and `NEWSAPI_API_KEY` (optional `GITHUB_API_TOKEN`) into `.env`;
   the main thread restarts the backend and runs the key half of B5 (one weather and one news request).
2. Owner acceptance of M0–M3 against the [baseline verification record](../development/baseline-verification.md),
   defects included.
3. M4 preparation: decisions 1–3 and 5–17, ADR 0002, contract acceptance; then M4a.
