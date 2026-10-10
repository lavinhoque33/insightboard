# Local setup

Everything runs on this machine. No cloud account, credentials, or paid service is needed or permitted.
Dependency downloads (crates, npm packages, the Postgres and Redis images) need internet access once. Provider
widgets call public APIs at request time; see "Provider keys" below.

Commands are run from the repository root unless a `cd` is shown. Steps below match `.env.example`,
`frontend/.env.example`, `docker-compose.yml`, `backend/src/config.rs`, `backend/src/main.rs`, and
`frontend/vite.config.ts`, and the root `Makefile` as of M4a.

## Network exposure (D3, mitigated by M4 decision 4)

Every listener binds to loopback and the data services need credentials from `.env` (M4 decision 4, approved
by the owner 2026-10-10; **locally verified** on this workstation 2026-10-10 as precondition B0 in the
[baseline verification record](baseline-verification.md)):

| Listener | Binding | Credentials |
| --- | --- | --- |
| Backend `:8080` | `APP_HOST`, default `127.0.0.1` (`backend/src/config.rs`, `backend/src/main.rs`) | API; anyone who can reach it can self-register |
| Postgres `:5432` | `127.0.0.1` (`docker-compose.yml`, `'127.0.0.1:5432:5432'`) | `POSTGRES_USER` / `POSTGRES_PASSWORD` from `.env`, required by compose |
| Redis `:6379` | `127.0.0.1` (`'127.0.0.1:6379:6379'`) | `REDIS_PASSWORD` from `.env`, required by compose |
| Vite `:5173` | `127.0.0.1` (`frontend/vite.config.ts`) | n/a |

**Observed** on this workstation on 2026-10-10: firewalld zone `FedoraWorkstation` on `wlp1s0` opens
`1025-65535/tcp` and `1025-65535/udp`. Before decision 4 the services listened on all interfaces and were
therefore reachable from the local network `[INFERENCE: not probed]`. With the loopback binds they listen on
`127.0.0.1` only, and connections to the workstation's LAN address on 5173, 8080, 5432, and 6379 are refused
(observed 2026-10-10 from the workstation itself; not probed from another host). Do not
change firewall rules. Setting `APP_HOST` to a non-loopback address is a deliberate owner-approved change; the
backend does not warn about it. Through the backend, D1 (status-widget SSRF) is still open.

**Precondition B0** (loopback binds and `.env` credentials in place) is met; the defect is catalogued in the
[M0–M3 baseline contract](../architecture/m0-m3-baseline-contract.md).

## Prerequisites

