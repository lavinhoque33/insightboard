---
applyTo: '**'
---

# InsightBoard — A Full‑Stack, Cloud‑Ready Dashboard (Rust backend + Vue 3)

A production‑grade, portfolio‑ready project that aggregates data from multiple external APIs (GitHub, Weather, News, Crypto, Status pings) into a customizable dashboard. Built to showcase:

## 🎯 IMPLEMENTATION PROTOCOL

**MANDATORY TODO LIST USAGE**: When implementing this project, you MUST use the todo list tool extensively for systematic progress tracking:

1. **Create Todo Entries for ALL Lists**: Any enumerated list, feature set, or multi-step process MUST have corresponding todo list entries
2. **One Todo Per Item**: Each widget, API endpoint, component, configuration file, or feature gets its own todo entry
3. **Granular Tracking**: Break down large tasks into atomic, completable units
4. **Status Management**: Mark items as in-progress when starting, completed immediately upon finishing
5. **Progressive Implementation**: Work through todos sequentially, never skip ahead

**MANDATORY STOPPING PROTOCOL**: After completing each major phase or significant milestone:

1. **STOP and explain** all changes made in that phase
2. **List files created/modified** with brief description of each
3. **Summarize what was accomplished** and how it fits into the overall architecture
4. **UPDATE THIS INSTRUCTION FILE** with current progress:
   - Mark completed phases with ✅
   - Update the "Last Updated" timestamp
   - Add new files to the "FILES CREATED" section
   - Update "NEXT STEP" with what comes next
   - Document any architectural decisions made
5. **Ask user for confirmation** before proceeding to the next phase
6. **Wait for explicit user approval** (e.g., "continue", "proceed", "yes") before moving forward
7. **Never implement more than one major phase** without user confirmation

**Examples of What Requires Todo Entries**:

-   Each of the 5 MVP widgets (GitHub, Weather, News, Crypto, Status)
-   Each API endpoint in the API Design section
-   Each database table in the schema
-   Each configuration file mentioned
-   Each frontend component or view
-   Each test suite or CI/CD workflow
-   Each deployment step

**Todo List Best Practices**:

-   Create comprehensive todo list BEFORE starting implementation
-   Update status in real-time as you work
-   Never complete multiple items at once - complete individually
-   Add newly discovered tasks immediately
-   Use descriptive titles that indicate the specific work

---

## 📊 CURRENT IMPLEMENTATION STATUS

**Last Updated**: October 31, 2025

**Overall Progress**: � IN PROGRESS - Backend Core Complete, Frontend Pending

### Phase 1: Project Foundation ✅ COMPLETED

-   [x] Backend Rust project initialization (Cargo.toml with dependencies)
-   [ ] Frontend Vue 3 project setup (Vite + TypeScript)
-   [ ] Docker compose for local Postgres & Redis
-   [ ] Environment configuration files (.env.example)
-   [x] Basic project structure and directories

### Phase 2: Backend Core ✅ COMPLETED

-   [x] Database connection and SQLx setup
-   [x] Redis cache client setup
-   [x] Configuration management (config.rs)
-   [x] Database migrations (users, dashboards tables)
-   [x] Basic Axum server with health check endpoint

### Phase 3: Authentication ✅ COMPLETED

-   [x] User registration endpoint (POST /api/auth/register)
-   [x] User login endpoint (POST /api/auth/login)
-   [x] JWT token generation and validation
-   [x] Auth middleware for protected routes
-   [x] Current user endpoint (GET /api/me)

### Phase 4: Dashboard Management ✅ COMPLETED

-   [x] Get user dashboards endpoint (GET /api/dashboards)
-   [x] Create dashboard endpoint (POST /api/dashboards)
-   [x] Update dashboard endpoint (PUT /api/dashboards/:id)
-   [x] Delete dashboard endpoint (DELETE /api/dashboards/:id)

### Phase 5: Widget Implementations ✅ COMPLETED

