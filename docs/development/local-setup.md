# Local setup

Everything runs on this machine. No cloud account, credentials, or paid service is needed or permitted.
Dependency downloads (crates, npm packages, the Postgres and Redis images) need internet access once. Provider
widgets call public APIs at request time; see "Provider keys" below.

Commands are run from the repository root unless a `cd` is shown. Steps below match `.env.example`,
`frontend/.env.example`, `docker-compose.yml`, `backend/src/config.rs`, `backend/src/main.rs`, and
`frontend/vite.config.ts` at `f15c6a3`.

## Warning: network exposure (D3) — read before starting anything

As built, InsightBoard does not stay on loopback:

| Listener | Binding at `f15c6a3` | Credentials |
| --- | --- | --- |
| Backend `:8080` | `0.0.0.0` (`backend/src/main.rs:90`), not configurable | API; anyone can self-register |
| Postgres `:5432` | all interfaces (`docker-compose.yml`, `'5432:5432'`) | `postgres` / `postgres` |
| Redis `:6379` | all interfaces (`'6379:6379'`) | none |
| Vite `:5173` | Vite default `localhost` | n/a |

**Observed** on this workstation on 2026-10-10: firewalld zone `FedoraWorkstation` on `wlp1s0` opens
`1025-65535/tcp` and `1025-65535/udp`. The three listeners above are therefore reachable from the local network
while they run `[INFERENCE: not probed]`. Through the backend, D1 (status-widget SSRF) is reachable too.

**Precondition B0** for every baseline drill, and the rule for any run on this workstation: do not start
`docker compose up` or the backend until the exposure is resolved by owner-approved loopback-only binds for the
compose ports and the backend (M4 decision 4, requested before the drills) or by another owner-approved
mitigation. Do not change firewall rules, `docker-compose.yml`, or backend code to work around this without that
approval. The defect is catalogued in the [M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md).

## Prerequisites

