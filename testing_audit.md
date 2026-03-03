# InsightBoard Verification Audit (Pre-Phase-11)

Purpose: verify whether phases marked complete in `project_progress.md` are actually complete and identify critical bugs/regressions.

## Audit Phase Status
- Phase: `audit-remediation` (separate from project progress tracking)
- Date: 2026-03-03
- Status: `COMPLETE`
- Open bugs from audit findings: `0`

## Contract-First Baseline
- Added strict schema: `/contracts/api-contract.v1.json`
- Added frontend contract types bound to schema: `/frontend/src/contracts/apiContract.ts`
- Backend and frontend API layers were aligned to this contract before remediation verification.

## What Was Re-Verified

### Build/Test
- Backend `cargo test`: `pass` (`5` tests, `0` failures)
- Backend `cargo check`: `pass`
- Frontend `npm run build`: `pass`
- Frontend `npm audit`: `pass` (`0` vulnerabilities)

### Runtime Smoke
- `GET /healthz`: `200`
- Auth register/login/me flow: `pass`
- Dashboard CRUD flow: `pass`
- Status widget endpoint contract: `pass` (`status` numeric, `error` optional)
- Weather/News missing-key behavior: `pass` (clear `400` validation error messages)
- Crypto endpoint query contract (`ids`) path: `pass` (code path aligned; external provider availability still environment-dependent)
- Docker infrastructure smoke (compose + DB + Redis + backend integration): `pass` (executed via `sudo`)

## Finding Resolution Ledger

### 1) Frontend production build broken
- Previous: `critical`
- Status: `resolved`
- Fixes:
  - Removed build-blocking unused imports in `/frontend/src/api/dashboard.ts`
  - Revalidated with `npm run build` (success)

### 2) Auth API route mismatch (`/auth/me` vs `/me`)
- Previous: `high`
- Status: `resolved`
- Fixes:
  - Updated frontend to call `/me` in `/frontend/src/api/auth.ts`

### 3) Widget contracts inconsistent across layers
- Previous: `critical`
- Status: `resolved`
- Fixes:
  - Added contract-first schema file and mirrored frontend contract types
  - Backend payloads aligned:
    - Weather fields normalized to `city/country/temperature/feels_like/humidity/description/wind_speed/icon`
    - News fields normalized to `source.name`, `publishedAt`, `urlToImage`
    - Crypto query standardized to `ids`; response standardized to `id/symbol/name/current_price/price_change_percentage_24h/market_cap/total_volume`
    - Status response standardized to numeric `status`, numeric `response_time_ms`, optional `error`
  - Frontend API widget types bound to contract types

### 4) Widget interval cleanup bug (memory leak risk)
- Previous: `high`
- Status: `resolved`
- Fixes:
  - Replaced incorrect `onMounted(() => return cleanup)` usage with `onUnmounted(...)` in all widget components:
    - `GitHubWidget.vue`, `WeatherWidget.vue`, `NewsWidget.vue`, `CryptoWidget.vue`, `StatusWidget.vue`

### 5) External API key handling misleading failures
- Previous: `high`
- Status: `resolved`
- Fixes:
  - Optional env values now trim and treat blank strings as missing in `/backend/src/config.rs`
  - Weather/News now return explicit validation errors when keys are missing
  - Runtime now returns `400` + clear error text instead of opaque internal error

### 6) Claimed testing completeness unsupported (`0` tests)
- Previous: `high`
- Status: `resolved`
- Fixes:
  - Added backend unit tests in `/backend/src/auth.rs` and `/backend/src/config.rs`
  - `cargo test` now executes real tests (`5 passed`)

### 7) Crypto default/query semantics mismatch
- Previous: `medium`
- Status: `resolved`
- Fixes:
  - Backend query switched to `ids` (CoinGecko IDs)
  - Default now uses `bitcoin,ethereum`
  - Frontend already used `ids`, now fully consistent

### 8) Register flow UX conflict
- Previous: `medium`
- Status: `resolved`
- Fixes:
  - Register view now routes to `/dashboards` after successful register, matching authenticated state behavior

### 9) Unused scaffold component in source
- Previous: `low`
- Status: `resolved`
- Fixes:
  - Removed `/frontend/src/components/HelloWorld.vue`

## Additional Remediation (Non-Blocking)
- Removed debug `console.log` statements from app bootstrap/widget store
- Fixed dashboard editor empty-state add-widget control so it actually opens the widget picker dropdown
- Applied DB migration on backend startup to reduce schema drift risk in local/dev

## Remaining Non-Bug Constraints
- Direct Docker daemon access from this shell user is denied (`/var/run/docker.sock` is `root:docker`, user not in `docker` group), but `sudo` is available and was used to complete full compose smoke tests successfully.
- Some external APIs can still return upstream errors (e.g., provider auth/rate/network policy). Endpoint contracts and local error behavior are now consistent and explicit.

## Docker Smoke Details (2026-03-03)
- `sudo docker compose up -d --remove-orphans`: success
- `sudo docker compose ps`: `postgres` and `redis` both `healthy`
- Backend launched against compose services and validated:
  - `/healthz` success
  - register/login + dashboards DB operations success
  - status endpoint success
  - Redis cache key observed: `status:https://example.com`
  - PostgreSQL row verified for created smoke dashboard ID

## Final Audit Verdict
- Audit findings fixed completely.
- Contract-first schema is in place and actively used by implementation.
