# InsightBoard

A production‑grade, full‑stack, cloud‑ready dashboard aggregating data from multiple external APIs into a customizable interface.

**[Live Demo](#)** | **[Documentation](#documentation)** | **[Contributing](#contributing)** | **[License](#license)**

---

## 🎯 Overview

InsightBoard normalizes and caches data from third‑party APIs (GitHub, Weather, News, Crypto, Status) into a single, modern dashboard. Built with **Rust** (Axum) backend and **Vue 3** frontend, designed for AWS serverless deployment and showcasing fullstack cloud‑native architecture.

> **Perfect for portfolio**: Demonstrates async concurrency, caching strategies, clean architecture, cloud infrastructure, and modern DevEx practices.

---

## ✨ Features

### MVP

-   🔐 **JWT Authentication** - Secure auth with Argon2 password hashing
-   📊 **5 Pre-built Widgets**
    -   GitHub Activity tracking
    -   Real-time Weather data
    -   News aggregation
    -   Cryptocurrency prices
    -   Service status checks
-   🎨 **Drag-and-drop Dashboard Layout** - Powered by GridStack.js
-   🔄 **Auto-refresh with Smart Caching** - Redis-backed cache with TTL
-   ⚡ **Real-time Data Updates** - Reactive Vue 3 component system
-   📱 **Responsive Design** - Mobile-friendly UI

### Advanced (Roadmap)

-   💾 **S3 File Uploads** - Dashboard snapshots and exports
-   🤖 **AI Summary Widget** - LLM-powered data insights
-   🌐 **Public Dashboards** - Share dashboards with link sharing
-   🔄 **Background Refresh Jobs** - EventBridge scheduled refreshes
-   🔌 **Cognito Integration** - Enterprise authentication

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Vue 3 SPA (Frontend)                    │
│              Vite · Pinia · Vue Router · Chart.js            │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTPS/WSS
┌──────────────────────▼──────────────────────────────────────┐
│                   API Gateway (AWS)                          │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│          Lambda Runtime (Rust/Axum Backend)                 │
│  • Auth Middleware (JWT)                                    │
│  • Widget Handlers (GitHub, Weather, News, Crypto, Status) │
│  • Rate Limiting & Caching Layer                            │
│  • Error Handling & Observability                           │
└──────────────────────┬──────────────────────────────────────┘
                       │
       ┌───────────────┼───────────────┐
       │               │               │
   ┌───▼───┐   ┌──────▼──────┐   ┌────▼────────┐
   │Postgres│   │ Redis Cache │   │External APIs│
   │ (Neon) │   │ (Upstash)   │   │  (3rd-party)│
   └────────┘   └─────────────┘   └─────────────┘
```

### Data Flow

1. **Request** → API Gateway → Lambda
2. **Auth** → JWT validation middleware
3. **Cache Check** → Redis (TTL-based)
4. **Cache Miss** → Fetch from external API
5. **Store** → Cache result in Redis
6. **Persist** → Optional save to Postgres
7. **Response** → Return to frontend

---

## 🛠️ Tech Stack

| Layer             | Technology                                          | Purpose                          |
| ----------------- | --------------------------------------------------- | -------------------------------- |
| **Backend**       | Rust + Axum                                         | Async, type-safe REST API        |
| **Frontend**      | Vue 3 + TypeScript + Vite                           | Reactive SPA dashboard           |
| **State Mgmt**    | Pinia                                               | Vue store management             |
| **Database**      | PostgreSQL (Neon/Supabase)                          | User & dashboard persistence     |
| **Cache**         | Redis (Upstash)                                     | Response caching & rate limiting |
| **UI Components** | Mantine UI / Tailwind CSS                           | Modern component library         |
| **Charting**      | Chart.js                                            | Data visualization               |
| **Layout**        | GridStack.js                                        | Drag-and-drop dashboard          |
| **Infra**         | AWS Lambda, API Gateway, S3, CloudFront             | Serverless deployment            |
| **CI/CD**         | GitHub Actions                                      | Automated build & test           |
| **Testing**       | Tokio + Criterion (Rust), Vitest + Playwright (Vue) | Comprehensive coverage           |

---

## 🚀 Quick Start

### Prerequisites

-   **Rust** 1.70+ (`rustup`)
-   **Node.js** 18+ & **pnpm**
-   **Docker** & **Docker Compose** (for local Postgres & Redis)
-   **Git**

### Installation

```bash
# Clone repository
git clone https://github.com/lavinhoque33/insightboard.git
cd insightboard

# 1. Start Docker services (Postgres + Redis)
docker-compose up -d

# Wait for services to be healthy (check with: docker-compose ps)

# 2. Configure environment
cp .env.example .env
# Edit .env with your API keys (optional for basic functionality)

# 3. Run database migrations
cd backend
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run

# 4. Build backend
cargo build

# 5. Frontend setup (in a new terminal)
cd ../frontend
pnpm install
```

### Running Locally

**Terminal 1 — Docker Services**

```bash
# Start Postgres and Redis
docker-compose up

# Or run in detached mode:
docker-compose up -d

# Check service health:
docker-compose ps

# View logs:
docker-compose logs -f
```

**Terminal 2 — Backend (Rust)**

```bash
cd backend
cargo run
# Runs on http://localhost:8080
```

**Terminal 3 — Frontend (Vue)**

```bash
cd frontend
pnpm dev
# Runs on http://localhost:5173
```

### Verify Everything Works

```bash
# Check Docker services are running
docker-compose ps

# Health check backend
curl http://localhost:8080/healthz
# Expected: "ok"

# Test database connection
psql postgres://postgres:postgres@localhost:5432/insightboard -c "SELECT 1"

# Test Redis connection
redis-cli -u redis://127.0.0.1:6379 ping
# Expected: "PONG"

# Frontend
open http://localhost:5173
```

### Stopping Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (clean slate)
docker-compose down -v
```

---

## 📋 Configuration

### Environment Variables

Copy `.env.example` to `.env` and configure:

```bash
cp .env.example .env
```

**Minimal configuration for local development** (using Docker Compose):

```env
# Server
APP_PORT=8080

# Database (Docker Compose default)
DATABASE_URL=postgres://postgres:postgres@localhost:5432/insightboard

# Redis (Docker Compose default)
REDIS_URL=redis://127.0.0.1:6379

# Authentication (change in production!)
JWT_SECRET=dev-secret-change-in-production

# Logging
RUST_LOG=info,insightboard=debug
```

**Optional API keys** (required for specific widgets):

```env
# GitHub Widget - Get at: https://github.com/settings/tokens
GITHUB_API_TOKEN=ghp_yourtoken

# Weather Widget - Get at: https://openweathermap.org/api
OPENWEATHER_API_KEY=your_key_here

# News Widget - Get at: https://newsapi.org/
NEWSAPI_API_KEY=your_key_here
```

**Production configuration:**

```env
# Use production database (Neon/Supabase)
DATABASE_URL=postgres://user:password@host:5432/dbname?sslmode=require

# Use production Redis (Upstash)
REDIS_URL=rediss://default:password@host:port

# Strong JWT secret (generate with: openssl rand -base64 32)
JWT_SECRET=your-random-generated-secret

# AWS credentials
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
```

> **Security Note**: Never commit `.env` to version control. The `.gitignore` file is configured to exclude it. Always use `.env.example` as a template.

---

## 📁 Project Structure

```
insightboard/
├─ .github/                      # GitHub-specific config
│  ├─ workflows/                 # CI/CD pipelines
│  ├─ ISSUE_TEMPLATE/            # Issue templates
│  └─ InsightBoard_Condensed_Project_Spec.md
│
├─ backend/                      # Rust/Axum server
│  ├─ src/
│  │  ├─ main.rs                # App entry & route setup
│  │  ├─ lib.rs                 # Public library interface
│  │  ├─ config.rs              # Configuration loader
│  │  ├─ auth.rs                # JWT middleware & logic
│  │  ├─ db.rs                  # Database connection pool
│  │  ├─ cache.rs               # Redis cache client
│  │  ├─ error.rs               # Custom error types
│  │  ├─ handlers/
│  │  │  ├─ mod.rs
│  │  │  ├─ auth.rs             # Auth endpoints (login, register)
│  │  │  └─ dashboards.rs       # Dashboard CRUD
│  │  ├─ widgets/
│  │  │  ├─ mod.rs
│  │  │  ├─ github.rs           # GitHub API integration
│  │  │  ├─ weather.rs          # Weather API integration
│  │  │  ├─ news.rs             # News API integration
│  │  │  ├─ crypto.rs           # Crypto price integration
│  │  │  └─ status.rs           # Status check widget
│  │  └─ models.rs              # Shared data structures
│  ├─ tests/                     # Integration tests
│  ├─ Cargo.toml
│  └─ sqlx-data.json             # SQLx offline query cache
│
├─ frontend/                     # Vue 3 + Vite SPA
│  ├─ src/
│  │  ├─ main.ts                # Vue app entry
│  │  ├─ App.vue                # Root component
│  │  ├─ pages/
│  │  │  ├─ Dashboard.vue       # Main dashboard
│  │  │  ├─ Login.vue           # Auth page
│  │  │  └─ Profile.vue         # User settings
│  │  ├─ components/
│  │  │  ├─ widgets/
│  │  │  │  ├─ GitHubWidget.vue
│  │  │  │  ├─ WeatherWidget.vue
│  │  │  │  ├─ NewsWidget.vue
│  │  │  │  ├─ CryptoWidget.vue
│  │  │  │  └─ StatusWidget.vue
│  │  │  ├─ DashboardGrid.vue   # Drag-drop layout
│  │  │  └─ Common.vue
│  │  ├─ store/
│  │  │  ├─ auth.ts             # Auth state (Pinia)
│  │  │  ├─ dashboard.ts        # Dashboard state
│  │  │  └─ widgets.ts          # Widget registry
│  │  ├─ services/
│  │  │  ├─ api.ts              # HTTP client (axios)
│  │  │  └─ auth.ts             # Auth service
│  │  ├─ types/
│  │  │  ├─ index.ts            # Global types
│  │  │  └─ models.ts           # API models
│  │  └─ router/
│  │     └─ index.ts            # Vue Router config
│  ├─ public/
│  ├─ package.json
│  ├─ vite.config.ts
│  └─ tsconfig.json
│
├─ infra/                        # Infrastructure as Code
│  ├─ terraform/
│  │  ├─ main.tf                # Lambda, API Gateway, S3
│  │  ├─ rds.tf                 # Postgres setup
│  │  ├─ redis.tf               # Redis cache
│  │  ├─ variables.tf
│  │  ├─ outputs.tf
│  │  └─ terraform.tfvars.example
│  └─ docker-compose.yml        # Local dev services
│
├─ .github/
│  └─ workflows/
│     ├─ backend-test.yml       # Rust test CI
│     ├─ frontend-test.yml      # Vue test CI
│     └─ deploy.yml             # Deploy to AWS
│
├─ docs/                         # Project documentation
│  ├─ API.md                     # API reference
│  ├─ ARCHITECTURE.md            # Deep dive
│  ├─ WIDGETS.md                 # Widget dev guide
│  └─ DEPLOYMENT.md              # Deploy guide
│
├─ .env.example                  # Environment template
├─ .gitignore
├─ docker-compose.yml            # Service orchestration
├─ README.md                      # This file
└─ LICENSE
```

---

## 🔐 Authentication

### Local JWT (Development)

1. **Register** → `POST /api/auth/register`
    ```json
    { "email": "user@example.com", "password": "secure_pwd" }
    ```
2. **Login** → `POST /api/auth/login`
    ```json
    { "email": "user@example.com", "password": "secure_pwd" }
    ```
    Returns: `{ "access_token": "jwt..." }`
3. **Authenticated requests** → Include header:
    ```
    Authorization: Bearer <access_token>
    ```

### Password Security

-   Argon2id hashing (via `argon2` crate)
-   Never store plaintext passwords
-   JWT signed with `HS256`

### Production (AWS Cognito)

-   Swap local JWT for Cognito token validation
-   Middleware extracts `cognito_user_id` from JWT claims
-   No password hashing needed

---

## 📡 API Reference

### Auth Endpoints

| Method | Endpoint             | Description         | Auth |
| ------ | -------------------- | ------------------- | ---- |
| POST   | `/api/auth/register` | Create user account | —    |
| POST   | `/api/auth/login`    | Get JWT token       | —    |
| GET    | `/api/auth/me`       | Get current user    | ✅   |
| POST   | `/api/auth/refresh`  | Refresh token       | ✅   |

### Dashboard Endpoints

| Method | Endpoint              | Description          | Auth |
| ------ | --------------------- | -------------------- | ---- |
| GET    | `/api/dashboards`     | List user dashboards | ✅   |
| POST   | `/api/dashboards`     | Create dashboard     | ✅   |
| GET    | `/api/dashboards/:id` | Get single dashboard | ✅   |
| PUT    | `/api/dashboards/:id` | Update dashboard     | ✅   |
| DELETE | `/api/dashboards/:id` | Delete dashboard     | ✅   |

### Data Endpoints (Widgets)

| Method | Endpoint            | Query Params                | Cache | Description          |
| ------ | ------------------- | --------------------------- | ----- | -------------------- |
| GET    | `/api/data/github`  | `username`                  | 5m    | GitHub user activity |
| GET    | `/api/data/weather` | `city`                      | 30m   | Weather forecast     |
| GET    | `/api/data/news`    | `topic`, `country`          | 1h    | News headlines       |
| GET    | `/api/data/crypto`  | `symbols` (comma-separated) | 5m    | Crypto prices        |
| GET    | `/api/data/status`  | `url`                       | 2m    | Service uptime check |

**Example Requests**:

```bash
curl -H "Authorization: Bearer <token>" \
  'http://localhost:8080/api/data/github?username=torvalds'

curl -H "Authorization: Bearer <token>" \
  'http://localhost:8080/api/data/weather?city=San%20Francisco'

curl -H "Authorization: Bearer <token>" \
  'http://localhost:8080/api/data/crypto?symbols=BTC,ETH'
```

---

## 🎨 Widget System

### Widget Structure

Each widget is independently modular:

1. **Backend Handler** (Rust) — Fetches and caches data
2. **Frontend Component** (Vue) — Displays widget UI
3. **Registry Entry** — Maps widget ID to component

### Adding a New Widget

**Step 1: Backend (Rust)**

```rust
// backend/src/widgets/example.rs
pub async fn example_handler(
    State(st): State<AppState>,
    Query(q): Query<ExampleQuery>,
) -> Result<impl IntoResponse, AppError> {
    let cache_key = format!("example:{}", q.param);

    // Check cache
    if let Some(cached) = st.cache.get(&cache_key).await? {
        return Ok(Json(cached));
    }

    // Fetch data
    let data = fetch_example_data(&q.param).await?;

    // Cache for 1 hour
    st.cache.put(&cache_key, &data, 3600).await?;

    Ok(Json(data))
}
```

**Step 2: Register Route** (in `main.rs`)

```rust
.nest("/api", routes![
    // ... existing routes
    post(example_handler).at("/data/example"),
])
```

**Step 3: Frontend Component** (Vue)

```vue
<!-- frontend/src/components/widgets/ExampleWidget.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '@/services/api';

const props = defineProps<{ param: string }>();
const data = ref(null);
const loading = ref(false);

onMounted(async () => {
	loading.value = true;
	try {
		const { data: result } = await api.get('/data/example', {
			params: { param: props.param },
		});
		data.value = result;
	} finally {
		loading.value = false;
	}
});
</script>

<template>
	<div class="widget">
		<div v-if="loading">Loading...</div>
		<div v-else>{{ data }}</div>
	</div>
</template>
```

**Step 4: Register in Store**

```typescript
// frontend/src/store/widgets.ts
export const widgetRegistry = {
	// ... existing widgets
	example: ExampleWidget,
};
```

---

## 🧪 Testing

### Backend Tests (Rust)

```bash
# Run all tests
cd backend
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test github_widget_fetch
```

Example test:

```rust
#[tokio::test]
async fn test_github_handler_caches() {
    let state = setup_test_state().await;
    let result1 = github_handler(state.clone(), query.clone()).await;
    let result2 = github_handler(state.clone(), query.clone()).await;

    assert_eq!(result1, result2);
    // Verify cache was hit (check metrics or timing)
}
```

### Frontend Tests (Vue)

```bash
# Run unit tests
cd frontend
pnpm test

# Watch mode
pnpm test:watch

# Coverage
pnpm test:coverage
```

### E2E Tests (Playwright)

```bash
# Run E2E tests
pnpm test:e2e

# Debug mode
pnpm test:e2e --debug
```

---

## 🚀 Deployment

### Prerequisites

-   AWS account (free tier eligible)
-   Terraform configured
-   Domain name (optional)

### Quick Deploy (AWS)

```bash
# 1. Configure AWS credentials
aws configure

# 2. Deploy infrastructure
cd infra
terraform init
terraform apply

# 3. Build backend for Lambda
cd ../backend
cargo build --release
# (Creates optimized binary for Lambda)

# 4. Deploy frontend to S3 + CloudFront
cd ../frontend
pnpm build
# (Manual upload or use GitHub Actions)
```

### Free-Tier Services Used

-   **Lambda**: 1M free requests/month
-   **API Gateway**: 1M free calls/month
-   **Neon Postgres**: 5GB free storage
-   **Upstash Redis**: 10,000 commands/day free
-   **S3**: 5GB free storage
-   **CloudFront**: 1TB free data transfer

**Total Monthly Cost**: ~$0-5 (after free tier)

### Continuous Deployment

GitHub Actions workflows automate:

1. Run tests on every push
2. Build Docker images
3. Deploy to Lambda on merge to `main`

See `.github/workflows/` for configuration.

---

## 📚 Documentation

| Document                                | Purpose                     |
| --------------------------------------- | --------------------------- |
| [API.md](docs/API.md)                   | Detailed API reference      |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design & decisions   |
| [WIDGETS.md](docs/WIDGETS.md)           | Guide to building widgets   |
| [DEPLOYMENT.md](docs/DEPLOYMENT.md)     | Production deployment guide |

---

## 🐛 Troubleshooting

### Backend won't start

```bash
# Check Postgres connection
psql $DATABASE_URL -c "SELECT 1"

# Check Redis connection
redis-cli -u $REDIS_URL ping

# View logs
RUST_LOG=debug cargo run
```

### Frontend build fails

```bash
# Clear cache
rm -rf node_modules pnpm-lock.yaml
pnpm install

# Check Node version
node --version  # Should be 18+
```

### CORS errors

Ensure backend has CORS middleware enabled:

```rust
.layer(CorsLayer::permissive())  // Dev only!
```

---

## 🤝 Contributing

We welcome contributions! Please follow these steps:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** changes (`git commit -m 'Add amazing feature'`)
4. **Push** to branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Code Standards

-   **Rust**: `cargo fmt` and `cargo clippy` (no warnings)
-   **Vue**: `pnpm lint` and `pnpm format`
-   **Git**: Conventional commits (feat:, fix:, docs:, etc.)

---

## 📋 Roadmap

-   [x] Core MVP (auth, 5 widgets, layout)
-   [ ] Background refresh jobs (EventBridge)
-   [ ] Cognito authentication
-   [ ] AI summary widget (LLM integration)
-   [ ] Public dashboard sharing
-   [ ] S3 file uploads
-   [ ] Mobile app (React Native)
-   [ ] Real-time WebSocket updates
-   [ ] Plugin marketplace

---

## 📄 License

This project is licensed under the **MIT License** — see [LICENSE](LICENSE) file for details.

---

## 👥 Acknowledgments

-   [Axum](https://github.com/tokio-rs/axum) — Ergonomic and modular web framework
-   [Vue 3](https://vuejs.org/) — Progressive JavaScript framework
-   [Tokio](https://tokio.rs/) — Async Rust runtime
-   [Pinia](https://pinia.vuejs.org/) — Store management
-   The awesome Rust & Vue communities

---

## 📞 Support

-   📧 Email: [ihoque33@yahoo.com]
-   🐛 Issues: [GitHub Issues](https://github.com/yourusername/insightboard/issues)

---

**⭐ If you found this useful, please star the repository!**

---

_Last updated: October 28, 2025_
