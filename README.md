# KCC Automation

Rust-backed automation platform for KCC (Kitchen, Countertops & Cabinet) quoting and drawing pipelines, with a Next.js operator UI.

## Architecture

This is a Cargo workspace plus a separate Next.js frontend.

```
kccautomation/
├── crates/              # Rust workspace
│   ├── kcc-api          # Axum HTTP API (port 3000)
│   ├── kcc-worker       # Background job runner
│   ├── kcc-core         # Shared domain types, DB, S3 helpers
│   ├── kcc-dxf          # DXF/DWG parsing & geometry
│   ├── kcc-report       # PDF / quote report generation
│   ├── erp-core         # ERP domain primitives
│   ├── erp-boq          # Bill of quantities
│   ├── erp-costs        # Costing engine
│   └── erp-assemblies   # Assembly composition
├── frontend/            # Next.js 15 admin UI (port 3001)
├── migrations/          # sqlx migrations (applied by kcc-api on startup)
└── docker/              # Dockerfiles per service
    ├── Dockerfile.api
    ├── Dockerfile.worker
    ├── Dockerfile.frontend
    └── docker-compose.yml
```

## Runtime services

Three independently deployable services, plus two stateful dependencies:

| Service       | Binary / entry       | Port | Purpose                         |
|---------------|----------------------|------|---------------------------------|
| `kcc-api`     | `kcc-api`            | 3000 | REST API, runs migrations on boot |
| `kcc-worker`  | `kcc-worker`         | —    | Pulls jobs from queue, no HTTP  |
| `kcc-frontend`| Next.js standalone   | 3001 | Operator UI                      |
| Postgres 16   | managed              | 5432 | Primary DB                       |
| Redis 7       | managed              | 6379 | Job queue + cache                |

All three app services share the same Postgres and Redis.

## Local development

```bash
# 1. Start Postgres + Redis locally
cd docker && docker compose up -d

# 2. Copy env template & fill in AWS / API keys
cp .env.example .env

# 3. Run the API (applies migrations on boot)
cargo run --bin kcc-api

# 4. In another terminal, run the worker
cargo run --bin kcc-worker

# 5. In another terminal, run the frontend
cd frontend && npm install && npm run dev
```

## Environment variables

Core vars (required in prod for every service that uses them):

- `DATABASE_URL` — Postgres connection string
- `REDIS_URL` — Redis connection string
- `AWS_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `S3_BUCKET` — asset storage
- `BRIGHTDATA_API_KEY`, `BRIGHTDATA_ZONE` — web scraping
- `OPENROUTER_API_KEY` — LLM inference
- `JWT_SECRET` — API auth (≥32 chars)
- `RUST_LOG` — log level (e.g. `info,kcc_api=debug`)

Frontend additionally reads:

- `NEXT_PUBLIC_API_URL` — public URL of `kcc-api`
- `NEXT_PUBLIC_CAD_VIEWER_ORIGIN` / `_PATH` — for the embedded CAD viewer

## Build

```bash
cargo build --release --bin kcc-api
cargo build --release --bin kcc-worker
cd frontend && npm run build
```

Or via Docker:

```bash
docker build -f docker/Dockerfile.api      -t kcc-api .
docker build -f docker/Dockerfile.worker   -t kcc-worker .
docker build -f docker/Dockerfile.frontend -t kcc-frontend .
```

## Migrations

`sqlx` migrations live under `migrations/`. `kcc-api` applies them automatically on startup via `sqlx::migrate!`. Do not run them manually against prod.

## Deployment

Designed for Dockploy — one Application per service, two managed Database services (Postgres, Redis). See the deploy runbook when preparing a new environment.
