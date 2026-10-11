# M6 verification record

Evidence for the [M6 AWS-readiness contract](../architecture/m6-aws-readiness-contract.md) (accepted 2026-10-10;
decisions in [ADR 0004](../decisions/ADR-0004-m6-aws-readiness.md)). Everything below ran on this workstation (Fedora
44, kernel 7.2.8, Docker 29.8.2, rustc 1.99.0, Node 20.19.0) on 2026-10-11 between 01:50 and 02:10 UTC (2026-10-10
local), on the M6 working tree based on `50be1e2`, unless it names GitHub Actions. **Observed** = run or inspected by
the main thread; **reported** = a delegated agent's result. No step used AWS credentials, called an AWS API, or
deployed anything. Session: [2026-10-10-08](sessions/2026-10-10-08-m6-contract.md).

## M6a — backend seams

Implemented by `implement-heavy` (M6aBackendSeams); reviewed, rerun, and committed by the main thread as `e84ed43`.

| Check | Result |
| --- | --- |
| Agent checks | **Reported**: `make backend-check` 101 passed; `--features lambda` clippy clean and 104 passed; no `openssl`, `native-tls`, or `hyper-tls` in either `cargo tree`; first TLS drill pass |
| Code review | **Observed**: `main.rs` shared `startup(RedisRequirement)` with the server `main` behind `cfg(not(feature = "lambda"))`; `lambda.rs` inserts `ConnectInfo` only from `RequestContext::ApiGatewayV2.http.source_ip`; `/readyz` Postgres-only branch when the cache is disabled; contract pair changed together (`ReadyResponse.cache`, const `"disabled"`, optional) |
| Accepted deviations | `libssl3` stays in the image as a dependency of Debian `curl` and `ca-certificates` (the Dockerfile no longer installs it; the binary links no OpenSSL). A Redis TLS failure stays fail-open. `/readyz` gains the optional `cache` field. Recorded in the contract and ADR 0004 |

## M6b — synth-only infrastructure

Implemented by `implement-heavy` (M6bInfraSynth); the main thread wired `make infra-check`, `make infra-synth`, and
the CI `infra` job, and committed them together as `014b277`.

| Check | Result |
| --- | --- |
| Agent checks | **Reported**: 18/18 `node:test` tests; 0 unacknowledged findings; `cfn-lint` 0 errors and 0 warnings; offline `--network none` synth passed and was refused with `AWS_ACCESS_KEY_ID` set |
| Accepted deviations | Minimal RDS-only VPC (contract decision 7 clarification); explicit least-privilege function role; CloudFormation-Validate W2508 and W9011 acknowledged with decision 8 reasons (W3010 is acknowledged by `aws-cdk-lib` itself); `cfn-lint` W3691 and W3005 ignored on their single resource; empty provider-key fields in the app secret |

## Drills

