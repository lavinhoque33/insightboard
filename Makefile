SHELL := /bin/sh
.DEFAULT_GOAL := help

# Project Node (gitignored .tools/ download) wins over host Node when present.
NODE_DIR := $(CURDIR)/.tools/node-v20.19.0-linux-x64/bin
ifneq ($(wildcard $(NODE_DIR)/node),)
export PATH := $(NODE_DIR):$(PATH)
endif
# rustup installs to ~/.cargo/bin, which non-login shells may not have on PATH.
ifneq ($(wildcard $(HOME)/.cargo/bin/cargo),)
export PATH := $(HOME)/.cargo/bin:$(PATH)
endif

BACKEND := backend
FRONTEND := frontend

# One source of truth for the Rust version: rust-toolchain.toml.
RUST_VERSION := $(shell sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)
RUST_IMAGE := rust:$(RUST_VERSION)-bookworm
HAVE_CARGO := $(shell command -v cargo 2>/dev/null)

.PHONY: help setup setup-env doctor db-up db-down db-reset backend-dev web-dev backend-check web-check e2e verify format require-env require-db require-cargo require-web-deps

help:
	@printf '%s\n' \
	  'InsightBoard development (run from the repository root; local-only, loopback)' \
	  '  setup          npm ci; create .env from .env.example with generated secrets if it is absent' \
	  '  doctor         Report tool versions, .env presence, and Docker access' \
	  '  db-up          Start Postgres and Redis on 127.0.0.1 and wait until healthy' \
	  '  db-down        Stop Postgres and Redis; data volumes are kept' \
	  '  db-reset       DESTRUCTIVE: remove containers and data volumes (needs CONFIRM=yes)' \
	  '  backend-dev    Run the API on 127.0.0.1:8080 with the root .env (needs host Rust)' \
	  '  web-dev        Run Vite on 127.0.0.1:5173 with the /api proxy' \
	  '  backend-check  cargo fmt --check, clippy -D warnings, test, build (host Rust, else container); needs db-up (GitHub Actions: uses DATABASE_URL/REDIS_URL)' \
	  '  web-check      ESLint + Prettier check, Vitest, vue-tsc + vite build' \
	  '  e2e            Playwright journeys (Chromium) on loopback fixtures: backend :8090, Vite :5175; needs db-up and host Rust' \
	  '  verify         backend-check and web-check; backend tests need Postgres and Redis (make db-up)' \
	  '  format         cargo fmt and Prettier write (needs host Rust)'

# --- setup ------------------------------------------------------------------

setup: setup-env
	cd $(FRONTEND) && npm ci

# Creates .env only when absent and never reads or overwrites an existing one.
# Secrets are passed to awk through its environment (not argv) and are never printed.
setup-env:
	@if [ -e .env ]; then \
	  printf '%s\n' '.env exists; leaving it untouched'; \
	else \
	  command -v openssl >/dev/null 2>&1 || { printf '%s\n' 'setup: openssl is required to generate secrets'; exit 1; }; \
	  umask 077; \
	  IB_PGU=insightboard_app IB_PGP="$$(openssl rand -hex 24)" IB_RDP="$$(openssl rand -hex 24)" IB_JWT="$$(openssl rand -hex 32)" \
	  awk '\
	    /^POSTGRES_USER=/ { print "POSTGRES_USER=" ENVIRON["IB_PGU"]; next } \
	    /^POSTGRES_PASSWORD=/ { print "POSTGRES_PASSWORD=" ENVIRON["IB_PGP"]; next } \
	    /^REDIS_PASSWORD=/ { print "REDIS_PASSWORD=" ENVIRON["IB_RDP"]; next } \
	    /^JWT_SECRET=/ { print "JWT_SECRET=" ENVIRON["IB_JWT"]; next } \
	    { gsub(/<POSTGRES_USER>/, ENVIRON["IB_PGU"]); gsub(/<POSTGRES_PASSWORD>/, ENVIRON["IB_PGP"]); gsub(/<REDIS_PASSWORD>/, ENVIRON["IB_RDP"]); print }' \
	    .env.example > .env.tmp && mv -n .env.tmp .env && rm -f .env.tmp && chmod 600 .env \
	    || { rm -f .env.tmp; printf '%s\n' 'setup: could not create .env'; exit 1; }; \
	  printf '%s\n' 'Created .env (mode 600) with generated secrets; provider keys are left empty'; \
	fi

