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
| M0 — Foundation | Toolchain, compose Postgres and Redis, Axum skeleton with config, migrations, and health; Vue shell | **Accepted** 2026-10-10 (owner), defects included; see the qualification below and the [baseline verification record](../development/baseline-verification.md) |
| M1 — Accounts and dashboards | Register and login with JWT sessions; owner-scoped dashboard CRUD; frontend auth flow and dashboard list | **Accepted** 2026-10-10 (owner), defects included; see the qualification below and the [baseline verification record](../development/baseline-verification.md) |
| M2 — Widgets and editor | Five provider widgets through a cached backend proxy; GridStack editor with persisted layout | **Accepted** 2026-10-10 (owner), defects included; see the qualification below and the [baseline verification record](../development/baseline-verification.md) |
| M3 — UI polish and contract alignment | daisyUI styling, auth and editor fixes, v1 payload contract, draft-on-add editor | **Accepted** 2026-10-10 (owner), defects included; see the qualification below and the [baseline verification record](../development/baseline-verification.md) |
| M4 — Quality gate and hardening | Reproducible builds, passing gates, loopback defaults, safe outbound calls, real test suites, CI | **Accepted** 2026-10-10 (owner) after M4a, M4b, M4c were locally verified and CI passed on GitHub; [contract](../architecture/m4-quality-gate-contract.md) ([ADR 0002](../decisions/ADR-0002-m4-quality-gate.md)); [record](../development/m4-verification.md) |
| M5 — Local release | A reproducible, documented, independently runnable local product | **Accepted** 2026-10-10 (owner) after M5a and M5b were locally verified, drills 1–10 passed, and CI passed on GitHub; [contract](../architecture/m5-local-release-contract.md) ([ADR 0003](../decisions/ADR-0003-m5-local-release.md)); [record](../development/m5-verification.md) |
| M6 — AWS readiness | Cloud adapters and an inspectable, synth-only experiment proposal, without deployment | **In progress**: M6a, M6b, M6c **locally verified** (drills 1–8 on this workstation; drill 9 CI pending); awaiting owner acceptance; [contract](../architecture/m6-aws-readiness-contract.md) ([ADR 0004](../decisions/ADR-0004-m6-aws-readiness.md)); [record](../development/m6-verification.md) |
| M7 — AWS experiment | A validated and cleaned-up cloud execution path | **Deferred**; owner approval and a $0 path required |

## Qualification for M0–M3

- Delivered 2025-10-28 to 2026-03-03 under the legacy stop-and-confirm phase protocol, which treated phases as
  milestones and marked them "COMPLETED & VERIFIED".
- **Reported** verified on a previous workstation (`/home/gui-13/PersProj/...`) between 2025-10-31 and 2026-03-03;
  no evidence artifacts beyond prose.
- The 2026-03-03 audit (`f15c6a3`) found and fixed defects inside phases already marked completed.
- The 2026-10-10 automated baseline on this workstation passes builds and tests; `cargo fmt --check`,
  `cargo clippy -D warnings`, and `npm audit` fail (D13, D14).
- Drills B0–B12 ran on this workstation on 2026-10-10 at `354e06c` after the decision 4 change (B0); they observed
  D9, D23, D25, and two new defects, D30 (crypto and news always 502: no `User-Agent`) and D31 (small-width
  layout).
- **Accepted** by the owner on 2026-10-10 against the
  [baseline verification record](../development/baseline-verification.md), defects included.

The binding as-built behaviour, the defect catalogue D1–D31, and drills B0–B12 are in the
[M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md) (**accepted** with M0–M3). Acceptance of M0–M3
records the product as it is, defects included; fixing defects is M4 work.

## M0 — Foundation (accepted)

**Objective (plan 12/M0):** a local development baseline that starts, persists data, and builds.

**Delivered (as built; legacy phases 1 and 2, 2025-10-28 to 2025-10-31):**

- `docker-compose.yml`: Postgres 16 and Redis 7 (AOF) with named volumes and healthchecks.
- `backend/`: Rust crate `insightboard-backend` on axum 0.7 with environment configuration, a Postgres pool,
  embedded migrations applied at startup, a Redis client, `GET /healthz`, CORS, compression, request tracing, and
  graceful shutdown.
- `frontend/`: Vue 3 + Vite + TypeScript shell with Pinia, Vue Router, and an axios client.
- `.env.example`, `frontend/.env.example`, `.nvmrc` (Node 20.19.0), ignore rules.

**Status:** **accepted** 2026-10-10 (qualification above).

