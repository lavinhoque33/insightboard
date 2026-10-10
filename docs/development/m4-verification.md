# M4 verification record

Evidence for the [M4 contract](../architecture/m4-quality-gate-contract.md) acceptance drills (section 14). Main thread
only. One section per slice; M4 is accepted only after M4a, M4b, and M4c are recorded and the owner accepts.

## M4a — reproducibility, gates, and exposure

- Status: **locally verified** on this workstation, 2026-10-10 (UTC). Not pushed; no CI.
- Commits: `d4bdca9` (format only: `cargo fmt`, Prettier), `2780fc0` (M4a implementation). Session
  [2026-10-10-03-m4a](sessions/2026-10-10-03-m4a.md).
- Environment: Fedora 44 (kernel 7.2.8), rustc 1.99.0 (rustup, pinned by `rust-toolchain.toml`), Node 20.19.0
  (project `.tools/`), Docker Compose 5.5.1, Postgres 16 and Redis 7 from `docker-compose.yml`.
- Scope: D4, D12, D13, D14, D18, D26, D29 (compose); decisions 1, 2, 3, 7, 14, 16, 17. D3 (decision 4) landed earlier
  in `354e06c` and is rechecked by drill 2.
- Implemented by `implement-light` (reformat) and `implement-medium` (rest); adversarial review by `verify`;
  drills 1–4 run by the main thread.

### Drills

| # | Drill | Expected | Observed (main thread) | Result |
| --- | --- | --- | --- | --- |
| 1 | Fresh clone → `make setup verify` | Green with host Rust or the container fallback | `git clone` of the local repository at `2780fc0` into `/tmp`, project Node linked in, `make setup verify` exit 0: `.env` created with mode 600; `cargo fmt --check`, clippy `-D warnings`, 14 backend tests, build; ESLint and Prettier, Vitest 2 files and 3 tests, `vue-tsc` and `vite build`. Clone removed afterwards | Pass (host Rust) |
| 2 | Compose and backend up; `ss -ltn` | Postgres, Redis, backend, Vite on `127.0.0.1` only | After `make db-up` recreated the containers under project names (`insightboard-postgres-1`, `insightboard-redis-1`; volumes `insightboard_postgres_data` and `insightboard_redis_data` kept, 3 users and 2 dashboards still present), the M4a backend and Vite started: listeners `127.0.0.1:5432`, `127.0.0.1:6379`, `127.0.0.1:8080`, `127.0.0.1:5173`; none on `0.0.0.0`, `[::]`, or `*` | Pass |
| 3 | Preflight and simple request from an origin not in `CORS_ORIGINS` | No CORS grant; the Vite dev origin is granted | Default `CORS_ORIGINS`. `OPTIONS /api/dashboards` and `GET /healthz` with `Origin: http://evil.example`: no `access-control-allow-origin`. Same requests from `http://127.0.0.1:5173` and `http://localhost:5173`: the exact origin echoed; methods `GET,POST,PUT,DELETE`; headers `authorization,content-type`; no credentials header | Pass |
| 4 | `npm audit --omit=dev` | 0 high or critical, or owner-accepted residuals | `npm audit --omit=dev`: 0 vulnerabilities; full `npm audit`: 0 (baseline: 8 and 17) | Pass; no residuals |

### Other checks

| Check | Who | Outcome |
| --- | --- | --- |
| `make verify` on the working tree | `verify` (reported) | Exit 0; same counts as drill 1 |
| Container fallback (`make backend-check` without host Rust, `rust:1.99.0-bookworm`) | `implement-medium` (reported) | Exit 0 in a copy of the tree; no root-owned files left. Not rerun by the main thread or `verify` (host Rust present) |
| `cargo metadata --locked`; `.gitignore` still ignores `.env`, `target/`, `node_modules/`, `dist/`, `.tools/` | `verify` (reported) | Lockfile consistent; ignores intact. Backend checks now run with `--locked` (main-thread change) |
| Startup errors without `.env` or with blank `DATABASE_URL`/`REDIS_URL`/`JWT_SECRET`, bad `APP_HOST`, `APP_PORT`, `CORS_ORIGINS` | `verify` (reported) | Exit 1 with a message naming the variable; no panic |
| `RUST_LOG` from `.env` | `verify` (reported) | Honoured (`trace` and `off` observed); the process environment still wins |
| `make setup` with an existing `.env`; generated `.env` | `implement-medium`, `verify` (reported) | Existing file untouched (byte-identical); new file mode 600, placeholders replaced, nothing printed |
| `make db-reset` without `CONFIRM=yes` | `implement-medium` (reported) | Refuses (exit 2); never run with `CONFIRM=yes` |
| Backend restart with retained data and rotated credentials | Main thread | Backend connected to Postgres and Redis after the recreate and after the credential rotation below |

### Incident: local credentials exposed in an agent transcript

During review, a `verify` probe ran `docker compose config` with the repository `.env` in scope and printed part of
the rendered file, so the local `POSTGRES_PASSWORD` and `REDIS_PASSWORD` reached that agent's session transcript.
They are loopback-only development credentials; no provider key or `JWT_SECRET` is referenced by the compose file. The
main thread rotated both the same day without printing them: `ALTER ROLE` with the new Postgres password, `.env`
rewritten (mode 600) with the new values and matching URLs, containers recreated, backend restarted and connected.
Temporary files from the probe were deleted.

### Carried forward (not M4a defects)

- `frontend/src/api/client.ts` formats 422 `details` with `join('; ')`, which renders `[object Object]` for the
  contract's `[{field, message}]` shape; the `errors`/`validation` fallbacks are unreachable (D16, M4b).
- `WidgetConfigModal.vue` casts form values to `string` before calling a field validator; safe only while validators
  sit on text fields (M4c).
- Two checkouts still share loopback ports 5432 and 6379 (documented in local setup).
- The workstation `.env` keeps the old `RUST_LOG` filter (`insightboard=debug`); `.env.example` now uses
  `insightboard_backend`. Cosmetic; owner's file.
