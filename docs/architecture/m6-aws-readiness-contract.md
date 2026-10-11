# M6 contract: AWS readiness

- Status: **accepted** 2026-10-10. The owner approved every ★ decision (1–12), the new-dependency list, and the
  one-time anonymous download of the public RDS CA bundle, as written; recorded in
  [ADR 0004](../decisions/ADR-0004-m6-aws-readiness.md).
- Sources: [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) section 4.1 (hard boundaries), section 12/M6, open
  decision 6 (section 17); [M5 contract](m5-local-release-contract.md); [system overview](system-overview.md);
  [M0–M3 baseline contract](m0-m3-baseline-contract.md) D21; [ADR 0003](../decisions/ADR-0003-m5-local-release.md);
  [ADR 0004](../decisions/ADR-0004-m6-aws-readiness.md); [coding practices](../../CODING_PRACTICES.md).
- Facts gathered on 2026-10-10 at `939c35c`. The code map came from `research-medium` M6CodeSeams (read-only). The
  public-documentation survey came from `research-medium` M6Ecosystem (web pages only, no AWS access). The main
  thread made the checks marked **observed**. Session:
  [2026-10-10-08](../development/sessions/2026-10-10-08-m6-contract.md).

M6 prepares InsightBoard for a later, separately approved AWS experiment (M7) without touching AWS. It adds three
things:

- a Lambda entry point behind a cargo feature;
- TLS to Postgres and Redis (D21);
- an infrastructure definition that is synthesised offline and inspected, with a cost estimate and a teardown plan.

No credentials, accounts, control-plane calls, or deploys are involved. The local product stays the product. Every
cloud seam is off by default, and the release image, `make` targets, and development flow behave as in 1.0.0. The
one exception is the TLS library change in decision 5, which the drills cover.

## Starting point (observed at `939c35c`)

**Backend structure**
- The backend is a single binary crate. Its unit and router tests live inside it, under `backend/src/tests/`.
- `main.rs` does the following, in order:
  1. loads an optional `.env`;
  2. reads the configuration;
  3. opens the Postgres pool (`max_connections` 10, 3 s acquire timeout);
  4. runs the embedded migrations;
  5. opens the Redis cache;
  6. builds `AppState` and `app::router(state)`;
  7. binds with `into_make_service_with_connect_info::<SocketAddr>()`.
- The router is built without a socket, and the tests already drive it with `oneshot`.

**Rate limiting**
- The auth rate limit keys on `ConnectInfo<SocketAddr>`. When the extension is absent, every client shares one
  bucket.
- Limiter state lives in process memory, and a spawned task cleans it every 60 s.

**TLS libraries (D21)**
- `sqlx` 0.8.6 and `redis` 0.27.6 are built without TLS features, so `sslmode=require` and `rediss://` URLs fail.
- `reqwest` 0.12.28 uses its default native TLS (OpenSSL). For that reason the release image installs `libssl3`
  (`Dockerfile:35-37`).

**Redis and readiness**
- `REDIS_URL` is required.
- The cache fails open when Redis is unreachable, but `/readyz` returns 503 unless both Postgres and Redis answer
  (`backend/src/handlers/health.rs:20-29`).

**Frontend and tokens**
- The frontend calls a relative `/api`, and the backend serves the built frontend when `STATIC_DIR` is set (M5).
- Tokens are HS256, signed with `JWT_SECRET`. Their claims are `sub` (the `users.id` UUID), `email`, `exp`, and
  `iat`.

**This workstation**
- No infrastructure code exists in the repository.
- The workstation has no `~/.aws` directory and no AWS credential variables. None of `aws`, `cdk`, `sam`, `tofu`,
  `terraform`, `cargo-lambda`, `zig`, or `cfn-lint` is installed.

**Compatibility checks**
- Observed: a throwaway crate outside the repository, since removed, ran `cargo check` cleanly with:
  - `axum` 0.7.9 and `lambda_http` 1.3.1 (`default-features = false`, `apigw_http`);
  - a `map_request` layer that inserts `ConnectInfo` from the API Gateway v2 request context.