doctor:
	@status=0; \
	printf 'node:    '; \
	if command -v node >/dev/null 2>&1; then \
	  v="$$(node -p process.versions.node)"; \
	  ok="$$(node -p "const [a,b]=process.versions.node.split('.').map(Number); (a===20&&b>=19)||(a===22&&b>=12)||a>22")"; \
	  src=host; case "$$(command -v node)" in $(CURDIR)/.tools/*) src=project;; esac; \
	  if [ "$$ok" = true ]; then printf '%s (%s; required ^20.19.0 || >=22.12.0)\n' "$$v" "$$src"; \
	  else printf '%s (%s) TOO OLD: need ^20.19.0 || >=22.12.0; see .nvmrc\n' "$$v" "$$src"; status=1; fi; \
	else printf '%s\n' 'MISSING (need Node 20.19+; see .nvmrc)'; status=1; fi; \
	printf 'npm:     '; \
	if command -v npm >/dev/null 2>&1; then npm --version; else printf '%s\n' 'MISSING'; status=1; fi; \
	printf 'docker:  '; \
	if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then \
	  printf '%s; ' "$$(docker --version)"; docker compose version --short 2>/dev/null | sed 's/^/compose /' || printf '%s\n' 'compose MISSING'; have_docker=1; \
	else printf '%s\n' 'not available (needed for db-up and the container fallback)'; have_docker=0; fi; \
	printf 'cargo:   '; \
	if command -v cargo >/dev/null 2>&1; then \
	  printf '%s (rustc %s; pinned by rust-toolchain.toml: $(RUST_VERSION))\n' "$$(cargo --version)" "$$(rustc --version | cut -d' ' -f2)"; \
	elif [ "$$have_docker" = 1 ]; then printf '%s\n' 'not installed; backend-check will use the container fallback ($(RUST_IMAGE))'; \
	else printf '%s\n' 'MISSING and no Docker for the container fallback'; status=1; fi; \
	printf 'openssl: '; \
	if command -v openssl >/dev/null 2>&1; then printf '%s\n' "$$(openssl version | cut -d' ' -f1-2)"; else printf '%s\n' 'MISSING (setup needs it to generate secrets)'; fi; \
	printf '.env:    '; \
	if [ -f .env ]; then printf '%s\n' 'present (contents not read)'; else printf '%s\n' 'absent; run make setup'; fi; \
	printf 'deps:    '; \
	if [ -d $(FRONTEND)/node_modules ]; then printf '%s\n' 'frontend/node_modules present'; else printf '%s\n' 'frontend/node_modules absent; run make setup'; fi; \
	exit $$status

# --- database ---------------------------------------------------------------

require-env:
	@[ -f .env ] || { printf '%s\n' 'No .env: run make setup'; exit 1; }

require-cargo:
	@[ -n "$(HAVE_CARGO)" ] || { printf '%s\n' 'This target needs host Rust (rustup); see docs/development/local-setup.md. backend-check works without it through the container fallback.'; exit 1; }

require-web-deps:
	@[ -d $(FRONTEND)/node_modules ] || { printf '%s\n' 'frontend/node_modules is missing: run make setup'; exit 1; }

# Backend tests use the compose Postgres (isolated #[sqlx::test] databases) and Redis
# (unique key prefixes). Checks state only; never starts, stops, or recreates containers.
# In GitHub Actions (GITHUB_ACTIONS=true) the workflow's service containers replace compose:
# the target only checks that DATABASE_URL and REDIS_URL are set, and .env is not needed.
# Keyed on GITHUB_ACTIONS, not CI: some local shells and agent harnesses export CI=true.
ifeq ($(GITHUB_ACTIONS),true)
require-db:
	@[ -n "$$DATABASE_URL" ] && [ -n "$$REDIS_URL" ] || { printf '%s\n' 'CI: DATABASE_URL and REDIS_URL must be set for backend-check'; exit 1; }
else
require-db: require-env
	@command -v docker >/dev/null 2>&1 || { printf '%s\n' 'backend-check needs Docker for Postgres and Redis'; exit 1; }
	@running="$$(docker compose ps --status running --services 2>/dev/null)"; \
	for svc in postgres redis; do \
	  printf '%s\n' "$$running" | grep -qx "$$svc" || { printf '%s\n' "backend-check needs Postgres and Redis running: run make db-up ($$svc is not running)"; exit 1; }; \
	done
endif

db-up: require-env
	docker compose up -d --wait postgres redis

db-down:
	docker compose stop postgres redis

# The only target allowed to remove volumes.
db-reset:
	@if [ "$(CONFIRM)" != "yes" ]; then \
	  printf '%s\n' 'Refusing: this deletes every local user, dashboard, and cache entry. Re-run as: make db-reset CONFIRM=yes'; exit 1; fi
	docker compose down --volumes

# --- run --------------------------------------------------------------------

backend-dev: require-env require-cargo
	cd $(BACKEND) && cargo run

web-dev: require-web-deps
	cd $(FRONTEND) && npm run dev

# --- checks -----------------------------------------------------------------

# Host Rust when present; otherwise a throwaway container. Tests read DATABASE_URL and
# REDIS_URL from .env (exported to cargo without printing) when it exists; in CI they come
# from the environment. The container mounts backend/
# read-only, builds in a copy inside the container (no root-owned files in the checkout),
# uses the host network to reach the 127.0.0.1 Postgres and Redis, and downloads crates on
# every run (no cache volume).
ifneq ($(HAVE_CARGO),)
backend-check: require-db
	cd $(BACKEND) && cargo fmt --check
	cd $(BACKEND) && cargo clippy --locked --all-targets -- -D warnings
	@printf '%s\n' 'cd $(BACKEND) && cargo test --locked  (environment from .env when present)'
	@cd $(BACKEND) && if [ -f ../.env ]; then set -a && . ../.env && set +a; fi && cargo test --locked
	cd $(BACKEND) && cargo build --locked
else
backend-check: require-db
	docker run --rm --network host --env-file "$(CURDIR)/.env" -v "$(CURDIR)/$(BACKEND):/src:ro" $(RUST_IMAGE) bash -c '\
	  set -ex; \
	  rustup component add rustfmt clippy; \
	  mkdir /work && tar -C /src --exclude=./target -cf - . | tar -C /work -xf -; \
	  cd /work; \
	  cargo fmt --check; \
	  cargo clippy --locked --all-targets -- -D warnings; \
	  cargo test --locked; \
	  cargo build --locked'
endif

web-check: require-web-deps
	cd $(FRONTEND) && npm run lint
	cd $(FRONTEND) && npm test
	cd $(FRONTEND) && npm run build

# Playwright journeys (frontend/e2e, frontend/playwright.config.ts). Playwright starts the
# provider/status fixture (18090/18091), the freshly built backend (8090), and Vite (5175);
# it never reuses the dev services. The config fixes the backend's provider base URLs (the
# fixture), provider keys (dummies), JWT secret, CORS origin, and status allowlist over any
# value in .env; DATABASE_URL and REDIS_URL come from .env (exported without printing) or,
# in CI, the workflow's service containers. One-time browser download:
# cd frontend && npx playwright install chromium. Not part of verify.
e2e: require-db require-cargo require-web-deps
	cd $(BACKEND) && cargo build --locked
	@printf '%s\n' 'cd $(FRONTEND) && npm run e2e  (DATABASE_URL and REDIS_URL from .env when present)'
	@cd $(FRONTEND) && if [ -f ../.env ]; then set -a && . ../.env && set +a; fi && npm run e2e

verify: backend-check web-check

format: require-cargo require-web-deps
	cd $(BACKEND) && cargo fmt
	cd $(FRONTEND) && npm run format
