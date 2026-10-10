# M5 contract: local release

- Status: **accepted** 2026-10-10. The owner approved every ★ decision (1–9) and the preparation values as written;
  recorded in [ADR 0003](../decisions/ADR-0003-m5-local-release.md).
- Sources: [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) section 12/M5 and open decision 5 (section 17),
  [M4 contract](m4-quality-gate-contract.md), [system overview](system-overview.md),
  [local setup](../development/local-setup.md), [ADR 0001](../decisions/ADR-0001-foundation-stack.md),
  [ADR 0002](../decisions/ADR-0002-m4-quality-gate.md), [coding practices](../../CODING_PRACTICES.md).
- Facts below were gathered read-only on 2026-10-10 at `12b1fdb` (`research-medium` M5Facts, session
  [2026-10-10-06](../development/sessions/2026-10-10-06-m5-contract.md)).

M5 turns the accepted M4 product into a release that someone can build, run, back up, and restart on one machine
by following the docs, without Rust or Node on the host. It adds packaging and release material and changes
application behaviour only where packaging requires it (serving the built frontend). Cloud, public hosting, and
domains stay out.

## Starting point (observed at `12b1fdb`)

- No Dockerfile, `.dockerignore`, release notes, runbook, screenshots, or git tags exist.
- The backend serves only `/healthz`, `/readyz`, and `/api/**`; unknown paths are a JSON 404. `tower-http` is built
  with `cors`, `trace`, and `compression-gzip` only; `ServeDir` needs its `fs` feature, which adds crates to
  `Cargo.lock` (at least `mime_guess`).
- Migrations are embedded with `sqlx::migrate!` and applied at every start; no separate migrate step exists.
- `reqwest` uses native TLS: a release build needs OpenSSL headers, and a runtime image needs `libssl3` and
  `ca-certificates`. sqlx and redis have no TLS features (D21, M6).
- `[profile.release]` already sets `lto = true`, `codegen-units = 1`, `strip = true`.
- The frontend calls a relative `/api` (`VITE_API_URL` baked at build time) and uses HTML5 history, so a static
  server needs an SPA fallback to `index.html`.
- The backend reads its configuration from the environment; `.env` is optional for the binary but required by
  compose and the `make` targets. `DATABASE_URL` and `REDIS_URL` in `.env` point at `127.0.0.1`, which a container
  cannot use for the compose services.
- The auth rate limit keys on the TCP peer address and reads no forwarding header.
- Versions: backend `0.1.0` (returned by `/healthz`), frontend `0.0.0`.
- CI builds no release artifact.

## Slices

Delivered in order; both are required for M5 acceptance.

- **M5a — Packaging:** sections 1–5 (static serving, image, compose profile, `make` targets, CI release job).
  Decisions 1–4, 7, 8.
- **M5b — Release material and drills:** sections 6–9 (version, release notes, runbook, README and screenshots,
  documented limits) and the acceptance drills. Decisions 5, 6, 9.

## Decisions

★ marks the recommended option.

1. **Packaging shape** (plan open decision 5).
   - ★ One application image: the backend serves the built frontend from the same origin (no CORS, no second
     server), with an SPA fallback. Enables the `tower-http` features `fs` and `set-header` (adds `mime_guess`,
     `http-range-header`, and related crates to `Cargo.lock`; recorded in ADR 0003).
   - Alternative: a separate static web container (for example `nginx`) serving `dist/` and proxying `/api`; no crate
     changes, two images, one more configuration file to keep secure.
   - Alternative: native release build only (`cargo build --release`, `vite build`), no application containers;
     needs Rust, Node, and OpenSSL headers on the host.
2. **Images** ★ Multi-stage build: `node:20.19.0-bookworm-slim` builds the frontend; `rust:1.99.0-bookworm` (the
   pinned toolchain, already used by the `make` container fallback) builds the backend with `--locked`; the runtime is
   `debian:bookworm-slim` with `libssl3`, `ca-certificates`, and a health probe, running as a non-root user. All base
   images are public and free; nothing is pushed to any registry.
3. **Exposure** ★ Inside its container the backend binds `0.0.0.0` (required for a published port); compose publishes
   it only as `127.0.0.1:${APP_PUBLISH_PORT}`. This is the only non-loopback `APP_HOST`, it is set by compose rather
   than `.env`, and drill 2 proves nothing listens beyond loopback on the host.
