# ADR 0003: M5 local release decisions

- Status: **accepted** 2026-10-10 — owner choices on M5 decisions 1–9, recorded by the main thread with the
  preparation values of the [M5 contract](../architecture/m5-local-release-contract.md).
- Milestone: M5 (slices M5a, M5b)
- Resolves: [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) section 17, open decision 5 (container images or a
  compose profile; whether to tag a version).
- Sources: [M5 contract](../architecture/m5-local-release-contract.md), session
  [2026-10-10-06](../development/sessions/2026-10-10-06-m5-contract.md).

## Context

M4 was accepted on 2026-10-10. At `12b1fdb` the product ran only as a development setup (host Rust and Node, Vite
dev server, compose for Postgres and Redis); there was no Dockerfile, release build path, runbook, release notes,
screenshot, or tag. The plan asks M5 for a reproducible, documented, independently runnable local product, proven
by a fresh-clone setup on a clean machine and a restart without data loss.

## Decisions

| # | Decision | Owner choice |
| --- | --- | --- |
| 1 | Packaging: one application image; the backend serves the built frontend on the same origin with an SPA fallback | ★ approved, including the `tower-http` features `fs` and `set-header` |
| 2 | Multi-stage image: `node:20.19.0-bookworm-slim`, `rust:1.99.0-bookworm`, runtime `debian:bookworm-slim` with `libssl3` and `ca-certificates`, non-root, nothing pushed | ★ approved |
| 3 | In-container bind `0.0.0.0`, published only on `127.0.0.1` | ★ approved |
| 4 | Compose profile `app` in the existing project, host port `8088` (owner changed it from the proposed `8000`, which StatusForge already uses on this workstation), shared Postgres and Redis data | ★ approved; port amended 2026-10-10 |
| 5 | Version `1.0.0` in both manifests; annotated tag `v1.0.0` only on an explicit request after M5 acceptance; no GitHub Release | ★ approved |
| 6 | `CHANGELOG.md` (Keep a Changelog) and `docs/operations/runbook.md` | ★ approved |
| 7 | Auth rate limit keeps keying on the peer address; the shared bucket is a documented limit | ★ approved |
| 8 | CI `release` job: build the image without pushing and smoke-test it on the runner | ★ approved |
| 9 | Clean-machine evidence: the CI `release` job, plus a fresh clone here with a Docker-only `PATH` and its own compose project; `docker compose down -v` approved **for that throwaway project only** | ★ approved |

Preparation values are listed in the contract.

## Consequences

- New locked crates come only from the `tower-http` `fs` and `set-header` features; any other new dependency still
  needs owner approval and a line here.
- Base images are pulled from Docker Hub (free); no registry push, no cloud resource.
- The release app and development share one database, so development and e2e test users exist beside real data
  (owner-scoped, invisible to other accounts).
- `APP_HOST=0.0.0.0` is allowed only inside the `app` container; host exposure stays loopback.
- Creating or pushing the `v1.0.0` tag remains an explicit owner request (`.omp/AGENTS.md`, Git publication).