**Exit evidence and where it is recorded:**

| Gate | Evidence |
| --- | --- |
| Services start; migrations apply; health answers | Drill B1 in the [baseline verification record](../development/baseline-verification.md) |
| Data survives a restart without data loss | Drill B4 |
| Builds and tests pass | Drill B11 (2026-10-10 automated baseline as reference: builds and tests pass; fmt, clippy, audit fail) |
| No unintended network exposure | Precondition B0; not met at `f15c6a3` (D3); met at `354e06c` (observed 2026-10-10) |

**Keep out:** cloud resources, public access, production builds.

## M1 — Accounts and dashboards (accepted)

**Objective (plan 12/M1):** a user can create an account, sign in, and manage their own dashboards.

**Delivered (as built; legacy phases 3, 4, and 6, 2025-10-31 to 2025-11-01, with 2025-11-04 and 2026-03-03 fixes):**

- `POST /api/auth/register`, `POST /api/auth/login`, `GET /api/me`; Argon2 hashing; HS256 JWT valid for 7 days.
- `GET|POST /api/dashboards`, `GET|PUT|DELETE /api/dashboards/:id`, all scoped to the token's user.
- Login and register views, guarded routes, session in localStorage, dashboard list with create, open, delete.

**Status:** **accepted** 2026-10-10 (qualification above).

| Gate | Evidence |
| --- | --- |
| Register, login, `/me`, and their error cases | Drill B2 |
| Dashboard CRUD and ownership isolation | Drill B3 |
| Dashboard list create and delete in the browser | Drills B7 and B9 |
| Invalid session handling | Drill B10 (D23 expected) |

**Keep out:** refresh tokens, Cognito, roles, public dashboards.

## M2 — Widgets and editor (accepted)

**Objective (plan 12/M2):** five useful widgets on a dashboard the user can arrange, with provider data fetched
and cached by the backend.

**Delivered (as built; legacy phases 5, 7, 8, and 9, 2025-10-31 to 2025-11-01):**

- `/api/data/{github,weather,news,crypto,status}` with per-provider Redis TTLs.
- Widget registry, `BaseWidget`, five widget components with interval refresh, the configuration modal.
- GridStack editor (12 columns) with drag, resize, add, configure, remove, and 2 s auto-save of `layout_json`.

**Status:** **accepted** 2026-10-10 (qualification above).

| Gate | Evidence |
| --- | --- |
| Providers answer at human scale within approved targets | Drill B5 |
| Responses are cached with the documented TTLs | Drill B6 |
| Layout and configuration persist across reload | Drill B7 |
| Existing widgets can be reconfigured | Drill B8 (D9 expected: recorded as an observed defect, not a pass) |
| Removal and dashboard deletion persist | Drill B9 |

**Keep out:** new providers, background refresh, charts.

## M3 — UI polish and contract alignment (accepted)

**Objective (plan 12/M3):** a consistent interface and one agreed payload shape across backend and frontend.

**Delivered (as built; legacy phase 10 on 2025-11-03, fixes 2025-11-04 to 2025-11-05, remediation 2026-03-03 in
`f15c6a3`):**

- daisyUI 5.0.0-beta.9 on Tailwind 4 across all views and widgets.
- Auth and editor fixes: login error display, `/me` route alignment, dashboard field mapping, widget menu click
  handling, interval cleanup.
- `contracts/api-contract.v1.json` and `frontend/src/contracts/apiContract.ts`; backend payloads aligned to them.
- Draft-on-add: a new widget exists only after **Save Changes**.
- 5 backend unit tests and 3 frontend component tests.

**Status:** **accepted** 2026-10-10 (qualification above).

| Gate | Evidence |
| --- | --- |
| Draft-on-add: Cancel creates nothing, Save creates the widget | Drill B7 |
| Payloads match the v1 contract | Responses from B2, B3, and B5 compared with `contracts/api-contract.v1.json` by inspection (no automated check, D15) |
| Layout at 390 px and keyboard use | Drill B12 (observations, D25) |
| Automated tests pass | Drill B11 |

**Keep out:** dark theme, new widgets, OpenAPI.

## M4 — Quality gate and hardening (accepted)

**Objective (plan 12/M4):** reproducible builds, passing gates, loopback defaults, safe outbound calls, real test
suites, and CI, fixing the defects the baseline recorded. Legacy phase 11 (Testing and CI/CD) folds in here.

The binding details are the [M4 contract](../architecture/m4-quality-gate-contract.md) (**accepted** 2026-10-10)
and [ADR 0002](../decisions/ADR-0002-m4-quality-gate.md).

