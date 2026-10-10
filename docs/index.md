# InsightBoard documentation

InsightBoard is a personal dashboard builder: accounts, owner-scoped dashboards, and five provider widgets on a
GridStack grid, served by a Rust/Axum API with Postgres and Redis and a Vue 3 client. M0–M3 are **implemented
but unverified** (delivered under a legacy phase protocol, **reported** verified on a previous workstation;
baseline drills on this workstation pending). M4 — Quality gate and hardening is **planned**, its contract
**proposed**.

## Start here

1. Read the [project plan](../INSIGHTBOARD_PROJECT_PLAN.md): product reference, constraints, milestones.
2. Follow [agent operations](../.omp/AGENTS.md) and [coding practices](../CODING_PRACTICES.md).
3. Read the [session handoff](development/session-handoff.md) for the current checkpoint and exact next action.
4. Check the [roadmap](product/roadmap.md) for milestone status and the next slice.
5. Use [local setup](development/local-setup.md) and [testing](development/testing.md) to run and check the
   product. Read the network exposure section in local setup before starting any service.

The root [README](../README.md) summarises status, limits, and layout.

## Product

| Document | Purpose |
| --- | --- |
| [Project plan](../INSIGHTBOARD_PROJECT_PLAN.md) | Product direction, constraints, milestones M0–M7, definition of done, backlog and non-goals |
| [Roadmap](product/roadmap.md) | Milestone tracker, M0–M3 qualification and exit evidence, M4 slices and pending decisions, approved external targets, next action |
| [Terminology](product/terminology.md) | Product objects, delivery words, evidence words, roles, status words |

## Architecture

| Document | Purpose |
| --- | --- |
| [System overview](architecture/system-overview.md) | Components, request flows, data model, cache keys, ports and binds, boundaries, known gaps by defect ID, planned seams |
| [M0–M3 baseline contract](architecture/m0-m3-baseline-contract.md) | **Proposed** as-built record at `f15c6a3`: routes, status codes, config, schema, widgets, editor; defect catalogue D1–D31; baseline drills B0–B12 |
| [M4 quality-gate contract](architecture/m4-quality-gate-contract.md) | **Proposed**: slices M4a–M4c, owner decisions, acceptance drills |

## Decisions

| Document | Purpose |
| --- | --- |
| [ADR 0001: foundation stack](decisions/ADR-0001-foundation-stack.md) | **Proposed** retroactive record of the stack chosen 2025-10-28 to 2026-03-03: Rust/Axum, SQLx, Postgres, Redis, Vue 3, Pinia, GridStack, Tailwind and daisyUI |

## Development

| Document | Purpose |
| --- | --- |
| [Local setup](development/local-setup.md) | Prerequisites, network exposure (D3, fixed), first run, configuration, stop and restart, troubleshooting |
| [Testing](development/testing.md) | Evidence vocabulary, commands (native and container), current suites, failing gates, drill conventions, M4 test targets |
| [Baseline verification](development/baseline-verification.md) | M0–M3 evidence: 2026-10-10 automated baseline, drill results B1–B12 |
| [Session handoff](development/session-handoff.md) | Current checkpoint, protocol, template, session index; entries under `development/sessions/` |
| [2026-10-10-01 workflow onboarding](development/sessions/2026-10-10-01-workflow-onboarding.md) | The session that moved InsightBoard onto this workflow |

## Guides (non-authoritative)

Educational walkthroughs written in 2025-10 and 2025-11 under the legacy protocol. They are kept for background,
are not maintained, and contain stale points listed in each banner. Where they disagree with the documents above,
the documents above win.

| Document | Purpose |
| --- | --- |
| [Backend walkthrough](guides/backend-walkthrough.md) | Rust backend explained for newcomers |
| [Frontend walkthrough](guides/frontend-walkthrough.md) | Vue client, stores, widgets, and styling explained |
| [Infrastructure walkthrough](guides/infrastructure-walkthrough.md) | Docker, Postgres, Redis, and environment primer |

## Machine-readable contracts

| File | Purpose |
| --- | --- |
| [`contracts/api-contract.v1.json`](../contracts/api-contract.v1.json) | JSON Schema 2020-12 of request and response payload shapes (no paths, statuses, or errors; D15) |
| [`frontend/src/contracts/apiContract.ts`](../frontend/src/contracts/apiContract.ts) | Hand-written TypeScript mirror; changes together with the JSON, one writer |

## Documentation authority

- The **plan** owns direction and constraints; change it only for intentional scope decisions.
- The **roadmap** owns milestone status; only reviewed evidence moves a status.
- **Contracts** under `architecture/` own milestone behaviour; the baseline contract owns the defect catalogue.
- **ADRs** own consequential technical decisions; `Cargo.toml`, `package.json`, the lockfiles, `.nvmrc`, and
  `docker-compose.yml` own exact versions.
- **Verification records** own evidence; the **handoff** owns session continuity and is main-thread-editable only.

Retired legacy documents (phase ledger, audits, run instructions, condensed spec, backend verification report)
remain readable at commit `f15c6a3`, for example
[`testing_audit.md`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/testing_audit.md).
