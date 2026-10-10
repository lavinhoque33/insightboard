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
URLs, locally or in CI. Provider behaviour is tested against local mock servers through configurable base URLs;
the URLs are hard-coded today (D11, M4b), so no provider test exists yet.

## Commands

The root `Makefile` is the gate (M4 decision 2). Run from the repository root; `make doctor` shows which tools it
found. Nothing below needs Postgres or Redis: `cargo test` has no database tests yet and the Vitest suites mock the
API.

| Target | What it runs |
| --- | --- |
| `make backend-check` | In `backend/`: `cargo fmt --check`, then `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`, `cargo build --locked` (`--locked` fails instead of silently changing the committed `Cargo.lock`) |
| `make web-check` | In `frontend/`: `npm run lint` (ESLint with `--max-warnings 0`, then `prettier --check .`), `npm test` (`vitest run`), `npm run build` (`vue-tsc -b && vite build`) |
| `make verify` | `backend-check` then `web-check`; works from a fresh clone after `make setup` |
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
| `cargo test jwt_roundtrip` | One backend test by name filter |

There are no `test:watch`, `test:coverage`, or `test:e2e` scripts; Playwright is not installed (M4c).

**Backend toolchain.** `rust-toolchain.toml` pins Rust 1.99.0 with `rustfmt` and `clippy`; `backend/Cargo.lock` is
committed, so builds use the locked dependency versions. With host rustup, `cargo` fetches the pinned toolchain on
first use inside the checkout.

**Backend, container fallback.** When `cargo` is not on `PATH`, `make backend-check` (and `make verify`) runs the
same four steps in a throwaway `rust:<version>-bookworm` container, where `<version>` is read from
`rust-toolchain.toml`. It mounts `backend/` read-only and builds in a copy inside the container, so nothing is
written to the checkout and no root-owned files appear. It needs Docker and internet access, and downloads the
locked crates on every run (no cache volume). `make backend-dev` and `make format` need host Rust.

## What exists

| Area | Test | Covers |
| --- | --- | --- |
| `backend/src/auth.rs` | `auth::tests::password_hash_roundtrip` | A hashed password verifies |
| `backend/src/auth.rs` | `auth::tests::password_hash_rejects_wrong_password` | A wrong password does not verify |
| `backend/src/auth.rs` | `auth::tests::jwt_roundtrip` | `sub` and `email` survive encode/decode; `exp > iat` |
| `backend/src/config.rs` | `config::tests::normalize_optional_strips_blank_values` | Unset, empty, and blank optional values become `None` |
| `backend/src/config.rs` | `config::tests::normalize_optional_trims_and_keeps_non_empty` | Optional values are trimmed |
| `backend/src/config.rs` | `config::tests::app_host_defaults_to_loopback` | Unset `APP_HOST` binds `127.0.0.1` (D3) |
| `backend/src/config.rs` | `config::tests::app_host_parses_ip_and_rejects_garbage` | An IP literal parses; a non-IP value fails naming `APP_HOST` |
| `backend/src/config.rs` | `config::tests::cors_origins_default_to_vite_dev_origin` | Unset or blank `CORS_ORIGINS` yields the two Vite dev origins (D4) |
| `backend/src/config.rs` | `config::tests::cors_origins_parse_a_trimmed_list` | A comma-separated list is trimmed; empty entries are skipped |
| `backend/src/config.rs` | `config::tests::cors_origins_reject_invalid_entries` | `*`, missing scheme, non-http scheme, paths, credentials, and wildcards fail naming `CORS_ORIGINS` |
| `backend/src/config.rs` | `config::tests::required_variables_name_themselves_when_missing_or_blank` | A missing or blank required variable errors with its name (D18) |
| `backend/src/main.rs` | `tests::preflight_from_allowed_origin_is_granted` | `tower::ServiceExt::oneshot` against the production middleware stack: an allowed origin gets `access-control-allow-origin`, methods, and no credentials header |
| `backend/src/main.rs` | `tests::preflight_from_foreign_origin_gets_no_grant` | A preflight from an origin outside the list gets no `access-control-allow-origin` |
| `backend/src/main.rs` | `tests::simple_request_from_foreign_origin_gets_no_grant` | A simple cross-origin request from a foreign origin is served but gets no CORS grant |
| `frontend/src/components/__tests__/BaseWidget.spec.ts` | "emits configure when Configure is clicked from options menu" | Options menu opens and Configure emits once |
| `frontend/src/views/__tests__/DashboardEditorView.spec.ts` | "does not create a widget until configuration is saved" | Draft-on-add: modal opens, Cancel leaves no grid item (stores and GridStack mocked) |
| `frontend/src/views/__tests__/DashboardEditorView.spec.ts` | "creates the widget only after saving valid configuration" | Save with a valid status URL adds one grid item; the save payload is not asserted |

Totals: 14 backend unit tests and 3 frontend component tests. The router test uses a stub route behind the real
middleware function, not the full application (which needs live Postgres and Redis). Nothing covers the real
routes, the database, Redis, providers, the API client, stores, router guards, the contract, the
configuration-modal prefill (D9), or end-to-end journeys; there is no CI (D11; M4b and M4c).

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

The gates above exist; the contract, provider-mock, `#[sqlx::test]`, Redis, frontend client/store/guard/modal/editor,
Playwright, and CI items are **pending** (M4b and M4c).

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
