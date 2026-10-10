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
the URLs are hard-coded today (D11), so no provider test exists yet.

## Commands

There is no `Makefile` yet (M4 decision 2). Until M4a lands one, these commands are the gate.

**Frontend** (from `frontend/`, Node 20.19.0; on this workstation
`export PATH="$PWD/../.tools/node-v20.19.0-linux-x64/bin:$PATH"`):

| Command | What it runs |
| --- | --- |
| `npm ci` | Clean install from `package-lock.json` |
| `npm run test` | `vitest run` (jsdom, setup `src/test/setup.ts`) |
| `npm run build` | `vue-tsc -b && vite build`: type-checks app and spec files, then bundles |
| `npm audit --omit=dev` | Advisories in production dependencies |
| `npx vitest run src/components/__tests__/BaseWidget.spec.ts` | One test file |

There are no `lint`, `format`, `test:watch`, `test:coverage`, or `test:e2e` scripts; ESLint, Prettier, and
Playwright are not installed.

**Backend, native** (from `backend/`, Rust stable with `rustfmt` and `clippy`):

| Command | What it runs |
| --- | --- |
| `cargo build` | Debug build; no database needed (runtime queries, embedded migrations) |
| `cargo test` | Unit tests |
| `cargo fmt --check` | rustfmt defaults; no `rustfmt.toml` |
| `cargo clippy --all-targets -- -D warnings` | Lints, warnings as errors |
| `cargo test jwt_roundtrip` | One test by name filter |

**Backend, container fallback** (from the repository root, when host Rust is absent; this workstation has host Rust since 2026-10-10).
The mount is read-only and the build runs in a throwaway copy, so nothing is written to the checkout. With no
committed `Cargo.lock` (D12), every run resolves dependencies afresh from crates.io:

```sh
docker run --rm -v "$PWD/backend":/src:ro rust:1-bookworm bash -c 'rustup component add rustfmt clippy >/dev/null 2>&1; cp -r /src /work && cd /work && cargo build && cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings'
```

The chain stops at the first failure; today that is `cargo fmt --check` (D13), so clippy runs only if you
split the chain.

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
| `frontend/src/components/__tests__/BaseWidget.spec.ts` | "emits configure when Configure is clicked from options menu" | Options menu opens and Configure emits once |
| `frontend/src/views/__tests__/DashboardEditorView.spec.ts` | "does not create a widget until configuration is saved" | Draft-on-add: modal opens, Cancel leaves no grid item (stores and GridStack mocked) |
| `frontend/src/views/__tests__/DashboardEditorView.spec.ts` | "creates the widget only after saving valid configuration" | Save with a valid status URL adds one grid item; the save payload is not asserted |

Totals: 7 backend unit tests and 3 frontend component tests. Nothing covers routes, the database, Redis,
providers, the API client, stores, router guards, the contract, the configuration-modal prefill (D9), or
end-to-end journeys; there is no CI (D11).

## Current results and known failing gates

**Observed** on 2026-10-10 on this workstation at `354e06c` (drill B11; frontend with Node 20.19.0 from `.tools/`;
backend native with rustup stable, rustc 1.99.0). The earlier run at `f15c6a3` in the container gave the same
results with 5 backend tests:

| Check | Result |
| --- | --- |
| `npm ci` | Pass; 246 packages |
| `npm run test` | Pass; 2 files, 3 tests |
| `npm run build` | Pass; CSS warning `Unknown at rule: @property` (D24) |
| `npm audit --omit=dev` | **Fail**: 8 findings, 7 high and 1 moderate (D14); full `npm audit`: 17 |
| `cargo build` | Pass; 2 dead-code warnings |
| `cargo test` | Pass; 7 tests |
| `cargo fmt --check` | **Fail**: 67 diff hunks across 16 source files (D13) |
| `cargo clippy --all-targets -- -D warnings` | **Fail**: the 2 dead-code warnings (D13) |

Details are in the [baseline verification record](baseline-verification.md). Until M4a fixes D13 and D14, a
change is judged on: builds and tests pass, no clippy warnings beyond the two known ones, new code written in
rustfmt style (without reformatting the crate), and no new audit findings. The legacy 2026-03-03 audit
**reported** `npm audit` with 0 findings on a previous workstation.

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
- **Gates (decision 3):** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint, Prettier,
  `vue-tsc`, Vitest, and `cargo test`, run by `make verify`.

None of this exists yet.

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
