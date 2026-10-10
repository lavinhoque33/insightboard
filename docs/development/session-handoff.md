# Session handoff

Resume work here without reconstructing a conversation. The [roadmap](../product/roadmap.md) owns milestone
status; the [baseline verification record](baseline-verification.md) holds M0–M3 evidence. This file holds only
the **current checkpoint**, the protocol, the template, and an index of session entries. Each session's full
entry lives in [`sessions/`](sessions/) so this file stays short as history grows.

## Current checkpoint — main thread only

| Field | Current state |
| --- | --- |
| Updated | 2026-10-10 (UTC) |
| Latest session | [2026-10-10-01-workflow-onboarding](sessions/2026-10-10-01-workflow-onboarding.md) |
| Milestone | M0–M3 **implemented but unverified**: delivered 2025-10-28 to 2026-03-03 under the retired phase protocol and **reported** verified on a previous workstation; the 2026-10-10 automated baseline passes builds and tests here, with `cargo fmt --check`, `cargo clippy -D warnings` and `npm audit` failing (D13, D14). Baseline drills B1–B12 not yet run. M4 — Quality gate and hardening **planned**; the [M4 contract](../architecture/m4-quality-gate-contract.md) is **proposed** with decisions 1–17 pending the owner. |
| Last accepted task | Workflow onboarding (session 2026-10-10-01): committed (`3521c16` and the session record) and pushed to `origin/main` at the owner's request on 2026-10-10. |
| Active task | None. |
| Active delegates / write ownership | None. The main thread owns everything. |
| Blockers | **B0 (D3):** compose publishes Postgres (`postgres/postgres`) and passwordless Redis on all interfaces, the backend binds `0.0.0.0:8080`, and firewalld zone `FedoraWorkstation` opens `1025-65535/tcp` on `wlp1s0` (observed 2026-10-10). Drills must not start until the owner approves loopback-only binds (M4 decision 4, ★ land it first) or another mitigation. Any push to the public origin needs an explicit request. |
| Exact next action | Owner decides B0 and how the backend runs for the drills: this workstation has no Rust toolchain, so either install rustup at user level or run the backend in a container published on `127.0.0.1:8080` `[INFERENCE: untested]`. Then run drills B1–B12 into the [baseline record](baseline-verification.md) (B5 with or without owner-supplied free-tier keys); then M4 preparation (present decisions 1–17, record choices, write ADR-0002, accept the contract) and M4a. |
| Workspace | This workstation (Fedora 44): clone `/home/axeleratorx/PersProject/InsightBoard`, `origin` = `https://github.com/lavinhoque33/insightboard.git` (**public**), `main` = `origin/main` with the onboarding commits on top of `f15c6a3`; working tree clean apart from ignored files. No `.env`; compose not running. Node 20.19.0 in gitignored `.tools/` (host Node 24.19.0); no host Rust, backend checks run in `rust:1-bookworm` (rustc 1.99.0, image pulled) per [testing](testing.md). `frontend/node_modules` and `frontend/dist` present (gitignored). Ignored local files `backend.log`, `frontend/dev.log`, `frontend/dev.pid` left as found. |

## Protocol

- **Only the main thread edits this file and anything under `sessions/`**, including attributed subagent
  summaries. Subagents return findings, changed paths, checks with outcomes, blockers, and proposed next
  steps in their tool response; the main thread records concise summaries naming the reporting agent.
- Label subagent results as **reported**. Only the main thread accepts, partially accepts, rejects, or marks
  blocked after inspecting files and evidence. Keep three things distinct: checks an agent ran, checks the main
  thread inspected, checks the main thread reran. Never convert missing evidence into a pass.
- Legacy claims from the retired phase ledger and audits are **reported** evidence from a previous workstation;
  they never count as verification on this workstation.
- On a meaningful session start or resume: read this checkpoint and the latest entry, inspect the workspace
  (`git status`, `docker compose ps`), then create or resume an entry. IDs are `YYYY-MM-DD-NN-short-topic`
  (UTC date); check `sessions/` before assigning `NN`. Attributed report sections use `<entry-id>/<task>`.
- Update at meaningful handoffs, blockers, or scope changes. Before ending, pausing, or planned compaction,
  record the last verified task, active work and delegates, uncommitted changes, ownership, blockers, and the
  exact next step. Informational-only exchanges need no entry.
- Keep entries concise and chronological. Link source paths, ADRs, and evidence. Never paste transcripts, raw
  logs, credentials, or unreviewed model output. Historical commands must be rechecked when relevant; the
  latest user steering takes precedence.

## Entry template

Create `sessions/YYYY-MM-DD-NN-short-topic.md`:

```markdown
# YYYY-MM-DD-NN-short-topic

- Request / acceptance: ...
- Milestone / constraints: ...
- Uncommitted changes at start: ...

## Report summary: <entry-id>/<task>

- Report source / role: main thread, or named agent and research/implementation role.
- Reported state: active | submitted | blocked.
- Ownership / changed paths: exact paths and owners; identify uncommitted work.
- Findings / work performed: concise results; link sources; label proposals.
- Checks / evidence: exact commands and outcomes; skipped checks and reasons.
- Remaining work / blockers: dependency, impact, action needed to resume.
- Proposed next action: ...

## Main-thread review: <entry-id>/<task>

- Decision: pending | accepted | partially accepted | changes requested | blocked.
- Evidence reviewed / checks executed: distinguish inspection from reruns.
- Gaps / follow-up owner: ...

## Close

- Milestone and slice:
- What changed:
- What was verified, and in which environment:
- What remains unverified or blocked:
- Decisions and permissions still needed:
- Next recommended action:
```

## Session index (newest first)

| Session | Outcome |
| --- | --- |
| [2026-10-10-01-workflow-onboarding](sessions/2026-10-10-01-workflow-onboarding.md) | Pulled `2e1c4ac` → `f15c6a3`. Three read-only audits (`research-heavy` ×2, `research-medium`); automated baseline run on this workstation; StatusForge workflow adopted: `.omp/AGENTS.md`, plan, roadmap (M0–M7), contracts, ADR-0001, verification record, setup and testing docs, README rewrite (`implement-customHeavy` ×2); legacy docs condensed, moved, or removed. Committed and pushed at the owner's request. |
