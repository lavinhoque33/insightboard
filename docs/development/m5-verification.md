# M5 verification record

Evidence for the [M5 local-release contract](../architecture/m5-local-release-contract.md) (accepted 2026-10-10;
decisions in [ADR 0003](../decisions/ADR-0003-m5-local-release.md)). Everything below ran on this workstation (Fedora
44, Docker 29.8.2, Compose 5.5.1) on 2026-10-10 unless it names GitHub Actions. **Observed** = run or inspected by
the main thread; **reported** = a delegated agent's result, with what the main thread inspected stated beside it.
Session: [2026-10-10-06](sessions/2026-10-10-06-m5-contract.md).

## M5a — packaging

Implemented by `implement-heavy` (M5aPackaging), verified by `verify` (M5aVerify), reviewed and committed by the
main thread as `9a14027`.

| Check | Result |
| --- | --- |
| `make verify` after M5a | **Reported** pass: 94 backend tests, Vitest 91 in 12 files; fmt, clippy, ESLint, Prettier, type-check, build clean |
| Static serving battery on `127.0.0.1:8088` | **Reported** pass: `/api`, `/api/`, `/api/unknown`, deep API paths → JSON 404; raw, encoded, and backslash traversal never serve a file outside `dist/`; `HEAD /` `no-cache` with the three security headers; `POST /` → 405 JSON; foreign-origin preflight gets no CORS grant; hashed assets `immutable`, gzip on request; missing asset → JSON 404 without cache header |
| Finding F1 (medium) | A path segment over 255 bytes or with `%00` returned an empty 500 and logged an ERROR per request. Fixed by M5aPackaging: `unservable()` guard in `backend/src/app.rs:179-201` answers a JSON 404 under `/assets/` and `index.html` elsewhere before `ServeDir`; router test `unservable_file_names_are_treated_as_missing_files`. **Reported** live recheck 200/404 as expected and 0 ERROR lines; **observed** guard code inspected and the test passing in drill 10 |
| Finding F2 (low) | With `STATIC_DIR` set, the security headers also attach to JSON 404/405 responses. Accepted as harmless |
| Backup, persistence, final state | **Reported** pass; repeated by the main thread in drills 4 and 7 |

## M5b — release material and drills

Version `1.0.0` (`implement-light` VersionBump: manifests and lockfile root entries only; **observed** diff and
`/healthz`), README rewrite, `CHANGELOG.md`, `docs/operations/runbook.md`, screenshots (main thread).

### Drills

