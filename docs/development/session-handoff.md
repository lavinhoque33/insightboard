# Session handoff

Resume work here without reconstructing a conversation. The [roadmap](../product/roadmap.md) owns milestone
status; the [baseline verification record](baseline-verification.md) holds M0–M3 evidence. This file holds only
the **current checkpoint**, the protocol, the template, and an index of session entries. Each session's full
entry lives in [`sessions/`](sessions/) so this file stays short as history grows.

## Current checkpoint — main thread only

| Field | Current state |
| --- | --- |
| Updated | 2026-10-10 (UTC) |
| Latest session | [2026-10-10-02-b0-loopback-and-drills](sessions/2026-10-10-02-b0-loopback-and-drills.md) |
| Milestone | M0–M3 **accepted** by the owner on 2026-10-10 against the [baseline record](baseline-verification.md), defects included (drills B0–B12 on this workstation at `354e06c`; new D30 crypto and news always 502 without a `User-Agent`, D31 small-width layout). M4 — Quality gate and hardening **planned**; the [M4 contract](../architecture/m4-quality-gate-contract.md) is **accepted** (2026-10-10) with owner decisions 1–17 in the roadmap and [ADR 0002](../decisions/ADR-0002-m4-quality-gate.md); decision 4 landed in `354e06c`. |
| Last accepted task | Session 2026-10-10-02: decision 4 (D3 fix, `354e06c`), drills B0–B12, owner acceptance of M0–M3, and M4 preparation (decisions, preparation values, ADR 0002, contract accepted), committed locally; not pushed. |
| Active task | None in progress. Next: M4a. |
| Active delegates / write ownership | None. The main thread owns everything. |
| Blockers | None. Any push to the public origin needs an explicit request. |
| Exact next action | M4a: brief `implement-medium` with contract sections 1, 2, 11 (audit part), and 13 and the M4a drills; then `verify`; record evidence in `docs/development/m4-verification.md` (new, main thread). |
| Workspace | This workstation (Fedora 44): clone `/home/axeleratorx/PersProject/InsightBoard`, `origin` public; `main` is ahead of `origin/main` by the local commits of session 2026-10-10-02 (not pushed). `.env` present (gitignored, generated secrets, `POSTGRES_USER=postgres` for the 2026-10-01 volume with the password rotated in place). Compose running on `127.0.0.1`; backend debug build running on `127.0.0.1:8080`; Vite stopped. Host Rust via user-level rustup (rustc 1.99.0) plus `openssl-devel`; Node 20.19.0 in `.tools/`. Ignored local files `backend.log`, `frontend/dev.log`, `frontend/dev.pid` left as found. |

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
| [2026-10-10-02-b0-loopback-and-drills](sessions/2026-10-10-02-b0-loopback-and-drills.md) | Owner approved decision 4, rustup, free-tier keys. Decision 4 landed (`implement-medium`, one main-thread fix) as `354e06c`; rustup and `openssl-devel` installed; drills B0–B12 run by the main thread; new D30 (crypto and news: no `User-Agent`) and D31. Owner accepted M0–M3 and settled M4 decisions 1–17; M4 contract accepted with ADR 0002. Committed locally, not pushed. |
| [2026-10-10-01-workflow-onboarding](sessions/2026-10-10-01-workflow-onboarding.md) | Pulled `2e1c4ac` → `f15c6a3`. Three read-only audits (`research-heavy` ×2, `research-medium`); automated baseline run on this workstation; StatusForge workflow adopted: `.omp/AGENTS.md`, plan, roadmap (M0–M7), contracts, ADR-0001, verification record, setup and testing docs, README rewrite (`implement-customHeavy` ×2); legacy docs condensed, moved, or removed. Committed and pushed at the owner's request. |
