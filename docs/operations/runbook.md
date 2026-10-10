# InsightBoard runbook

How to install, run, back up, and recover the local release. It covers the Docker-only path; the development setup
(host Rust and Node, Vite) is in [local setup](../development/local-setup.md). Decisions behind this layout are in
the [M5 contract](../architecture/m5-local-release-contract.md) and
[ADR 0003](../decisions/ADR-0003-m5-local-release.md).

Everything here runs on one machine and listens on `127.0.0.1` only. Nothing is deployed, published, or exposed to
the network.

## What runs

| Service | Image | Host address | Data |
| --- | --- | --- | --- |
| `app` | `insightboard:local`, built from the root `Dockerfile` | `http://127.0.0.1:8088` (`APP_PUBLISH_PORT`) | none; the backend also serves the frontend |
| `postgres` | `postgres:16-alpine` | `127.0.0.1:5432` | volume `insightboard_postgres_data` |
| `redis` | `redis:7-alpine` (append-only file) | `127.0.0.1:6379` | volume `insightboard_redis_data` (cache only) |

The app container binds `0.0.0.0:8080` inside its own network namespace; compose publishes it only on the host's
loopback. The development backend (`make backend-dev`, port 8080) and the release app share the same Postgres and
Redis data.

## Prerequisites

Docker with the Compose plugin, `make`, `openssl`, `awk`, and Git. No Rust, Node, or database tools on the host. The
first build downloads base images, crates, and npm packages (a few minutes; later builds reuse the cache).

## First start

```sh
git clone https://github.com/lavinhoque33/insightboard.git
cd insightboard
make setup-env   # writes .env (mode 600) with generated database, Redis, and JWT secrets; never overwrites
make release     # builds insightboard:local
make up          # starts postgres, redis, and app; waits until all are healthy
```

Open `http://127.0.0.1:8088`, register an account, and create a dashboard. Migrations run automatically when the app
starts.

`make doctor` reports which prerequisites are present and whether Docker alone is enough.

## Provider keys

GitHub, CoinGecko, and status checks work without keys. Weather and News need free keys from OpenWeather and
NewsAPI; without them those widgets show a "not configured" error (HTTP 503). Put the keys in `.env`
(`OPENWEATHER_API_KEY`, `NEWSAPI_API_KEY`, optionally `GITHUB_API_TOKEN`) and run `make up` again so compose
recreates the app with the new values. Never commit or share `.env`.

## Daily operation

| Task | Command |
| --- | --- |
| Start or apply changed `.env` values | `make up` |
| Stop everything (data kept) | `make down` |
| Restart only the app | `docker compose --profile app restart app` |
| Follow the app log | `make logs` |
| Status and health | `docker compose --profile app ps` (all `healthy`); `curl -fsS http://127.0.0.1:8088/readyz` |
| Version | `curl -fsS http://127.0.0.1:8088/healthz` |

`make down` removes the containers and the network but never the volumes. If you also use the development backend,
run `make db-up` afterwards to bring Postgres and Redis back.

## Upgrade

```sh
git pull
make release
make up        # recreates the app on the new image; migrations run at start
```

Take a backup first (below). If the app does not become healthy, read `make logs`; a failed migration stops the app
before it serves requests.

## Backup and restore

`make backup` writes a compressed custom-format dump of the database to `backups/insightboard-<UTC time>.dump`
(directory mode 700, file mode 600, ignored by Git). A failed or empty dump is deleted and the command fails. Redis
holds only a cache and needs no backup. Copy dumps somewhere safe; there are no automatic backups.

Rehearse a restore into a throwaway database without touching the live one:

```sh
docker compose exec -T postgres sh -c 'createdb -U "$POSTGRES_USER" restore_check'
docker compose exec -T postgres sh -c 'pg_restore -U "$POSTGRES_USER" -d restore_check --no-owner' \
  < backups/<file>.dump
docker compose exec -T postgres sh -c \
  'psql -U "$POSTGRES_USER" -d restore_check -tAc "select count(*) from users; select count(*) from dashboards"'
docker compose exec -T postgres sh -c 'dropdb -U "$POSTGRES_USER" restore_check'
```

Restoring over the live database (`-d insightboard --clean`) replaces all accounts and dashboards; stop the app first
(`docker compose --profile app stop app`) and take a fresh backup before doing it.

## Rotating secrets

- `JWT_SECRET`: replace it in `.env` (at least 32 bytes, for example `openssl rand -hex 32`) and run `make up`.
  Every existing session ends; users sign in again.
- Postgres password: change it inside the database first, then in `.env`, then `make up` (steps in
  [local setup](../development/local-setup.md), troubleshooting).
- Redis password: change `REDIS_PASSWORD` in `.env` and run `make up` (Redis restarts with the new password).

## Reset (deletes data)

`make db-reset CONFIRM=yes` removes the containers **and** the Postgres and Redis volumes: every account and dashboard
is gone. Take a backup first. It is the only `make` target that deletes data.

## Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `Bind for 127.0.0.1:8088 failed: port is already allocated` | Another program uses the port. Set `APP_PUBLISH_PORT=<free port>` in `.env` and run `make up`; open that port instead |
| `make up` or `make release` says `.env` is missing | Run `make setup-env` |
| App stays `starting` or `unhealthy` | `make logs`; `/readyz` returns 503 while Postgres or Redis is unreachable |
| Weather or News show "not configured" | Add the provider key to `.env`, then `make up` |
| Status widget rejects a URL | Only public `http`/`https` addresses are checked; loopback, private, and metadata addresses are refused by design |
| Too many login attempts (429) | Auth is limited to 10 requests per minute with a burst of 5, shared by every browser on this machine (all arrive through the same published port); wait a minute |
| Signed out unexpectedly | Sessions last 24 hours, or `JWT_SECRET` changed |

Never paste the output of `docker compose config` anywhere: it prints every secret from `.env`.

## Limits

- Loopback only and plain HTTP; no TLS between the app, Postgres, and Redis (D21, planned for M6).
- The session token is kept in the browser's `localStorage` for 24 hours; there is no refresh or revocation.
- Anyone who can reach the port can register an account.
- All clients share one authentication rate-limit bucket.
- One instance on one machine; no automatic backups; development, test, and release data share one database.
- Weather and News need the owner's free provider keys.
- Moving and resizing widgets needs a mouse or touch; keyboard layout editing is in the backlog.
