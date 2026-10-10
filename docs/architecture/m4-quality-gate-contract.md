# M4 contract: quality gate and hardening

- Status: **proposed** — owner decisions pending. The owner settles decisions 1–17 in M4 preparation; the main
  thread then records the choices in the [roadmap](../product/roadmap.md), writes ADR 0002, and marks this contract
  **accepted**. Until then nothing here is binding and no M4 implementation starts, except decision 4 (D3), which
  the owner approved on 2026-10-10 and which landed before the baseline drills (`354e06c`).
- Sources: [M0–M3 baseline contract](m0-m3-baseline-contract.md) (behaviour as built and the canonical defect list
  D1–D31), [baseline verification record](../development/baseline-verification.md) (automated baseline
  2026-10-10), [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) sections 4, 10–12, [ADR 0001](../decisions/ADR-0001-foundation-stack.md),
  [system overview](system-overview.md), [coding practices](../../CODING_PRACTICES.md).

M4 turns the as-built product into one that can be rebuilt, checked, and run locally without exposing the owner's
machine or credentials. It changes behaviour only where a defect requires it. Every defect ID below refers to the
baseline contract section 12; this document does not restate them.

## Slices

Delivered in order; all three are required for M4 acceptance.

- **M4a — Reproducibility, gates, and exposure:** D3 (landed early in `354e06c`; M4a drill 2 still applies), D4,
  D12, D13, D14, D18, D26, D29 (compose). Decisions 1, 2, 3, 4, 7, 14, 16, 17.
- **M4b — Outbound safety, API correctness, and contract:** D1, D2, D5, D6, D7, D8, D15, D16, D17, D19, D20,
  D29 (cache keys), D30. Decisions 5, 6, 8, 9, 10, 11.
- **M4c — Frontend correctness, test suites, and CI:** D9, D10, D11, D22, D23, D24, D25, D27, D28, D31. Decisions
  12, 13, 15 (and the daisyUI part of 14).

D21 (TLS features) belongs to M6.

## Decisions

★ marks the recommended option. "Owner calls" are choices with no recommendation in this draft.

1. ★ Commit `Cargo.lock`; add `rust-toolchain.toml` (stable channel pinned to a version) and `rust-version`
   in `backend/Cargo.toml`.
2. ★ Root `Makefile` mirroring StatusForge: `setup`, `doctor`, `db-up`, `db-down`, `db-reset CONFIRM=yes`,
   `backend-dev`, `web-dev`, `backend-check`, `web-check`, `verify`, `format`; works with host Rust or the
   container fallback.