| Tool | Version | Notes |
| --- | --- | --- |
| Docker + Compose v2 | any current (Docker 29.8.2, Compose v5.5.1 on this workstation) | Runs Postgres and Redis only. Add your user to the `docker` group so `sudo` is unnecessary (already the case here); the legacy instructions' `sudo docker` is not needed. |
| Node.js | 20.19.0 (`.nvmrc`); `frontend/package.json` `engines.node` is `^20.19.0 \|\| >=22.12.0` | Vite 7 needs `^20.19 \|\| >=22.12`. The Makefile puts the project Node first when `.tools/node-v20.19.0-linux-x64/bin` exists and otherwise uses host Node (`make doctor` checks the version). On this workstation the project Node lives in the gitignored `.tools/node-v20.19.0-linux-x64/bin`; put it first on `PATH` (below). Host Node 24.19.0 also satisfies Vite 7. npm comes with Node; the project uses npm, not pnpm. |
| Rust | 1.99.0, pinned by `rust-toolchain.toml` (with `rustfmt` and `clippy`); `backend/Cargo.toml` declares `rust-version = "1.89"`, the highest MSRV declared among the locked dependencies | Install [rustup](https://rustup.rs/); inside the checkout it fetches the pinned toolchain on first use. reqwest uses native TLS, so a native build needs OpenSSL development headers and `pkg-config`: **observed** on Fedora 44 that `openssl-sys` fails without `openssl-devel`. Without host Rust, `make backend-check` (and so `make verify`) falls back to a container ([testing](testing.md)), but running the backend (`make backend-dev`) and `make format` need host Rust. |
| make, awk | any | GNU make drives the targets below; `make setup` uses `awk` and `openssl` to write `.env`. |
| git, curl, openssl | any | `openssl` generates the JWT secret. |

`sqlx-cli` is not required: the backend applies migrations at startup.

## First run

Quick path (this is what `make` automates; the manual steps follow):

```sh
make setup      # npm ci; creates .env with generated secrets if it is absent
make doctor     # tool versions, .env presence, Docker access
make db-up      # Postgres and Redis on 127.0.0.1, waits until healthy
make backend-dev   # terminal 1: API on 127.0.0.1:8080
make web-dev       # terminal 2: Vite on 127.0.0.1:5173
```

| Target | What it does |
| --- | --- |
| `make help` | Lists the targets |
| `make setup` | Runs `npm ci` in `frontend/`. Creates `.env` only if it is absent: copies `.env.example`, generates `POSTGRES_PASSWORD`, `REDIS_PASSWORD` (24 random bytes, hex) and `JWT_SECRET` (32 random bytes, hex) with `openssl rand`, fills the matching `DATABASE_URL`/`REDIS_URL`, sets mode 600, and prints no secret. An existing `.env` is never read or changed. |
| `make doctor` | Reports Node, npm, Docker and Compose, cargo and rustc (or whether the container fallback will be used), openssl, whether `.env` exists (never its contents), and whether `frontend/node_modules` is installed. Exits non-zero if Node is too old or missing. |
| `make db-up` / `make db-down` | `docker compose up -d --wait postgres redis` / `docker compose stop postgres redis`; data volumes are kept. `db-up` needs `.env`. |
| `make db-reset CONFIRM=yes` | **Destructive**: `docker compose down --volumes` deletes the Postgres and Redis data. Refuses without `CONFIRM=yes`; the only target that removes volumes, and it needs owner approval to run. |
| `make backend-dev` | `cargo run` in `backend/` with the root `.env`; needs host Rust. |
| `make web-dev` | `npm run dev` in `frontend/`. |
| `make backend-check`, `make web-check`, `make verify`, `make format` | The quality gates; see [testing](testing.md). |

Manual path, when you cannot or do not want to use `make setup`: complete step 1 before starting anything, because
compose refuses to start without the secrets.

1. **Backend and compose environment.** Copy the template and fill the secrets with locally generated values,
   without printing them. Compose (interpolation) and the backend (`dotenvy`) read the same root `.env`;
   `DATABASE_URL` and `REDIS_URL` must carry the same user and passwords. Hex values are URL-safe:

   ```sh
   cp .env.example .env
   PGPASS=$(openssl rand -hex 24); REDISPASS=$(openssl rand -hex 24)
   sed -i \
     -e "s|^JWT_SECRET=.*|JWT_SECRET=$(openssl rand -hex 32)|" \
     -e "s|^POSTGRES_USER=.*|POSTGRES_USER=insightboard_app|" \
     -e "s|^POSTGRES_PASSWORD=.*|POSTGRES_PASSWORD=$PGPASS|" \
     -e "s|^REDIS_PASSWORD=.*|REDIS_PASSWORD=$REDISPASS|" \
     -e "s|<POSTGRES_USER>|insightboard_app|; s|<POSTGRES_PASSWORD>|$PGPASS|; s|<REDIS_PASSWORD>|$REDISPASS|" .env
   unset PGPASS REDISPASS
   chmod 600 .env
   ```

   `.env` is gitignored. Never commit, log, or paste it. Leave the provider keys empty unless you have free-tier
   keys (see "Provider keys"). Keep values plain: no quotes and no `${}` references.
2. **Frontend environment (optional).** `cp frontend/.env.example frontend/.env`. It sets `VITE_API_URL=/api`,
   which is also the default in `frontend/src/api/client.ts`.
3. **Start Postgres and Redis.**

   ```sh
   docker compose up -d --wait
   docker compose ps     # the postgres and redis services should be healthy
   ```

4. **Run the backend** (terminal 1). `dotenvy` finds the root `.env` from `backend/` by searching parent
   directories.

   ```sh
   cd backend
   cargo run
   ```

   Startup loads `.env` (so `RUST_LOG` in it applies), connects to Postgres, applies the embedded migrations,
   connects to Redis, and logs `Listening on 127.0.0.1:8080`. A missing or empty `DATABASE_URL`, `REDIS_URL`, or
   `JWT_SECRET` exits non-zero with an error naming the variable; an unparsable `APP_HOST`, `APP_PORT`, or
   `CORS_ORIGINS` also fails startup with an error; an unreachable Postgres or Redis exits with an error. Check it
   with `curl -s http://127.0.0.1:8080/healthz`
   (`{"status":"ok","service":"insightboard-backend","version":"0.1.0"}`; it checks no dependency).
5. **Run the frontend** (terminal 2).

   ```sh
   export PATH="$PWD/.tools/node-v20.19.0-linux-x64/bin:$PATH"   # this workstation only
   cd frontend
   npm ci
   npm run dev
   ```

6. **Open** `http://127.0.0.1:5173`, register an account, create a dashboard, and add a widget. The Vite dev
   server proxies `/api` to `http://127.0.0.1:8080` (`frontend/vite.config.ts`).

## Configuration

`.env` is read by the backend at startup (`dotenvy`, before logging initialises). Variables
(`backend/src/config.rs`, `backend/src/main.rs`):

| Variable | Default | Rule |
| --- | --- | --- |
| `APP_HOST` | `127.0.0.1` | IP address the backend binds (`std::net::IpAddr`); unparsable values fail startup. Non-loopback values are a deliberate owner-approved change. |
| `APP_PORT` | `8080` | Port only. Changing it also needs the Vite proxy target changed. |
| `POSTGRES_USER`, `POSTGRES_PASSWORD` | none, required by compose | Read by compose only; the container is initialised with them on first start. Database name is `insightboard`. |
| `REDIS_PASSWORD` | none, required by compose | Read by compose only (`--requirepass`, and `REDISCLI_AUTH` inside the container so `docker compose exec redis redis-cli ...` works without `-a`). |
| `DATABASE_URL` | none, required | `postgres://<POSTGRES_USER>:<POSTGRES_PASSWORD>@127.0.0.1:5432/insightboard`; same values as above. |
| `REDIS_URL` | none, required | `redis://:<REDIS_PASSWORD>@127.0.0.1:6379`; same password as above. |
| `JWT_SECRET` | none, required | Must be non-empty; no strength check yet (D17, M4b enforces 32 bytes); generate with `openssl rand -hex 32` (`make setup` does). |
| `GITHUB_API_TOKEN` | empty | Optional; sent to GitHub when set. Blank values count as unset. |
| `OPENWEATHER_API_KEY` | empty | Weather widget returns 400 "OpenWeather API key not configured" without it. |
| `NEWSAPI_API_KEY` | empty | News widget returns 400 "NewsAPI key not configured" without it. |
| `RUST_LOG` | `insightboard_backend=debug,tower_http=debug` | Standard `tracing` filter. The value in `.env` is honoured because `.env` loads before logging starts; `.env.example` sets `info,insightboard_backend=debug`. |
| `CORS_ORIGINS` | `http://127.0.0.1:5173,http://localhost:5173` (the Vite dev origin) | Comma-separated exact origins (`scheme://host[:port]`; no path, trailing slash, wildcard, or credentials). The backend allows only those origins, the methods GET/POST/PUT/DELETE, and the `Authorization` and `Content-Type` headers, without credentials; other origins get no CORS grant. An unparsable entry fails startup. The Vite proxy makes browser calls same-origin in development, so CORS matters only for direct cross-origin calls. |

`COINMARKETCAP_API_KEY`, `SENTRY_DSN`, and `AWS_REGION` were removed from `.env.example` because nothing read them.

The production-style URLs commented in `.env.example` (`sslmode=require`, `rediss://`) would fail: sqlx and redis
are built without TLS features (D21).

Frontend: `VITE_API_URL` (default `/api`). Vite binds `127.0.0.1` (`server.host` in `vite.config.ts`); the port is
the Vite default.

## Ports

| Port | Service | Binding |
| --- | --- | --- |
| 8080 | Backend API | `127.0.0.1` (`APP_HOST`) |
| 5173 | Vite dev server | `127.0.0.1` |
| 5432 | Postgres (compose service `postgres`) | `127.0.0.1` |
| 6379 | Redis (compose service `redis`) | `127.0.0.1` |

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
  restarts. Compose scopes them to the project, so they appear as `<project>_postgres_data` and
  `<project>_redis_data` (`insightboard_postgres_data` in a checkout directory named `insightboard`). `docker compose up -d` starts them again.
- Both services use `restart: unless-stopped`, so they start again with the Docker daemon unless you stopped
  them.
- Containers and the network are named by Compose from the checkout directory (no `container_name`, no fixed
  network name), so a second checkout gets its own stack and volumes. Both checkouts publish the same loopback
  ports (5432, 6379), so only one stack can run at a time unless the ports are remapped (an owner-approved
  change). Use `docker compose exec postgres|redis ...` rather than `docker exec <name>`.
- **Destructive operations need owner approval:** `docker compose down -v`, removing either volume, or dropping
  the database deletes every user and dashboard.

Inspect data without exposing secrets:

```sh
docker compose exec -T postgres sh -c 'psql -U "$POSTGRES_USER" -d insightboard \
  -c "SELECT id, email, created_at FROM users ORDER BY created_at DESC LIMIT 20;"'
docker compose exec -T postgres sh -c 'psql -U "$POSTGRES_USER" -d insightboard \
  -c "SELECT id, user_id, name, updated_at FROM dashboards ORDER BY updated_at DESC LIMIT 20;"'
docker compose exec -T redis redis-cli --scan --pattern 'weather:*'
docker compose exec -T redis redis-cli TTL '<key>'
```

## Troubleshooting

- **`cargo: command not found`** — install Rust with rustup and open a new shell; for checks only, `make
  backend-check` uses the container fallback ([testing](testing.md)).
- **Backend exits with `DATABASE_URL must be set to a non-empty value`** (or `REDIS_URL`, `JWT_SECRET`) — `.env` is
  missing, empty for that variable, or not found; run `make setup` or create it at the repository root as above.
- **Backend exits with `CORS_ORIGINS has an invalid origin`** — fix the entry to `scheme://host[:port]` or unset it.
- **Backend exits connecting to Postgres or Redis** — `docker compose ps`; wait for `healthy`, then restart the
  backend. The backend does not reconnect at startup.
- **Cannot connect to the Docker daemon** — `systemctl status docker`; start it, and check `docker` group
  membership with `id`.
- **Port 5432 or 6379 already in use** — another Postgres or Redis is running; stop it. Remapping means editing
  `docker-compose.yml` and `DATABASE_URL`/`REDIS_URL` together, which is an owner-approved change.
- **`password authentication failed for user "..."`** — an existing `postgres_data` volume keeps the
  credentials it was initialised with; changing `POSTGRES_USER`/`POSTGRES_PASSWORD` in `.env` does not change
  them. Rotate the password in place without deleting data, using the old user and a prompt so the password is
  not echoed or placed on the command line (`psql` inside the container connects through the local socket):

  ```sh
  docker compose exec postgres psql -U <old-user> -d insightboard
  # inside psql:  \password <old-user>      (prompts for the new password, no echo)
  ```

  Equivalent: `ALTER ROLE <old-user> PASSWORD '...'` with the password read from stdin, never typed into
  shell history. Then put the same value in `POSTGRES_PASSWORD` and `DATABASE_URL`. Keep the old user name in
  `.env` unless you also rename the role (`ALTER ROLE <old-user> RENAME TO <new-user>;`, then reset its
  password). Resetting the volume with `docker compose down -v` deletes all data and needs owner approval.
- **`NOAUTH` or `WRONGPASS` from Redis, or compose says `set REDIS_PASSWORD in .env`** — `.env` is missing the
  value, or `REDIS_URL` carries a different password than `REDIS_PASSWORD`. Recreate the container after
  changing it: `docker compose up -d redis` (the append-only data is kept).
- **Weather or News show "not configured"** — expected without keys; after adding keys, restart the backend.
- **GitHub or Crypto show "External service error" (502)** — the provider failed, rate-limited, or returned an
  unexpected value; the backend hides the cause from the client and logs it.
- **Status widget shows "No response from server"** — the backend checks URLs one at a time with 5 s each, and
  the client gives up after 10 s (D20); use fewer URLs.
- **Signed out unexpectedly** — a 401 on any request other than login or register clears the stored session
  and reloads `/login`; tokens last 7 days (24 hours after M4b).
- **`npm run dev` or `npm ci` fails on engine or syntax errors** — Node is older than 20.19; put the `.tools/`
  Node first on `PATH` or use `nvm use`.

## What is not set up

No Dockerfile, application containers, CI workflow (M4c), or production configuration. The legacy run
instructions remain readable at
[`manual run instructions.md` at `f15c6a3`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/manual%20run%20instructions.md);
their absolute paths, `sudo docker`, and `httpstat.us` target no longer apply.
