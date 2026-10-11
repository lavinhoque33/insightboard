# Local setup

Everything runs on this machine. No cloud account, credentials, or paid service is needed or permitted.
Dependency downloads (crates, npm packages, the Postgres and Redis images) need internet access once. Provider
widgets call public APIs at request time; see "Provider keys" below.

Commands are run from the repository root unless a `cd` is shown. Steps below match `.env.example`,
`frontend/.env.example`, `docker-compose.yml`, `backend/src/config.rs`, `backend/src/main.rs`,
`frontend/vite.config.ts`, the root `Makefile`, `Dockerfile`, and `.dockerignore` as of M5a.

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
| Release app `:8088` (`make up`, M5) | Published as `127.0.0.1:${APP_PUBLISH_PORT:-8088}` (`docker-compose.yml`); inside its container the backend binds `0.0.0.0:8080`, which is reachable only through that loopback publication | API and frontend; anyone who can reach it can self-register |
| `make e2e` only: fixtures `:18090`/`:18091`, backend `:8090`, Vite `:5175` | `127.0.0.1` (`frontend/playwright.config.ts`, `frontend/e2e/fixtures/server.mjs`); they stop when the run ends | Fixture endpoints accept only the run's dummy keys; the backend uses a throwaway e2e `JWT_SECRET` |

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
| Docker + Compose v2 | any current (Docker 29.8.2, Compose v5.5.1 on this workstation) | Runs Postgres and Redis, and builds and runs the release image ([Run the release](#run-the-release-docker-only)). Add your user to the `docker` group so `sudo` is unnecessary (already the case here); the legacy instructions' `sudo docker` is not needed. |
| Node.js | 20.19.0 (`.nvmrc`); `frontend/package.json` `engines.node` is `^20.19.0 \|\| >=22.12.0` | Vite 7 needs `^20.19 \|\| >=22.12`. The Makefile puts the project Node first when `.tools/node-v20.19.0-linux-x64/bin` exists and otherwise uses host Node (`make doctor` checks the version). On this workstation the project Node lives in the gitignored `.tools/node-v20.19.0-linux-x64/bin`; put it first on `PATH` (below). Host Node 24.19.0 also satisfies Vite 7. npm comes with Node; the project uses npm, not pnpm. |
| Rust | 1.99.0, pinned by `rust-toolchain.toml` (with `rustfmt` and `clippy`); `backend/Cargo.toml` declares `rust-version = "1.89"`, the highest MSRV declared among the locked dependencies | Install [rustup](https://rustup.rs/); inside the checkout it fetches the pinned toolchain on first use. Since M6 the backend uses rustls only (no `openssl-sys` in `cargo tree`), so a native build no longer needs OpenSSL development headers. Without host Rust, `make backend-check` (and so `make verify`) falls back to a container ([testing](testing.md)), but running the backend (`make backend-dev`) and `make format` need host Rust. |
| make, awk | any | GNU make drives the targets below; `make setup` and `make setup-env` use `awk` and `openssl` to write `.env`. |
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
| `make doctor` | Reports Node, npm, Docker and Compose, cargo and rustc (or whether the container fallback will be used), openssl, whether `.env` exists (never its contents), whether `frontend/node_modules` is installed, and on a final `release:` line whether Docker alone is enough for the release path. Exits non-zero if Node is too old or missing (development needs it; the release does not). |
| `make db-up` / `make db-down` | `docker compose up -d --wait postgres redis` / `docker compose stop postgres redis`; data volumes are kept. `db-up` needs `.env`. |
| `make db-reset CONFIRM=yes` | **Destructive**: `docker compose --profile app down --volumes` removes the containers (including the release app) and deletes the Postgres and Redis data. Refuses without `CONFIRM=yes`; the only target that removes volumes, and it needs owner approval to run. |
| `make backend-dev` | `cargo run` in `backend/` with the root `.env`; needs host Rust. |
| `make web-dev` | `npm run dev` in `frontend/`. |
| `make backend-check`, `make web-check`, `make verify`, `make format` | The quality gates; see [testing](testing.md). |
| `make e2e` | Playwright browser journeys on their own loopback ports (fixture 18090/18091, backend 8090, Vite 5175), next to running dev services; needs `make db-up`, host Rust, and a one-time `cd frontend && npx playwright install chromium`. See [testing](testing.md#browser-journeys-playwright). |
| `make lambda-build` | M6, Docker only: builds the frontend and a static x86_64 musl `bootstrap` with `--features lambda` in containers (`backend/Dockerfile.lambda`) and writes `backend/target/lambda/insightboard-lambda.zip` (`bootstrap`, `static/`, `certs/rds-global-bundle.pem`). No AWS tool or credentials; nothing is uploaded. |
| `make infra-check` / `make infra-synth` | M6, synth only: `npm ci`, type-check, tests, the guarded offline synth (cdk-nag gate), and `cfn-lint` in a container for `infra/`; `infra-synth` runs the synth alone. Both need `make lambda-build` first. See [`infra/README.md`](../../infra/README.md). |

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
   `JWT_SECRET` exits non-zero with an error naming the variable; a `JWT_SECRET` shorter than 32 bytes, or an
   unparsable `APP_HOST`, `APP_PORT`, `CORS_ORIGINS`, base URL, timeout, rate limit, or fixture entry also fails
   startup with an error naming it; an unreachable Postgres exits with an error. An unreachable Redis only logs a
   warning: caching stays off until Redis answers. Check it with `curl -s http://127.0.0.1:8080/healthz`
   (`{"status":"ok","service":"insightboard-backend","version":"1.0.0"}`; it checks no dependency) and
   `curl -s http://127.0.0.1:8080/readyz` (`{"status":"ready"}` when Postgres and Redis answer, 503 otherwise).
5. **Run the frontend** (terminal 2).

   ```sh
   export PATH="$PWD/.tools/node-v20.19.0-linux-x64/bin:$PATH"   # this workstation only
   cd frontend
   npm ci
   npm run dev
   ```

6. **Open** `http://127.0.0.1:5173`, register an account, create a dashboard, and add a widget. The Vite dev
   server proxies `/api` to `http://127.0.0.1:8080` (`frontend/vite.config.ts`); the `API_PROXY_TARGET`
   environment variable overrides the target (the Playwright harness sets `http://127.0.0.1:8090`).

## Run the release (Docker only)

The release (M5) is one image, `insightboard:local`, built from the root `Dockerfile`: the backend serves the built
frontend from the same origin. It needs Docker with Compose, `make`, `openssl`, and `awk`; no host Rust or Node.
`make doctor` ends with a `release:` line that says whether Docker alone is enough on this machine.

```sh
make setup-env   # .env with generated secrets, if absent (openssl + awk only; no npm)
make release     # build insightboard:local and insightboard:<version>; nothing is pushed
make up          # Postgres, Redis, and the app; waits until all three are healthy
# open http://127.0.0.1:8088
make logs        # follow the app logs (Ctrl-C stops following, not the app)
make backup      # pg_dump into backups/ (gitignored), mode 600
make down        # stop and remove the containers; volumes are kept
```

| Target | What it does |
| --- | --- |
| `make setup-env` | The `.env` half of `make setup` (no `npm ci`): creates `.env` only if it is absent, with generated secrets and mode 600; prints no secret. |
| `make release` | `docker compose --profile app build app`, then tags the image `insightboard:<version>` (the backend version) beside `insightboard:local`; needs `.env` because compose interpolates `docker-compose.yml`. |
| `make up` | `docker compose --profile app up -d --wait`: starts Postgres, Redis, and the app (building the image if it is missing) and waits up to 180 s until all three are healthy; prints the published address. After a new `make release`, `make up` recreates the app with the new image; migrations run at startup. |
| `make down` | `docker compose --profile app down`: stops and removes the app, Postgres, and Redis containers and the network. **Never removes volumes.** A running `make backend-dev` loses its database until `make db-up`. |
| `make logs` | `docker compose --profile app logs -f app`. |
| `make backup` | `pg_dump --format=custom` inside the `postgres` container with that container's own `POSTGRES_USER` (no password leaves `.env`), written to `backups/insightboard-<UTC timestamp>.dump` with mode 600 in a mode-700 directory; an empty or failed dump is deleted and the target fails. Postgres must be running (`make db-up` or `make up`). Restore with `pg_restore` (runbook, M5b). |

How the release app runs (`Dockerfile`, `docker-compose.yml` service `app`, profile `app`):

- **Image.** Multi-stage: `node:20.19.0-bookworm-slim` runs `npm ci` and `npm run build`; `rust:1.99.0-bookworm`
  runs `cargo build --release --locked` (migrations are embedded in the binary); the runtime is `debian:bookworm-slim`
  with `ca-certificates` and `curl` (healthcheck) installed; the binary uses rustls and links no OpenSSL (M6). It ships `/app/insightboard-backend` and the built
  frontend in `/app/dist` (`STATIC_DIR=/app/dist`), owned by root, and runs as the unprivileged user `insightboard`
  (uid 10001). `.dockerignore` keeps `.env*` (except the examples), `.tools/`, `.git/`, `target/`, `node_modules/`,
  `dist/`, logs, Playwright output, and `backups/` out of the build context; the image holds no secret.
- **Configuration.** Compose passes an explicit environment, not `.env` itself: `DATABASE_URL` and `REDIS_URL` are
  built from `POSTGRES_USER`, `POSTGRES_PASSWORD`, and `REDIS_PASSWORD` with the service names `postgres` and `redis`
  (the `127.0.0.1` URLs in `.env` would point at the container itself); `JWT_SECRET` and the provider keys come from
  `.env`; the optional base URLs, timeouts, and rate limits are forwarded when set in `.env` (blank means the
  default). `STATUS_ALLOWED_FIXTURES` is not forwarded. `CORS_ORIGINS` is the published origin
  (`http://127.0.0.1:<port>`, `http://localhost:<port>`); `RUST_LOG` is `info`.
- **Exposure.** Inside its container the backend binds `0.0.0.0:8080` (`APP_HOST` is set by compose, not by `.env`
  or the image), because a published port cannot reach a loopback-only listener. The host side is published only as
  `127.0.0.1:${APP_PUBLISH_PORT:-8088}`; nothing listens on `0.0.0.0` or `::` on the host. This is the only
  non-loopback `APP_HOST` (M5 decision 3).
- **Ports.** 8088 by default, so the release runs beside `make backend-dev` (8080), Vite (5173), and `make e2e`
  (8090, 5175). Set `APP_PUBLISH_PORT` in `.env` to change it; `CORS_ORIGINS` follows.
- **Shared data.** The app joins the same compose project and uses the same Postgres and Redis volumes as
  development: development and e2e test users exist beside release data (each owner sees only their own dashboards).
- **Health.** The compose healthcheck is `curl -fsS http://127.0.0.1:8080/readyz` inside the container; the policy
  is `restart: unless-stopped`, as for Postgres and Redis.
- **Rate limit.** The auth rate limit keys on the TCP peer address, and behind Docker's loopback port publishing every
  browser on this machine reaches the app from the same Docker-side address `[INFERENCE: Docker port forwarding]`, so
  all clients share one auth bucket (10 register or login requests per minute, burst 5), as through the Vite proxy
  (M5 decision 7). Raise `AUTH_RATE_LIMIT_*` in `.env` and run `make up` if that gets in the way.
- **Static serving.** `/api/**`, `/healthz`, and `/readyz` behave as in development (`/api/<unknown>` stays a JSON
  404). Files under `/assets/` (content-hashed by Vite) get `Cache-Control: public, max-age=31536000, immutable`; a
  missing `/assets/` file is a JSON 404. Every other `GET` for a missing file returns `index.html` (`no-cache`) with
  200, so deep links such as `/dashboards/<id>` load. Static responses carry `X-Content-Type-Options: nosniff`,
  `Referrer-Policy: no-referrer`, and `X-Frame-Options: DENY`, and are gzip-compressed when the client asks.

## Configuration

`.env` is read by the backend at startup (`dotenvy`, before logging initialises). Variables
(`backend/src/config.rs`, `backend/src/main.rs`):

| Variable | Default | Rule |
| --- | --- | --- |
| `APP_HOST` | `127.0.0.1` | IP address the backend binds (`std::net::IpAddr`); unparsable values fail startup. Non-loopback values are a deliberate owner-approved change. |
| `APP_PORT` | `8080` | Port only. Changing it also needs the Vite proxy target changed (`API_PROXY_TARGET`). |
| `STATIC_DIR` | unset | Directory of the built frontend (`frontend/dist` after `npm run build`) served beside the API, with the SPA fallback described in [Run the release](#run-the-release-docker-only). Unset: unknown paths stay a JSON 404 and Vite serves the frontend. When set it must be an existing directory containing `index.html`, or startup fails naming `STATIC_DIR`. Relative paths start at the working directory. The image sets `/app/dist`. |
| `APP_PUBLISH_PORT` | `8088` | Read by compose only: the host port of the release app, published on `127.0.0.1`. |
| `POSTGRES_USER`, `POSTGRES_PASSWORD` | none, required by compose | Read by compose only; the container is initialised with them on first start. Database name is `insightboard`. |
| `REDIS_PASSWORD` | none, required by compose | Read by compose only (`--requirepass`, and `REDISCLI_AUTH` inside the container so `docker compose exec redis redis-cli ...` works without `-a`). |
| `DATABASE_URL` | none, required | `postgres://<POSTGRES_USER>:<POSTGRES_PASSWORD>@127.0.0.1:5432/insightboard`; same values as above. Remote databases add `?sslmode=verify-full&sslrootcert=<ca.pem>` (see TLS below). |
| `DB_MAX_CONNECTIONS` | `10` | Postgres pool size per process, 1 to 100; anything else fails startup. The Lambda synth sets 2. |
| `REDIS_URL` | none, required | `redis://:<REDIS_PASSWORD>@127.0.0.1:6379`; same password as above. `rediss://` for TLS (see below). Only the Lambda build (`--features lambda`) accepts it unset, which disables the cache. |
| `SSL_CERT_FILE` | unset | Optional PEM bundle that replaces the system CA store for `rediss://` only; Postgres and the providers ignore it. |
| `JWT_SECRET` | none, required | At least 32 bytes, or startup fails; generate with `openssl rand -hex 32` (`make setup` does). Tokens last 24 hours. |
| `GITHUB_API_TOKEN` | empty | Optional; sent to GitHub as a bearer token when set. Blank values count as unset. |
| `OPENWEATHER_API_KEY` | empty | Without it the weather endpoint returns 503 `{error}`. OpenWeather accepts the key only in the query string; the backend never logs request URLs. |
| `NEWSAPI_API_KEY` | empty | Without it the news endpoint returns 503 `{error}`. Sent in the `X-Api-Key` header. |
| `GITHUB_API_BASE_URL`, `OPENWEATHER_API_BASE_URL`, `NEWSAPI_API_BASE_URL`, `COINGECKO_API_BASE_URL` | the real APIs (`https://api.github.com`, `https://api.openweathermap.org`, `https://newsapi.org`, `https://api.coingecko.com`) | `http(s)` URL with a host, no query or fragment; tests and drills point them at local mocks. |
| `PROVIDER_CONNECT_TIMEOUT_MS`, `PROVIDER_TIMEOUT_MS` | `3000`, `8000` | Shared provider client timeouts; a timed-out call is 502. |
| `STATUS_CONNECT_TIMEOUT_MS`, `STATUS_TIMEOUT_MS` | `2000`, `4000` | Per-URL status-check timeouts (10 URLs at most, 5 at a time). |
| `STATUS_ALLOWED_FIXTURES` | empty | Development only: comma-separated exact loopback `ip:port` fixtures (for example `127.0.0.1:18080`) the status widget may reach. Non-loopback entries fail startup. Hostnames such as `localhost` are not accepted; use the IP in the widget URL. |
| `REQUEST_TIMEOUT_MS` | `10000` | Server-side limit per request; a slower request gets 504 `{error}`. |
| `AUTH_RATE_LIMIT_PER_MINUTE`, `AUTH_RATE_LIMIT_BURST` | `10`, `5` | Register and login, per client IP; over the limit is 429 `{error}`. Through the Vite proxy every browser arrives from `127.0.0.1`, so all local clients share one bucket; raise these if several people use one backend. |
| `DATA_RATE_LIMIT_PER_MINUTE`, `DATA_RATE_LIMIT_BURST` | `60`, `30` | `/api/data/*`, per signed-in user. |
| `RUST_LOG` | `insightboard_backend=debug,tower_http=debug` | Standard `tracing` filter. The value in `.env` is honoured because `.env` loads before logging starts; `.env.example` sets `info,insightboard_backend=debug`. |
| `CORS_ORIGINS` | `http://127.0.0.1:5173,http://localhost:5173` (the Vite dev origin) | Comma-separated exact origins (`scheme://host[:port]`; no path, trailing slash, wildcard, or credentials). The backend allows only those origins, the methods GET/POST/PUT/DELETE, and the `Authorization` and `Content-Type` headers, without credentials; other origins get no CORS grant. An unparsable entry fails startup. The Vite proxy makes browser calls same-origin in development, so CORS matters only for direct cross-origin calls. |

`COINMARKETCAP_API_KEY`, `SENTRY_DSN`, and `AWS_REGION` were removed from `.env.example` because nothing read them.

**TLS (D21, M6).** One rustls stack for Postgres, Redis, and the providers; certificate verification is always on.

- Postgres: `sslmode=verify-full` checks the certificate and host name against the bundled public roots plus the CA
  in `sslrootcert`, if given. `sslmode=require` and the default `prefer` encrypt without verifying the certificate,
  as in libpq, so use `verify-full` for any remote database. A verification failure (unknown CA, wrong host name)
  stops startup with `could not connect to Postgres (DATABASE_URL): … invalid peer certificate …`.
- Redis: `rediss://` verifies against the system CA store (`ca-certificates`), or against `SSL_CERT_FILE` for a
  private CA. A verification failure is treated like an unreachable Redis: a startup warning, caching off, and
  `/readyz` 503 until Redis answers.
- Local development keeps the plain loopback URLs; the local Postgres offers no TLS and `prefer` falls back to plain.

Frontend: `VITE_API_URL` (default `/api`). Vite binds `127.0.0.1` (`server.host` in `vite.config.ts`); the port is
the Vite default.

## Ports

| Port | Service | Binding |
| --- | --- | --- |
| 8080 | Backend API | `127.0.0.1` (`APP_HOST`) |
| 5173 | Vite dev server | `127.0.0.1` |
| 8088 | Release app (`make up`; `APP_PUBLISH_PORT`) | `127.0.0.1` (published; `0.0.0.0:8080` inside the container only) |
| 5432 | Postgres (compose service `postgres`) | `127.0.0.1` |
| 6379 | Redis (compose service `redis`) | `127.0.0.1` |

## Provider keys

Keys are optional and must be free-tier keys the owner supplies; never buy a plan.

- **No keys:** Weather and News answer 503 "not configured"; GitHub works unauthenticated within
  GitHub's anonymous rate limit; Crypto uses CoinGecko without a key; Status needs no key.
- **With keys:** add them to `.env` and restart the backend.
- Manual provider checks stay at human scale (a handful of requests). Status-widget checks use only
  `https://example.com` and loopback fixtures listed in `STATUS_ALLOWED_FIXTURES`; `httpstat.us`, used by the legacy
  run instructions, is not approved. Automated tests never call providers.
- The NewsAPI key travels in a header. The OpenWeather key must travel in the query string; reqwest errors are
  logged without their URL and provider bodies are never logged. Still, keep `RUST_LOG` at its default levels (do not
  enable `reqwest` or `hyper` debug/trace logging) when keys are set.

## Stop, restart, and data

- Stop the backend and Vite with Ctrl-C; stop the release app with `make down`.
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
docker compose exec -T redis redis-cli --scan --pattern 'insightboard:weather:*'
docker compose exec -T redis redis-cli TTL '<key>'
```

## Troubleshooting

- **`cargo: command not found`** — install Rust with rustup and open a new shell; for checks only, `make
  backend-check` uses the container fallback ([testing](testing.md)).
- **Backend exits with `DATABASE_URL must be set to a non-empty value`** (or `REDIS_URL`, `JWT_SECRET`) — `.env` is
  missing, empty for that variable, or not found; run `make setup` or create it at the repository root as above.
- **Backend exits with `CORS_ORIGINS has an invalid origin`** — fix the entry to `scheme://host[:port]` or unset it.
- **Backend exits connecting to Postgres** — `docker compose ps`; wait for `healthy`, then restart the backend. A
  Redis outage does not stop the backend (`/readyz` reports 503 until Redis answers).
- **Cannot connect to the Docker daemon** — `systemctl status docker`; start it, and check `docker` group
  membership with `id`.
- **Port 5432 or 6379 already in use** — another Postgres or Redis is running; stop it. Remapping means editing
  `docker-compose.yml` and `DATABASE_URL`/`REDIS_URL` together, which is an owner-approved change.
- **`make up` fails with `Bind for 127.0.0.1:8088 failed: port is already allocated`** — another program uses the
  release port; set `APP_PUBLISH_PORT` in `.env` to a free port and run `make up` again (`CORS_ORIGINS` follows).
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
- **Weather or News show "not configured" (503)** — expected without keys; after adding keys, restart the backend.
- **A widget shows "External service error" (502)** — the provider failed, timed out (8 s), rate-limited, or
  returned an unexpected value; the backend hides the cause from the client and logs it.
- **Status widget shows `blocked_address`** — the URL is, or resolves to, a loopback, private, link-local,
  metadata, or other non-public address; only public `http(s)` targets are checked. For a local fixture, add its exact
  `127.0.0.1:port` to `STATUS_ALLOWED_FIXTURES` and use that IP in the URL.
- **429 "Too many requests"** — the auth (10/min per IP) or data (60/min per user) rate limit; wait or raise the
  limits in `.env` for local experiments.
- **Signed out unexpectedly** — a 401 on any request other than login or register clears the stored session
  and reloads `/login`; tokens last 24 hours (client-side expiry check: M4c).
- **`npm run dev` or `npm ci` fails on engine or syntax errors** — Node is older than 20.19; put the `.tools/`
  Node first on `PATH` or use `nvm use`.

## What is not set up

No deployment, cloud account, or registry. TLS is available for a remote Postgres and Redis (see
[Configuration](#configuration)); the M6 Lambda package and `infra/` are build- and synth-only. The release image and
compose profile are described in
[Run the release](#run-the-release-docker-only). CI is `.github/workflows/ci.yml` (see [testing](testing.md#ci)). The legacy run
instructions remain readable at
[`manual run instructions.md` at `f15c6a3`](https://github.com/lavinhoque33/insightboard/blob/f15c6a3/manual%20run%20instructions.md);
their absolute paths, `sudo docker`, and `httpstat.us` target no longer apply.
