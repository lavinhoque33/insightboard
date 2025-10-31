# 🏗️ InsightBoard Infrastructure Setup - Complete Guide

> A comprehensive guide to local development infrastructure for developers starting from scratch

---

## 📚 Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Understanding Docker Compose](#docker-compose)
4. [PostgreSQL Database Setup](#postgresql)
5. [Redis Cache Setup](#redis)
6. [Environment Configuration](#environment)
7. [Project File Structure](#structure)
8. [Step-by-Step Setup](#setup)
9. [Verifying Your Setup](#verification)
10. [Troubleshooting](#troubleshooting)
11. [Daily Development Workflow](#workflow)
12. [Production Considerations](#production)

---

## <a name="overview"></a>🎯 Overview

This guide will help you set up the complete development infrastructure for InsightBoard, including:

-   **PostgreSQL 16** - Relational database for user accounts and dashboards
-   **Redis 7** - In-memory cache for API response caching
-   **Docker Compose** - Orchestrates both services with one command
-   **Environment Configuration** - Secure secrets management

**Why this approach?**

-   ✅ **Consistent Environment**: Everyone on the team uses the same database/cache versions
-   ✅ **Easy Setup**: One command starts everything
-   ✅ **Isolated**: Services run in containers, won't interfere with other projects
-   ✅ **Disposable**: Can reset to clean state anytime
-   ✅ **Production-Like**: Mirrors production environment

---

## <a name="prerequisites"></a>📋 Prerequisites

### Required Software

#### 1. **Docker** & **Docker Compose**

**What is Docker?**
Docker is a platform for running applications in isolated containers. Think of containers as lightweight virtual machines that package everything an application needs to run.

**What is Docker Compose?**
Docker Compose is a tool for defining and running multi-container Docker applications. You describe your services in a YAML file, and it handles starting everything.

**Installation:**

**Linux:**

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add your user to docker group (avoid needing sudo)
sudo usermod -aG docker $USER

# Log out and back in for group changes to take effect

# Install Docker Compose
sudo apt-get install docker-compose-plugin
```

**macOS:**

-   Download and install [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop/)
-   Docker Compose is included

**Windows:**

-   Download and install [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop/)
-   Docker Compose is included

**Verify Installation:**

```bash
docker --version
# Expected: Docker version 24.0+ or higher

docker compose version
# Expected: Docker Compose version 2.0+ or higher
```

#### 2. **Git** (for cloning the repository)

```bash
# Check if installed
git --version

# If not installed:
# Linux: sudo apt-get install git
# macOS: brew install git
# Windows: Download from https://git-scm.com/
```

#### 3. **Rust Toolchain** (for backend development)

**What is Rust?**
Rust is the programming language used for the InsightBoard backend. The Rust toolchain includes the compiler (`rustc`), package manager (`cargo`), and standard library.

**Installation via rustup (recommended):**

```bash
# Install rustup (Rust installer and version manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts - use default options for standard installation

# Reload your shell configuration
source $HOME/.cargo/env

# Verify installation
rustc --version
# Expected: rustc 1.70+ or higher

cargo --version
# Expected: cargo 1.70+ or higher
```

**What gets installed:**

-   `rustc` - The Rust compiler
-   `cargo` - Rust's package manager and build tool
-   `rustup` - Tool to manage Rust versions
-   Standard library and documentation

**Platform-specific notes:**

**Linux:**

-   May need build essentials: `sudo apt-get install build-essential`
-   OpenSSL development libraries (required for backend): `sudo apt-get install libssl-dev pkg-config`

**macOS:**

-   Requires Xcode Command Line Tools: `xcode-select --install`

**Windows:**

-   Requires Visual Studio C++ Build Tools
-   Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

#### 4. **sqlx-cli** (for database migrations)

**What is sqlx-cli?**
The SQLx CLI is a command-line tool for running database migrations. It's required to set up the initial database schema.

**Installation:**

```bash
# Install sqlx-cli with PostgreSQL support only
cargo install sqlx-cli --no-default-features --features postgres

# This may take 1-2 minutes to compile

# Verify installation
sqlx --version
# Expected: sqlx-cli 0.7+ or higher
```

**When do you need this?**

-   First-time setup (to run database migrations)
-   Creating new migrations
-   You only need to install this once per development machine

#### 5. **Text Editor or IDE**

Recommended options:

-   **VS Code** with Rust and Docker extensions
    -   Install `rust-analyzer` extension for code completion
    -   Install `Docker` extension for container management
-   **IntelliJ IDEA** with Rust plugin
-   **Vim/Neovim** with rust-analyzer

---

## <a name="docker-compose"></a>🐳 Understanding Docker Compose

### What Does Our docker-compose.yml Do?

Let's break down the configuration file line by line:

```yaml
version: '3.8'
```

-   Specifies the Docker Compose file format version
-   Version 3.8 is modern and well-supported

```yaml
services:
    postgres:
        image: postgres:16-alpine
```

-   **services**: Defines what containers to run
-   **postgres**: Name of our first service (you can reference this name)
-   **image**: Which Docker image to use
    -   `postgres:16` = PostgreSQL version 16
    -   `-alpine` = Lightweight Linux distribution (smaller image size)

```yaml
container_name: insightboard-postgres
```

-   Gives the container a friendly name
-   **Why?** Easier to identify in `docker ps` output

```yaml
restart: unless-stopped
```

-   **Restart policy**: Automatically restart if container crashes
-   **unless-stopped**: Restart on crashes, but not if you manually stopped it

```yaml
environment:
    POSTGRES_DB: insightboard
    POSTGRES_USER: postgres
    POSTGRES_PASSWORD: postgres
```

-   **Environment variables** passed to the container
-   **POSTGRES_DB**: Name of the database to create automatically
-   **POSTGRES_USER**: Admin username (default: postgres)
-   **POSTGRES_PASSWORD**: Admin password
    -   ⚠️ **Note**: This is for development only! Production uses secure passwords

```yaml
ports:
    - '5432:5432'
```

-   **Port mapping**: `host:container`
-   Maps port 5432 on your computer to port 5432 in the container
-   **Why?** So your backend can connect to the database at `localhost:5432`

```yaml
volumes:
    - postgres_data:/var/lib/postgresql/data
```

-   **Volumes**: Persistent storage for data
-   **Why?** Without this, all data would be lost when you stop the container!
-   `postgres_data` is a named volume (defined at the bottom)
-   `/var/lib/postgresql/data` is where PostgreSQL stores data inside the container

```yaml
healthcheck:
    test: ['CMD-SHELL', 'pg_isready -U postgres']
    interval: 10s
    timeout: 5s
    retries: 5
```

-   **Health check**: Verifies the service is ready
-   **test**: Command to run - `pg_isready` checks if PostgreSQL is accepting connections
-   **interval**: Check every 10 seconds
-   **timeout**: Wait up to 5 seconds for response
-   **retries**: Try 5 times before marking as unhealthy

### Redis Service (Similar Structure)

```yaml
redis:
    image: redis:7-alpine
    container_name: insightboard-redis
    restart: unless-stopped
    ports:
        - '6379:6379'
    volumes:
        - redis_data:/data
    healthcheck:
        test: ['CMD', 'redis-cli', 'ping']
        interval: 10s
        timeout: 5s
        retries: 5
    command: redis-server --appendonly yes
```

**Key differences:**

-   **Port 6379**: Standard Redis port
-   **Command**: `--appendonly yes` enables persistence (saves data to disk)
-   **Health check**: `redis-cli ping` should return "PONG"

### Volumes Definition

```yaml
volumes:
    postgres_data:
        driver: local
    redis_data:
        driver: local
```

-   **Named volumes**: Docker manages the storage location
-   **driver: local**: Stores on your local machine
-   **Persistence**: Data survives container restarts and rebuilds

### Networks

```yaml
networks:
    default:
        name: insightboard-network
```

-   Creates a custom network for the services
-   **Why?** Services can communicate using their service names (postgres, redis)
-   All services in the same compose file share this network

---

## <a name="postgresql"></a>🐘 PostgreSQL Database Setup

### What is PostgreSQL?

PostgreSQL (often called "Postgres") is a powerful, open-source relational database management system (RDBMS).

**Why PostgreSQL?**

-   ✅ **ACID Compliant**: Guarantees reliable transactions
-   ✅ **JSONB Support**: Can store and query JSON data efficiently
-   ✅ **Advanced Features**: Full-text search, foreign keys, triggers
-   ✅ **Scalable**: Handles millions of rows efficiently
-   ✅ **Free & Open Source**: No licensing costs

**What version are we using?**

-   **PostgreSQL 16** (latest stable as of 2024)
-   **Alpine variant**: Smaller image size (~80MB vs ~300MB)

### Database Configuration in Docker

**Default Settings:**

-   **Database Name**: `insightboard`
-   **Username**: `postgres`
-   **Password**: `postgres`
-   **Port**: `5432`
-   **Connection String**: `postgres://postgres:postgres@localhost:5432/insightboard`

**Data Storage:**

-   Data is stored in a Docker volume named `postgres_data`
-   Location: Managed by Docker (usually `/var/lib/docker/volumes/`)
-   **Important**: Data persists even if you remove the container

### How to Connect

**From your Rust backend:**

```rust
let database_url = "postgres://postgres:postgres@localhost:5432/insightboard";
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(database_url)
    .await?;
```

**Using psql (PostgreSQL CLI):**

```bash
# Install psql first (if not already installed)
# Linux: sudo apt-get install postgresql-client
# macOS: brew install postgresql
# Windows: Included in PostgreSQL installation

# Connect to database
psql postgres://postgres:postgres@localhost:5432/insightboard

# You should see:
# insightboard=#
```

**Using a GUI tool:**

-   **pgAdmin**: https://www.pgadmin.org/
-   **DBeaver**: https://dbeaver.io/
-   **TablePlus**: https://tableplus.com/

Connection details:

-   Host: `localhost`
-   Port: `5432`
-   Database: `insightboard`
-   User: `postgres`
-   Password: `postgres`

### Database Migrations

**What are migrations?**
Migrations are version-controlled SQL files that evolve your database schema over time.

**Our migrations:**

**001_create_users.sql:**

```sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
```

**What this does:**

-   Creates `users` table with UUID primary keys
-   `email` is unique (can't have duplicate accounts)
-   `password_hash` stores hashed passwords (never plaintext!)
-   `created_at` tracks when user registered
-   Index on `email` makes login lookups fast

**002_create_dashboards.sql:**

```sql
CREATE TABLE IF NOT EXISTS dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    layout_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    settings_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dashboards_user_id ON dashboards(user_id);
CREATE INDEX IF NOT EXISTS idx_dashboards_updated_at ON dashboards(updated_at DESC);
```

**What this does:**

-   Creates `dashboards` table
-   `user_id` is a foreign key to `users` table
-   **ON DELETE CASCADE**: When user is deleted, their dashboards are too
-   `layout_json` stores widget positions (JSONB type for efficient queries)
-   `settings_json` stores user preferences
-   Indexes for fast queries on `user_id` and `updated_at`

**Running migrations:**

```bash
# Install sqlx-cli (Rust migration tool)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
cd backend
sqlx migrate run

# Create new migration
sqlx migrate add <migration_name>
```

---

## <a name="redis"></a>🔴 Redis Cache Setup

### What is Redis?

Redis is an in-memory data structure store used as a database, cache, and message broker.

**Why Redis?**

-   ✅ **Extremely Fast**: All data in RAM, microsecond latency
-   ✅ **Simple**: Key-value storage (like a giant HashMap)
-   ✅ **TTL Support**: Automatically expire old data
-   ✅ **Atomic Operations**: Thread-safe by default
-   ✅ **Persistence Options**: Can save to disk for durability

**What version are we using?**

-   **Redis 7** (latest stable)
-   **Alpine variant**: Small footprint
-   **Persistence enabled**: `--appendonly yes` (AOF mode)

### How We Use Redis for Caching

**Purpose: Reduce external API calls**

Example flow:

1. Frontend requests GitHub data for user "torvalds"
2. Backend checks Redis: `GET github:torvalds`
3. **Cache hit?** Return cached data instantly (no GitHub API call)
4. **Cache miss?** Fetch from GitHub API, store in Redis with 5-minute TTL
5. Next request within 5 minutes uses cached data

**Benefits:**

-   ⚡ **Faster responses**: Redis lookup is ~0.1ms vs API call ~200ms
-   💰 **Lower costs**: External APIs often charge per request
-   🛡️ **Rate limit protection**: Won't hit API rate limits
-   📉 **Reduced load**: Less strain on external services

### Cache Key Patterns

We use structured keys for organization:

```
github:<username>           # GitHub events cache
weather:<city>              # Weather data cache
news:<topic>                # News articles cache
crypto:<symbols>            # Crypto prices cache
status:<urls>               # URL health checks cache
```

**TTL (Time-To-Live) Strategy:**

-   GitHub: 5 minutes (events don't change rapidly)
-   Weather: 10 minutes (weather changes slowly)
-   News: 15 minutes (news updates periodically)
-   Crypto: 5 minutes (prices volatile, but not real-time)
-   Status: 2 minutes (want recent health checks)

### Redis Configuration

**Default Settings:**

-   **Port**: `6379` (standard Redis port)
-   **Persistence**: AOF (Append-Only File) mode
-   **Data Storage**: Docker volume `redis_data`

**Connecting from Rust:**

```rust
let redis_url = "redis://127.0.0.1:6379";
let client = redis::Client::open(redis_url)?;
let mut conn = client.get_connection()?;

// Set value with TTL
conn.set_ex("key", "value", 300)?;  // 300 seconds = 5 minutes

// Get value
let value: Option<String> = conn.get("key")?;
```

**Using redis-cli:**

```bash
# Connect to Redis
redis-cli

# Should see:
# 127.0.0.1:6379>

# Test commands
PING                        # Should return PONG
SET mykey "Hello"          # Set a key
GET mykey                  # Get a key
KEYS *                     # List all keys
TTL mykey                  # Check time-to-live
DEL mykey                  # Delete a key
FLUSHDB                    # Clear all keys (use carefully!)
```

### Monitoring Cache Performance

**Check cache stats:**

```bash
redis-cli INFO stats
```

**Useful metrics:**

-   `keyspace_hits`: Number of successful key lookups
-   `keyspace_misses`: Number of failed key lookups
-   **Hit rate**: `hits / (hits + misses) * 100%`
    -   Goal: >80% hit rate means caching is effective

---

## <a name="environment"></a>🔐 Environment Configuration

### Understanding .env Files

**What are .env files?**
Environment files store configuration as key-value pairs. They keep secrets out of your code.

**Our setup:**

-   `.env.example` - Template file (committed to git)
-   `.env` - Your local file with real secrets (NEVER commit this!)
-   `.gitignore` - Configured to exclude `.env` but allow `.env.example`

### Setting Up Your .env File

**Step 1: Copy the example file**

```bash
cp .env.example .env
```

**Step 2: Edit .env with your values**

### Environment Variables Explained

#### **Server Configuration**

```bash
APP_PORT=8080
```

-   **What**: Port the Rust backend listens on
-   **Default**: 8080
-   **When to change**: If 8080 is already in use

#### **Database Configuration**

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/insightboard
```

-   **Format**: `postgres://username:password@host:port/database`
-   **Development**: Uses Docker Compose defaults
-   **Production**: Replace with Neon/Supabase connection string

**Production example (Neon):**

```bash
DATABASE_URL=postgres://user:password@ep-cool-name-123456.us-east-2.aws.neon.tech/insightboard?sslmode=require
```

**Important parts:**

-   `sslmode=require` - Forces encrypted connection (required for production)
-   Host includes region and unique ID
-   Password is auto-generated by Neon

#### **Redis Configuration**

```bash
REDIS_URL=redis://127.0.0.1:6379
```

-   **Format**: `redis://host:port`
-   **Development**: Localhost
-   **Production**: Use Upstash or AWS ElastiCache

**Production example (Upstash):**

```bash
REDIS_URL=rediss://default:password@region.upstash.io:6379
```

-   `rediss://` - TLS-encrypted connection (note the extra 's')
-   Includes password and region

#### **Authentication**

```bash
JWT_SECRET=dev-secret-change-in-production
```

-   **What**: Secret key used to sign JWT tokens
-   **Security**: CRITICAL - must be random and secret
-   **Development**: Use placeholder
-   **Production**: Generate a strong random secret

**Generate a secure secret:**

```bash
# Using OpenSSL
openssl rand -base64 32

# Using Python
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# Using Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
```

**Example output:**

```
3f8b9c1d2e4a6f7c8e9b0a1d3e5f7g9h1j2k4l6m8n0p2q4r6s8t0u2v4w6x8y0
```

#### **External API Keys (Optional)**

**GitHub Token:**

```bash
GITHUB_API_TOKEN=ghp_1234567890abcdefghijklmnop
```

-   **Why?** Increases rate limit from 60 to 5,000 requests/hour
-   **How to get**: https://github.com/settings/tokens
-   **Scopes needed**: `public_repo` (read-only)
-   **Optional**: Widget works without it, just with lower limits

**OpenWeather API Key:**

```bash
OPENWEATHER_API_KEY=abcdef1234567890abcdef1234567890
```

-   **Why?** Required for weather widget
-   **How to get**: https://openweathermap.org/api
-   **Free tier**: 1,000 calls/day
-   **Limits**: 60 calls/minute

**NewsAPI Key:**

```bash
NEWSAPI_API_KEY=1234567890abcdef1234567890abcdef
```

-   **Why?** Required for news widget
-   **How to get**: https://newsapi.org/
-   **Free tier**: 100 requests/day (developer plan)
-   **Limits**: Recent news only (48 hours)

**Note on API keys:**

-   Not required for basic functionality
-   Backend will skip widgets if keys are missing
-   Can add later when you need those features

#### **Logging Configuration**

```bash
RUST_LOG=info,insightboard=debug
```

-   **Format**: `level,module=level`
-   **Levels**: trace, debug, info, warn, error
-   `info`: Show important events
-   `insightboard=debug`: Show debug logs for our app only

**Development logging:**

```bash
RUST_LOG=debug  # Show all debug logs
```

**Production logging:**

```bash
RUST_LOG=info  # Only show important events
```

### Security Best Practices

**DO:**

-   ✅ Use `.env` for local development
-   ✅ Use environment variables in production (AWS Systems Manager, etc.)
-   ✅ Generate strong random secrets
-   ✅ Rotate secrets periodically
-   ✅ Use different secrets for dev/staging/production

**DON'T:**

-   ❌ Never commit `.env` to git
-   ❌ Never hardcode secrets in source code
-   ❌ Never share secrets in chat/email
-   ❌ Never use the same secrets across environments
-   ❌ Never use default/example secrets in production

---

## <a name="structure"></a>📁 Project File Structure

Understanding where everything lives:

```
InsightBoard/
├── .env.example                    # Template environment file
├── .env                            # Your local secrets (git-ignored)
├── .gitignore                      # Specifies files git should ignore
├── docker-compose.yml              # Defines Docker services
├── README.md                       # Project overview
│
├── backend/                        # Rust backend
│   ├── Cargo.toml                 # Rust dependencies
│   ├── src/                       # Source code
│   │   ├── main.rs               # Entry point
│   │   ├── config.rs             # Reads .env variables
│   │   └── ...
│   └── migrations/                # SQL migration files
│       ├── 001_create_users.sql
│       └── 002_create_dashboards.sql
│
├── frontend/                       # Vue 3 frontend (to be created)
│   ├── package.json
│   └── src/
│
└── Documentation/                  # Project documentation
    ├── BACKEND_ARCHITECTURE.md
    └── INFRASTRUCTURE_SETUP.md     # This file
```

### Important Files

**docker-compose.yml**

-   Defines PostgreSQL and Redis services
-   Configuration for ports, volumes, health checks
-   Single source of truth for local infrastructure

**.env.example**

-   Template with all required variables
-   Committed to git
-   Documentation for what variables are needed

**.env**

-   Your local copy with real values
-   Never committed to git
-   Created by copying .env.example

**.gitignore**

-   Tells git which files to ignore
-   Includes `.env`, `target/`, `node_modules/`, etc.
-   Prevents accidentally committing secrets

---

## <a name="setup"></a>🚀 Step-by-Step Setup

Follow these steps to get your development environment running:

> **📝 Note on Prerequisites**: Before starting, ensure you have installed all required software from the [Prerequisites](#prerequisites) section above. This includes Docker, Rust, sqlx-cli, and Git. These are **one-time installations** per development machine.

### Step 1: Clone the Repository

```bash
git clone https://github.com/lavinhoque33/insightboard.git
cd insightboard
```

### Step 2: Verify Prerequisites

**Check that all required tools are installed:**

```bash
# Verify Docker
docker --version && docker compose version

# Verify Rust
rustc --version && cargo --version

# Verify sqlx-cli
sqlx --version

# Verify Git
git --version
```

**If any of these commands fail**, go back to the [Prerequisites](#prerequisites) section and install the missing tools.

### Step 3: Verify Docker is Running

```bash
# Check Docker daemon is running
docker ps

# Should show a list of running containers (might be empty)
# If you see an error, start Docker Desktop or run: sudo systemctl start docker
```

### Step 4: Create Your Environment File

```bash
# Copy example to .env
cp .env.example .env

# Edit with your preferred editor
nano .env    # or vim, code, etc.
```

**Minimal configuration for local development:**

```bash
APP_PORT=8080
DATABASE_URL=postgres://postgres:postgres@localhost:5432/insightboard
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=dev-secret-change-in-production
RUST_LOG=info,insightboard=debug
```

**Optional: Add API keys if you want to test specific widgets**

### Step 5: Start Docker Services

> **🔄 Daily Step**: This step and the following steps are what you'll do each time you start development. Steps 1-4 are one-time setup.

```bash
# Start PostgreSQL and Redis
docker compose up -d

# -d flag runs in detached mode (background)
```

**What's happening:**

1. Docker downloads postgres:16-alpine image (~80MB)
2. Docker downloads redis:7-alpine image (~30MB)
3. Creates two containers
4. Creates two named volumes for data persistence
5. Creates network for containers to communicate
6. Starts both services
7. Runs health checks

**Expected output:**

```
[+] Running 5/5
 ✔ Network insightboard-network         Created
 ✔ Volume "insightboard_postgres_data"  Created
 ✔ Volume "insightboard_redis_data"     Created
 ✔ Container insightboard-postgres      Started
 ✔ Container insightboard-redis         Started
```

### Step 6: Verify Services Are Running

```bash
# Check container status
docker compose ps

# Should show:
# NAME                    STATUS                  PORTS
# insightboard-postgres   Up (healthy)           0.0.0.0:5432->5432/tcp
# insightboard-redis      Up (healthy)           0.0.0.0:6379->6379/tcp
```

**Healthy status means:**

-   Container is running
-   Health checks are passing
-   Services are ready to accept connections

### Step 7: Test Database Connection

> **⚠️ One-Time Step**: This is optional verification. Skip to Step 9 if you want to proceed quickly.

```bash
# Connect to PostgreSQL
psql postgres://postgres:postgres@localhost:5432/insightboard

# Should see:
# psql (16.x)
# Type "help" for help.
# insightboard=#

# Test a query
SELECT version();

# Exit
\q
```

### Step 8: Test Redis Connection

> **⚠️ One-Time Step**: This is optional verification. Skip to Step 9 if you want to proceed quickly.

```bash
# Connect to Redis
redis-cli

# Test ping
PING

# Should return: PONG

# Exit
exit
```

### Step 9: Run Database Migrations

> **📝 One-Time Step**: Migrations only need to be run once (or when new migrations are added).

```bash
# Install sqlx-cli (first time only)
cargo install sqlx-cli --no-default-features --features postgres

# Navigate to backend directory
cd backend

# Run migrations
sqlx migrate run

# Expected output:
# Applied 001_create_users.sql
# Applied 002_create_dashboards.sql
```

**Verify migrations worked:**

```bash
psql postgres://postgres:postgres@localhost:5432/insightboard

# List tables
\dt

# Should show:
#          List of relations
#  Schema |    Name     | Type  | Owner
# --------+-------------+-------+----------
#  public | users       | table | postgres
#  public | dashboards  | table | postgres
```

### Step 10: Build and Run Backend

> **🔄 Daily Step**: This is what you'll run each time you want to start the backend server.

```bash
# Still in backend directory
cargo build

# This will:
# - Download and compile all dependencies
# - Compile the backend
# - First time takes ~5-10 minutes
# - Subsequent builds much faster

# Run the backend
cargo run

# Expected output:
# Compiling insightboard v0.1.0
# Finished dev [unoptimized + debuginfo] target(s) in 2.34s
# Running `target/debug/insightboard`
# 2024-10-31T15:30:00.123Z INFO insightboard: Server starting on 0.0.0.0:8080
# 2024-10-31T15:30:00.124Z INFO insightboard: Connected to database
# 2024-10-31T15:30:00.125Z INFO insightboard: Connected to Redis
# 2024-10-31T15:30:00.126Z INFO insightboard: Server ready to accept connections
```

### Step 11: Test the Backend

> **✅ Verification Step**: Quick check to ensure everything is working.

**Open a new terminal and test the health endpoint:**

```bash
curl http://localhost:8080/healthz

# Should return: ok
```

**Test registration:**

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'

# Should return:
# {"token":"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...","user":{"id":"...","email":"test@example.com"}}
```

**Save the token for testing protected endpoints!**

---

## 📋 Setup Summary: One-Time vs Daily Steps

### One-Time Setup (Do Once Per Machine)

These steps only need to be done once when setting up a new development machine:

1. **Install Prerequisites** (from [Prerequisites](#prerequisites) section)

    - Docker & Docker Compose
    - Git
    - Rust toolchain (rustup, cargo, rustc)
    - sqlx-cli
    - Text editor/IDE

2. **Clone Repository** (Step 1)

    ```bash
    git clone https://github.com/lavinhoque33/insightboard.git
    ```

3. **Create Environment File** (Step 4)

    ```bash
    cp .env.example .env
    # Edit .env with your configuration
    ```

4. **Run Database Migrations** (Step 9)
    ```bash
    cd backend
    sqlx migrate run
    ```

### Daily Development Workflow

These steps are what you'll do each time you start working on the project:

1. **Start Docker Services** (Step 5)

    ```bash
    docker compose up -d
    ```

2. **Start Backend Server** (Step 10)

    ```bash
    cd backend
    cargo run
    ```

3. **Stop Services When Done**
    ```bash
    # Stop backend: Ctrl+C in terminal
    # Stop Docker services:
    docker compose down
    ```

### Optional Verification Steps

These are useful for debugging but not required for daily development:

-   Test database connection (Step 7)
-   Test Redis connection (Step 8)
-   Test backend endpoints (Step 11)

---

## <a name="verification"></a>✅ Verifying Your Setup

### Checklist

Run through this checklist to ensure everything is working:

-   [ ] Docker is installed and running
-   [ ] `docker compose ps` shows both containers as "healthy"
-   [ ] Can connect to PostgreSQL with psql
-   [ ] Can connect to Redis with redis-cli
-   [ ] Database migrations ran successfully
-   [ ] Backend compiles without errors
-   [ ] Backend starts and logs "Server ready"
-   [ ] `/healthz` endpoint returns "ok"
-   [ ] Can register a new user
-   [ ] Can login with registered user

### Quick Verification Script

Save this as `verify-setup.sh`:

```bash
#!/bin/bash

echo "🔍 Verifying InsightBoard setup..."
echo ""

# Check Docker
echo "✓ Checking Docker..."
docker --version || { echo "❌ Docker not found"; exit 1; }

# Check services
echo "✓ Checking Docker services..."
docker compose ps | grep -q "healthy" || { echo "❌ Services not healthy"; exit 1; }

# Check PostgreSQL
echo "✓ Checking PostgreSQL..."
psql postgres://postgres:postgres@localhost:5432/insightboard -c "SELECT 1" &> /dev/null || { echo "❌ PostgreSQL connection failed"; exit 1; }

# Check Redis
echo "✓ Checking Redis..."
redis-cli ping | grep -q "PONG" || { echo "❌ Redis connection failed"; exit 1; }

# Check backend
echo "✓ Checking backend..."
curl -s http://localhost:8080/healthz | grep -q "ok" || { echo "❌ Backend not responding"; exit 1; }

echo ""
echo "✅ All checks passed! Your setup is ready."
```

Make it executable and run:

```bash
chmod +x verify-setup.sh
./verify-setup.sh
```

---

## <a name="troubleshooting"></a>🔧 Troubleshooting

### Docker Issues

**Problem: "Cannot connect to Docker daemon"**

```
Error: Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

**Solution:**

```bash
# Check if Docker is running
sudo systemctl status docker

# Start Docker
sudo systemctl start docker

# Or on macOS/Windows, start Docker Desktop
```

---

**Problem: "Port already in use"**

```
Error: Bind for 0.0.0.0:5432 failed: port is already allocated
```

**Solution 1: Stop conflicting service**

```bash
# Check what's using port 5432
sudo lsof -i :5432

# Stop existing PostgreSQL
sudo systemctl stop postgresql
```

**Solution 2: Change port in docker-compose.yml**

```yaml
ports:
    - '5433:5432' # Use 5433 on host, 5432 in container
```

Then update DATABASE_URL:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5433/insightboard
```

---

**Problem: "Volume mount denied"**

```
Error: error while mounting volume: path ... is mounted on / but it is not a shared mount
```

**Solution (Linux):**

```bash
# Make root mount shared
sudo mount --make-rshared /
```

---

### PostgreSQL Issues

**Problem: "Connection refused"**

```
psql: error: connection to server at "localhost", port 5432 failed: Connection refused
```

**Solutions:**

1. Check if container is running:

```bash
docker compose ps
# Make sure postgres shows "Up (healthy)"
```

2. Check health status:

```bash
docker compose logs postgres
# Look for "database system is ready to accept connections"
```

3. Wait for startup (takes ~5-10 seconds):

```bash
# Watch logs until ready
docker compose logs -f postgres
```

---

**Problem: "Password authentication failed"**

```
psql: error: connection to server at "localhost", port 5432 failed:
FATAL: password authentication failed for user "postgres"
```

**Solutions:**

1. Verify password in .env matches docker-compose.yml
2. Reset database:

```bash
docker compose down -v  # Removes volumes
docker compose up -d     # Recreates with fresh database
```

---

**Problem: "Database does not exist"**

```
psql: error: connection to server at "localhost", port 5432 failed:
FATAL: database "insightboard" does not exist
```

**Solution:**

1. Connect to default database and create it:

```bash
psql postgres://postgres:postgres@localhost:5432/postgres
CREATE DATABASE insightboard;
\q
```

2. Or recreate containers:

```bash
docker compose down -v
docker compose up -d
```

---

### Redis Issues

**Problem: "Connection refused"**

```
Error: Connection refused (os error 111)
```

**Solutions:**

1. Check if Redis is running:

```bash
docker compose ps
# redis should show "Up (healthy)"
```

2. Test with redis-cli:

```bash
redis-cli ping
# Should return PONG
```

3. Check logs:

```bash
docker compose logs redis
```

---

**Problem: "DENIED Redis is running in protected mode"**

**Solution:**
Protected mode is only for external connections. From localhost it should work. If not:

```bash
# Update docker-compose.yml
command: redis-server --appendonly yes --protected-mode no
```

---

### Backend Issues

**Problem: "Failed to connect to database"**

**Check:**

1. Is PostgreSQL running and healthy?
2. Is DATABASE_URL correct in .env?
3. Did migrations run?

**Debug:**

```bash
# Check environment variable is loaded
cd backend
cargo run 2>&1 | grep -i database
```

---

**Problem: "Failed to connect to Redis"**

**Check:**

1. Is Redis running?
2. Is REDIS_URL correct in .env?

**Test Redis connection:**

```bash
redis-cli -u $REDIS_URL ping
```

---

**Problem: "Compilation errors"**

**Solution 1: Update dependencies**

```bash
cargo update
cargo build
```

**Solution 2: Clean and rebuild**

```bash
cargo clean
cargo build
```

**Solution 3: Check Rust version**

```bash
rustc --version
# Should be 1.70+

# Update if needed
rustup update stable
```

---

### Migration Issues

**Problem: "Migration has already been applied"**

**Solution:**

```bash
# This is normal - migrations only run once
# To re-run, reset database:
docker compose down -v
docker compose up -d
cd backend
sqlx migrate run
```

---

**Problem: "No such file or directory"**

**Check:**

```bash
# Make sure you're in backend directory
cd backend
ls migrations/
# Should show .sql files
```

---

### Performance Issues

**Problem: Docker uses too much CPU/memory**

**Solution: Limit resources in docker-compose.yml**

```yaml
services:
    postgres:
        deploy:
            resources:
                limits:
                    cpus: '1.0'
                    memory: 1G
```

---

**Problem: Slow queries**

**Check:**

1. Are indexes created? (migrations should have created them)
2. Is connection pool sized correctly?

**Monitor:**

```bash
# PostgreSQL slow queries
psql postgres://postgres:postgres@localhost:5432/insightboard
SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;
```

---

## <a name="workflow"></a>📅 Daily Development Workflow

### Starting Your Day

```bash
# 1. Start Docker services
docker compose up -d

# 2. Check services are healthy
docker compose ps

# 3. Start backend (in one terminal)
cd backend
cargo run

# 4. Start frontend (in another terminal - when implemented)
cd frontend
pnpm dev

# You're ready to develop!
```

### During Development

**View logs:**

```bash
# Backend logs (if running with cargo run, logs show in terminal)

# Docker logs
docker compose logs -f          # Follow all logs
docker compose logs -f postgres # Just PostgreSQL
docker compose logs -f redis    # Just Redis
```

**Database queries during development:**

```bash
# Connect to database
psql postgres://postgres:postgres@localhost:5432/insightboard

# Useful commands:
\dt                  # List tables
\d users            # Describe users table
\d dashboards       # Describe dashboards table
SELECT * FROM users; # Query users
```

**Redis debugging:**

```bash
# Connect to Redis
redis-cli

# Useful commands:
KEYS *              # List all keys (use sparingly in production!)
GET github:torvalds # Get specific key
TTL github:torvalds # Check time-to-live
FLUSHDB             # Clear all keys (development only!)
```

### Ending Your Day

```bash
# Stop services (keeps data)
docker compose stop

# Or stop and remove containers (keeps data in volumes)
docker compose down
```

### Resetting Everything

```bash
# Nuclear option - removes everything including data
docker compose down -v

# Then start fresh
docker compose up -d
cd backend
sqlx migrate run
```

---

## <a name="production"></a>🚀 Production Considerations

### What Changes for Production?

#### Database (PostgreSQL)

**Development:**

-   Docker container on localhost
-   Default credentials
-   No SSL
-   No backups

**Production (Neon/Supabase):**

-   Managed PostgreSQL service
-   Strong random password
-   SSL required
-   Automatic backups
-   Connection pooling
-   Monitoring and alerts

**Migration to production:**

```bash
# Get your Neon connection string from dashboard
DATABASE_URL="postgres://user:pass@host.neon.tech/dbname?sslmode=require"

# Run migrations on production
cd backend
sqlx migrate run

# Test connection
psql "$DATABASE_URL" -c "SELECT version()"
```

#### Cache (Redis)

**Development:**

-   Docker container on localhost
-   No persistence
-   No authentication

**Production (Upstash):**

-   Managed Redis service
-   TLS encryption
-   Authentication required
-   Persistence enabled
-   Global replication
-   99.99% uptime SLA

**Setup:**

```bash
# Get connection string from Upstash dashboard
REDIS_URL="rediss://default:password@region.upstash.io:6379"

# Test connection
redis-cli -u "$REDIS_URL" ping
```

#### Secrets Management

**Development:**

-   `.env` file with plaintext secrets
-   Local only

**Production:**

-   **AWS Systems Manager Parameter Store**
-   **AWS Secrets Manager**
-   **HashiCorp Vault**
-   Environment variables in Lambda/ECS

**Example (AWS Lambda):**

```bash
# Store secret
aws ssm put-parameter \
  --name "/insightboard/prod/jwt-secret" \
  --value "your-secret" \
  --type SecureString

# Lambda reads from environment variable
# Set in Lambda configuration:
JWT_SECRET=/insightboard/prod/jwt-secret
```

#### Security Hardening

**Production checklist:**

-   [ ] Strong random JWT_SECRET (32+ bytes)
-   [ ] SSL/TLS for all connections
-   [ ] Firewall rules (allow only necessary ports)
-   [ ] Rotate secrets regularly
-   [ ] Enable audit logging
-   [ ] Set up monitoring and alerts
-   [ ] Rate limiting on API endpoints
-   [ ] CORS configured for your domain only
-   [ ] Input validation and sanitization
-   [ ] Regular security updates

#### Monitoring

**Production monitoring:**

-   **Logs**: CloudWatch, Datadog, or Grafana Loki
-   **Metrics**: Prometheus + Grafana
-   **Traces**: OpenTelemetry + Jaeger
-   **Errors**: Sentry or Rollbar
-   **Uptime**: UptimeRobot or Pingdom

**Key metrics to track:**

-   Request latency (p50, p95, p99)
-   Error rate
-   Database connection pool usage
-   Redis hit/miss ratio
-   API rate limit usage
-   Memory and CPU usage

---

## 📊 Quick Reference

### Essential Commands

```bash
# Docker Compose
docker compose up -d           # Start services in background
docker compose down            # Stop and remove containers
docker compose down -v         # Stop and remove volumes (data loss!)
docker compose ps              # List running services
docker compose logs -f         # Follow logs
docker compose restart         # Restart services

# PostgreSQL
psql $DATABASE_URL                                  # Connect
\dt                                                 # List tables
\d tablename                                        # Describe table
SELECT * FROM users;                               # Query
\q                                                 # Quit

# Redis
redis-cli                                          # Connect
PING                                               # Test connection
KEYS *                                             # List keys
GET key                                            # Get value
TTL key                                            # Check expiration
FLUSHDB                                            # Clear all keys

# Backend
cargo build                                        # Build
cargo run                                          # Run
cargo test                                         # Run tests
cargo clean                                        # Clean build artifacts

# Migrations
sqlx migrate run                                   # Apply migrations
sqlx migrate revert                                # Revert last migration
sqlx migrate add <name>                            # Create new migration
```

### Connection Strings

**PostgreSQL (Development):**

```
postgres://postgres:postgres@localhost:5432/insightboard
```

**Redis (Development):**

```
redis://127.0.0.1:6379
```

**PostgreSQL (Production - Neon):**

```
postgres://user:pass@host.neon.tech/dbname?sslmode=require
```

**Redis (Production - Upstash):**

```
rediss://default:pass@host.upstash.io:6379
```

---

## 🎓 Learning Resources

### Docker & Docker Compose

-   [Docker Official Documentation](https://docs.docker.com/)
-   [Docker Compose Documentation](https://docs.docker.com/compose/)
-   [Play with Docker](https://labs.play-with-docker.com/) - Browser-based learning

### PostgreSQL

-   [PostgreSQL Official Documentation](https://www.postgresql.org/docs/)
-   [PostgreSQL Tutorial](https://www.postgresqltutorial.com/)
-   [Use The Index, Luke](https://use-the-index-luke.com/) - SQL indexing

### Redis

-   [Redis Official Documentation](https://redis.io/docs/)
-   [Redis University](https://university.redis.com/) - Free courses
-   [Redis CLI Tutorial](https://redis.io/topics/rediscli)

### Environment Variables

-   [The Twelve-Factor App](https://12factor.net/config) - Config best practices
-   [dotenv](https://github.com/motdotla/dotenv) - Understanding .env files

---

## 🤝 Getting Help

If you encounter issues not covered here:

1. **Check logs:**

    ```bash
    docker compose logs
    cargo run 2>&1 | tee backend.log
    ```

2. **Search existing issues:**

    - GitHub Issues: https://github.com/lavinhoque33/insightboard/issues

3. **Ask for help:**

    - Open a new GitHub issue
    - Include logs and error messages
    - Describe what you tried

4. **Community resources:**
    - [Rust Users Forum](https://users.rust-lang.org/)
    - [r/rust](https://reddit.com/r/rust)
    - [Docker Community Forums](https://forums.docker.com/)

---

## ✅ Summary

You now have:

-   ✅ **PostgreSQL 16** running in Docker with persistent storage
-   ✅ **Redis 7** configured for caching with persistence
-   ✅ **Environment configuration** for secure secret management
-   ✅ **Database migrations** for schema version control
-   ✅ **Health checks** ensuring services are ready
-   ✅ **Development workflow** for productive daily work

**Next steps:**

-   Proceed to **Phase 6: Frontend Foundation**
-   Build Vue 3 frontend
-   Create widget components
-   Implement dashboard UI

---

**Last Updated**: October 31, 2025  
**Version**: 1.0.0  
**Status**: Phase 1 Infrastructure ✅ Complete