-   [x] GitHub widget backend (GET /api/data/github)
-   [x] Weather widget backend (GET /api/data/weather)
-   [x] News widget backend (GET /api/data/news)
-   [x] Crypto widget backend (GET /api/data/crypto)
-   [x] Status ping widget backend (GET /api/data/status)

### Phase 6: Frontend Foundation ⬜ NOT STARTED

-   [ ] API service layer (axios setup)
-   [ ] Pinia store setup (auth, dashboard stores)
-   [ ] Vue Router configuration
-   [ ] Authentication views (Login, Register)
-   [ ] Protected route guards

### Phase 7: Frontend Dashboard UI ⬜ NOT STARTED

-   [ ] Dashboard list view
-   [ ] Dashboard editor view
-   [ ] Widget registry system
-   [ ] GridStack drag-and-drop integration
-   [ ] Widget configuration modal

### Phase 8: Widget Components ⬜ NOT STARTED

-   [ ] GitHub widget component
-   [ ] Weather widget component
-   [ ] News widget component
-   [ ] Crypto widget component
-   [ ] Status widget component

### Phase 9: Polish & Features ⬜ NOT STARTED

-   [ ] Auto-refresh functionality
-   [ ] Loading states and error handling
-   [ ] Responsive design
-   [ ] Dark mode support (optional)
-   [ ] Widget settings persistence

### Phase 10: Testing & CI/CD ⬜ NOT STARTED

-   [ ] Backend unit tests
-   [ ] Backend integration tests
-   [ ] Frontend component tests (Vitest)
-   [ ] E2E tests (Playwright)
-   [ ] GitHub Actions CI workflow

### Phase 11: Deployment Prep ⬜ NOT STARTED

-   [ ] Dockerfile for backend
-   [ ] Production build configuration
-   [ ] AWS Lambda packaging
-   [ ] Terraform/CloudFormation templates
-   [ ] Deployment documentation

**NEXT STEP**: Complete Phase 1 infrastructure (Docker compose + environment config), then proceed to Phase 6 (Frontend Foundation)

**FILES CREATED IN BACKEND**:

-   `/backend/Cargo.toml` - Complete Rust dependencies configuration
-   `/backend/src/main.rs` - Axum server with routing, middleware, graceful shutdown
-   `/backend/src/error.rs` - Custom error types and HTTP response mapping
-   `/backend/src/config.rs` - Environment variable configuration management
-   `/backend/src/db.rs` - PostgreSQL connection pool with SQLx
-   `/backend/src/cache.rs` - Redis cache wrapper with TTL support
-   `/backend/src/auth.rs` - JWT generation/validation, Argon2 password hashing, UserCtx extractor
-   `/backend/src/models/mod.rs` - Model module organization
-   `/backend/src/models/user.rs` - User model and auth DTOs
-   `/backend/src/models/dashboard.rs` - Dashboard model and CRUD DTOs
-   `/backend/src/handlers/mod.rs` - Handler module organization
-   `/backend/src/handlers/health.rs` - Health check endpoint
-   `/backend/src/handlers/auth.rs` - Register, login, and me endpoints
-   `/backend/src/handlers/dashboard.rs` - Full CRUD operations for dashboards
-   `/backend/src/widgets/mod.rs` - Widget module organization
-   `/backend/src/widgets/github.rs` - GitHub events widget with caching
-   `/backend/src/widgets/weather.rs` - OpenWeather API integration
-   `/backend/src/widgets/news.rs` - NewsAPI integration with topic filtering
-   `/backend/src/widgets/crypto.rs` - CoinGecko API for crypto prices
-   `/backend/src/widgets/status.rs` - URL status checker with response times
-   `/backend/migrations/001_create_users.sql` - Users table migration
-   `/backend/migrations/002_create_dashboards.sql` - Dashboards table migration

**BACKEND ARCHITECTURE COMPLETED**:
✅ Clean modular structure with separation of concerns
✅ Async Axum web server with Tower middleware
✅ PostgreSQL integration with SQLx (compile-time query checking)
✅ Redis caching layer with TTL support
✅ JWT-based authentication with Argon2 password hashing
✅ Protected route middleware using UserCtx extractor
✅ Complete CRUD API for dashboards
✅ Five fully functional widget backends with caching
✅ Comprehensive error handling and type-safe responses
✅ Production-ready features: CORS, compression, tracing, graceful shutdown

