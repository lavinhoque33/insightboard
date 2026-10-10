# Baseline verification (M0–M3)

Maintained by the **main thread** after reviewing and rerunning relevant work. Subagents may read this record and
return check results in their reports; only the main thread edits it. Reported findings remain unaccepted until
main-thread review; owner acceptance of M0–M3 is recorded in the [roadmap](../product/roadmap.md). The expected
behaviour and the drills are defined in the [M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md).

## Status

M0 — Foundation, M1 — Accounts and dashboards, M2 — Widgets and editor, and M3 — UI polish and contract alignment
are **implemented but unverified**:

- Delivered 2025-10-28 to 2026-03-03 under the legacy stop-and-confirm phase protocol, which treated phases as
  milestones and marked them "COMPLETED & VERIFIED".
- **Reported** verified on a previous workstation (`/home/gui-13/PersProj/...`) between 2025-10-31 and 2026-03-03;
  no evidence artifacts beyond prose.
- The 2026-03-03 audit (`f15c6a3`) found and fixed defects inside phases already marked completed.
- The 2026-10-10 automated baseline on this workstation passes builds and tests; `cargo fmt --check`,
  `cargo clippy -D warnings`, and `npm audit` fail (section "Automated baseline").
- Journeys have not been re-exercised on this workstation. Owner acceptance of M0–M3 follows baseline drills
  B1–B12 below. The drills are blocked on precondition B0 (D3).

## Environment

This workstation, observed 2026-10-10:

- Fedora 44, x86_64. Clone `/home/axeleratorx/PersProject/InsightBoard` at `main` = `origin/main` = `f15c6a3`;
  origin `https://github.com/lavinhoque33/insightboard.git` is public.
- Docker 29.8.2, Compose v5.5.1; the user is in the `docker` group, so `sudo` is not needed.
- Node 20.19.0 from the gitignored `.tools/node-v20.19.0-linux-x64/bin` (matches `.nvmrc`); host Node 24.19.0 also
  satisfies Vite 7 (`^20.19 || >=22.12`).
- No host Rust. Backend checks run in an ephemeral `rust:1-bookworm` container (rustc 1.99.0) with the backend
  mounted read-only and no lockfile (`Cargo.lock` is gitignored, D12).
- Host firewall: firewalld zone `FedoraWorkstation` on `wlp1s0` opens `1025-65535/tcp` and `/udp`. With compose
  publishing `5432` and `6379` on all interfaces (default Postgres credentials, passwordless Redis) and the backend
  binding `0.0.0.0:8080`, these services would be reachable from the LAN while running
  `[INFERENCE: not probed]`. This is D3 and the reason for precondition B0.

## Legacy evidence (reported)

Claims from the legacy documents, written by agents on a previous workstation (`/home/gui-13/PersProj/...`). They
are **reported** evidence only: none was rerun here, none produced artifacts beyond prose, and several were later
contradicted. The documents are removed from the working tree in this session and cited at `f15c6a3`.

| Date (UTC) | Source | Reported claim | Notes |
| --- | --- | --- | --- |
| 2025-10-31 (completed 20:08) | [`Documentation/BACKEND_VERIFICATION.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/Documentation/BACKEND_VERIFICATION.md) | Postgres 16 and Redis 7 healthy in compose; migrations applied; register, login, protected routes, and dashboard CRUD exercised with curl; all five widget endpoints answered; cache improvement for the status widget (`status:https://example.com`, TTL 120 s) from 266 ms to 11 ms total time; built with rustc 1.91.0 after three build fixes; status "ALL TESTS PASSING" | No automated tests existed on that date. The latency figures are **reported**, not re-measured. The report also states "checked concurrently" (the code is sequential, D20) and compile-time SQL checks (queries are runtime). It recorded `settings_json: null` on create without flagging it (D8). |
| 2025-10-31 to 2025-11-03 | [condensed spec](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/.github/instructions/Insightboard_Condensed_spec.instructions.md) status section | Phases 1–7 "COMPLETED & VERIFIED", phases 8–9 merged into 7, phase 10 "COMPLETED" (2025-11-03 21:30); dev server verified on port 5173; "KNOWN ISSUES: None"; "all 5 widgets fully functional"; daisyUI 4.12.10 | Contradicted by the 2026-03-03 audit (build broken, `/me` route mismatch, contract drift, no tests) and by the lockfile (daisyUI 5.0.0-beta.9). |
| 2026-03-02 21:20 | [`project_progress.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/project_progress.md) | Phases 1–10 `COMPLETED`; phase 11 (Testing and CI/CD) and phase 12 (Deployment Preparation) `NOT STARTED` | Phase 11 was stale a day later: `f15c6a3` added 5 backend and 3 frontend tests. |
| 2026-03-03 | [`testing_audit.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/testing_audit.md) | `cargo test` 5 passed; `cargo check` pass; `npm run build` pass; `npm audit` 0 vulnerabilities; runtime smoke of `/healthz`, auth, dashboard CRUD, status shape, and 400 for missing weather and news keys; compose smoke via `sudo` with cache key `status:https://example.com`; nine findings resolved; "Open bugs: 0"; contract "actively used by implementation" | `npm audit` observed 2026-10-10 reports 8 production findings (D14) `[INFERENCE: the advisories may postdate 2026-03-03]`. The audit did not assess outbound safety, timeouts, or exposure (D1–D7). The contract is used only as TypeScript types (D15). |
| 2026-03-03 16:59–17:19 | [`manual_audit.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/manual_audit.md) | Add Widget now opens configuration; the widget menu Configure works; header contrast fixed; draft-on-add (a widget exists only after Save); `npm run test` 2 then 3 tests passed; `npm run build` passed; ad hoc Playwright smoke with widget count 2 → 2 → 2 → 3 | Times converted from CST. No Playwright script was committed. The configure flow opens with an empty form (D9), which this audit did not detect. |

Run instructions from the same period
([`manual run instructions.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/manual%20run%20instructions.md))
are condensed into [local setup](local-setup.md) and [testing](testing.md); they named `httpstat.us`, which is not an
approved target.

