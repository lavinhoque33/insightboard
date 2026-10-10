# Terminology

Shared vocabulary for code, documentation, and the interface. Terms come from the
[project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) and the
[M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md). A term here is a definition, not evidence
that the capability works; the [roadmap](roadmap.md) records status.

## Product objects

| Term | Meaning | Not to be confused with |
| --- | --- | --- |
| **Dashboard** | A named, owner-scoped page of widgets, stored as one `dashboards` row with `name`, `layout_json`, and `settings_json`. Another user's dashboard is indistinguishable from a missing one (404). | The dashboard list page (`/dashboards`). |
| **Widget** | A tile on a dashboard that shows data from one provider and refreshes on an interval. | A widget type. |
| **Widget type** | One of the five registered kinds (`github`, `weather`, `news`, `crypto`, `status`) with its component, configuration schema, default configuration, refresh interval, and sizes (`frontend/src/composables/widgetRegistry.ts`). | A provider (one widget type uses one provider, but the type is the frontend definition). |
| **Widget instance** | One placed widget: `{id, type, x, y, w, h, config}` inside `layout_json.widgets`. Its `id` is generated in the browser (`widget-<ms>-<rand>`). | A widget type. |
| **Draft widget** | A widget instance that exists only in the configuration modal while it is being added. It joins the grid and the saved layout only on **Save Changes**; **Cancel** discards it (draft-on-add, M3). | A saved widget with an empty configuration. |
| **Layout (`layout_json`)** | The JSONB column holding `{"widgets":[…]}`: every widget instance with position, size, and configuration. Written by auto-save. | GridStack's in-memory grid state. |
| **Settings (`settings_json`)** | The JSONB column for dashboard-level settings; written once at creation as `{title, theme}`. `title` duplicates `name` and `theme` is unused (D28). | Widget configuration (which lives in the layout). |
| **Provider** | An external service a widget type depends on: GitHub, OpenWeather, NewsAPI, CoinGecko, or the user-supplied URLs of the status widget. Called only by the backend. | A data endpoint. |
| **Data endpoint** | A backend route under `/api/data/*` that validates input, checks the cache, calls the provider, and returns a normalised payload. | The provider's own API. |
| **Cache TTL** | How long a data endpoint's Redis entry lives (github 300 s, weather 600 s, news 900 s, crypto 300 s, status 120 s). Distinct from, and not aligned with, the widget's client refresh interval. | The client refresh interval. |
| **Auto-save** | The editor's debounced save: 2 s after the last change it sends `PUT /api/dashboards/:id` with the layout. Failures are not shown today (D10). | An explicit Save button (none exists for layouts). |
| **Contract** | The v1 payload schema `contracts/api-contract.v1.json` (JSON Schema 2020-12) and its hand-written TypeScript mirror `frontend/src/contracts/apiContract.ts`. Payload shapes only; not enforced (D15). Also: a milestone contract under `docs/architecture/` (context decides). | OpenAPI (not used). |

## Delivery words

| Term | Meaning |
| --- | --- |
| **Milestone** | One of M0–M7 in the [roadmap](roadmap.md): an observable outcome with exit evidence, accepted only by the owner. The legacy phases 1–12 map onto milestones (plan section 13). |
| **Slice** | A delivery unit inside a milestone (for example M4a), with its own defect IDs and decisions. Completing a slice does not complete the milestone. |
| **Milestone contract** | The binding behaviour description of a milestone under `docs/architecture/`, **proposed** until the owner accepts it. |
| **Drill** | A numbered, reproducible check with an expected result: B0–B12 for the baseline, numbered drills in later contracts. Results go to a verification record. |
| **Defect ID** | A stable identifier `D1`–`D31` from the canonical list in the baseline contract section 12. Never renumbered or reused; new defects are proposed to the main thread for an ID. |

## Evidence words

| Term | Meaning |
| --- | --- |
| **Reported** | A claim made by someone else and not rerun: legacy documents from a previous workstation, or a subagent's report before main-thread review. |
| **Observed** | Seen by the main thread on this workstation, with the command or steps recorded and a UTC date. |
| **Inferred** (`[INFERENCE]`) | A deduction from code or library defaults that has not been observed. Marked inline. |

## Roles

| Term | Meaning |
| --- | --- |
| **Owner** | The single human who owns the repository, makes product and permission decisions, and accepts milestones. |
| **Main thread** | The orchestrating agent session: plans, delegates, reviews evidence, and alone edits the tracking documents (roadmap, plan, verification records, session handoff). |
| **Delegate** | A subagent given a bounded task with explicit file ownership; its results are **reported** until the main thread reviews them. |

## Status vocabulary for tracking documents

**planned** · **in progress** · **implemented but unverified** · **locally verified** · **accepted** ·
**blocked** · **deferred** — contracts and ADRs additionally use **proposed**. Qualify by environment ("on this
workstation", "on a previous workstation") where it matters.