**INSTRUCTIONS FOR NEW CHAT SESSIONS**:

1. Read this status section to understand current progress
2. Check the last completed phase
3. Start from the first uncompleted item in the next phase
4. Update this section after completing each major milestone
5. Mark phases as: ⬜ NOT STARTED | 🟡 IN PROGRESS | ✅ COMPLETED

---

-   **Rust backend** (Axum) with async concurrency, caching, rate limiting, auth, and clean architecture.
-   **Vue 3 frontend** with a pluggable **widget** system, drag‑and‑drop layout, and real‑time refresh.
-   **AWS‑ready infrastructure** (Lambda + API Gateway + S3 + CloudFront + EventBridge + SQS) deployable mostly within free tiers.
-   **DevEx**: OpenAPI, CI/CD, testing (unit/integration/E2E), observability.

> **One‑liner for your README**  
> _“InsightBoard: a modular, cloud‑native dashboard that normalizes and caches data from 3rd‑party APIs, demonstrating Rust microservices, Vue UI engineering, and AWS serverless.”_

---

## Table of Contents

-   [Goals & Non‑Goals](#goals--non-goals)
-   [Architecture Overview](#architecture-overview)
-   [Tech Stack](#tech-stack)
-   [Feature Set](#feature-set)
-   [Repository Layout](#repository-layout)
-   [Local Development](#local-development)
-   [Configuration & Secrets](#configuration--secrets)
-   [Database Schema](#database-schema)
-   [Authentication](#authentication)
-   [API Design](#api-design)
-   [Backend Notes (Rust)](#backend-notes-rust)
-   [Frontend Notes (Vue 3)](#frontend-notes-vue-3)
-   [Widget System](#widget-system)
-   [Testing](#testing)
-   [Deployment](#deployment)
-   [Roadmap](#roadmap)

---

## Goals & Non‑Goals

**Goals**

-   Demonstrate fullstack skills (Rust backend, Vue frontend, AWS deploy).
-   Modular, cloud‑native, async backend with caching.
-   Production‑style architecture for portfolio.

**Non‑Goals**

-   Not a full commercial SaaS.
-   Not a real‑time collaborative tool.

---

## Architecture Overview

```
Frontend (Vue3) <-> API Gateway <-> Lambda (Rust/Axum)
                                |
                        +-------+---------+
                        | Postgres (Neon) |
                        | Redis (Upstash) |
                        +-----------------+
                                |
                           External APIs
```

---

## Tech Stack

| Layer    | Tech                                    | Purpose               |
| -------- | --------------------------------------- | --------------------- |
| Backend  | Rust + Axum                             | Async REST API        |
| Database | Postgres (Neon/Supabase)                | User & dashboard data |
| Cache    | Redis (Upstash)                         | API response cache    |
| Frontend | Vue 3 + Vite + Pinia                    | SPA dashboard UI      |
| Infra    | AWS Lambda, API Gateway, S3, CloudFront | Free‑tier deploy      |
| CI/CD    | GitHub Actions                          | Build, test, deploy   |

---

## Feature Set

**MVP**

-   Auth (JWT or Cognito)
-   Dashboard CRUD
-   5 widgets (GitHub, Weather, News, Crypto, Status)
-   Drag‑and‑drop layout
-   Auto refresh

**Advanced**

-   S3 uploads
-   AI summary widget
-   Public dashboards
-   Background job refresh

---

## Repository Layout

```
insightboard/
├─ backend/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ widgets/
│  │  │  ├─ github.rs
│  │  │  ├─ weather.rs
│  │  │  └─ ...
│  │  ├─ auth.rs
│  │  ├─ db.rs
│  │  ├─ cache.rs
│  │  └─ config.rs
├─ frontend/
│  ├─ src/
│  │  ├─ components/widgets/
│  │  ├─ store/
│  │  └─ services/
└─ infra/
   └─ terraform/
```

---

## Local Development

**Requirements**

-   Rust (stable), Node 18+, Docker (for Postgres & Redis)
-   `sqlx-cli` and `cargo-watch` optional

**Setup**

```bash
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:16
docker run -d -p 6379:6379 redis:7
cp .env.example .env
```

**Run**

```bash
cd backend && cargo run
cd frontend && pnpm dev
```

---

## Configuration & Secrets

`.env.example`

```
APP_PORT=8080
DATABASE_URL=postgres://postgres:postgres@localhost:5432/insight
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=dev-secret
OPENWEATHER_API_KEY=
NEWSAPI_API_KEY=
```

---

## Database Schema (Simplified)

```sql
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT UNIQUE NOT NULL,
  password_hash TEXT,
  created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE dashboards (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id),
  layout_json JSONB DEFAULT '[]',
  settings_json JSONB DEFAULT '{}'
);
```

---

## Authentication

Two options:

1. Local JWT (for dev) using Argon2 password hashing.
2. Cognito JWT validation (for prod).

Middleware checks `Authorization: Bearer <token>` and injects `UserCtx`.

---

## API Design

### Auth

-   `POST /api/auth/register`
-   `POST /api/auth/login`
-   `GET /api/me`

### Dashboards

-   `GET /api/dashboards`
-   `POST /api/dashboards`
-   `PUT /api/dashboards/:id`

### Widgets

-   `GET /api/data/github?username=...`
-   `GET /api/data/weather?city=...`
-   `GET /api/data/news?topic=...`

---

## Backend Notes (Rust)

Use **Axum + SQLx + Redis + JWT**.

### Example: main.rs

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .nest("/api", api::routes());
    axum::serve(TcpListener::bind("0.0.0.0:8080").await.unwrap(), app).await.unwrap();
}
```

### Example: Widget Handler

```rust
pub async fn github_activity(
  State(st): State<AppState>,
  Query(q): Query<GithubQuery>
) -> impl IntoResponse {
  let cache_key = format!("github:{}", q.username);
  if let Some(cached) = st.cache_get(&cache_key).await? {
      return Json(cached);
  }
  let res = reqwest::get(format!("https://api.github.com/users/{}/events", q.username))
      .await?.json::<Vec<_>>().await?;
  st.cache_put(&cache_key, &res, 300).await?;
  Json(res)
}
```

---

## Frontend Notes (Vue 3)

### Setup

```bash
pnpm create vite frontend --template vue-ts
cd frontend && pnpm add axios pinia vue-router chart.js gridstack
```

### Example: Widget Component

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '@/services/api';
const props = defineProps<{ username: string }>();
const items = ref([]);
onMounted(async () => {
	const { data } = await api.get('/data/github', {
		params: { username: props.username },
	});
	items.value = data;
});
</script>

<template>
	<div class="widget">
		<h3>GitHub: {{ props.username }}</h3>
		<ul>
			<li v-for="ev in items">{{ ev.type }}</li>
		</ul>
	</div>
</template>
```

---

## Widget System

Each widget has:

-   Backend route `/api/data/{widget}`
-   Config JSON schema
-   Frontend Vue component

Widget registry maps ID → component.

To add a new widget:

1. Add backend route in Rust.
2. Add component under `components/widgets`.
3. Update registry in store.

---

## Testing

**Backend:** `cargo test` using mock responses.  
**Frontend:** `vitest` and `playwright` for E2E.  
**CI:** GitHub Actions build/test workflows.

---

## Deployment

Free‑tier AWS deployment:

-   Lambda + API Gateway for backend
-   Neon or Supabase for DB
-   Upstash Redis
-   S3 + CloudFront (or Vercel) for frontend
-   EventBridge for background refresh jobs

No paid servers required.

---

## Roadmap

-   [x] Auth (JWT)
-   [x] 5 widgets
-   [ ] Background refresh job
-   [ ] Cognito integration
-   [ ] AI summary widget
-   [ ] Public dashboards

---

## License

MIT