## Automated baseline — 2026-10-10 (observed)

Run by the main thread on this workstation at about 03:36 UTC at `f15c6a3`. Frontend in `frontend/` with Node
20.19.0 from `.tools/`: `npm ci`, `npm run test`, `npm run build`, `npm audit --omit=dev`, `npm audit`. Backend in
`rust:1-bookworm` (rustc 1.99.0) with a read-only mount and no lockfile:

```
docker run --rm -v "$PWD/backend":/src:ro rust:1-bookworm bash -c 'rustup component add rustfmt clippy >/dev/null 2>&1; cp -r /src /work && cd /work && cargo build && cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings'
```

| Check | Result |
| --- | --- |
| `npm ci` | 246 packages; deprecation warning for `glob@10.5.0` |
| `npm run test` (Vitest 4.0.18) | 2 files, 3 tests, all pass |
| `npm run build` (`vue-tsc -b && vite build`) | Pass; CSS warning `Unknown at rule: @property` (daisyUI radialprogress); index JS 195.60 kB (68.29 kB gzip), editor chunk 93.47 kB, CSS 80.99 kB |
| `npm audit --omit=dev` | 8 findings: 7 high, 1 moderate (axios direct: NO_PROXY SSRF, prototype pollution; vue/@vue/server-renderer XSS; postcss; source-map-js; nanoid; form-data; follow-redirects moderate) — D14 |
| `npm audit` (all) | 17 findings: 1 critical, 13 high, 3 moderate (includes Vite dev-server advisories) — D14 |
| `cargo build` | Pass; 2 dead-code warnings (`UserCtx.email` `src/auth.rs:34`, `AppError::Forbidden` `src/error.rs:30`) |
| `cargo test` | 5 passed: `config::tests::normalize_optional_strips_blank_values`, `config::tests::normalize_optional_trims_and_keeps_non_empty`, `auth::tests::jwt_roundtrip`, `auth::tests::password_hash_rejects_wrong_password`, `auth::tests::password_hash_roundtrip` |
| `cargo fmt --check` | Fail: 67 diff hunks across all 15 source files — D13 |
| `cargo clippy --all-targets -- -D warnings` | Fail: only the 2 dead-code warnings — D13 |

Because there is no lockfile, the backend resolved its dependencies fresh (axum 0.7.9, sqlx 0.8.6, redis 0.27.6,
reqwest 0.12.28, jsonwebtoken 9.3.1, tokio 1.53.2, tower-http 0.5.2); a later run may resolve differently (D12).

Not run this session: compose, backend runtime, migrations, any HTTP request, browser journeys, provider calls.
Those are drills B1–B12.

## Drills

Defined in the [baseline contract](../architecture/m0-m3-baseline-contract.md) section 13. Provider drills stay
within the approved external targets in the [roadmap](../product/roadmap.md).

| # | Drill | Result | Evidence |
| --- | --- | --- | --- |
| B0 | Precondition: D3 resolved by owner-approved loopback-only binds (or another approved mitigation); `.env` from `.env.example` with a locally generated `JWT_SECRET`; no provider keys unless the owner supplies free-tier keys | **pending** (owner decision) | — |
| B1 | Compose up, containers healthy, backend starts, migrations apply, `GET /healthz` | **pending** | — |
| B2 | Auth: register, login, `/me`, duplicate, 7-byte password, wrong password, no token | **pending** | — |
| B3 | Dashboard CRUD and ownership | **pending** | — |
| B4 | Persistence across `docker compose restart` and a backend restart | **pending** | — |
| B5 | Providers at human scale (weather and news without keys, one github username, crypto defaults, status with `https://example.com` only) | **pending** | — |
| B6 | Cache hit and key TTL in Redis | **pending** | — |
| B7 | Browser editor journey with draft-on-add, drag, resize, reload | **pending** | — |
| B8 | Configure an existing widget (expected to expose D9) | **pending** | — |
| B9 | Remove a widget and delete a dashboard | **pending** | — |
| B10 | Invalid `auth_token` in localStorage | **pending** | — |
| B11 | Automated gate rerun | **pending** (2026-10-10 baseline above is the reference) | — |
| B12 | 390 px width and keyboard-only pass (observations, not pass/fail) | **pending** | — |

## Defects found

The canonical list is section 12 of the [baseline contract](../architecture/m0-m3-baseline-contract.md); this
record does not duplicate it.

- **Observed 2026-10-10 on this workstation:** D13 (fmt and clippy failures), D14 (npm audit findings), D12 (fresh
  dependency resolution without a lockfile), D24 in part (the `@property` build warning), and the firewall half of
  D3.
- **From code reading (2026-10-10 audits), not yet observed at runtime:** every other ID. Drills expected to expose
  some of them: B8 (D9), B10 (D23), B12 (D25); B5 and B6 touch the paths of D1, D5, D7, D19, and D29 but stay within
  approved targets and do not attempt to exploit them.
- Defects found during the drills that are not in the catalogue are recorded here by description and proposed to
  the main thread for an ID; IDs are never assigned in this file.

## Limits of this record

Local verification only, on one workstation. No CI exists. The legacy rows are reported claims from another
workstation and do not count as verification here. Package counts, sizes, and test counts above are from the
2026-10-10 run and will change; the automated gate after M4a is `make verify`, not these numbers.