**Slices, in order:**

- **M4a — Reproducibility, gates, and exposure:** D3 (landed early in `354e06c`, 2026-10-10), D4, D12, D13, D14,
  D18, D26, D29 (compose). Decisions 1, 2, 3, 4, 7, 14, 16, 17. **Locally verified** 2026-10-10 (`d4bdca9`,
  `2780fc0`; drills 1–4 in the [M4 verification record](../development/m4-verification.md)).
- **M4b — Outbound safety, API correctness, and contract:** D1, D2, D5, D6, D7, D8, D15, D16, D17, D19, D20,
  D29 (cache keys), D30. Decisions 5, 6, 8, 9, 10, 11. **Locally verified** 2026-10-10 (`b20986e`; drills 5–11 in
  the [M4 verification record](../development/m4-verification.md)).
- **M4c — Frontend correctness, test suites, and CI:** D9, D10, D11, D22, D23, D24, D25, D27, D28, D31. Decisions
  12, 13, 15 (and the daisyUI part of 14), plus the owner's visual overhaul with light and dark themes (2026-10-10).
  **Locally verified** 2026-10-10 (`e20a0bc`, `5caff27`, `13e9018`; drills 12–16 in the
  [M4 verification record](../development/m4-verification.md)).

**Decisions (owner, 2026-10-10):** full text in the contract; preparation values (status-widget cap and timeouts,
rate limits, dashboard cap, 422 shape, `/readyz`) in the contract's "Preparation values" table.

| # | Decision | Recommendation | Owner choice | Date |
| --- | --- | --- | --- | --- |
| 1 | Commit `Cargo.lock`; pin the toolchain; `rust-version` | ★ as proposed | **approved** ★ | 2026-10-10 |
| 2 | Root `Makefile` mirroring StatusForge, host Rust or container fallback | ★ as proposed | **approved** ★ | 2026-10-10 |
| 3 | fmt and clippy gates; ESLint and Prettier; `vue-tsc` in `build` | ★ as proposed | **approved** ★ | 2026-10-10 |
| 4 | Loopback defaults, compose on `127.0.0.1`, credentials from `.env` (before drills, B0) | ★ as proposed | **approved** ★; landed in `354e06c` as the B0 mitigation | 2026-10-10 |
| 5 | Outbound policy for the status widget and all providers | ★ as proposed | **approved** ★ | 2026-10-10 |
| 6 | `governor` rate limiting; Argon2 in `spawn_blocking` | ★ as proposed | **approved** ★ | 2026-10-10 |
| 7 | CORS allowlist from `CORS_ORIGINS` | ★ as proposed | **approved** ★ | 2026-10-10 |
| 8 | Auth hardening; owner calls: token TTL, localStorage vs httpOnly cookie | no ★ | **approved**; TTL 24 hours; localStorage | 2026-10-10 |
| 9 | Dashboard JSON defaults, migration and backfill, shape validation | no ★ | **approved** as written | 2026-10-10 |
| 10 | Error model; owner calls: 400 vs 422 with details, 400 vs 503 for a missing key | no ★ | **approved**; 422 with details; 503 | 2026-10-10 |
| 11 | Contract tests on both sides; OpenAPI later | ★ as proposed | **approved** ★ | 2026-10-10 |
| 12 | Test suites with local mocks, no live providers | ★ as proposed | **approved** ★ | 2026-10-10 |
| 13 | GitHub Actions CI without secrets or live providers | ★ as proposed | **approved** ★; enabling needs a push request (granted as a standing push authorization, 2026-10-10) | 2026-10-10 |
| 14 | Dependency fixes, daisyUI stable, unused removals, `engines` | no ★ | **approved** as written | 2026-10-10 |
| 15 | Frontend defects; owner calls: title source (`name` ★), auto-save with visible errors ★, keyboard layout editing | partial ★ | **approved**; `name`; auto-save with error and retry; keyboard layout editing to the backlog | 2026-10-10 |
| 16 | MIT `LICENSE` | ★ as proposed | **approved** ★ | 2026-10-10 |
| 17 | Keep `docs/guides/` non-authoritative | ★ as proposed | **approved** ★ | 2026-10-10 |

**Agents** (tiers per [`.omp/AGENTS.md`](../../.omp/AGENTS.md)):

