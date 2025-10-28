# InsightBoard — A Full‑Stack, Cloud‑Ready Dashboard (Rust backend + Vue 3)

A production‑grade, portfolio‑ready project that aggregates data from multiple external APIs (GitHub, Weather, News, Crypto, Status pings) into a customizable dashboard. Built to showcase:

- **Rust backend** (Axum) with async concurrency, caching, rate limiting, auth, and clean architecture.
- **Vue 3 frontend** with a pluggable **widget** system, drag‑and‑drop layout, and real‑time refresh.
- **AWS‑ready infrastructure** (Lambda + API Gateway + S3 + CloudFront + EventBridge + SQS) deployable mostly within free tiers.
- **DevEx**: OpenAPI, CI/CD, testing (unit/integration/E2E), observability.

> **One‑liner for your README**  
> *“InsightBoard: a modular, cloud‑native dashboard that normalizes and caches data from 3rd‑party APIs, demonstrating Rust microservices, Vue UI engineering, and AWS serverless.”*

---

## Table of Contents

- [Goals & Non‑Goals](#goals--non-goals)
- [Architecture Overview](#architecture-overview)
- [Tech Stack](#tech-stack)
- [Feature Set](#feature-set)
- [Repository Layout](#repository-layout)
- [Local Development](#local-development)
- [Configuration & Secrets](#configuration--secrets)
- [Database Schema](#database-schema)
- [Authentication](#authentication)
- [API Design](#api-design)
- [Backend Notes (Rust)](#backend-notes-rust)
- [Frontend Notes (Vue 3)](#frontend-notes-vue-3)
- [Widget System](#widget-system)
- [Testing](#testing)
- [Deployment](#deployment)
- [Roadmap](#roadmap)

---

## Goals & Non‑Goals

**Goals**
- Demonstrate fullstack skills (Rust backend, Vue frontend, AWS deploy).
- Modular, cloud‑native, async backend with caching.
- Production‑style architecture for portfolio.

**Non‑Goals**
- Not a full commercial SaaS.
- Not a real‑time collaborative tool.

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

| Layer | Tech | Purpose |
|-------|------|----------|
| Backend | Rust + Axum | Async REST API |
| Database | Postgres (Neon/Supabase) | User & dashboard data |
| Cache | Redis (Upstash) | API response cache |
| Frontend | Vue 3 + Vite + Pinia | SPA dashboard UI |
| Infra | AWS Lambda, API Gateway, S3, CloudFront | Free‑tier deploy |
| CI/CD | GitHub Actions | Build, test, deploy |

---

## Feature Set

**MVP**
- Auth (JWT or Cognito)
- Dashboard CRUD
- 5 widgets (GitHub, Weather, News, Crypto, Status)
- Drag‑and‑drop layout
- Auto refresh

**Advanced**
- S3 uploads
- AI summary widget
- Public dashboards
- Background job refresh

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
- Rust (stable), Node 18+, Docker (for Postgres & Redis)
- `sqlx-cli` and `cargo-watch` optional

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
- `POST /api/auth/register`
- `POST /api/auth/login`
- `GET /api/me`

### Dashboards
- `GET /api/dashboards`
- `POST /api/dashboards`
- `PUT /api/dashboards/:id`

### Widgets
- `GET /api/data/github?username=...`
- `GET /api/data/weather?city=...`
- `GET /api/data/news?topic=...`

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
  const { data } = await api.get('/data/github', { params: { username: props.username } });
  items.value = data;
});
</script>

<template>
  <div class="widget">
    <h3>GitHub: {{ props.username }}</h3>
    <ul><li v-for="ev in items">{{ ev.type }}</li></ul>
  </div>
</template>
```

---

## Widget System

Each widget has:
- Backend route `/api/data/{widget}`
- Config JSON schema
- Frontend Vue component

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
- Lambda + API Gateway for backend
- Neon or Supabase for DB
- Upstash Redis
- S3 + CloudFront (or Vercel) for frontend
- EventBridge for background refresh jobs

No paid servers required.

---

## Roadmap

- [x] Auth (JWT)
- [x] 5 widgets
- [ ] Background refresh job
- [ ] Cognito integration
- [ ] AI summary widget
- [ ] Public dashboards

---

## License

MIT