- In that crate, `lambda_http` 1.3.1 and `lambda_runtime` 1.4.0 pulled no TLS crate, and `tower` resolved to 0.5.3.
- Observed from the npm registry: `aws-cdk-lib` 2.273.0 declares `node >=20.0.0`, and `cdk-nag` 3.0.2 declares
  `>=18.12.0`. Both are compatible with the repository's Node 20.19.0.

## Slices

Delivered in order; all three are required for M6 acceptance.

- **M6a — Backend seams:** sections 1–4: the Lambda entry point, the build and package, TLS (D21), and the
  cache-disabled mode. Decisions 3–6, 9, 11.
- **M6b — Synth-only infrastructure:** sections 5–7: the infrastructure package, synth guardrails, and CI.
  Decisions 1, 7, 8, 10, 12.
- **M6c — Experiment proposal and drills:** sections 8–9 (cost, cleanup, M7 preconditions, documentation) and the
  acceptance drills. Decision 2 is recorded here.

## Decisions

★ marks the recommended option.

1. **Infrastructure-as-code tool** (plan open decision 6a).
   - ★ **AWS CDK v2 in TypeScript**, in a new `infra/` package with its own lockfile, on the repository's Node
     20.19.0.
     - It installs only the libraries (`aws-cdk-lib`, `constructs`, `cdk-nag`), not the `aws-cdk` CLI. Synthesis
       calls `app.synth()` from a script, so there is no deploy command in the repository at all.
     - The output is real CloudFormation JSON under `infra/cdk.out/`.
     - Checks: `cdk-nag` (AwsSolutions rules) runs during synthesis, `node:test` assertion tests check the template,
       and `cfn-lint` validates it.
     - All three are free (Apache-2.0, Apache-2.0, MIT-0).
   - Alternative: **hand-written CloudFormation YAML** plus `cfn-lint`. It has the fewest dependencies and no
     abstraction, but no type checking, and is more verbose to review.
   - Alternative: **OpenTofu** (MPL-2.0).
     - `init` downloads providers from the registry, and `validate` works offline.
     - `plan` without credentials relies on the provider's skip flags; reported as unreliable, not tried.
     - Its plan output only becomes inspectable after `plan`.
   - Rejected:
     - SAM: `sam validate` requires credentials, according to AWS's documentation.
     - Terraform ≥ 1.6: BUSL licence.
     - Pulumi: preview calls the provider with credentials.
2. **Cognito** (plan open decision 6b).
   - ★ **Defer to the backlog**; M6 makes no auth change. The current HS256 and Argon2 auth runs unchanged in
     Lambda. Accepting Cognito tokens would need:
     - a frontend sign-in flow (Hosted UI or SRP);
     - first-sight user provisioning, because a Cognito `sub` has no `users` row and dashboards reference
       `users.id`;
     - an email-collision policy against `users.email UNIQUE`;
     - a second, RS256 verification path, kept strictly apart from HS256.

     A backend-only seam could not be used by anyone and would add a migration and an attack surface without a
     consumer. Cognito's 10,000-MAU free tier (Lite and Essentials) keeps it available for a later milestone.
   - Alternative: **backend verifier seam.** `AUTH_MODE=cognito` would verify RS256 tokens against a cached JWKS using
     `jsonwebtoken` 9.3.1 (`jwk::JwkSet`, `DecodingKey::from_jwk`; no new crate). It would add an `external_sub`
     migration with first-sight provisioning, and disable the password routes in that mode.
     - It would be proven with a locally generated RSA key and a loopback JWKS fixture.
     - Local mode stays the default.
   - Alternative: **full Cognito.** The verifier seam, plus a frontend Hosted UI flow, plus a user pool in the synth.
3. **Lambda entry point.**
   - ★ **Off-by-default cargo feature `lambda` that switches the binary's entry point.**
     - `lambda_http` 1.3 becomes an optional dependency (`default-features = false`, `apigw_http`).
     - Startup steps 2–6 move into one shared function. The server path binds a socket as today; the `lambda` path
       hands the router to `lambda_http::run`, without reading `.env` and without the signal handler.
     - No library split is needed. Default builds, tests, and the release image never compile `lambda_http`.
   - Alternative: detect Lambda at run time (`AWS_LAMBDA_RUNTIME_API`) in every build. Simpler to build, but
     `lambda_http` would then ship in the local binary and image.
   - Alternative: split into a library crate with two binaries. Cleaner targets, but a larger restructuring of a
     crate whose tests live inside it.