| Slice | Implementation | Verification |
| --- | --- | --- |
| M4a | `implement-medium`; the fmt reformat commit may go to `implement-light` | `verify` |
| M4b | `implement-heavy` (backend, contract pair, and client change together) | `verify` |
| M4c | `implement-heavy` | `verify-browser` (desktop, 390 px, keyboard) |

Research as needed: `research-medium` by default, `research-heavy` for cross-layer audits.

**Accepted** by the owner 2026-10-10. **Next:** M5 — local release.

**Keep out:** cloud work, app container images (M5), OpenAPI generation, new widgets, public dashboards, keyboard
layout editing (backlog). Light and dark themes with a toggle joined M4c with the owner's visual overhaul
(2026-10-10).

## M5 — Local release (accepted)

Scope, exit evidence, and keep-out follow plan section 12/M5 and the
[M5 contract](../architecture/m5-local-release-contract.md) (**accepted** 2026-10-10; owner decisions 1–9 in
[ADR 0003](../decisions/ADR-0003-m5-local-release.md)). Legacy phase 12 (local half).

- **M5a — Packaging:** the backend serves the built frontend; one multi-stage image; compose profile `app` on
  `127.0.0.1:8088`; `make release`, `up`, `down`, `logs`, `backup`; CI `release` job. **Locally verified**
  (`9a14027`; verify finding F1 fixed; CI run 38096580273 release job green).
- **M5b — Release material and drills:** version `1.0.0`, `CHANGELOG.md`, runbook, README release path and
  screenshots, documented limits, drills 1–10. **Locally verified** (`cdb9359` and the records commit; evidence in
  the [M5 verification record](../development/m5-verification.md); CI run 38097611740 green).

**Accepted** by the owner 2026-10-10 (version `1.0.0`). Annotated tag `v1.0.0` on `7c90107` (code identical to the
accepted `e6abe8c`; dated changelog and product README added), pushed at the owner's request. **Next:** M6 — AWS
readiness.

**Keep out:** cloud, registries, public hosting, domains, TLS (D21, M6), a GitHub Release object, the backlog.

## M6 — AWS readiness (in progress)

Scope, exit evidence, and keep-out follow plan section 12/M6: a Lambda-compatible packaging seam, synth-only IaC
(tool chosen by the owner), a Cognito-compatible auth seam unless deferred, TLS features for sqlx and redis (D21),
and a cost and cleanup proposal. No AWS credentials, no deploy. Legacy phase 12 (cloud half).

The [M6 contract](../architecture/m6-aws-readiness-contract.md) and [ADR 0004](../decisions/ADR-0004-m6-aws-readiness.md)
are **accepted** (2026-10-10): the owner approved decisions 1–12, the new-dependency list, and the RDS CA bundle
download. Slices:

- **M6a — Backend seams:** `lambda` feature, musl package, rustls TLS (D21 fixed), cache-disabled mode. **Locally
  verified** 2026-10-11 (UTC).
- **M6b — Synth-only infrastructure:** CDK v2 package in `infra/`, synth guardrails, CI. **Locally verified**
  2026-10-11 (UTC).
- **M6c — Experiment proposal and drills:** [experiment proposal](../operations/aws-experiment-proposal.md) (cost
  $18.43 a month while deployed, teardown, M7 preconditions), documentation, drills 1–9. **Locally verified**
  2026-10-11 (UTC); drill 9 (CI) pending.

Evidence: [M6 verification record](../development/m6-verification.md).

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
| AWS public documentation and pricing pages (`aws.amazon.com`, `docs.aws.amazon.com`, `b0.p.awsstatic.com` price list) | M6 research and cost sourcing | Anonymous reads at human scale; no account, API, or credential | M6 research rule (public pages only), session 2026-10-10-08 |
| `truststore.pki.rds.amazonaws.com` | Public RDS CA bundle | One anonymous download, done 2026-10-10 | Owner, 2026-10-10 (M6 decision 8) |
| Docker Hub `amazon/aws-lambda-provided:al2023` | Runtime Interface Emulator for M6 drill 3 | Image pull only; runs on loopback | Owner, 2026-10-10 (M6 dependency list) |

Nothing on this list is approved for automated tests or CI; those use local mocks only. `httpstat.us` (from the old
run instructions) is not approved. Never load-test any target. Add a row (target, purpose, limits, approval date)
before any other external request.

## Next action

1. M6: CI green on all five jobs (drill 9), then owner acceptance of M6. M7 stays deferred; its contract would start
   from the [experiment proposal](../operations/aws-experiment-proposal.md) preconditions.
2. Tag `v1.0.0`: done 2026-10-10 on `7c90107`.
