# InsightBoard Agent Instructions

This AGENTS.md applies to the entire repository.

## Primary Working Source
- Use `/project_progress.md` as the operational progress ledger for what is complete, in progress, and next.

## Project Requirements Analysis

### Product Goals
- Build a production-grade, portfolio-ready dashboard that aggregates external data into customizable widgets.
- Demonstrate full-stack capability: Rust backend engineering, Vue 3 frontend architecture, and cloud-ready deployment practices.
- Optimize for modularity, maintainability, and clear separation of concerns.

### Non-Goals
- Do not treat this project as a full commercial SaaS platform.
- Do not scope into real-time collaborative editing features.

### Architecture Requirements
- Frontend (Vue 3 SPA) communicates with a Rust/Axum API.
- Backend integrates with PostgreSQL for persistent user/dashboard data.
- Redis is required for API response caching and performance.
- External APIs include GitHub, Weather, News, Crypto, and URL status checks.
- Deployment target is AWS-ready serverless architecture (Lambda + API Gateway + S3/CloudFront + EventBridge/SQS patterns).

### Stack and Tooling Constraints
- Backend: Rust + Axum + SQLx + Redis + JWT/Argon2 auth.
- Frontend: Vue 3 + Vite + Pinia + Vue Router (+ GridStack for dashboard layout).
- Infrastructure: PostgreSQL + Redis locally, AWS-oriented deployment targets for production.
- CI/CD expectations: GitHub Actions with automated build/test coverage.
- Avoid introducing alternative frameworks, databases, or state systems unless explicitly requested.

### Core Functional Requirements
- Authentication: register, login, and current-user endpoints with JWT-based auth.
- Dashboard management: authenticated CRUD operations for user-owned dashboards.
- Widget system: pluggable components and backend data routes for 5 MVP widgets.
- Dashboard editor: drag-and-drop layout and widget configuration persistence.
- Auto-refresh and resilient loading/error states across widgets.
- Advanced backlog targets include S3 uploads, AI summary widget, public dashboards, and background refresh jobs.

### Data and Domain Requirements
- `users` table must support unique email, hashed password storage, and created timestamps.
- `dashboards` table must relate to users and persist layout/settings JSON payloads.
- Widget configuration and layout data are first-class persisted entities, not transient UI state.

### API Contract Baseline
- Auth: `POST /api/auth/register`, `POST /api/auth/login`, `GET /api/me`
- Dashboards: `GET /api/dashboards`, `POST /api/dashboards`, `PUT /api/dashboards/:id`, `DELETE /api/dashboards/:id`
- Widget data: `GET /api/data/github`, `GET /api/data/weather`, `GET /api/data/news`, `GET /api/data/crypto`, `GET /api/data/status`

### Auth and Security Requirements
- Development path supports local JWT auth with Argon2 password hashing.
- Production-ready path must remain compatible with Cognito/JWT validation model.
- Protected endpoints require bearer-token middleware and user context extraction.
- Preserve rate-limit/caching behavior where relevant to third-party API protections.

### Quality and Delivery Requirements
- Testing scope includes backend unit/integration tests, frontend component tests, E2E tests, and CI workflows.
- Documentation must explain architecture and rationale (not only change logs), and should be updated when behavior changes.
- Work is phase-based; current unfinished phases are Testing/CI and Deployment Preparation.
- Observability and production-style reliability are expected outcomes (health checks, structured logging/tracing, graceful behavior under failure).

### Implementation Workflow Requirements
- Treat phase progression as explicit milestones; do not skip unfinished core phase work.
- Keep documentation in `Documentation/` educational and rationale-driven when behavior/architecture changes.
- For major milestone completions, update progress state and remaining next-step focus before starting unrelated work.

## Mandatory Progress Tracking (`/project_progress.md`)
- ALWAYS read `/project_progress.md` before starting any implementation task.
- ALWAYS align new work to the active phase and `Next Step` in `/project_progress.md` unless user instructions override it.
- ALWAYS update `/project_progress.md` immediately after meaningful progress, including:
  - `Last Updated` timestamp (with timezone)
  - Phase/subtask status changes
  - What was completed in the session
  - Files created/modified
  - Next concrete step
- NEVER mark work complete in responses without first reflecting it in `/project_progress.md`.
- If implementation direction changes, document the rationale and impact in `/project_progress.md` the same session.
- Treat `/project_progress.md` as mandatory context for every future session in this repository.

## Conventions
- Keep changes minimal and focused; prefer surgical edits over broad refactors unless requested.
- Adhere to the existing stack and tooling; do not introduce new dependencies without explicit approval.
- Match existing code style, naming, and folder structure.
- Update adjacent documentation when changing behavior or public APIs.

## Precedence
- More deeply nested `AGENTS.md` files may refine or override guidance within their directory scope.
- Direct system/developer/user instructions in prompts take precedence over this file.
