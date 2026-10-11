# AWS experiment proposal (M7 input)

- Status: **proposed** input for a later M7 contract. Nothing described here exists in any AWS account. M6 synthesises
  the template offline and inspects it; it never deploys, bootstraps, or calls AWS.
- Sources: [M6 contract](../architecture/m6-aws-readiness-contract.md) sections 7–8 and decisions 7–11,
  [ADR 0004](../decisions/ADR-0004-m6-aws-readiness.md), [`infra/`](../../infra/README.md), and the
  [M6 verification record](../development/m6-verification.md) (template checksum and inspection).
- Whether to deploy at all is decided in M7's own contract, with explicit owner approval and a $0 path for the owner's
  actual account (plan section 4.1).

## What the stack would create

One environment-agnostic CloudFormation stack, `InsightBoardExperiment`, synthesised by `make infra-synth` into
`infra/cdk.out/InsightBoardExperiment.template.json` (gitignored). 25 resources, all tagged `project=insightboard`
and `purpose=m7-experiment` where the type supports tags:

| Resource type | Count | Purpose |
| --- | --- | --- |
| `AWS::Lambda::Function` | 1 | `InsightBoardExperiment`: `provided.al2023`, x86_64, `bootstrap`, 512 MB, 15 s, outside any VPC; the `make lambda-build` package (binary, built frontend, RDS CA bundle) |
| `AWS::Lambda::Url` | 1 | Public HTTPS endpoint, auth `NONE`, buffered, no CORS; serves the app and `/api` from one origin |
| `AWS::Lambda::Permission` | 2 | Public invoke through the Function URL only (`lambda:InvokeFunctionUrl` with auth `NONE`, and `lambda:InvokeFunction` restricted to invocations via the Function URL) |
| `AWS::IAM::Role`, `AWS::IAM::Policy` | 1 + 1 | Function role: `logs:CreateLogStream` and `logs:PutLogEvents` on its own log group only |
| `AWS::Logs::LogGroup` | 1 | `/aws/lambda/InsightBoardExperiment`, 14-day retention, deleted with the stack |
| `AWS::RDS::DBInstance` | 1 | PostgreSQL 16, `db.t4g.micro`, 20 GB gp3, single-AZ, encrypted, publicly accessible, backups 1 day |
| `AWS::RDS::DBParameterGroup` | 1 | `postgres16` with `rds.force_ssl=1` |
| `AWS::RDS::DBSubnetGroup` | 1 | The two public subnets |
| `AWS::EC2::VPC`, `Subnet`, `RouteTable`, `SubnetRouteTableAssociation`, `Route`, `InternetGateway`, `VPCGatewayAttachment` | 1, 2, 2, 2, 2, 1, 1 | Minimal VPC hosting RDS only: two public subnets in two zones, no NAT gateway, no endpoints |
| `AWS::EC2::SecurityGroup` | 1 | TCP 5432 from any IPv4 address (the function has no fixed address; decision 8) |
| `AWS::SecretsManager::Secret` | 2 | Generated database credentials, and the app secret (generated `JWT_SECRET`, empty provider-key fields) |
| `AWS::SecretsManager::SecretTargetAttachment` | 1 | Links the database secret to the instance |

Not in the template: NAT gateway, Elastic IP, VPC endpoints, custom resources or helper Lambdas, CloudFront, S3
buckets, API Gateway, ElastiCache, Cognito, and any secret value. Secrets reach the function environment as
CloudFormation dynamic references (`{{resolve:secretsmanager:…}}`); `DATABASE_URL` uses `sslmode=verify-full` with
`sslrootcert=/var/task/certs/rds-global-bundle.pem`.

## Cost while the stack exists

On-demand list prices for US East (N. Virginia), monthly figures at 730 hours. Prices change; M7 must re-check them
for the chosen region before any deploy.