4. **Port and data** ★ The release service joins the existing compose project as profile `app` on host port `8000`
   (configurable), so it can run beside `make backend-dev` (8080), Vite (5173), and `make e2e` (8090, 5175), and it
   uses the same Postgres and Redis volumes: one machine, one database. Alternative: a separate compose project with
   its own volumes and ports, isolating release data from development and test data.
5. **Version and tag** (plan open decision 5).
   - ★ Version `1.0.0` in `backend/Cargo.toml` and `frontend/package.json`; an annotated tag `v1.0.0` on the accepted
     M5 commit, created and pushed only when the owner asks after accepting M5. No GitHub Release object.
   - Alternatives: `0.1.0` with a tag; or a version bump without any tag.
6. **Release notes and runbook** ★ `CHANGELOG.md` at the root (Keep a Changelog format; one `1.0.0` entry
   summarising M0–M5, linking the defect catalogue); `docs/operations/runbook.md` for start, stop, upgrade, logs,
   health, backup and restore, secret rotation, reset, and troubleshooting.
7. **Rate limiting behind the published port** ★ Keep keying on the peer address and document the limit: through
   Docker's loopback port publishing every client shares one auth bucket (as already through the Vite proxy). No
   forwarding-header support; this is a single-owner, loopback-only product.
8. **CI** ★ A `release` job builds the image (no push) and smoke-tests it on the clean runner: compose with throwaway
   credentials, the app profile up, `/healthz` version, `/readyz` 200, `index.html` served, a deep link served by
   the SPA fallback, `/api/unknown` a JSON 404. No secrets, no provider keys, no live providers.
9. **Clean-machine evidence** ★ Two parts: (a) the CI `release` job above on a fresh runner; (b) on this workstation,
   a fresh `git clone` into a temporary directory, run with a `PATH` that has no Rust or Node and with its own compose
   project, `.env`, and volumes, following only the README and runbook. Afterwards the throwaway project is removed
   with `docker compose down -v` **for that temporary project only** (owner approval requested here; the dev volumes
   are never touched). A separate physical machine or VM is not available to the main thread.

### Preparation values (main thread, proposed)

