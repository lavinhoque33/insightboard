# Session handoff

Resume work here without reconstructing a conversation. The [roadmap](../product/roadmap.md) owns milestone
status; the [baseline verification record](baseline-verification.md) holds M0–M3 evidence. This file holds only
the **current checkpoint**, the protocol, the template, and an index of session entries. Each session's full
entry lives in [`sessions/`](sessions/) so this file stays short as history grows.

## Current checkpoint — main thread only

| Field | Current state |
| --- | --- |
| Updated | 2026-10-10 (UTC) |
| Latest session | [2026-10-10-06-m5-contract](sessions/2026-10-10-06-m5-contract.md) |
| Milestone | M0–M4 **accepted** 2026-10-10 (M0–M3 defects included; M4 after M4a, M4b, M4c locally verified, [M4 verification record](m4-verification.md)). M5 — local release **in progress**: [contract](../architecture/m5-local-release-contract.md) **accepted** 2026-10-10 with owner decisions 1–9 ([ADR 0003](../decisions/ADR-0003-m5-local-release.md)). ADR 0001 **accepted** 2026-10-10. |
| Last accepted task | Session 2026-10-10-05: M4c — `e20a0bc` (CI), `5caff27` (frontend correctness, daisyUI stable 5, Vitest), `13e9018` (visual overhaul with light and dark themes, Playwright, CI e2e job); drills 12–16 pass; CI run 38087833114 backend, web, e2e success. Pushed over SSH: `a27a8de..e20a0bc`, `e20a0bc..5caff27`, `5caff27..13e9018`, then the `docs(m4c)` records commit directly after `13e9018`. |
| Active task | M5b — release material and drills (session 06). M5a accepted in `9a14027`. |
| Active delegates / write ownership | None active. Main thread owns `README.md`, `CHANGELOG.md`, `docs/operations/runbook.md`, screenshots, and tracking documents. |
| Blockers | None. Standing push authorization since 2026-10-10 (scope in `.omp/AGENTS.md`); force-push, tags, releases, PRs, and visibility changes still need an explicit request. HTTPS pushes that touch `.github/workflows/` need the `workflow` OAuth scope (`gh auth refresh -s workflow`); SSH pushes work. |
| Exact next action | Push M5a and the drafts; observe CI including `release`; version `1.0.0`; screenshots; drills 1–10; record `docs/development/m5-verification.md`; ask the owner to accept M5. |
| Workspace | This workstation (Fedora 44): clone `/home/axeleratorx/PersProject/InsightBoard`, `origin` public and in sync with `main`. `.env` present (gitignored, mode 600). Compose containers `insightboard-postgres-1`, `insightboard-redis-1` on `127.0.0.1`; backend `13e9018` debug build on `127.0.0.1:8080`; Vite on `127.0.0.1:5173`. Dev database holds drill, browser, and e2e test users. Backend tests and `make e2e` need `make db-up`; Playwright Chromium installed. rustc 1.99.0 (pinned); Node 20.19.0 in `.tools/`. |

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
| [2026-10-10-06-m5-contract](sessions/2026-10-10-06-m5-contract.md) | In progress: M5 contract draft. |
| [2026-10-10-05-m4c](sessions/2026-10-10-05-m4c.md) | M4c locally verified: CI workflow `e20a0bc` (`implement-medium`), frontend correctness `5caff27` and visual overhaul with light and dark themes plus Playwright `13e9018` (`implement-heavy` ×3); `verify` and `verify-browser` (one dialog-focus regression fixed); drills 12–16 by the main thread; CI green incl. e2e. Pushed over SSH (`a27a8de..8cc1e54`). Owner accepted M4. |
| [2026-10-10-04-m4b](sessions/2026-10-10-04-m4b.md) | ADR 0001 accepted; `main` pushed (`42555eb`). M4b locally verified: `b20986e` (`implement-heavy` backend and contract, `implement-medium` frontend; `ajv-formats` rejected), `verify` and `verify-browser` reviews, drills 5–11 by the main thread. Owner granted standing push authorization; pushed `42555eb..20ba7ce`. |
| [2026-10-10-03-m4a](sessions/2026-10-10-03-m4a.md) | M4a locally verified: format-only commit `d4bdca9` (`implement-light`), M4a commit `2780fc0` (`implement-medium`, main-thread fixes), `verify` review; drills 1–4 pass; local Postgres and Redis passwords rotated after a transcript exposure. Committed locally, not pushed. |
| [2026-10-10-02-b0-loopback-and-drills](sessions/2026-10-10-02-b0-loopback-and-drills.md) | Owner approved decision 4, rustup, free-tier keys. Decision 4 landed (`implement-medium`, one main-thread fix) as `354e06c`; rustup and `openssl-devel` installed; drills B0–B12 run by the main thread; new D30 (crypto and news: no `User-Agent`) and D31. Owner accepted M0–M3 and settled M4 decisions 1–17; M4 contract accepted with ADR 0002. Committed locally, not pushed. |
| [2026-10-10-01-workflow-onboarding](sessions/2026-10-10-01-workflow-onboarding.md) | Pulled `2e1c4ac` → `f15c6a3`. Three read-only audits (`research-heavy` ×2, `research-medium`); automated baseline run on this workstation; StatusForge workflow adopted: `.omp/AGENTS.md`, plan, roadmap (M0–M7), contracts, ADR-0001, verification record, setup and testing docs, README rewrite (`implement-customHeavy` ×2); legacy docs condensed, moved, or removed. Committed and pushed at the owner's request. |
