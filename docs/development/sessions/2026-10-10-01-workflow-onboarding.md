# 2026-10-10-01-workflow-onboarding

- **Request / acceptance:** the owner asked to pull the latest InsightBoard, then bring it onto the StatusForge
  and HouseSync workflow, thoroughly: (1) an agent operating guide in the project's `.omp/` folder modelled on
  `StatusForge/AGENTS.md` (orchestration loop, agent tiers, delegation briefs, write ownership, session handoff,
  hard boundaries, git policy); (2) the scattered docs condensed into StatusForge-style milestones, contracts,
  ADRs, verification records, and an index; (3) a session handoff recording what was done and what comes next.
  Acceptance: an agent started in this folder can continue milestone by milestone as in StatusForge.
- **Milestone / constraints:** workflow onboarding; no milestone work and no application code changes.
  Implementation delegation only to `implement-customHeavy` (owner instruction); read-only research agents
  allowed. No commit or push requested at the start; the origin is public. The owner's unstaged deletion of root
  `AGENTS.md` is intentional. The owner wrote "AGENT.md"; the guide is `.omp/AGENTS.md` because omp loads only that
  filename from `.omp/`.
- **Uncommitted changes at start:** ` D AGENTS.md` (owner). `main` fast-forwarded `2e1c4ac` → `f15c6a3`
  (`git pull --ff-only`, 44 files, +3963 −1149); `main` = `origin/main`.

## Main-thread record: discovery and baseline

- StatusForge conventions read: `AGENTS.md`, `CODING_PRACTICES.md`, plan sections 14 and 20–23, `docs/index.md`,
  roadmap, terminology, system overview, M5 contract, ADR 0006, M5 verification, testing, session handoff, and two
  session entries. `HouseSync/AGENTS.md` compared; it is an earlier variant, so StatusForge is the template.