| # | Drill | Expected | Observed | Result |
| --- | --- | --- | --- | --- |
| 1 | Default build unchanged | All green; no `lambda_http` or OpenSSL in the default tree; no OpenSSL linked; plain local URLs work | `make backend-check` 101 passed (fmt, clippy, build clean); `make web-check` Vitest 91 in 12 files, ESLint, Prettier, build clean; `make e2e` 2 passed (24.1 s). `cargo tree -e normal`: 0 matches for `openssl`, `openssl-sys`, `native-tls`, `hyper-tls`, `lambda_http`, `aws-lc-rs`. `make release` rebuilt `insightboard:local`; `ldd /app/insightboard-backend` lists only `libgcc_s`, `libm`, `libc`, and the loader; `dpkg -l` shows one `libssl` package (pulled by `curl`/`ca-certificates`, see deviation). App on `127.0.0.1:8088` via `docker compose --profile app up -d --wait --no-deps app`: `/healthz` 200 `version 1.0.0`, `/readyz` 200, `/` 200 `text/html`; removed afterwards, compose Postgres and Redis untouched | Pass |
| 2 | `--features lambda` clippy and tests | Clean; adapter tests pass | `cargo clippy --locked --all-targets --features lambda -- -D warnings` clean; `cargo test --locked --features lambda` 104 passed (the 3 `tests::lambda` tests included). The lambda tree contains `lambda_http` 1.3.1 and no OpenSSL, native-tls, or AWS SDK crate | Pass |
| 3 | `bootstrap` under the Lambda Runtime Interface Emulator | Under the size limits; v2 events succeed; throwaway data removed | `make lambda-build` (zip deleted first): 4,353,156 bytes zipped, 9,709,370 bytes unzipped (limits 50 MB and 250 MB [MEMORY]); `bootstrap` is a static-pie x86_64 ELF; contents `bootstrap`, `static/`, `certs/`. `amazon/aws-lambda-provided:al2023` (amd64, 2025-01-14) on network `insightboard_default`, published `127.0.0.1:9100` only, `REDIS_URL` unset, throwaway `JWT_SECRET`, `DB_MAX_CONNECTIONS=2`. 13/13 checks: `/healthz` 200 `1.0.0`; `/readyz` 200 `{"cache":"disabled","status":"ready"}`; `/` and `/dashboards/some-id` the same `index.html`; `/assets/index-*.js` 200 `text/javascript`, `immutable`; with `accept-encoding: gzip` the body is base64 (`isBase64Encoded: true`) and gunzips to the identical bytes; `/api/unknown` JSON 404; register 201, create dashboard 201, list returns exactly it, delete 204; auth limit per `sourceIp` (A: 5×401 then 429; B: 401). Log: migrations applied, "REDIS_URL is unset: provider-response caching is disabled", init 49 ms. Container, package copy, and env file removed; the throwaway user row deleted (1 row, no dashboards) | Pass |
| 4 | TLS to Postgres and Redis (D21) | `/readyz` 200 over TLS; wrong CA or host fails clearly with no client leak; throwaways removed | Throwaway CA and `localhost` server certificate; `postgres:16-alpine` with `ssl=on` on `127.0.0.1:55432`, `redis:7-alpine` TLS-only (plaintext ping reset) on `127.0.0.1:56379`, random passwords. Server binary under `env -i` with `sslmode=verify-full&sslrootcert=`, `rediss://`, `SSL_CERT_FILE`: `/readyz` 200. Postgres wrong CA: exit 1, `invalid peer certificate: UnknownIssuer`. Postgres host `127.0.0.1` (not in SAN): exit 1, `certificate not valid for name "127.0.0.1"`. Redis wrong CA: running, WARN `Redis unavailable at startup … invalid peer certificate: UnknownIssuer`, `/readyz` 503 `{"error":"Service not ready"}`. No password or secret in any log. Containers (no volumes), CA, keys, and logs removed | Pass |
| 5 | Offline synth | Template produced offline; refused with credentials | `node:20.19.0-bookworm-slim`, `--network none`, `--user` of the invoking user, empty `HOME`, no `AWS_*` variables: template written, SHA-256 `94a9d3407f758169a0ecb397e55003055f15fc954dd02c5327c6ba83109e94b6`, 0 unacknowledged findings. Same with `AWS_ACCESS_KEY_ID=dummy`: exit 1, "synth refused: AWS_ACCESS_KEY_ID set" | Pass |
| 6 | Template inspection (`jq` over `infra/cdk.out/InsightBoardExperiment.template.json`, 24,698 bytes) | Matches decision 7; nothing account-bound; settings and tags present; gates clean | 25 resources exactly as in the [proposal](../operations/aws-experiment-proposal.md); 0 NAT, EIP, VPC endpoint, custom resource. Function `provided.al2023`, `x86_64`, 512 MB, 15 s, no `VpcConfig`, environment keys `DATABASE_URL`, `DB_MAX_CONNECTIONS`, `GITHUB_API_TOKEN`, `JWT_SECRET`, `NEWSAPI_API_KEY`, `OPENWEATHER_API_KEY`, `RUST_LOG`, `STATIC_DIR` (8 `resolve:secretsmanager` references; `DATABASE_URL` with `sslmode=verify-full&sslrootcert=/var/task/certs/rds-global-bundle.pem`). URL auth `NONE`, `BUFFERED`, no CORS. RDS `postgres` `16`, `db.t4g.micro`, 20 GB gp3, single-AZ, public, encrypted, deletion protection off, backups 1 day, `DeleteAutomatedBackups`, `DeletionPolicy`/`UpdateReplacePolicy` `Delete`; parameter group `rds.force_ssl=1`. Security group: TCP 5432 from `0.0.0.0/0` only. Log group 14 days, `Delete`. Both secrets generated (no `SecretString`), `Delete`. Subnet zones via `Fn::GetAZs`. No 12-digit account ID, no region literal, no lookup. Every taggable resource tagged. IAM policy: `logs:CreateLogStream`, `logs:PutLogEvents` on the function's log group only. Acknowledgements: AwsSolutions EC23, RDS3, RDS10, RDS11, SMG4, VPC7 and CloudFormation-Validate W2508, W9011 each cite a decision; W3010 is `aws-cdk-lib`'s own. `cfn-lint` 1.57.2: no errors or warnings | Pass |
| 7 | Credential hygiene | No AWS tooling or credentials; secret scan clean | No `~/.aws`; no AWS credential variables; `aws`, `cdk`, `sam`, `cargo-lambda` absent. `infra/package-lock.json`: 0 `aws-sdk`, `@aws-sdk`, `aws-cdk` CLI, or `cdk-cli` packages. Backend lambda tree: 0 `aws-sdk`, `aws-config`, `aws-sigv4`, `aws-smithy` crates. No `cdk deploy`, `cdk bootstrap`, or AWS CLI command in `infra/`, `Makefile`, or `.github/`. Secret scan of the M6 commit range: see "Publication" | Pass |
| 8 | Proposal and docs | Every cost figure sourced and dated; docs match the code | [Experiment proposal](../operations/aws-experiment-proposal.md): paid lines from AWS pricing pages and AWS's public price list (RDS list published 2026-10-06), retrieved 2026-10-11 UTC; free-tier lines attributed to M6Ecosystem (reported 2026-10-10) or marked [MEMORY]. `local-setup`, `testing`, `system-overview`, `runbook`, README, `CHANGELOG.md`, `.env.example`, and `infra/README.md` checked against `Cargo.toml`, `config.rs`, `Dockerfile`, `backend/Dockerfile.lambda`, the `Makefile`, `ci.yml`, and `infra/src/stack.ts` | Pass |
| 9 | CI | `backend` (with `lambda`), `web`, `e2e`, `release`, `infra` green | Pending the push; see "Publication" | Pending |

## Observations

- The Runtime Interface Emulator image runs `/var/runtime/bootstrap` (custom-runtime layout); the zip-deployed
  runtime runs `/var/task/bootstrap`. Drill 3 mounted the same binary at both paths. The first attempt, without the
  second mount, failed with `Runtime.InvalidEntrypoint` before reaching the binary.
- During drill 4 a broad `pkill -f target/debug/insightboard-backend` also stopped the long-running development
  backend on `127.0.0.1:8080` for about 17 seconds; the main thread restarted it from the same binary. No data was
  affected. Later cleanup killed only recorded process IDs.
- `quinn` is listed in `Cargo.lock` but compiled by neither feature set (`cargo tree -i quinn` is empty).

## External requests

- AWS public pricing pages (RDS for PostgreSQL, VPC, Secrets Manager, Lambda) and one static price-list file behind
  the RDS page, anonymous HTTPS, 2026-10-11 UTC. No AWS API, account, or credential.
- Docker Hub: one pull of `amazon/aws-lambda-provided:al2023` (approved in the contract's dependency list).
- No provider was called; drill 3 and drill 4 used only loopback services.

## Publication

To be completed after the push.