3. ★ Gates: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` (one reformat commit plus the
   two dead-code fixes); ESLint and Prettier for the web with a `lint` script; `vue-tsc` stays in `build`.
4. ★ Loopback defaults: a bind-host variable defaulting to `127.0.0.1`; compose ports prefixed `127.0.0.1:`;
   Redis password and non-default Postgres credentials from `.env`; Vite host stays loopback. Requested before
   baseline drills (B0).
5. ★ Outbound policy. Status widget: `http`/`https` only; resolve and reject loopback, private, link-local,
   and metadata addresses (including `169.254.169.254`) and re-check at connect; no redirects or re-validated
   redirects; URL count cap; bounded concurrency; generic error classes; dev-only loopback fixture allowlist.
   All providers: encode user input; one shared client with connect and total timeouts; keys out of URLs where
   the provider allows headers, and never logged.
6. ★ Rate limiting with `governor` on auth routes and `/api/data/*`; Argon2 in `spawn_blocking`.
7. ★ CORS allowlist from `CORS_ORIGINS` (default the Vite dev origin).
8. Auth hardening: `JWT_SECRET` ≥ 32 bytes enforced at startup; email trimmed, lower-cased, validated; 409 for
   duplicates (race included); `/me` for a missing user → 401; generic 401 text. Owner calls: token TTL (7 days
   today) and localStorage vs httpOnly cookie.
9. Dashboard JSON: server defaults `{"widgets":[]}` and `{}`; migration changes the DB default and backfills
   `null`/`[]`; shape validation against the contract; empty name rejected on update.
10. Error model: extractor rejections wrapped as `{error}`; owner picks 400 vs 422-with-details for validation
    and 400 vs 503 for a missing provider key; frontend client follows.
11. ★ Contract tests on both sides (Rust `jsonschema` crate against handler output; ajv in Vitest against
    fixtures and `apiContract.ts`-typed payloads); OpenAPI (utoipa) later, not in M4.
12. ★ Tests: configurable provider base URLs with local mock servers; router tests via `tower::ServiceExt::oneshot`;
    `#[sqlx::test]` against compose Postgres; Redis cache tests; frontend client, store, guard, modal, and
    editor tests; Playwright journeys against a provider fixture. No live provider calls anywhere.
13. ★ GitHub Actions CI (free for public repos): backend and web jobs with Postgres and Redis services; no
    secrets; no live providers.
14. Dependencies: resolve the D14 advisories (bump axios, vue, vite, transitive); move daisyUI to stable 5 with
    a class audit (D24); remove unused dependencies (D26); add `engines`. Each new dependency needs owner approval.
15. Frontend defects D9, D10, D22, D23, D24, D25, D27, D28. Owner calls: single source of truth for the
    dashboard title (`name` ★ vs `settings.title`); explicit Save vs auto-save with visible errors (★ keep
    auto-save, add visible error and retry); keyboard layout editing now or backlog.
16. ★ Add an MIT `LICENSE` (the README already intends MIT).
17. ★ Keep `docs/guides/` as non-authoritative reference; do not rewrite them in M4.

Owner calls collected: decision 8 (token TTL; localStorage vs httpOnly cookie), decision 10 (400 vs 422 with
details; 400 vs 503 for a missing provider key), decision 15 (title source of truth; Save vs auto-save; keyboard
layout editing now or backlog). Decisions without ★ (8, 9, 10, 14, 15) still need explicit owner approval; new
dependencies named in any decision (for example `jsonschema`, ajv, ESLint, Prettier, Playwright) are approved only
by that decision.

## 1. Reproducibility and gates (M4a; decisions 1–3; D12, D13)

- `Cargo.lock` is committed and no longer ignored; `rust-toolchain.toml` pins a stable version; `backend/Cargo.toml`
  declares `rust-version`. `frontend/package.json` declares `engines` matching `.nvmrc` and Vite 7's floor.
- `make setup`, `make doctor`, and `make verify` work from a fresh clone on host Rust or, when Rust is absent, the
  container fallback documented in [testing](../development/testing.md). `make db-reset` refuses without
  `CONFIRM=yes`.
- `make backend-check`: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `cargo build`. `make web-check`: lint (ESLint and Prettier), `vitest run`, `vue-tsc -b && vite build`. All pass.
- `vitest.config.ts` is covered by a `tsconfig` and type-checked; ESLint rejects new explicit `any`, and the existing
  uses named in D13 are replaced with precise types.
- The reformat lands as its own commit, separate from behaviour changes.

## 2. Exposure and CORS (M4a; decisions 4, 7; D3, D4, D18, D29)

- The backend bind host comes from configuration and defaults to `127.0.0.1`; a non-loopback value is a deliberate,
  owner-approved change, never a default.
- Compose publishes Postgres and Redis on `127.0.0.1` only; Postgres credentials and a Redis password come from
  `.env`; `.env.example` carries placeholders, not working secrets. The obsolete `version` key is removed, and the
  fixed `container_name` values are dropped so a second checkout can run its own stack.
- The Vite dev server stays on loopback.
- CORS allows only the origins in `CORS_ORIGINS` (default the Vite dev origin); a request from any other origin
  gets no CORS grant.
- Configuration: `.env` is loaded before tracing starts so `RUST_LOG` in `.env` takes effect; missing required
  variables produce a clear startup error instead of a panic; variables that nothing reads are removed from
  `.env.example` or implemented.

## 3. Outbound safety (M4b; decision 5; D1, D2, D5, D7, D20, D29)

Status widget:

- Accepts `http` and `https` only. Each host is resolved and every resulting address is rejected if loopback,
  private, link-local, unspecified, or a cloud metadata address (including `169.254.169.254`); the check is repeated
  against the address actually dialled.
- Redirects are not followed, or each hop is re-validated under the same rules (chosen in preparation).
- At most a fixed number of URLs per request and bounded concurrency; values set in M4 preparation so the worst case
  fits the client timeout.
- Failures return generic error classes, never library error text.
- A development-only allowlist admits loopback fixtures for tests and drills; it is off by default.

All providers:

- User input is encoded as a path segment or query value, never interpolated raw; a GitHub username containing `/`
  or `..` is rejected or encoded.
- One shared HTTP client with connect and total timeouts.
- Provider keys move from query strings to headers where the provider supports it; keys never appear in logs.
- A null numeric value from CoinGecko degrades that item rather than failing the whole response (shape decided with
  decision 11).
- Cache keys are normalised and bounded; whether status results are cached per user is set in preparation.

## 4. Rate limiting (M4b; decision 6; D6)

- `governor` limits auth routes and `/api/data/*`; over-limit requests get 429 with `{error}`. Limits per route
  class are set in M4 preparation.
- Argon2 hashing and verification run in `spawn_blocking`.

## 5. Auth (M4b; decision 8; D17)

- Startup rejects a `JWT_SECRET` shorter than 32 bytes.
- Email is trimmed, lower-cased, and validated on register and login.
- A duplicate email returns 409, including the race between the existence check and the insert.
- `/me` for a user that no longer exists returns 401.
- 401 bodies are generic; the JWT error kind is not returned.
- Token lifetime and storage (localStorage or httpOnly cookie) follow the owner's call; if storage changes, the
  frontend session handling in section 12 changes with it.

## 6. Dashboard JSON (M4b; decision 9; D8)

- Create without `layout_json`/`settings_json` stores `{"widgets":[]}` and `{}`.
- A migration changes the `layout_json` default to `{"widgets":[]}` and backfills stored `null` and `[]` values;
  `settings_json` `null` is backfilled to `{}`.
- `layout_json` and `settings_json` are validated against the contract shapes on create and update; invalid shapes
  return the validation error of section 7.
- Update rejects an empty or blank name.
- The dashboard list is bounded (paging or a per-user cap, chosen in preparation) (D29).

## 7. Error model (M4b; decision 10; D16)

- Every error response, including axum extractor rejections, is JSON `{error}` (with optional details if the owner
  picks 422).
- Validation status (400, or 422 with details) and missing-provider-key status (400 or 503) follow the owner's call;
  the frontend client formats exactly what the backend sends and drops unreachable branches.

## 8. Contract enforcement (M4b; decision 11; D15)

- `contracts/api-contract.v1.json` gains the shapes needed by sections 5–7; it and
  `frontend/src/contracts/apiContract.ts` change together under one writer.
- Backend tests validate handler output against the schema with the `jsonschema` crate.
- Vitest validates fixtures and typed payloads against the schema with ajv.
- A deliberately broken fixture makes the contract tests fail.
- OpenAPI generation is not part of M4.

## 9. Tests (M4b and M4c; decision 12; D11, D19)

- Provider base URLs and timeouts are configurable so tests use local mock servers; no test calls a live provider
  or an arbitrary status URL.
- Backend: router tests with `tower::ServiceExt::oneshot`; `#[sqlx::test]` against compose Postgres; Redis cache
  tests; status-policy tests for every rejected address class; ownership-isolation tests.
- Frontend: API client (Bearer header, 401 handling), stores, router guards, configuration modal (including
  prefill), editor (add, configure, remove, save failure, unmount).
- Playwright journeys against a provider fixture cover login, dashboard create, widget add and configure, reload.
- Health: whether `/healthz` gains dependency checks or a separate readiness route is set in preparation (D19).

## 10. CI (M4c; decision 13)

- GitHub Actions workflow with backend and web jobs and Postgres and Redis service containers; it runs
  `make backend-check` and `make web-check` (or their documented equivalents).
- No secrets, no provider keys, no live provider calls, no deploy step.
- Enabling CI is a push to the public origin and needs an explicit owner request.

## 11. Dependencies (M4a and M4c; decision 14; D14, D24, D26)

- Resolve the `npm audit --omit=dev` findings recorded in the baseline; any residual is listed with the owner's
  acceptance in the M4 verification record.
- daisyUI moves to a stable 5.x release with a class audit of every component (M4c).
- Unused dependencies and dead code listed in D26 are removed.
- New dependencies need owner approval; each is recorded in ADR 0002.

## 12. Frontend correctness (M4c; decision 15; D9, D10, D22, D23, D24, D25, D27, D28)

- The configuration modal pre-fills the current configuration for existing widgets and the registry defaults for new
  ones (D9).
- Load and save failures are visible with a retry; an unknown or foreign dashboard id shows a not-found state
  (D10).
- The editor loads one dashboard by id, applies `minSize`, serialises saves, sends no request after leaving, and
  lets one owner (Vue or GridStack) manage widget DOM nodes (D22).
- Session: a 401 keeps the intended destination; token expiry is checked; logout resets every store (D23).
- Styling uses daisyUI 5 class names and theme variables; global overrides and inert configuration are removed
  (D24).
- Accessibility: labelled inputs, keyboard-reachable logout and menus, dialog semantics with focus management and
  Escape, `aria-label` on icon buttons, live regions for saving and loading (D25). Keyboard layout editing follows
  the owner's call.
- Display and copy fixes in D27; dashboard title source and settings in D28 follow the owner's call.

## 13. Licensing and docs (M4a; decisions 16, 17)

- An MIT `LICENSE` is committed; the README license line changes from "MIT intended" to MIT.
- `docs/guides/` stays non-authoritative and is not rewritten.
- [Local setup](../development/local-setup.md), [testing](../development/testing.md), the
  [system overview](system-overview.md), and the [baseline contract](m0-m3-baseline-contract.md) (as superseded
  sections) are updated with each slice.

## 14. Acceptance drills

Evidence goes to `docs/development/m4-verification.md` (main thread). Provider behaviour is exercised only against
local mocks; manual provider smoke stays within the approved external targets in the roadmap.

### M4a

| # | Drill | Expected |
| --- | --- | --- |
| 1 | Fresh clone → `make setup verify` | Green on this workstation, with host Rust or the container fallback |
| 2 | Start compose and the backend; `ss -ltn` | Postgres, Redis, backend, and Vite listen on `127.0.0.1` only; nothing on `0.0.0.0` or `::` |
| 3 | Preflight and simple requests from an origin not in `CORS_ORIGINS` | No CORS grant; the Vite dev origin is granted |
| 4 | `npm audit --omit=dev` | 0 high or critical findings, or owner-accepted residuals recorded |

### M4b

| # | Drill | Expected |
| --- | --- | --- |
| 5 | Status widget with `http://127.0.0.1`, `http://169.254.169.254`, a private-range address, a non-http scheme, and more URLs than the cap | Each rejected with a generic error class; no request leaves the machine for the rejected entries |
| 6 | GitHub username containing `/` or `..` | Rejected or encoded; no other GitHub path is reached |
| 7 | Each provider against a slow local mock | The configured timeout ends the request with a generic error |
| 8 | Exceed the auth and data rate limits | 429 with `{error}` |
| 9 | Create a dashboard without JSON fields; update with an empty name | `{"widgets":[]}` and `{}` stored; empty name rejected |
| 10 | Register a duplicate email, including two concurrent registrations | 409 |
| 11 | Contract tests with a deliberately broken fixture | Tests fail; restored fixture passes |

### M4c

| # | Drill | Expected |
| --- | --- | --- |
| 12 | Configure an existing widget in the browser (the D9 prefill drill) | The form shows the current configuration; Save keeps unchanged fields |
| 13 | Make a save fail (backend stopped) | A visible error with retry; no silent loss |
| 14 | Edit, then leave the editor | No extra PUT after leaving |
| 15 | CI on a pushed branch (after an explicit push request) | fmt, clippy, ESLint, Vitest, and `cargo test` green |
| 16 | Rerun baseline drills B1–B10 | All green; B8 now passes as drill 12 |

## 15. Out of scope

Cloud work of any kind; Docker images for app services (M5); OpenAPI generation; new widgets; dark theme; public
dashboards; TLS features for sqlx and redis (D21, M6); the unscheduled backlog in the
[project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) section 5.
