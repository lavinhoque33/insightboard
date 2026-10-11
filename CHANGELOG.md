# Changelog

All notable changes to InsightBoard. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions follow [Semantic Versioning](https://semver.org/). Defect IDs (D1–D31) refer to the catalogue in the
[M0–M3 baseline contract](docs/architecture/m0-m3-baseline-contract.md) section 12.

## [Unreleased]

## [1.1.0] - 2026-10-11

AWS readiness (M6, accepted 2026-10-11): TLS to a remote Postgres and Redis, a Lambda build of the same app, and a
synth-only infrastructure definition with a cost and teardown proposal. Nothing is deployed; local use is unchanged.
See the [M6 verification record](docs/development/m6-verification.md).

### Added

- An AWS Lambda entry point behind the off-by-default cargo feature `lambda` (Function URL and HTTP API payload v2;
  the per-IP sign-in limit uses the request's source address). `make lambda-build` packages a static x86_64 musl
  `bootstrap`, the built frontend, and the RDS CA bundle using Docker only. Nothing is deployed.
- `infra/`: a synth-only AWS CDK v2 definition of a possible experiment (one Lambda function with a Function URL,
  RDS PostgreSQL, Secrets Manager), checked by cdk-nag, assertion tests, and `cfn-lint`, and refusing to run with AWS
  credentials set. `make infra-check` and `make infra-synth`; the
  [experiment proposal](docs/operations/aws-experiment-proposal.md) lists its cost and teardown.
- `DB_MAX_CONNECTIONS` (Postgres pool size, default 10).
- `/readyz` reports `"cache": "disabled"` when the Lambda build runs without `REDIS_URL`.
- CI lints and tests the `lambda` feature and runs a new `infra` job.

### Changed

- One TLS stack: rustls for Postgres, Redis, and the providers. The binary links no OpenSSL, the image no longer
  installs `libssl3` explicitly, and a native build no longer needs OpenSSL headers. Like the OpenSSL defaults before
  it, the status widget accepts TLS 1.2 and 1.3 only.

### Fixed

- TLS to a remote Postgres (`sslmode=verify-full`, `sslrootcert`) and Redis (`rediss://`, `SSL_CERT_FILE`) now works
  (D21).

## [1.0.0] - 2026-10-10

The first release: a local, single-machine product that builds and runs from a fresh clone with Docker alone. It
covers milestones M0–M5 (2025-10-28 to 2026-10-10); see the [roadmap](docs/product/roadmap.md) and the verification
records under [`docs/development/`](docs/development/).

### Added

- Accounts with email and password (Argon2), 24-hour JWT sessions; owner-scoped dashboards with create, list, open,
  rename, update, and delete.
- Five widgets proxied and cached in Redis by the backend: GitHub activity, OpenWeather, NewsAPI, CoinGecko prices,
  and URL status checks.
- GridStack editor: add (configure first), drag, resize, configure with the current settings pre-filled, remove;
  auto-save with a visible "not saved" state and Retry; a one-column layout below 768 px.
- A redesigned interface with light and dark themes that follow the system setting, with a Light/Dark/System
  toggle; keyboard-accessible menus and dialogs; reduced motion respected.
- Release packaging (M5): one container image in which the backend also serves the built frontend; a compose
  profile `app` on `127.0.0.1:8088`; `make release`, `up`, `down`, `logs`, and `backup`; a
  [runbook](docs/operations/runbook.md).
- Health endpoints: `/healthz` (liveness, reports the version) and `/readyz` (Postgres and Redis).
- Quality gates and tests: `make verify` (rustfmt, clippy, cargo test, ESLint, Prettier, Vitest, type-check, build),
  Playwright journeys (`make e2e`) against a loopback provider fixture, and GitHub Actions CI (backend, web, e2e,
  release).
- A versioned JSON Schema payload contract (`contracts/api-contract.v1.json`) enforced by tests on both sides; MIT
  `LICENSE`.

### Changed

- Every service binds to `127.0.0.1`; Postgres and Redis credentials come from `.env` (D3).
- Validation errors are 422 with field details; a duplicate email is 409; a missing provider key is 503; every
  error body is JSON (D16, D17).
- Tokens last 24 hours instead of 7 days; the client checks expiry and returns to the requested page after login
  (D23).
- daisyUI moved from a 5.0 beta to stable 5; dependencies updated until `npm audit` reports 0 findings (D12, D14,
  D24).

### Fixed

- Status widget server-side request forgery: only public `http`/`https` targets, no redirects, capped and timed
  (D1).
- Crypto and news always failing for lack of a `User-Agent` (D30); unencoded provider input, including GitHub path
  injection (D2, D7); missing timeouts and sequential status checks (D5, D20); CORS allowing any origin (D4).
- Rate limits on authentication and data routes (D6); dashboard JSON defaults and validation (D8).
- Configuration form not pre-filling (D9), silent save failures (D10), extra saves after leaving the editor (D22),
  small-width layout (D31), accessibility gaps (D25), and display and copy issues (D27, D28).

### Security

- The release image runs as a non-root user and contains no secrets; `.env` stays on the host.
- Known limits are listed in the [README](README.md) and the [runbook](docs/operations/runbook.md): loopback only,
  no TLS (D21, planned for M6), the token in `localStorage`, open registration on the local port, and one shared
  authentication rate-limit bucket.

[Unreleased]: https://github.com/lavinhoque33/insightboard/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/lavinhoque33/insightboard/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/lavinhoque33/insightboard/tree/v1.0.0