| Item | Value | Section |
| --- | --- | --- |
| Host port | `127.0.0.1:8000` via `APP_PUBLISH_PORT` | 3 |
| Image name | `insightboard:<version>` and `insightboard:local`, built locally only | 2 |
| Static caching | Hashed files under `/assets/` long-lived (`immutable`); `index.html` `no-cache` | 1 |
| Response headers on static files | `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `X-Frame-Options: DENY` | 1 |
| Health probe | Compose healthcheck on `/readyz` | 3 |
| Restart policy | `unless-stopped`, as for Postgres and Redis | 3 |
| Screenshots | 4–6 images under `docs/assets/screenshots/`, light and dark, desktop and 390 px, each ≤ 250 KB | 9 |

## 1. Static serving (M5a; decision 1)

- When a frontend directory is configured (for example `STATIC_DIR`), the backend serves it; when unset, behaviour is
  unchanged (development keeps Vite).
- `/api/**`, `/healthz`, and `/readyz` keep their JSON responses: an unknown `/api/...` path stays a JSON 404 and
  never falls back to `index.html`.
- Any other `GET` for a missing file returns `index.html` with 200 so deep links such as `/dashboards/<id>` load.
- Caching and headers follow the preparation values. Compression applies to static responses.
- Tests: router tests for a static file, the SPA fallback, `/api` 404 staying JSON, and the unset-directory case.

## 2. Image (M5a; decision 2)

- A `Dockerfile` (multi-stage, as in decision 2) and a `.dockerignore` that excludes `.env*` (except
  `.env.example`), `.tools/`, `target/`, `node_modules/`, `dist/`, `.git/`, logs, and Playwright output.
- The build uses `npm ci` and `cargo build --release --locked`; no network access at runtime other than the
  providers the user configures.
- The runtime runs as a non-root user, contains no secrets, and carries the migrations inside the binary.
- Provider base URLs, timeouts, and rate limits stay configurable through the environment.

## 3. Compose profile (M5a; decisions 3, 4, 7)

- Service `app` under profile `app`: depends on healthy `postgres` and `redis`; `DATABASE_URL` and `REDIS_URL` point
  at the service names and are built by compose from `POSTGRES_USER`, `POSTGRES_PASSWORD`, and `REDIS_PASSWORD`;
  `JWT_SECRET` and provider keys come from `.env` without being written into the image; `APP_HOST=0.0.0.0` inside,
  published only on `127.0.0.1`; `CORS_ORIGINS` limited to the published origin; healthcheck and restart policy
  from the preparation values.
- `make db-up` and the development flow are unchanged; Postgres and Redis keep their loopback ports.

## 4. Make targets (M5a)

- `make release` builds the image; `make up` and `make down` start and stop the full stack (`down` never removes
  volumes); `make logs` follows the app; `make backup` writes a `pg_dump` to a gitignored directory.
- `make doctor` reports whether Docker alone is enough for the release path.

## 5. CI (M5a; decision 8)

- The `release` job as in decision 8, beside the existing `backend`, `web`, and `e2e` jobs; the report or logs are
  uploaded only on failure.

## 6. Version and release notes (M5b; decisions 5, 6)

- Both manifests carry the release version; `/healthz` returns it.
- `CHANGELOG.md` per decision 6.
- The tag is created and pushed only on an explicit owner request after M5 acceptance.

## 7. Runbook (M5b; decision 6)

`docs/operations/runbook.md` covers: prerequisites (Docker with Compose, `make`, `openssl`, Git); first start from a
fresh clone; provider keys; start, stop, restart, and upgrade (rebuild and restart, migrations run on start); logs
and health; backup with `pg_dump` and a restore rehearsed into a throwaway database; rotating `JWT_SECRET` (signs
everyone out) and the Postgres and Redis passwords; reset (needs explicit confirmation, deletes data);
troubleshooting (port in use, Postgres not ready, missing keys → 503). It never asks for `docker compose config`
output, which prints secrets.

## 8. README and screenshots (M5b; decision 9)

- The README gains a "Run the release" path (Docker only) beside the development path and the screenshots from
  the preparation values.
- Screenshots come from the release build with a throwaway account and the loopback provider fixture used by
  `make e2e`: no real names, emails, keys, or personal data.
- Stale statements are corrected (for example "no Dockerfile or CI workflow yet", and 400 instead of 503 for a
  missing provider key).

## 9. Documented limits (M5b)

The README and runbook state at least: loopback only, single machine; no TLS anywhere (plain HTTP on loopback; TLS
to Postgres and Redis is D21, M6); the JWT stays in `localStorage` for 24 hours with no refresh or revocation;
registration is open to anyone who can reach the port; all clients share one auth rate-limit bucket; Weather and
News need the owner's free keys; the status widget checks only public addresses; keyboard layout editing is in the
backlog; no automatic backups.

## 10. Acceptance drills

Evidence goes to `docs/development/m5-verification.md` (main thread). Automated drills use local fixtures only;
manual provider smoke stays within the approved external targets in the roadmap.

| # | Drill | Expected |
| --- | --- | --- |
| 1 | `make release` from a clean tree; inspect the image | Builds; runs as non-root; no `.env`, `.tools/`, `node_modules/`, or source tree in the image |
| 2 | `make up`; `ss -ltn` | App on `127.0.0.1:8000`; Postgres and Redis on `127.0.0.1`; nothing on `0.0.0.0` or `::` |
| 3 | Browser on the release app (desktop and 390 px) | Register, create, add and configure a widget, rename, reload a deep link; `/api/unknown` JSON 404; `/healthz` shows the release version |
| 4 | Restart without data loss | `docker compose restart`, then `make down` and `make up`: same users and dashboards; a token issued before still works |
| 5 | Fresh clone, Docker-only `PATH`, own compose project (decision 9b) | Working app by following README and runbook alone; throwaway project removed afterwards |
| 6 | CI `release` job (decision 9a) | Green on the clean runner |
| 7 | Backup and restore | `make backup` writes a dump; restored into a throwaway database it has the same row counts |
| 8 | Manual provider smoke through the release app | One request each for weather, news, GitHub, crypto, and `https://example.com`: 200 |
| 9 | Docs check | README, runbook, changelog, and limits match `docker-compose.yml`, the Makefile, `.env.example`, and the image |
| 10 | Existing gates | `make verify`, `make e2e`, and CI `backend`, `web`, `e2e` stay green |

## 11. Out of scope

Cloud work of any kind, registries, public hosting, domains, TLS (D21, M6), reverse proxies beyond decision 1's
alternative, multiple instances, automatic backups, forwarding-header trust, a GitHub Release object, and the
unscheduled backlog in the [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) section 5.