4. **Build and package.**
   - ★ **Static `x86_64-unknown-linux-musl` binary built in the pinned `rust:1.99.0-bookworm` container.**
     - The container adds `musl-tools` and the rustup musl target; neither is installed on the host.
     - The binary is zipped as `bootstrap` together with `static/` (the built frontend) for the `provided.al2023`
       runtime (deprecation date 2029-06-30, per AWS's runtime table).
     - A static binary does not depend on the glibc version of Amazon Linux.
     - This route needs no cross-compiler, Zig, or `cargo-lambda`.
   - Alternative: `cargo lambda build --release --arm64 --output-format zip`. arm64 is about 20% cheaper per
     GB-second, which makes no difference inside the free tier. It needs `cargo-lambda` and Zig, two new tools.
5. **TLS stack (D21).**
   - ★ **rustls for sqlx, redis, and reqwest; OpenSSL removed everywhere.**
     - Certificate verification is always on, and no insecure feature (for example redis `tls-rustls-insecure`) is
       compiled in.
     - Plain local URLs keep working:
       - sqlx's default `sslmode=prefer` tries TLS and falls back when the local Postgres offers none [INFERENCE;
         drill 1 proves it];
       - `redis://` stays plaintext.
     - The release image drops `libssl3` and keeps `ca-certificates`.
     - The status widget's TLS changes library. Like OpenSSL 3's defaults on bookworm, rustls accepts only TLS 1.2
       and 1.3.
   - Alternative: **native TLS everywhere.** The musl build would then need vendored OpenSSL, built from source.
   - Alternative: **mixed stacks** (rustls for sqlx and redis, native TLS for reqwest). This leaves two TLS stacks and
     OpenSSL in the Lambda build.
6. **Client address in Lambda.**
   - ★ **Use the request context's source address.** The `lambda` entry point inserts `ConnectInfo` from
     `requestContext.http.sourceIp`. AWS sets this from the TCP connection; it is not a header.
     - This is valid because the shape in decision 7 puts no proxy in front of the function.
     - Forwarding headers stay untrusted, as in M5.
     - If the context has no source address, the existing shared-bucket fallback applies.
     - Limiter state is per execution environment, so limits are per instance, not global. This is a documented
       limit.
   - Alternative: no injection, so every caller shares one auth bucket.
7. **Experiment shape** (what the synthesis describes).
   - ★ **One Lambda function, outside any VPC, with a Function URL** (auth type `NONE`) that serves the single-page
     app and the API from one origin.
     - The function is the same application as the M5 release, so no CORS or second build is needed and the client
       address is real.
     - The stack also contains:
       - RDS PostgreSQL (decision 8);
       - Secrets Manager secrets (decision 10);
       - an explicit log group with retention;
       - project tags on every resource.
     - No Redis (decision 9). No NAT gateway, Elastic IP, CloudFront, S3, or API Gateway.
   - Alternative: **CloudFront plus S3 for the app, and a Function URL or HTTP API for `/api`.**
     - Adds caching at the edge, but more resources to keep secure and to tear down.
     - The source address becomes the CloudFront edge, so client-address trust would need its own decision.
     - Lambda origin access control for POST and PUT expects a payload hash from the viewer [INFERENCE; not
       researched].
   - Rejected: **Lambda inside a VPC with private RDS and ElastiCache.** The widgets need internet egress, which
     means a NAT gateway: about $33 a month idle, plus RDS and cache minimums (about $55–140 a month in total).