| Item | Basis | Source (retrieved 2026-10-11 UTC unless stated) | Monthly |
| --- | --- | --- | --- |
| RDS `db.t4g.micro`, Single-AZ, PostgreSQL | $0.016 per hour | AWS price list behind [RDS for PostgreSQL pricing](https://aws.amazon.com/rds/postgresql/pricing/) (`rds-postgresql-ondemand`, published 2026-10-06) | $11.68 |
| RDS storage, 20 GB gp3, Single-AZ | $0.115 per GB-month | Same price list | $2.30 |
| Public IPv4 address on the RDS instance | $0.005 per hour, in use or idle | [Amazon VPC pricing](https://aws.amazon.com/vpc/pricing/), "Public IPv4 Address" | $3.65 |
| Secrets Manager, 2 secrets | $0.40 per secret per month; API calls $0.05 per 10,000 (dynamic references resolve at deploy time, so negligible) | [AWS Secrets Manager pricing](https://aws.amazon.com/secrets-manager/pricing/) | $0.80 |
| Lambda requests and compute | Free tier: 1 million requests and 400,000 GB-seconds a month (about 800,000 s at 512 MB) | [AWS Lambda pricing](https://aws.amazon.com/lambda/pricing/) | $0 at experiment scale |
| Function URL | No charge beyond Lambda | M6Ecosystem, reported 2026-10-10 | $0 |
| CloudWatch Logs | 5 GB ingestion free | M6Ecosystem, reported 2026-10-10 | $0 at experiment scale |
| Data transfer out | 100 GB a month free, aggregated across services | M6Ecosystem, reported 2026-10-10 | $0 at experiment scale |
| VPC, subnets, route tables, internet gateway, security group | No charge for the resources themselves | [MEMORY]; M7 re-checks | $0 |
| RDS backup storage, 1-day retention | Free up to the provisioned storage size | [MEMORY]; M7 re-checks | $0 |
| **Total** | | | **$18.43 a month, about $0.61 a day** |

A three-day experiment costs about $1.85 at list price. Free-plan credits (accounts created from 2025-07-15) or the
legacy 12-month free tier (750 RDS micro hours a month) could bring that to $0, but neither is assumed: M7 must show
the $0 path for the owner's actual account (M6 contract, section 8).

If M7 bootstraps CDK to upload the Lambda package, the `CDKToolkit` stack adds an S3 staging bucket and an ECR
repository; the 4.4 MB package is a negligible storage charge [INFERENCE], but both must be removed at teardown.

## Guardrails (M7)

- A monthly AWS Budget of $5 with email alerts at 50, 80, and 100 percent, created before the deploy (alerting
  budgets are free; M6Ecosystem, reported 2026-10-10).
- Teardown date fixed before the experiment starts; the experiment runs for days, not weeks.
- A cost check one day and seven days after teardown (Billing console, Cost Explorer).
- Human-scale use only: no load tests, and provider calls stay within the approved targets.

## Teardown

Deleting the stack removes everything it created:

- the RDS instance with no final snapshot (`DeletionPolicy: Delete`) and its automated backups
  (`DeleteAutomatedBackups: true`), with deletion protection off;
- the log group (declared in the stack, because Lambda never deletes its own log groups);
- both secrets (`DeletionPolicy: Delete`). CloudFormation deletes a secret immediately with
  `ForceDeleteWithoutRecovery` (AWS CloudFormation `DeletionPolicy` attribute reference, retrieved 2026-10-10). CDK's
  default for secrets is `RETAIN`, so the stack sets `DESTROY` explicitly;
- the VPC, subnets, routes, internet gateway, security group, function, URL, role, and policy.

Post-teardown checklist:

1. No RDS instances, manual snapshots, or automated backups in the region.
2. No `/aws/lambda/InsightBoard*` log groups.
3. No Secrets Manager secrets, including any scheduled for deletion.
4. No Elastic IPs, NAT gateways, or leftover network interfaces.
5. If CDK was bootstrapped: the `CDKToolkit` stack, its S3 staging bucket (emptied first), and its ECR repository
   are removed.
6. The budget shows no new charges at the one-day and seven-day checks.

## M7 preconditions

M6 records these; M7's contract must settle each before any deploy.

- **Account and region:** the owner chooses a dedicated or Free-plan account and a region; prices are re-checked
  there.
- **Registration control:** a public Function URL lets anyone register and spend the owner's provider quotas; M7
  decides on a control (for example an invite code or a closed registration) before deploying.
- **Provider terms and quotas:** NewsAPI's developer plan is for development use; without a cache in the cloud
  (decision 9) every widget view calls its provider, so quotas are used faster.
- **Provider keys:** the owner fills the app secret's three provider-key fields in the console or leaves them
  empty. M7 verifies that dynamic references to the empty fields resolve to empty values (the backend treats blank
  as unset) [INFERENCE; untested without AWS].
- **Engine version:** the template pins major version `16` (cfn-lint W3691 ignored on that resource). M7 confirms
  that RDS accepts a major-only version at create time or pins a minor version.
- **Asset upload:** the template references the default CDK asset bucket
  (`cdk-hnb659fds-assets-<account>-<region>`) and the bootstrap-version rule. M7 chooses between `cdk bootstrap`
  (then removes `CDKToolkit` at teardown) and another upload path (for example a custom synthesizer with an
  owner-created bucket).
- **One copy per account and region:** the function and log group have fixed names, so only one copy of the stack
  can exist in an account and region.
- **Stated risks carried forward (ADR 0004):** a public Postgres endpoint protected by TLS and a generated password;
  secrets readable by anyone who can read the function configuration; per-instance rate limits; no cache.
