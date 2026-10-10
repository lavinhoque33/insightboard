# ADR 0002: M4 quality gate and hardening decisions

- Status: **accepted** 2026-10-10 — owner choices on M4 decisions 1–17, recorded by the main thread with the
  preparation values the [M4 contract](../architecture/m4-quality-gate-contract.md) left open.
- Milestone: M4 (slices M4a, M4b, M4c)
- Amends: [ADR 0001](ADR-0001-foundation-stack.md) (follow-ups listed there)
- Sources: [M4 contract](../architecture/m4-quality-gate-contract.md),
  [M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md) (defects D1–D31),
  [baseline verification record](../development/baseline-verification.md), session
  [2026-10-10-02](../development/sessions/2026-10-10-02-b0-loopback-and-drills.md).

## Context

The owner accepted M0–M3 on 2026-10-10 as built, defects included, after baseline drills B0–B12 on this workstation.
The drills confirmed the catalogued risks and added two defects: D30 (crypto and news always fail because no
`User-Agent` is sent) and D31 (small-width layout). The M4 contract proposed 17 decisions to make the product
reproducible, gated, and safe to run locally. Decision 4 (loopback binds, `.env` credentials) was approved first as
the B0 mitigation and landed in `354e06c`; the owner settled the rest the same day.

## Decisions

| # | Decision | Owner choice |
| --- | --- | --- |
| 1 | Commit `Cargo.lock`; `rust-toolchain.toml` pinned to a stable version; `rust-version` | ★ approved |
| 2 | Root `Makefile` (setup, doctor, db targets, dev, checks, verify, format), host Rust or container fallback | ★ approved |
| 3 | `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`; ESLint and Prettier with a `lint` script; `vue-tsc` in `build` | ★ approved |
| 4 | Loopback binds; compose on `127.0.0.1`; credentials from `.env` | ★ approved; landed in `354e06c` |
| 5 | Outbound policy for the status widget and all providers | ★ approved |
| 6 | `governor` rate limiting; Argon2 in `spawn_blocking` | ★ approved |
| 7 | CORS allowlist from `CORS_ORIGINS` | ★ approved |
| 8 | Auth hardening | Approved; token lifetime **24 hours** (was 7 days); token stays in **localStorage** |
| 9 | Dashboard JSON defaults, migration and backfill, shape validation, non-empty name | Approved as written |
| 10 | Error model | **422 with details** for validation; **503** for a missing provider key |
| 11 | Contract tests on both sides (`jsonschema` crate, ajv); OpenAPI later | ★ approved |
| 12 | Test suites with local mocks; Playwright journeys; no live providers | ★ approved |
| 13 | GitHub Actions CI without secrets or live providers | ★ approved; enabling it still needs an explicit push request |
| 14 | Audit fixes, daisyUI stable 5, unused-dependency removal, `engines` | Approved as written |
| 15 | Frontend defects D9, D10, D22–D25, D27, D28 | Title from **`name`**; **auto-save** with visible error and retry; **keyboard layout editing to the backlog** |
| 16 | MIT `LICENSE` | ★ approved |
| 17 | `docs/guides/` stays non-authoritative | ★ approved |

### Dependencies approved through these decisions

| Dependency | Side | Decision | Use |
| --- | --- | --- | --- |
| `jsonschema` | backend (dev) | 11 | Validate handler output against `contracts/api-contract.v1.json` |
| `tower` with `util` | backend (dev) | 12 | `ServiceExt::oneshot` router tests; already a regular dependency with `util`, so M4a added nothing |
| ajv | frontend (dev) | 11 | Validate fixtures and typed payloads against the contract |
| ESLint, `eslint-plugin-vue`, `typescript-eslint`, `eslint-config-prettier` | frontend (dev) | 3 | `lint` script |
| Prettier | frontend (dev) | 3 | Formatting gate |
| Playwright (`@playwright/test`) | frontend (dev) | 12 | Browser journeys against a provider fixture |

Already declared and now used: `governor` (6), `mockito` (12). Version bumps under decision 14 (axios, vue, vite,
transitive packages, daisyUI stable 5) are covered by that decision. Any other new dependency needs owner approval
and a row here.

### Preparation values

Set by the main thread on 2026-10-10 where the contract deferred a value to M4 preparation; the owner may change any
of them before its slice starts. Details are in the contract's "Preparation values" table.

- Status widget: redirects not followed; at most 10 URLs per request, 5 at a time, 2 s connect and 4 s total
  timeout per URL (worst case 8 s, inside the frontend's 10 s timeout); results cached per user.
- Shared provider client: 3 s connect and 8 s total timeout and a fixed `User-Agent` (fixes D30).
- Rate limits: auth 10 per minute per client IP, burst 5; `/api/data/*` 60 per minute per user, burst 30;
  configurable.
- Dashboards: per-user cap of 100; creating beyond it returns 409.
- Validation body: `{error, details:[{field, message}]}`, `field` dotted for nested JSON.
- Health: `/healthz` liveness only; `/readyz` checks Postgres and Redis.

## Alternatives considered

- **Token lifetime:** keep 7 days (convenient; a leaked token stays valid for a week) or 8 hours (more frequent
  logins). 24 hours chosen.
- **httpOnly cookie for the token:** keeps the token away from page scripts, but needs SameSite and CSRF handling,
  CORS credentials, backend cookie code, and contract and client changes. Not chosen; the shorter lifetime and the
  CORS allowlist reduce the localStorage exposure.
- **400 `{error}` for validation:** smallest change; chosen against in favour of field details that forms can show.
- **400 for a missing provider key:** the request is valid and the server is not configured, so 503.
- **`settings.title` as the title source; an explicit Save button; keyboard layout editing in M4c:** not chosen.
- **Following status-widget redirects with re-validation:** more useful results, more code and more outbound
  requests; not following them is simpler and safer.
- **Dashboard paging:** changes the contract and the list UI; a per-user cap bounds the list with less change.

## Consequences

- M1's 7-day token becomes 24 hours in M4b; users sign in again daily. The client checks expiry (D23).
- The error contract changes: 422 with details and 503 for missing keys replace today's 400s. The contract files,
  handlers, client, and tests change together under one writer.
- Keyboard layout editing joins the unscheduled backlog in plan section 5; the rest of D25 stays in M4c.
- Every outbound provider call goes through one client with timeouts and a `User-Agent`, which also fixes D30.
- New dev tooling (ESLint, Prettier, Playwright, ajv, `jsonschema`) adds weight to `npm ci` and test runs.
- CI exists only after the owner explicitly asks for a push.

## Follow-ups

Agents per slice are recorded in the [roadmap](../product/roadmap.md). Each slice records its evidence in
`docs/development/m4-verification.md`. Cloud seams (Lambda adapter, IaC tool, TLS features, Cognito) are decided in
M6.
