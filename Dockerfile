# InsightBoard release image (M5 decision 2): the backend serving the built frontend.
# Build with `make release`; run with `make up` (docker-compose.yml, profile `app`).
# Nothing here reads .env or holds a secret: configuration arrives at run time from compose.

# --- frontend: npm ci + vue-tsc + vite build ---------------------------------------------
FROM node:20.19.0-bookworm-slim AS frontend
WORKDIR /build/frontend
# Lockfile-only layer: dependencies are reinstalled only when the manifests change.
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci --no-audit --no-fund
# vue-tsc also type-checks the contract test, which imports ../contracts/api-contract.v1.json.
COPY contracts/ /build/contracts/
COPY frontend/ ./
RUN npm run build

# --- backend: cargo build --release --locked ----------------------------------------------
# Same toolchain as rust-toolchain.toml (not copied: rustup would try to add rustfmt/clippy).
FROM rust:1.99.0-bookworm AS backend
WORKDIR /build/backend
# Dependency layer: build the locked dependencies against a stub binary, so source-only
# changes reuse this layer instead of recompiling every crate.
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src
# Migrations are embedded by sqlx::migrate! at compile time.
COPY backend/migrations/ migrations/
COPY backend/src/ src/
# touch: the stub's build artefacts are newer than the copied sources' mtimes.
RUN touch src/main.rs && cargo build --release --locked

# --- runtime -------------------------------------------------------------------------------
FROM debian:bookworm-slim
# libssl3 + ca-certificates: reqwest native TLS to the providers; curl: the compose healthcheck.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libssl3 \
    && rm -rf /var/lib/apt/lists/*
RUN groupadd --system --gid 10001 insightboard \
    && useradd --system --uid 10001 --gid insightboard --no-create-home \
       --home-dir /nonexistent --shell /usr/sbin/nologin insightboard
WORKDIR /app
# Root-owned and read-only for the runtime user.
COPY --from=backend /build/backend/target/release/insightboard-backend /app/insightboard-backend
COPY --from=frontend /build/frontend/dist/ /app/dist/
# APP_HOST is deliberately unset (loopback default); compose sets 0.0.0.0 for the published port.
ENV STATIC_DIR=/app/dist \
    APP_PORT=8080
USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["/app/insightboard-backend"]
