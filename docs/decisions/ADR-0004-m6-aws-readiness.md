# ADR 0004: M6 AWS readiness decisions

- Status: **accepted** 2026-10-10. The owner approved every ★ recommendation of the
  [M6 contract](../architecture/m6-aws-readiness-contract.md) (decisions 1–12), its new-dependency list, and the
  one-time anonymous download of the public RDS CA bundle ("Take all the recommendations and start").
- Milestone: M6 (slices M6a, M6b, M6c).
- Resolves: [project plan](../../INSIGHTBOARD_PROJECT_PLAN.md) section 17, open decision 6: the infrastructure-as-code
  tool, and whether Cognito lands in M6 or stays in the backlog. It also resolves defect D21 (no TLS for sqlx and
  redis).
- Sources: [M6 contract](../architecture/m6-aws-readiness-contract.md), session
  [2026-10-10-08](../development/sessions/2026-10-10-08-m6-contract.md).

## Context

M5 was accepted on 2026-10-10, and release 1.0.0 is tagged `v1.0.0`. The plan asks M6 for three things:

- cloud adapters;
- TLS to Postgres and Redis;
- an inspectable, synth-only proposal for a later AWS experiment (M7).

The exit evidence is synth output that has been inspected, with no AWS credentials used and nothing deployed. The
hard boundaries in plan section 4.1 forbid credentials, control-plane calls, resources, deploys, and spending. The
local product must stay unchanged when cloud features are off.

## Decisions

★ marks the contract's recommendation. The owner approved every one on 2026-10-10.

| # | Decision | ★ Recommendation | Owner choice |
| --- | --- | --- | --- |
| 1 | Infrastructure-as-code tool | AWS CDK v2 in TypeScript in `infra/`; libraries only, with no CDK CLI; synthesised offline; checked with cdk-nag, assertion tests, and `cfn-lint` | ★ approved |
| 2 | Cognito | Defer to the backlog; no auth change in M6 | ★ approved |
| 3 | Lambda entry point | Off-by-default cargo feature `lambda` that switches the single binary's entry point (`lambda_http` 1.3, `apigw_http` only) | ★ approved |
| 4 | Build and package | Static x86_64 musl `bootstrap` built in the pinned Rust container, zipped with `static/`, for `provided.al2023` | ★ approved |
| 5 | TLS stack (D21) | rustls for sqlx, redis, and reqwest; OpenSSL removed; verification always on | ★ approved |
| 6 | Client address in Lambda | `ConnectInfo` from the request context's `sourceIp`; forwarding headers stay untrusted | ★ approved |
| 7 | Experiment shape | One Lambda function outside any VPC, with a public Function URL, serving the app and the API on one origin | ★ approved |
| 8 | Database | RDS PostgreSQL 16 `db.t4g.micro`, public with forced TLS and `verify-full` against the committed public RDS CA bundle | ★ approved, including the bundle download |
| 9 | Cache in the cloud | None: an unset `REDIS_URL` disables the cache in the `lambda` entry point only | ★ approved |
| 10 | Secrets | CloudFormation-generated Secrets Manager secrets, passed to the function as dynamic references | ★ approved |
| 11 | Migrations and pool | Migrations at startup, as locally; optional `DB_MAX_CONNECTIONS` (default 10; 2 in the synth) | ★ approved |
| 12 | CI | `backend` job adds `--features lambda`; new `infra` job; no credentials or secrets | ★ approved |

### As implemented (M6a and M6b, 2026-10-10)

| Area | Exact versions and features |
| --- | --- |
| Lambda entry point | `lambda_http` 1.3.1 (optional, `default-features = false`, `apigw_http`) behind cargo feature `lambda = ["dep:lambda_http"]`; resolves `lambda_runtime` 1.4.0 and `aws_lambda_events` 1.2.0. No AWS SDK crate in either tree. |
| sqlx 0.8.6 | `tls-rustls-ring-webpki` added; `sslrootcert=` adds a CA to the bundled webpki roots |
| redis 0.27.6 | `tokio-rustls-comp` added; trust from `rustls-native-certs` 0.7.3, so `SSL_CERT_FILE` replaces the system store for Redis only |
| reqwest 0.12.28 | `default-features = false` with `json`, `rustls-tls`, `charset`, `http2` |
| TLS stack | rustls 0.23.45 with ring 0.17.14, tokio-rustls 0.26.6, hyper-rustls 0.27.10 |
| `Cargo.lock` | Removed: `openssl`, `openssl-sys`, `openssl-macros`, `native-tls`, `hyper-tls`, `tokio-native-tls`, `foreign-types`, `tempfile`, `rustix` and others. `openssl-probe` moved from 0.2.1 to 0.1.6 (pure Rust, through `rustls-native-certs`). `quinn` is listed but compiled by neither feature set (`cargo tree -i quinn` is empty). |
| `infra/` | `aws-cdk-lib` 2.273.0, `constructs` 10.8.1, `cdk-nag` 3.0.2 (v3 `Validations` plugin API), `typescript` ~5.9.3, `@types/node` 20.19.43; `cfn-lint` 1.57.2 in `python:3.12-slim` |
| Package | `backend/Dockerfile.lambda` builds `x86_64-unknown-linux-musl` in `rust:1.99.0-bookworm` with `musl-tools`; the zip holds `bootstrap` (static-pie), `static/`, and `certs/rds-global-bundle.pem` |

## Consequences

- **New dependencies**, approved with the contract:
  - Rust: `lambda_http` (optional) and the TLS features of sqlx, redis, and reqwest. These add rustls crates to
    `Cargo.lock` and remove `openssl`, `native-tls`, and `hyper-tls`.
  - The new `infra/` package: `aws-cdk-lib`, `constructs`, `cdk-nag`, `typescript`, `@types/node`.
  - Container-only tools: `cfn-lint`, `musl-tools`, and the Lambda base image for a local emulator drill.
  - A one-time, anonymous download of AWS's public RDS CA bundle: done 2026-10-10 from
    `https://truststore.pki.rds.amazonaws.com/global/global-bundle.pem` into `infra/certs/rds-global-bundle.pem`
    (111 certificates, 169,984 bytes, SHA-256 `fe45bbebf92ad3e27a583bbb2ddd1553c521ed4d49af5514dc0a40372ea5395c`).
- **Release image:** the Dockerfile no longer installs `libssl3` and the binary links no OpenSSL; `libssl3` stays in
  the image only as a dependency of Debian's `curl` and `ca-certificates`. The status widget and the providers move
  from OpenSSL to rustls; both accept TLS 1.2 and 1.3 only.
- **Readiness contract:** `/readyz` gains an optional `cache: "disabled"` field (Lambda without `REDIS_URL`);
  `contracts/api-contract.v1.json` and `apiContract.ts` changed together.
- **Redis TLS:** a verification failure stays fail-open (warning, cache misses, `/readyz` 503), as for an
  unreachable Redis; a Postgres verification failure stops startup.
- **Default build:** the repository gains no deploy path. `cdk deploy`, `cdk bootstrap`, and the AWS CLI stay
  forbidden until an approved M7 contract.
- **Cognito:** it stays in the plan's section 5 backlog.
- **Accepted risks of the experiment shape**, carried into M7 as preconditions:
  - a public Postgres endpoint;
  - secrets readable by anyone who can read the function's configuration;
  - open registration on a public URL;
  - per-instance rate limits;
  - no cache in the cloud.