| Tool | Version | Notes |
| --- | --- | --- |
| Docker + Compose v2 | any current (Docker 29.8.2, Compose v5.5.1 on this workstation) | Runs Postgres and Redis only. Add your user to the `docker` group so `sudo` is unnecessary (already the case here); the legacy instructions' `sudo docker` is not needed. |
| Node.js | 20.19.0 (`.nvmrc`) | Vite 7 needs `^20.19 \|\| >=22.12`. On this workstation the project Node lives in the gitignored `.tools/node-v20.19.0-linux-x64/bin`; put it first on `PATH` (below). Host Node 24.19.0 also satisfies Vite 7. npm comes with Node; the project uses npm, not pnpm. |
| Rust | stable via [rustup](https://rustup.rs/); no version pinned (D12) | Built with rustc 1.99.0 in the 2026-10-10 baseline. This workstation has no host Rust: backend **checks** can run in a container ([testing](testing.md)), but running the backend needs host Rust. reqwest uses native TLS; the legacy verification **reported** needing OpenSSL development headers (Fedora: `openssl-devel` and `pkg-config` `[INFERENCE]`). |
| git, curl, openssl | any | `openssl` generates the JWT secret. |

`sqlx-cli` is not required: the backend applies migrations at startup.

## First run

Satisfy precondition B0 (warning above) first.

1. **Backend environment.** Copy the template and replace the placeholder JWT secret with a locally generated
   one, without printing it:

   ```sh
   cp .env.example .env
   sed -i "s|^JWT_SECRET=.*|JWT_SECRET=$(openssl rand -hex 32)|" .env
   ```

   `.env` is gitignored. Never commit, log, or paste it. Leave the provider keys empty unless you have free-tier
   keys (see "Provider keys").
2. **Frontend environment (optional).** `cp frontend/.env.example frontend/.env`. It sets `VITE_API_URL=/api`,
   which is also the default in `frontend/src/api/client.ts`.
3. **Start Postgres and Redis.**

   ```sh
   docker compose up -d
   docker compose ps     # insightboard-postgres and insightboard-redis should become healthy
   ```

4. **Run the backend** (terminal 1). `dotenvy` finds the root `.env` from `backend/` by searching parent
   directories `[INFERENCE: dotenvy behaviour]`.

   ```sh
   cd backend
   cargo run
   ```

   Startup connects to Postgres, applies the embedded migrations, connects to Redis, and logs
   `Listening on 0.0.0.0:8080`. A missing `DATABASE_URL`, `REDIS_URL`, or `JWT_SECRET` panics at startup; an
   unreachable Postgres or Redis exits with an error. Check it with `curl -s http://localhost:8080/healthz`
   (`{"status":"ok","service":"insightboard-backend","version":"0.1.0"}`; it checks no dependency).
5. **Run the frontend** (terminal 2).

   ```sh
   export PATH="$PWD/.tools/node-v20.19.0-linux-x64/bin:$PATH"   # this workstation only
   cd frontend
   npm ci
   npm run dev
   ```

6. **Open** `http://localhost:5173`, register an account, create a dashboard, and add a widget. The Vite dev
   server proxies `/api` to `http://localhost:8080` (`frontend/vite.config.ts`).

## Configuration

`.env` is read by the backend at startup (`dotenvy`, after logging has initialised). Variables at `f15c6a3`
(`backend/src/config.rs:18-34`, `backend/src/main.rs:38-48`):

| Variable | Default | Rule |
| --- | --- | --- |
| `APP_PORT` | `8080` | Port only; the host is always `0.0.0.0` (D3). Changing it also needs the Vite proxy target changed. |
| `DATABASE_URL` | none, required | `postgres://postgres:postgres@localhost:5432/insightboard` matches compose. |
| `REDIS_URL` | none, required | `redis://127.0.0.1:6379` matches compose. |
| `JWT_SECRET` | none, required | No strength check (D17); generate as above. |
| `GITHUB_API_TOKEN` | empty | Optional; sent to GitHub when set. Blank values count as unset. |
| `OPENWEATHER_API_KEY` | empty | Weather widget returns 400 "OpenWeather API key not configured" without it. |
| `NEWSAPI_API_KEY` | empty | News widget returns 400 "NewsAPI key not configured" without it. |
| `COINMARKETCAP_API_KEY` | empty | Read but unused; Crypto uses CoinGecko without a key (D18). |
| `RUST_LOG` | `insightboard_backend=debug,tower_http=debug` | The `.env` value is ignored because logging starts before `.env` loads (D18). Set it in the shell instead: `RUST_LOG=info cargo run`. |
| `CORS_ORIGINS`, `SENTRY_DSN`, `AWS_REGION` | commented out | Never read. CORS is permissive (D4). |

The production-style URLs commented in `.env.example` (`sslmode=require`, `rediss://`) would fail: sqlx and redis
are built without TLS features (D21).

Frontend: `VITE_API_URL` (default `/api`). Vite has no host or port overrides in `vite.config.ts`.

## Ports

| Port | Service | Binding today | Target (M4a) |
| --- | --- | --- | --- |
| 8080 | Backend API | `0.0.0.0` | `127.0.0.1` |
| 5173 | Vite dev server | `localhost` | `127.0.0.1` |
| 5432 | Postgres (`insightboard-postgres`) | all interfaces | `127.0.0.1` |
| 6379 | Redis (`insightboard-redis`) | all interfaces | `127.0.0.1` |

## Provider keys

Keys are optional and must be free-tier keys the owner supplies; never buy a plan.

- **No keys:** Weather and News show their "not configured" errors; GitHub works unauthenticated within
  GitHub's anonymous rate limit; Crypto uses CoinGecko without a key; Status needs no key.
- **With keys:** add them to `.env` and restart the backend.
- Manual provider checks stay at human scale (a handful of requests). Status-widget checks use only
  `https://example.com`; `httpstat.us`, used by the legacy run instructions, is not approved. Automated tests
  never call providers.
- Weather and News keys travel in provider query strings and can reach backend logs on transport errors (D7);
  do not share backend logs from runs with keys set.

## Stop, restart, and data

- Stop the backend and Vite with Ctrl-C.
- `docker compose stop` stops the containers; `docker compose down` removes them. Both keep the named volumes
  `postgres_data` (users and dashboards) and `redis_data` (cache, append-only file enabled), so data survives
  restarts. `docker compose up -d` starts them again.
- Both services use `restart: unless-stopped`, so they start again with the Docker daemon unless you stopped
  them.
- Container names are fixed (`insightboard-postgres`, `insightboard-redis`), so a second checkout cannot run
  its own stack alongside this one.
- **Destructive operations need owner approval:** `docker compose down -v`, removing either volume, or dropping
  the database deletes every user and dashboard.

Inspect data without exposing secrets:

```sh
docker compose exec -T postgres psql -U postgres -d insightboard \
  -c "SELECT id, email, created_at FROM users ORDER BY created_at DESC LIMIT 20;"
docker compose exec -T postgres psql -U postgres -d insightboard \
  -c "SELECT id, user_id, name, updated_at FROM dashboards ORDER BY updated_at DESC LIMIT 20;"
docker exec insightboard-redis redis-cli --scan --pattern 'weather:*'
docker exec insightboard-redis redis-cli TTL '<key>'
```

## Troubleshooting

- **`cargo: command not found`** — install Rust with rustup and open a new shell; for checks only, use the
  container command in [testing](testing.md).
- **Backend panics with `DATABASE_URL must be set`** (or `REDIS_URL`, `JWT_SECRET`) — `.env` is missing or not
  found; create it at the repository root as above.
- **Backend exits connecting to Postgres or Redis** — `docker compose ps`; wait for `healthy`, then restart the
  backend. The backend does not reconnect at startup.
- **Cannot connect to the Docker daemon** — `systemctl status docker`; start it, and check `docker` group
  membership with `id`.
- **Port 5432 or 6379 already in use** — another Postgres or Redis is running; stop it. Remapping means editing
  `docker-compose.yml` and `DATABASE_URL`/`REDIS_URL` together, which is an owner-approved change.
- **`password authentication failed for user "postgres"`** — an existing `postgres_data` volume was initialised
  with other credentials. Resetting it with `docker compose down -v` deletes all data and needs owner approval.
- **Weather or News show "not configured"** — expected without keys; after adding keys, restart the backend.
- **GitHub or Crypto show "External service error" (502)** — the provider failed, rate-limited, or returned an
  unexpected value; the backend hides the cause from the client and logs it.
- **Status widget shows "No response from server"** — the backend checks URLs one at a time with 5 s each, and
  the client gives up after 10 s (D20); use fewer URLs.
- **Signed out unexpectedly** — a 401 on any request other than login or register clears the stored session
  and reloads `/login`; tokens last 7 days.
- **`npm run dev` or `npm ci` fails on engine or syntax errors** — Node is older than 20.19; put the `.tools/`
  Node first on `PATH` or use `nvm use`.

## What is not set up

No `Makefile`, Dockerfile, application containers, CI workflow, or production configuration. The legacy run
instructions remain readable at
[`manual run instructions.md` at `f15c6a3`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/manual%20run%20instructions.md);
their absolute paths, `sudo docker`, and `httpstat.us` target no longer apply. M4a proposes loopback binds,
a `Makefile` with `setup`, `doctor`, and `db-*` targets, and pinned toolchains
([M4 contract](../architecture/m4-quality-gate-contract.md), **proposed**).