8. **Database for the experiment.**
   - ★ **RDS PostgreSQL 16**, with these settings:
     - instance: `db.t4g.micro`, single-AZ, 20 GB gp3, storage encryption on;
     - access: publicly accessible with `rds.force_ssl=1`, and inbound 5432 open to any address, because a Lambda
       outside a VPC has no fixed address. Access is protected by TLS and a generated password; this is a stated
       risk.
     - connection: `sslmode=verify-full` with AWS's public RDS CA bundle.
     - the bundle is committed under `infra/` after one anonymous HTTPS download from AWS's public truststore, with
       its source URL, date, and SHA-256 recorded. That download is an **approval item**: it is a public file, but
       it comes from an AWS host.
     - deletion: no final snapshot (see section 8).
   - Alternative: **Aurora DSQL.** It has an always-free allowance (100,000 DPU and 1 GB a month), scales to zero, and
     needs no VPC. But:
     - it authenticates with IAM tokens, which needs SigV4 signing and therefore a new AWS crate;
     - it uses optimistic concurrency, so the application would need retry semantics;
     - whether our migrations and sqlx's migration locking are supported is unverified.
   - Alternative: Aurora Serverless v2 with a minimum of 0 ACU. Storage is billed while paused, resumes take
     15–30 s, and active hours cost $0.12 per ACU-hour.
   - Alternative: a non-AWS free Postgres with TLS. A new third-party service and key.
9. **Cache in the cloud.**
   - ★ **None.** In the `lambda` entry point, an unset `REDIS_URL` disables the cache:
     - no connection attempts, and a startup log line;
     - `/readyz` reports the cache as disabled rather than down.

     The server binary keeps requiring `REDIS_URL`, so the local behaviour is unchanged. Every widget view then
     calls its provider, still subject to the per-user data rate limit, so provider quotas are used faster. Section 8
     records this as an M7 precondition.
   - Alternative: **ElastiCache.** It is VPC-only, so it brings back decision 7's rejected shape.
   - Alternative: **a third-party free TLS Redis.** A new service and key; it would exercise `rediss://` for real.
10. **Secrets and configuration.**
    - ★ **CloudFormation-generated Secrets Manager secrets** hold the database credentials (generated without
      URL-reserved characters) and `JWT_SECRET`.
      - Provider keys are entered by the owner in M7, never in the repository.
      - The values reach the Lambda environment through CloudFormation dynamic references.
      - No secret value appears in the template or the repository, and the function needs no AWS SDK at run time.
      - Anyone allowed to read the function's configuration in the account can read these environment values; this
        is a stated limit.
      - Cost: $0.40 per secret per month.
    - Alternative: **SSM SecureString** read at run time through the Parameters and Secrets Lambda extension. This
      needs fetch code and a layer, and CloudFormation cannot create SecureString parameters.
11. **Migrations and pool in Lambda.**
    - ★ **Migrations run at startup, as they do locally.** sqlx's migration lock serialises concurrent cold starts,
      and today's migrations are small.
      - A new optional `DB_MAX_CONNECTIONS` (default 10, so local is unchanged) is set to 2 in the synth. One
        execution environment handles one request at a time, so concurrency multiplied by pool size stays small.
    - Alternative: a separate migration invocation, and no migrations at startup in Lambda.
12. **CI.**
    - ★ **Extend the backend job and add one job, still without credentials or secrets.**
      - The `backend` job also runs clippy and tests with `--features lambda`.
      - A new `infra` job runs `make lambda-build` and `make infra-check`: type-check, assertion tests,
        synthesis with cdk-nag, and `cfn-lint`.
      - `permissions: contents: read` stays.
    - Alternative: infrastructure checks run locally only.

### New dependencies and tools (owner approval requested)

| Item | Where | Why | Decision |
| --- | --- | --- | --- |
| `lambda_http` 1.3 (optional; `default-features = false`, `apigw_http`) | `backend/Cargo.toml` | Lambda entry point | 3 |
| sqlx `tls-rustls-ring-webpki` (or the equivalent 0.8 feature), redis `tokio-rustls-comp`, reqwest `rustls-tls` with `default-features = false` (keeping `json`, `charset`, `http2`) | `backend/Cargo.toml` | D21; one TLS stack. Adds rustls crates to `Cargo.lock` and removes `openssl`, `native-tls`, `hyper-tls` | 5 |
| `aws-cdk-lib` 2.273.x, `constructs` 10.x, `cdk-nag` 3.x, `typescript` ~5.9.3, `@types/node` 20.x | new `infra/package.json` and lockfile | Infrastructure as code and its checks; no `aws-cdk` CLI | 1 |
| `cfn-lint` (pinned) in a throwaway `python:3.12-slim` container | `Makefile`, CI | Template validation; nothing installed on the host | 1 |
| `musl-tools` and the rustup `x86_64-unknown-linux-musl` target, inside the build container | `Makefile` | Static Lambda binary | 4 |
| `amazon/aws-lambda-provided:al2023` from Docker Hub (amd64; tag last updated 2025-01-14) | drill 3 only | Runtime Interface Emulator to run `bootstrap` locally | drills |
| AWS RDS global CA bundle (public file, one anonymous download, committed) | `infra/` | `sslmode=verify-full` against RDS | 8 |