- omp discovery checked in source (`oh-my-pi/packages/coding-agent/src/discovery/builtin.ts:91-100,914-945`):
  the nearest non-empty `.omp/` between cwd and the repository root supplies `AGENTS.md`.
  `discovery/github.ts:203-218` turns `.github/instructions/*.instructions.md` with `applyTo: '**'` into an
  always-apply rule, so the legacy condensed spec was injected into every session with an obsolete protocol
  (mandatory todo, stop after each phase, rewrite the spec's status, educational-only docs). Removed this session.
- Host firewall **observed**: firewalld zone `FedoraWorkstation` on `wlp1s0` opens `1025-65535/tcp` and `/udp`.
  With compose ports on all interfaces and the backend on `0.0.0.0`, running the stack would expose Postgres
  (`postgres/postgres`), passwordless Redis, and the API to the LAN `[INFERENCE: not probed]`. Recorded as D3 and
  precondition B0; compose and the backend were not started.
- Automated baseline **observed** about 03:36 UTC at `f15c6a3`: frontend `npm ci`, `npm run test` (3 pass),
  `npm run build` (pass), `npm audit --omit=dev` (8 findings: 7 high, 1 moderate), full `npm audit` (17); backend
  in `rust:1-bookworm` (rustc 1.99.0, read-only mount, no lockfile): `cargo build` (pass, 2 dead-code warnings),
  `cargo test` (5 pass), `cargo fmt --check` (fail, 67 hunks in 15 files), `cargo clippy --all-targets -- -D
  warnings` (fail, the 2 warnings). Full table in the [baseline verification record](../baseline-verification.md).

## Report summary: 2026-10-10-01/backend-audit

- Report source / role: `research-heavy` (BackendReality), read-only.
- Reported state: submitted.
- Findings: module map, route table, configuration, persistence, cache, auth, error model, security posture,
  tests, dependencies, contract alignment, BACKEND_ARCHITECTURE.md drift, prioritised risks, owner questions.
  Critical: status-widget SSRF and GitHub path injection with the server token. High: no outbound timeouts, no
  rate limiting, permissive CORS and `0.0.0.0`, unencoded provider input, dashboard JSON integrity.
- Checks: code reading only; no builds or requests.

## Report summary: 2026-10-10-01/frontend-audit

- Report source / role: `research-heavy` (FrontendReality), read-only.
- Reported state: submitted.
- Findings: routes and guards, API layer, stores, widget registry, editor flow, contract binding, styling, a11y,
  tooling, tests, FRONTEND_ARCHITECTURE.md drift. High: configuration modal never pre-fills, silent save and load
  failures, unenforced contract, dead 422 path, false tooling claims (Node 18, pnpm, lint/e2e scripts, daisyUI
  4.12.10).
- Checks: code reading only.

## Report summary: 2026-10-10-01/docs-infra-audit

- Report source / role: `research-medium` (DocsInfraAudit), read-only.
- Reported state: submitted; it could not run git, so the main thread rebuilt the timeline from `git log`.
- Findings: document inventory, README false claims, infrastructure guide drift, compose and environment facts,
  condensed-spec split, no CI or delivery artifacts, contract file structure, cross-document contradictions.

## Main-thread review: 2026-10-10-01/audits

- Decision: accepted as code-reading evidence. Spot-checked against code: `backend/src/widgets/status.rs:29-75`
  (no scheme or address check, raw error text at `:68`), `backend/src/main.rs:38-48,82,90` (tracing before dotenv,
  permissive CORS, `0.0.0.0`), `frontend/src/components/WidgetConfigModal.vue:26-34` with
  `DashboardEditorView.vue:100-122,164-167,458-460` (prefill defect), `DashboardEditorView.vue:184-262`
  (autosave timer never reset), validation messages in `backend/src/handlers/auth.rs` and `widgets/*.rs`.
- The findings were consolidated into the stable defect catalogue D1–D29 (canonical in the
  [baseline contract](../../architecture/m0-m3-baseline-contract.md) section 12). The audits themselves are session
  artifacts, not repository files; their D26 dead-code list is carried into the catalogue with code paths.

## Report summary: 2026-10-10-01/governance-docs

- Report source / role: `implement-customHeavy` (GovernanceDocs), implementation, docs only.
- Reported state: submitted.
- Ownership / changed paths: created `.omp/AGENTS.md`, `CODING_PRACTICES.md`, `docs/index.md`,
  `docs/development/local-setup.md`, `docs/development/testing.md`; rewrote `README.md`; inserted banners in
  `docs/guides/{backend,frontend,infrastructure}-walkthrough.md` (bodies untouched; no relative links needed
  fixing).
- Checks: one read-only relative-link and emoji check over its files; no builds, tests, compose, provider calls,
  or git writes.
- Uncatalogued findings proposed: remaining `any` uses; fixed compose `container_name` values. Proposed: an
  interim gate until M4a (no crate-wide `cargo fmt`; no clippy warnings beyond the 2 known; no new audit findings).
- Open questions: B0 mitigation; whether the M4 `Makefile` should run the backend in a container when host Rust is
  absent.

## Report summary: 2026-10-10-01/product-architecture-docs

- Report source / role: `implement-customHeavy` (ProductArchitectureDocs), implementation, docs only.
- Reported state: submitted.
- Ownership / changed paths: created `INSIGHTBOARD_PROJECT_PLAN.md`, `docs/product/roadmap.md`,
  `docs/product/terminology.md`, `docs/architecture/system-overview.md`,
  `docs/architecture/m0-m3-baseline-contract.md`, `docs/architecture/m4-quality-gate-contract.md`,
  `docs/decisions/ADR-0001-foundation-stack.md`, `docs/development/baseline-verification.md`.
- Checks: one read-only link and emoji check; legacy commit dates rechecked with `git log`.
- Corrections to the design brief: legacy phase 6 and 7 times are the legacy docs' claims, the commits are later
  (`efa7c07` 2025-11-01 02:16 UTC, `2e6bc97` 04:07 UTC); `c06b31f`/`4f9318f` are dated 2025-11-05 05:32 UTC
  (fetched 2025-11-09). Both recorded in the plan section 13.
- Uncatalogued findings proposed: `.env.example` `RUST_LOG` targets `insightboard` instead of
  `insightboard_backend`; login skips Argon2 for unknown emails (timing difference); `vitest.config.ts` never
  type-checked; dashboard list unpaged with no per-user cap.

## Main-thread review: 2026-10-10-01/docs

- Decision: accepted with main-thread corrections. Every file read in full against the code facts and each
  other; legacy figures spot-checked against the removed files (`BACKEND_VERIFICATION.md` 266 ms → 11 ms, rustc
  1.91.0, 20:08 UTC; `testing_audit.md` build, `/me`, and 0-test findings); commit facts checked with `git log`.
- Corrections made by the main thread:
  - Uncatalogued findings folded into existing IDs instead of new ones: D13 (no ESLint or Prettier,
    `vitest.config.ts` unchecked, remaining `any`), D17 (login timing), D18 (`.env.example` log filter target),
    D29 (unpaged dashboard list, fixed container names); the M4 contract sections 1, 2, and 6 now cover them.
  - D26 refs replaced with repository code paths (they pointed at session-only audit files).
  - B11 reworded to name the commands in [testing](../testing.md) instead of a design-brief section number.
  - README: container images belong to M5, not M4.
  - `docs/index.md` rows for the roadmap, terminology, and system overview aligned with their contents.
- Interim gate proposed by GovernanceDocs (no crate-wide reformat before M4a) accepted as written in
  `CODING_PRACTICES.md` and `testing.md`.

## Main-thread record: integration

- Moved with `git mv` before delegation: `Documentation/BACKEND_ARCHITECTURE.md`, `FRONTEND_ARCHITECTURE.md`,
  `INFRASTRUCTURE_SETUP.md` → `docs/guides/backend-walkthrough.md`, `frontend-walkthrough.md`,
  `infrastructure-walkthrough.md` (non-authoritative; banners list stale points).
- Removed with `git rm` after condensing (readable at `f15c6a3`): `project_progress.md` (into the plan, roadmap,
  and this handoff), `testing_audit.md`, `manual_audit.md`, `Documentation/BACKEND_VERIFICATION.md` (into the
  baseline verification record), `manual run instructions.md` (into local setup and testing),
  `.github/instructions/Insightboard_Condensed_spec.instructions.md` (into the plan; stops the always-apply
  injection), `frontend/README.md` (unmodified Vite template).
- Kept in place: `contracts/api-contract.v1.json`, `frontend/src/contracts/apiContract.ts`, `docker-compose.yml`,
  `.env.example`, `frontend/.env.example`, `.nvmrc`. Ignored local files `backend.log`, `frontend/dev.log`,
  `frontend/dev.pid` left as found.
- Wrote [session-handoff.md](../session-handoff.md) and this entry.

## Main-thread record: publication

- The owner then asked to commit and push (about 04:50 UTC). `origin/main` was still `f15c6a3` after `git fetch`.
- `3521c16` `docs: move InsightBoard onto the milestone workflow`: the workflow docs, guide moves, and legacy
  removals. It includes the owner's deletion of root `AGENTS.md`, which pointed agents at the removed
  `project_progress.md` ledger and the retired phase protocol; `.omp/AGENTS.md` replaces it.
- `docs(session): record the workflow onboarding session`: this entry and the handoff checkpoint.
- Both pushed to `origin/main` as a fast-forward. Staged paths checked for `.env`, logs, pid files, and tool or
  build directories before committing: none.

## Close

- **Milestone and slice:** none; workflow onboarding. M0–M3 **implemented but unverified**; M4 **planned** with
  its contract **proposed**.
- **What changed:** see the two report summaries, the integration record, and the publication record. Committed
  and pushed; the working tree holds only the ignored local files.
- **What was verified, and where (this workstation):**
  - omp discovery end to end: `omp -p` from the repository root (throwaway session directory, removed afterwards)
    reported `.omp/AGENTS.md` as its project context file, first heading `# InsightBoard agent operating guide`,
    and no `.github/instructions` rule loaded.
  - Relative-link check over every tracked and new Markdown file outside dependencies: no broken links.
  - Stale-reference search over the new and rewritten docs for the removed files, `pnpm`, `/api/auth/me`, and
    `4.12.10`: matches only where the docs name them as retired or wrong.
  - Milestone names and status strings, defect IDs, decisions 1–17, and drills B0–B12 consistent across the plan,
    roadmap, contracts, README, index, verification record, and this handoff.
  - Automated baseline as recorded above. Docs-only change: no build or test rerun.
- **What remains unverified or blocked:** drills B1–B12 (blocked on B0, D3); every runtime claim in the baseline
  contract marked `[INFERENCE]`; LAN reachability (not probed).
- **Decisions and permissions still needed:** B0 mitigation (★ approve M4 decision 4 and land loopback binds
  first); how the backend runs for the drills, since this workstation has no Rust toolchain (user-level rustup, or
  the backend in a container on loopback `[INFERENCE: untested]`); whether to supply free-tier OpenWeather and
  NewsAPI keys for B5; confirmation of the approved external targets in the roadmap; M4 decisions 1–17.
- **Next recommended action:** the B0 decision, then session `2026-10-10-02-baseline-drills` (or the next free ID)
  running B1–B12 into the baseline verification record, then M4 preparation.
