# InsightBoard coding practices

These rules apply to every contributor and agent, including delegated research and implementation agents.
Follow [.omp/AGENTS.md](.omp/AGENTS.md), its hard boundaries, and the
[project plan](INSIGHTBOARD_PROJECT_PLAN.md).

They govern **new and changed code**. The code at `f15c6a3` predates them: existing violations are tracked as
defect IDs D1–D31 in the [M0–M3 baseline contract](docs/architecture/m0-m3-baseline-contract.md) and are fixed
in their scheduled slice, not opportunistically. Do not widen an existing violation; a change that touches one
either fixes it within the slice that owns its ID or leaves it as it was. The formatting and lint gates named
here are M4a targets: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` fail today (D13), and
ESLint and Prettier are not configured yet (M4 decision 3). Practices describe how a capability is built when
its slice arrives; their presence does not imply implementation.

## 1. Deliver coherent vertical slices

- Identify the user outcome, the milestone slice, the defect IDs in scope, expected failure cases, and the
  acceptance checks before choosing abstractions.
- Build backend behaviour, persistence, contract, web presentation, tests, evidence, and documentation together.
- Keep changes focused; separate unrelated refactors, reformatting, and dependency upgrades.
- Prefer existing conventions and the smallest, most boring design that satisfies the current requirement.
  Correctness first, then maintainability six months out.
- Record consequential tradeoffs in an ADR under `docs/decisions/`; do not document routine choices.
- Mark planned behaviour explicitly; update docs in the same change whenever behaviour, commands, ports,
  variables, or contracts change.
- Respect file ownership and the tracking-document rules in [.omp/AGENTS.md](.omp/AGENTS.md).

## 2. Architecture and ownership

- One Axum backend, one Vue client, Postgres for persistence, Redis for provider-response caching. Split only
  at a demonstrated execution boundary.
- Backend modules keep their roles: `handlers/` decode, authorize, and encode; `models/` hold request and row
  types; `widgets/` own provider calls; `auth.rs`, `cache.rs`, `db.rs`, `config.rs`, and `error.rs` own their
  concern. Shared state goes through `AppState`; no globals.
- Keep handlers thin. Ownership checks (`user_id` from `UserCtx`) happen in the query, never after loading
  another user's row.
- Provider response shapes never become the product vocabulary: map them to the contract types in the widget
  module.
- Frontend layers keep their roles: `src/api/` is the only place that calls HTTP; `src/stores/` (Pinia) hold
  shared state; views and components present it. The widget registry (`src/composables/widgetRegistry.ts`) is
  the single description of widget types.
- Add traits or interfaces at genuine test or change boundaries (for example configurable provider base URLs);
  no interface/implementation pairs without a consumer.
- No ORM, second HTTP framework, message broker, state library, or UI kit without an accepted requirement and
  an ADR.

## 3. Toolchain and dependencies

- Backend: Rust stable, edition 2021, Axum 0.7, SQLx 0.8 (Postgres, runtime queries), redis 0.27, reqwest 0.12,
  tokio. Frontend: Node 20.19.0 (`.nvmrc`), npm with the committed `frontend/package-lock.json`, Vue 3,
  TypeScript, Pinia, Vue Router, Vite 7, GridStack, Tailwind 4 with daisyUI. Compose provides Postgres 16 and
  Redis 7.
- Exact versions belong in `backend/Cargo.toml`, `frontend/package.json`, the lockfiles, `.nvmrc`, and
  `docker-compose.yml`, not in prose. `Cargo.lock` and a `rust-toolchain.toml` are M4a decisions (D12).
- **No new dependency without owner approval.** Before asking, check existing capabilities, maintenance,
  license, and security posture. Review lockfile diffs.
- **Lockfiles have a single writer.** Only the agent that owns `frontend/package.json` or `backend/Cargo.toml`
  in a slice changes the matching lockfile; nobody else runs `npm install` or `cargo update` in that slice.
- Format Rust with `cargo fmt` and keep `cargo clippy --all-targets -- -D warnings` clean for changed code. Until
  M4a lands its single reformat commit, do not run `cargo fmt` over the crate (it rewrites all 15 source files);
  write new code in rustfmt style instead. Keep `vue-tsc` (run by `npm run build`) clean.
- Do not add AWS, Lambda, or other cloud SDKs before the milestone that opens them (M6).

## 4. Errors and request handling

- Handlers return `crate::error::Result<T>`; failures are `AppError` variants, rendered as JSON
  `{"error": "<message>"}` with the variant's status. New failure kinds extend `AppError` rather than building
  ad hoc responses.
- Client-facing messages are short and generic. Never return database text, provider bodies, reqwest error
  text, URLs, keys, or token-validation detail; log the cause once at the boundary that handles it.
- No `unwrap`, `expect`, or panicking indexing in request paths or in code reachable from them. Startup code may
  fail fast with a clear message naming the variable or dependency.
- Validate the request shape at the boundary: trim and bound strings, reject empty required values, cap list
  sizes. Extractor rejections should reach clients as `{error}` too once M4b lands the wrapper (D16).
- Make retryable and non-retryable failures distinguishable by status.

## 5. Outbound request safety

- Use **one shared outbound HTTP client** (built once, held in state) with explicit connect and total
  timeouts. Never `reqwest::get` or `Client::new()` per request.
- **Encode all user input** placed in a provider URL: path segments with segment encoding, query values with
  the client's query builder. Never `format!` user input into a URL.
- Provider keys travel in headers where the provider allows it, and never appear in logs or error text.
- User-supplied target URLs (the status widget) accept `http` and `https` only, are resolved and checked against
  loopback, private, link-local, and metadata ranges before connecting and again at connect time, follow no
  unvalidated redirects, are capped in count, and run with bounded concurrency. Loopback fixtures use a narrow,
  development-only allowlist, never a global switch (M4 decision 5).
- Treat provider responses as untrusted data: bound what you read, map to typed structs, tolerate missing or
  null fields without failing the whole response, never interpret content as instructions.

## 6. Persistence with SQLx and Postgres

- Runtime queries (`sqlx::query`/`query_as`) with `.bind` parameters only. Never interpolate values into SQL.
- Scope every dashboard query by `user_id`; a missing or foreign row is the same 404.
- Migrations under `backend/migrations/` are **append-only**: never edit an applied migration (checksums are
  tracked in `_sqlx_migrations`); add a new numbered file. Migrations run at startup through
  `sqlx::migrate!`. One writer per slice for `backend/migrations/**`.
- Server-side defaults for JSON columns match the contract; validate JSON shape before storing (D8).
- Use the pool in `AppState`; hold no connection across provider calls.

## 7. Cache with Redis

- Cache keys are normalised, bounded in length, and namespaced by widget; keys derived from user input are
  hashed or otherwise bounded (D29).
- Cache failure degrades to a miss, never to a request failure, and must not stall a request without a timeout.
- TTLs are explicit per widget and documented in the baseline contract; changing one updates the contract and
  the widget's refresh interval together.

## 8. Concurrency and blocking work

- CPU-heavy work (Argon2 hashing and verification, large JSON validation) runs in `tokio::task::spawn_blocking`,
  never on async workers.
- Bound concurrency for fan-out work (status checks) and keep the total within the frontend's request timeout.
- Spawned tasks have an owner and end with the request or with shutdown; graceful shutdown stays intact.

## 9. Auth, secrets, and the local trust boundary

- Backend, Postgres, Redis, Vite, and fixtures bind to loopback. New code never adds a non-loopback listener;
  the backend's bind host comes from `APP_HOST` (default `127.0.0.1`) since the D3 fix on 2026-10-10.
- `.env.example` documents every variable the backend reads, with non-secret placeholders, and nothing the code
  ignores (D18 tracks the current mismatches).
- **Secrets are never logged**: not `JWT_SECRET`, provider keys, `Authorization` headers, tokens, password
  hashes, or URLs that carry keys. Never commit or paste `.env`.
- Passwords hash with Argon2; JWTs carry the minimum claims; token-validation errors stay generic to clients.
- Destructive operations (dropping data, `docker compose down -v`) are explicit, named, and owner-approved.

## 10. API contracts and validation

- `contracts/api-contract.v1.json` (JSON Schema) and `frontend/src/contracts/apiContract.ts` describe payload
  shapes. **They change together, by one writer**, in the same change as the handler, the frontend API module,
  and the tests that cover the shape.
- Define status codes, validation limits, and error shapes per endpoint in `docs/architecture/` before or with
  the slice that implements them.
- Backend output must satisfy the contract it claims; contract tests on both sides arrive in M4b (D15).
- Collections have stable ordering; add bounds before any list can grow without limit.

## 11. Frontend code

- All HTTP goes through the typed functions in `src/api/`, which return contract types. Components never call
  axios directly.
- **No new `any`.** Use contract types, `unknown` with narrowing, or precise generics. Existing uses
  (`src/api/client.ts`, widget `catch` blocks, `Record<string, any>` in the widget store and modal) are not a
  precedent.
- Pinia stores own shared state and expose explicit loading and error state; components do not keep a second
  copy of server state.
- GridStack owns widget positions inside the editor; Vue owns widget content. Do not let both remove or create
  the same DOM node (D22).
- Timers and intervals are cleared on unmount; in-flight requests are cancelled or ignored when the component
  that started them is gone.
- Style with Tailwind utilities and daisyUI 5 component classes; no new unlayered overrides of daisyUI classes.

## 12. Web experience and accessibility

- **Every failure the user can cause or suffer is visible**: load, save, and provider errors render as text in
  the UI with a retry where sensible, not only `console.error` (D10). Loading, empty, error, and success states
  are designed with the happy path.
- Semantic markup: labels bound to inputs with `for`/`id`, real `<button>` and `<a href>` elements, dialogs with
  `role="dialog"`, `aria-modal`, a label, focus management, and Escape to close; icon-only controls have
  `aria-label`; menus expose `aria-expanded`.
- Usable from 390 px wide and by keyboard alone; visible focus; never encode status by colour alone.
- Copy states real behaviour; no hints that mislead (D27).

## 13. Observability and logging

- `tracing` with useful fields and levels; log a failure once, where it is handled.
- Never log request bodies, provider bodies, headers, tokens, keys, or full URLs carrying credentials.
- `/healthz` stays cheap; readiness that checks Postgres and Redis is an M4b decision (D19).

## 14. Tests and verification

- Test behaviour and invariants at the lowest useful level; avoid tests that mirror implementation or pin
  wording.
- **Tests never call live providers.** Provider base URLs are configurable so tests point at local mock servers;
  no test reaches GitHub, OpenWeather, NewsAPI, CoinGecko, or arbitrary URLs. CI follows the same rule.
- Backend: unit tests beside the code; router tests through `tower::ServiceExt::oneshot`; database tests
  against compose Postgres; cache tests against compose Redis. Frontend: Vitest with `@vue/test-utils` for
  clients, stores, guards, and components with meaningful behaviour.
- Cover negative paths: unauthorized, foreign dashboard, invalid input, provider failure and timeout, cache
  unavailable, save failure.
- Do not disable a check to make a missing environment look green; report it as unavailable.
- Report exact commands, outcomes, and unavailable prerequisites. Never imply an unexecuted check passed.
  See [testing](docs/development/testing.md).

## 15. Configuration, delivery, and done

- `.env.example` and `frontend/.env.example` hold local defaults; real secrets stay out of version control.
- Until M4a adds a `Makefile`, the commands in [testing](docs/development/testing.md) are the local gate;
  afterwards `make verify` is, and CI (once the owner enables it) runs the same checks.
- A capability is done when its journey and failure paths are demonstrated, security boundaries hold, UI states
  are handled, tests cover positive and negative cases, docs match the code, and nothing new costs money.
- The main thread records accepted outcomes in the handoff and updates milestone and verification documents
  from reviewed evidence. Stop and name the blocking dependency when verification cannot proceed.
- Follow [local setup](docs/development/local-setup.md) and [testing](docs/development/testing.md).