Exact versions and feature names are fixed during implementation and recorded in ADR 0004.

### Preparation values (main thread, proposed)

| Item | Value | Section |
| --- | --- | --- |
| Lambda runtime and architecture | `provided.al2023`, `x86_64`, handler `bootstrap` | 2, 5 |
| Lambda memory and timeout | 512 MB (Argon2 uses about 19 MiB per hash; more memory also means more CPU); 15 s (above `REQUEST_TIMEOUT_MS` 10 s) | 5 |
| Function URL | Auth `NONE`, invoke mode buffered, no CORS configuration (same origin) | 5 |
| Lambda environment | `STATIC_DIR=/var/task/static`, `DB_MAX_CONNECTIONS=2`, `RUST_LOG=insightboard_backend=info,tower_http=info`; `REDIS_URL` unset; secrets by dynamic reference | 1, 4, 5 |
| Log group | `/aws/lambda/<function>`, retention 14 days, removed with the stack | 5 |
| RDS | PostgreSQL 16, `db.t4g.micro`, 20 GB gp3, single-AZ, encrypted, `rds.force_ssl=1`, backup retention 1 day, no deletion protection, no final snapshot, automated backups deleted with the instance | 5, 8 |
| Stack | One stack, `InsightBoardExperiment`, environment-agnostic (no account or region in the template); tags `project=insightboard`, `purpose=m7-experiment` | 5 |
| Package | `backend/target/lambda/insightboard-lambda.zip` (gitignored) containing `bootstrap` and `static/` | 2 |
| Synth output | `infra/cdk.out/` (gitignored); the template is attached to the verification record by path and checksum, not committed | 6 |

## 1. Lambda entry point (M6a; decisions 3, 6, 11)

- With the `lambda` feature off, the binary, its tests, the image, and `make` targets behave exactly as in 1.0.0. A
  default `cargo tree` does not contain `lambda_http`.
- Startup is shared by both entry points. The `lambda` path:
  - reads only the process environment, not `.env`;
  - runs migrations;
  - inserts `ConnectInfo` from the request context's source address;
  - logs without ANSI colours;
  - serves through `lambda_http::run`.
- `DB_MAX_CONNECTIONS` is optional for both entry points and defaults to 10.
- Tests, feature-gated, run in-process: Function URL v2 event fixtures go through the Lambda adapter, covering:
  - `/healthz`;
  - a static file;
  - an SPA deep link;
  - `/api/unknown`, which must stay a JSON 404;
  - register and login from two different source addresses, which must land in separate buckets;
  - an event without a source address, which falls back to the shared bucket.

## 2. Build and package (M6a; decision 4)

- `make lambda-build` builds the frontend and the musl binary in containers and writes the package from the
  preparation values. It needs no Rust, Node, AWS tool, or credentials on the host.
- The package contains no `.env`, secrets, source tree, or `node_modules`, and stays within Lambda's zip limits
  (50 MB zipped, 250 MB unzipped [MEMORY]; checked in drill 3).

## 3. TLS to Postgres and Redis (M6a; decision 5; D21)

- `DATABASE_URL` accepts `sslmode` and `sslrootcert`, and `REDIS_URL` accepts `rediss://`, with no code-specific URL
  handling.
- Verification failures (an unknown CA or a hostname mismatch) stop startup with a clear log line. Clients still
  receive only generic errors.
- `.env.example`, local setup, and the runbook document the TLS URL forms and how to supply a custom CA.
- The release image drops `libssl3`, and the M5 smoke checks still pass.

## 4. Cache-disabled mode (M6a; decision 9)

- This mode exists only in the `lambda` entry point, and only when `REDIS_URL` is unset:
  - cache reads miss and writes are skipped, with no connection attempts;
  - `/readyz` checks Postgres only and reports `cache: disabled`.