| # | Drill | Expected | Observed | Result |
| --- | --- | --- | --- | --- |
| 1 | `make release` from a clean tree; inspect the image | Builds; non-root; no `.env`, `.tools/`, `node_modules/`, or source tree | Main thread, clean tree at `cdb9359`: tagged `insightboard:local` and `insightboard:1.0.0`, 146 MB. `Config.User` `10001:10001`; entrypoint `/app/insightboard-backend`; image env only `PATH`, `STATIC_DIR=/app/dist`, `APP_PORT=8080`. `/app` holds the binary (root-owned, 755) and `dist/` only. `find / -xdev` for `.env*`, `.tools`, `node_modules`, `*.rs`, `Cargo.toml`, `package.json`, `target`: nothing | Pass |
| 2 | `make up`; `ss -ltn` | App on `127.0.0.1:8088`; Postgres and Redis on `127.0.0.1`; nothing on `0.0.0.0` or `::` | Main thread: `ss -ltn` before and after `make up` differ only by `127.0.0.1:8088`; stack ports `127.0.0.1:5432`, `127.0.0.1:6379`, `127.0.0.1:8088`; no wildcard listener on them. All three containers healthy; `/healthz` → `"version":"1.0.0"` | Pass |
| 3 | Browser on the release app, desktop and 390 px | Register, create, add and configure a widget, rename, deep-link reload; `/api/unknown` JSON 404; `/healthz` shows the version | **Reported** by `verify-browser` (M5Drill3Browser), harness Chromium at 1440×900 and 390×844: theme switch, register 201, create, Crypto `bitcoin` added through the configuration dialog with live data, Configure shows the saved value, rename persists after reload, deep link served by the SPA fallback (200 `text/html`, also with cache disabled), `/api/unknown` 404 JSON, `/healthz` 1.0.0; keyboard: header tab order, account menu and add-widget dialog open and close by Enter, Space, Escape with visible focus and no trap; 390 px one-column read-only notice. 0 console errors, no failed requests besides the intended 404. Main thread inspected the editor and 390 px screenshots | Pass |
| 4 | Restart without data loss | `docker compose restart`, then `make down` and `make up`: same data; an earlier token works | Main thread: a token issued before the restarts created dashboard "Drill 4 persistence". `docker compose --profile app restart` (plain `restart` skips profile services): users 64, dashboards 65 before and after; `/api/me` 200 and the dashboard read back. `make down` (volumes `insightboard_postgres_data` and `insightboard_redis_data` kept), `make up`: same counts, same token 200, same dashboard | Pass |
| 5 | Fresh clone, Docker-only `PATH`, own compose project | Working app by README and runbook alone; throwaway project removed | Main thread: dev stack stopped with `make down` to free 5432, 6379, 8088; `env -i` shell with a `PATH` of `/usr/bin` links minus `node`, `npm`, `npx`, `cargo*`, `rust*`. `git clone` of the public origin into a directory named `ibfresh` (compose project `ibfresh`, so its volumes cannot touch the dev project) at `cdb9359`; then exactly the README commands: `make setup-env` (`.env` mode 600), `make release` (138 s), `make up` (all healthy). `/healthz` 1.0.0, `/readyz` 200, `/` and a deep link 200 `text/html`, `/api/unknown` 404 JSON, register 201, dashboard created and listed, crypto 200, weather without a key 503 "not configured" as the runbook says. With an empty `HOME`, `make doctor` reports no cargo and `make -n setup-env release up` invokes only `docker`. Removed with `docker compose -p ibfresh --profile app down -v` (only `ibfresh_*` volumes and network); dev volumes intact; `make db-up` restored the dev stack with users and dashboards unchanged (64, 65) | Pass |
| 6 | CI `release` job | Green on the clean runner | GitHub Actions run 38096580273 (`f998146`): backend, web, e2e, release success (release 2 min 56 s). Run 38097611740 (`cdb9359`, version 1.0.0): all four success | Pass |
| 7 | Backup and restore | Dump written; restored into a throwaway database with the same row counts | Main thread, runbook commands verbatim: `make backup` wrote `backups/insightboard-20261011T000448Z.dump` (20 739 bytes, file 600, directory 700, ignored by Git); `createdb restore_check`, `pg_restore --no-owner` exit 0; users 63 and dashboards 62 in both `insightboard` and `restore_check`; `dropdb restore_check`; dump deleted | Pass |
| 8 | Manual provider smoke through the release app | One request each for weather, news, GitHub, crypto, `https://example.com`: 200 | Main thread, `127.0.0.1:8088` with the owner's keys: weather `London` 200 (London, GB, few clouds); news `technology` 200 (10 articles); GitHub `octocat` 200 with an empty list; crypto `ethereum` 200 (price present); status `https://example.com` 200 with check result 200. GitHub also returned an empty list for `rust-lang` through the app, and one direct request to `api.github.com` for the same account returned `[]`, so the backend passes GitHub's answer through. Non-empty GitHub parsing is covered by `tests::providers` and the Playwright fixture | Pass (see O1) |
| 9 | Docs check | README, runbook, changelog, limits match compose, Makefile, `.env.example`, image | **Reported** by `verify` (M5DocsCheck): targets, ports, variables, paths, modes, rate limits, TTLs, timeouts, and version match; runbook covers every contract section 7 topic and never asks for `docker compose config`; all contract section 9 limits stated; 16 changelog claims match source; README links and images resolve. Findings fixed by the main thread: stale "no Dockerfile" and three-job CI text in `docs/architecture/system-overview.md`; `awk` missing from the README prerequisites; `/healthz` example version `0.1.0` in `local-setup.md`; stale gate-status paragraph in `testing.md`; status-widget limit added to the runbook Limits list. Changelog heading gets its date on acceptance | Pass after fixes |
| 10 | Existing gates | `make verify`, `make e2e`, CI backend, web, e2e green | Main thread at `cdb9359`: `make verify` exit 0 (95 backend tests, Vitest 91 in 12 files, build); `make e2e` exit 0 (2 passed). CI runs in drill 6 | Pass |

### Screenshots

Six WebP images in `docs/assets/screenshots/` (light and dark, 1440 px and 390 px, 14–66 KB each), shown in the
README. Captured with a throwaway account (`demo-…@example.com`) from the release-built frontend (`npm run build`,
the same `dist/` the image copies) served by the backend with `STATIC_DIR` on `127.0.0.1:8095`, against a throwaway
loopback demo fixture (`127.0.0.1:18190`, `18191`) with dummy keys and fake data; Redis database 1 kept the demo
cache apart. **Deviation from contract section 8**, which names the `make e2e` fixture: that fixture's data ("Fixture
headline", one event) is too sparse for a representative image. The safety properties are unchanged: no live
providers, no real names, emails, or keys. The fixture and capture scripts were deleted after capture.

### Observations

- **O1 — GitHub public events.** `api.github.com/users/<name>/events/public` returned `[]` for `octocat` and
  `rust-lang` on 2026-10-10; the widget shows its empty state. Not an InsightBoard defect.
- **O2 — `make doctor` exit status.** In a Docker-only environment it prints "Docker alone is enough" and exits 1
  because Node is missing. The runbook now says so; the release path is unaffected.
- **O3 — Makefile adds `~/.cargo/bin` to `PATH`** when present (M4a), so `make doctor` with the real `HOME` saw host
  cargo during drill 5. The release recipes call only `docker`, shown by the dry run with an empty `HOME`.
- **O4 — Shared build cache.** Drill 5's `make release` could reuse this workstation's Docker build cache; the cold
  build on a clean machine is drill 6.
- **O5 — Dev backend after a stack restart.** The development backend's `/readyz` returned 503 for under a minute
  after Postgres and Redis came back, then 200 without a restart.
- **Test data.** The dev database holds the throwaway accounts from these drills (`drill3-…`, `drill8-…`,
  `drill8b-…`, `drill8c-…`, `demo-…`, `dbg-…`, `verify-m5a-…`, `m5a-…`) besides earlier test users.

### External requests

All within the approved targets and at human scale: OpenWeather 1, NewsAPI 1, CoinGecko 3 (drills 3, 5, 8), GitHub 3
(two through the app, one direct comparison), `https://example.com` 2 (drill 8 and its follow-up summary), plus one
read-only `git ls-remote` and the drill 5 `git clone` of the public repository. Five early drill 8 requests used a
wrong path (`/api/widgets/…`) and got JSON 404s without reaching any provider.