- The server binary still fails at startup without `REDIS_URL`, as today.

## 5. Infrastructure package (M6b; decisions 1, 7, 8, 10)

- `infra/` holds the following:
  - the CDK app and the one stack described in decisions 7, 8, and 10 and the preparation values;
  - `node:test` assertion tests;
  - a synth script that fails on any cdk-nag error or on an unsuppressed warning;
  - a `README` explaining that M6 is synth-only.
- Every cdk-nag suppression carries a written reason that names a decision in this contract (for example: RDS
  publicly accessible, decision 8; Function URL without IAM auth, decision 7).
- The template contains:
  - no account ID, no region, no secret value, and no context lookup (`fromLookup` and similar methods are
    forbidden);
  - the function outside any VPC;
  - the deletion behaviour from section 8.

## 6. Synth guardrails (M6b)

- The synth script refuses to run if any of these is set: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`,
  `AWS_SESSION_TOKEN`, `AWS_PROFILE`, or `AWS_WEB_IDENTITY_TOKEN_FILE`.
- Synthesis works inside a container with `--network none` and an empty home directory (drill 5).
- `infra/` declares no `aws-cdk` CLI, AWS SDK, or deploy script. `cdk deploy`, `cdk bootstrap`, and any AWS CLI use
  remain forbidden until an approved M7.

## 7. CI (M6b; decision 12)

- The `backend` job runs clippy and tests with `--features lambda`, in addition to `make backend-check`.
- A new `infra` job runs `make lambda-build` and `make infra-check`. It has no services, secrets, or AWS credentials.
- The existing `web`, `e2e`, and `release` jobs are unchanged and stay green.

## 8. Experiment proposal: cost, cleanup, M7 preconditions (M6c)

`docs/operations/aws-experiment-proposal.md` is new. It lists the following:

- the synthesised resources, by type;
- a cost table with sources and retrieval dates;
- guardrails;
- the teardown procedure;
- the conditions M7 must meet before any deploy.

M7's own contract decides whether to deploy at all.

### Preliminary cost (list prices, US East, retrieved 2026-10-10 by M6Ecosystem; re-check before M7)

| Item | Basis | Monthly while the stack exists |
| --- | --- | --- |
| Lambda | Always free: 1M requests and 400,000 GB-s (about 800,000 s at 512 MB) | $0 at experiment scale |
| Function URL | No charge beyond Lambda | $0 |
| RDS `db.t4g.micro` | $0.016 per hour (third-party listing, not AWS) | about $11.68 |
| RDS storage, 20 GB gp3 | about $0.115 per GB-month [MEMORY; not fetched] | about $2.30 |
| Public IPv4 address on RDS | $0.005 per hour | about $3.65 |
| Secrets Manager, 2 secrets | $0.40 per secret per month | $0.80 |
| CloudWatch Logs | 5 GB free | $0 |
| Data transfer out | 100 GB free, aggregated | $0 |
| **Total** | | **about $18.43 a month, about $0.61 a day** |

A three-day M7 experiment would cost about $1.85 at list price. New accounts on AWS's Free plan (from 2025-07-15)
get up to $200 in credits over six months, and the plan closes the account at the end. Accounts on the legacy
12-month free tier include 750 RDS micro hours a month. Neither is assumed: M7 must show a $0 path for the owner's
actual account.

### Cleanup proposal

- Deleting the stack removes everything it created:
  - The RDS instance is deleted with no final snapshot, and its automated backups are deleted with it.
  - The log group is declared in the stack, because Lambda never deletes its own log groups.
  - Secrets are deleted without a recovery window. Otherwise CDK's default schedules deletion with a recovery
    window and the secrets linger [INFERENCE; confirmed in implementation].
- Post-teardown checklist (M7): no RDS instances, snapshots, or automated backups; no `/aws/lambda/InsightBoard*`
  log groups; no secrets, including those pending deletion; no Elastic IPs or NAT gateways. If M7 bootstraps CDK,
  the `CDKToolkit` stack and its staging bucket and repository must be removed as well.
- Guardrails (M7):
  - a monthly budget of $5 with email alerts at 50, 80, and 100 percent (alerting budgets are free);
  - teardown scheduled before the experiment starts;
  - a cost check one day and seven days after teardown.

### M7 preconditions to record (not M6 work)

- **Owner choices:** the account (a dedicated or Free-plan account) and the region.
- **Registration exposure:** a public Function URL lets anyone register and spend the owner's provider quotas. M7
  must decide on a registration control before deploying.
- **Provider terms:** NewsAPI's developer plan is for development, and calls from a server without a cache use
  quotas faster.
- **Provider keys:** entered by the owner into the secret, never committed.

## 9. Documentation (M6c)

- `infra/README.md`: synth-only, the commands, and the guardrails.
- `docs/architecture/system-overview.md`: the `lambda` entry point, the cache-disabled mode, and the TLS URL forms.
- `docs/development/local-setup.md`, `.env.example`, and `docs/operations/runbook.md`: TLS URL forms, a custom CA, and
  `DB_MAX_CONNECTIONS`.
- `CHANGELOG.md`: an `Unreleased` entry.
- README limits: TLS to Postgres and Redis becomes available; local use stays plain HTTP on loopback.
- The main thread writes the verification record, the ADR, and the roadmap.

## 10. Acceptance drills

Evidence goes to `docs/development/m6-verification.md`, written by the main thread. No drill uses AWS credentials,
calls an AWS API, or deploys anything.

| # | Drill | Expected |
| --- | --- | --- |
| 1 | Default build unchanged: `make verify`, `make e2e`, `make release` and `make up` smoke; default `cargo tree` | All green. `lambda_http` and `openssl` are absent from the default tree. The image has no `libssl3`. The local plain Postgres and Redis URLs still work. |
| 2 | `cargo clippy` and `cargo test` with `--features lambda` | Clean. The Lambda adapter tests from section 1 pass. |
| 3 | `make lambda-build`, then run `bootstrap` in the Lambda base image with the Runtime Interface Emulator. The container joins the compose network, is published only on `127.0.0.1`, and has `REDIS_URL` unset. | Under the size limits. The following v2 events all succeed: `/healthz` returns 1.0.0; `/readyz` returns 200 with the cache disabled; `index.html` and a hashed asset come back with the correct type and body; register, create a dashboard, and read it with a throwaway account. Throwaway account and container are removed afterwards. |
| 4 | TLS (D21): throwaway Postgres 16 and Redis 7 containers with TLS on loopback ports and a throwaway CA; the server binary with `sslmode=verify-full`, `sslrootcert`, and `rediss://` | `/readyz` 200 over TLS. A wrong CA or hostname fails with a clear log and no client leak. The throwaway containers, CA, and keys are removed (no named volumes). |
| 5 | Offline synth: `make infra-synth` in a container with `--network none`, no `AWS_*` variables, and an empty home directory; then again with a dummy `AWS_ACCESS_KEY_ID` | The first run produces `cdk.out/InsightBoardExperiment.template.json`. The second run is refused. |
| 6 | Template inspection | The resources match decision 7 exactly; no account, region, secret value, or lookup appears; the function has no VPC; RDS has force-SSL, encryption, and the deletion settings; the log group has retention; every resource is tagged; cdk-nag reports zero unsuppressed findings and every suppression has a reason; `cfn-lint` reports no errors. |
| 7 | Credential hygiene | No AWS CLI, CDK CLI, SDK, or credential file exists on the workstation or in the repository. A secret scan of the M6 range finds nothing. |
| 8 | Proposal and docs check | Every cost figure has a source and date. Docs match `Cargo.toml`, `.env.example`, the `Makefile`, `infra/`, and the image. |
| 9 | CI | `backend` (including the `lambda` feature), `web`, `e2e`, `release`, and `infra` jobs are all green. |

## 11. Out of scope

- Any deploy, `cdk bootstrap`, AWS CLI, credentials, account creation or change, and spending.
- Domains, CloudFront, WAF, API Gateway, VPCs, NAT, ElastiCache, DynamoDB, and Aurora (DSQL or Serverless).
- Cognito (decision 2), registration controls (an M7 precondition), and a per-application database role.
- Forwarding-header trust, a global rate limit across instances, multiple regions, and IPv6.
- A GitHub Release object, and the unscheduled backlog in the
  [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) section 5.
