# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Product Twin is a proof-of-concept digital twin / knowledge graph system built with Rust, Neo4j, and Redis. The Rust service exposes a JSON API under `/api/*` (Axum), reads configuration from `.env`, persists graph data in Neo4j, and serves a React SPA (`frontend/`) for the UI.

## Commands

### Development

```bash
cargo build              # debug build
cargo build --release    # release build
cargo run                # run API on :3000 (requires .env)
RUST_LOG=info cargo run  # run with log output
cargo test               # run tests
cargo clippy             # lint
```

Copy `.env.example` to `.env` and fill in values before running locally.

### Frontend

```bash
cd frontend
npm install     # install dependencies
npm run dev     # Vite dev server on :5173, proxies /api to :3000
npm run build   # production build -> frontend/dist (served by Axum)
npm run lint    # eslint
```

For local development, run `cargo run` and `npm run dev` in separate terminals and browse `http://localhost:5173`. In production, Axum serves `frontend/dist` directly from `:3000`.

### Docker / Infrastructure

```bash
./run.sh start           # start Neo4j (and app if built)
./run.sh stop            # stop containers
./run.sh restart         # restart containers
./run.sh status          # show container status
./run.sh logs [svc] [n]  # tail logs
./run.sh reset           # stop and wipe all data volumes
./run.sh build           # rebuild images and start
./run.sh shell           # bash shell inside Neo4j container

docker compose up -d     # equivalent to ./run.sh start
```

Access points when running:
- Application: `http://localhost:3000`
- Neo4j Browser: `http://localhost:7474`
- Neo4j Bolt: `bolt://localhost:7687`

## Architecture

```
┌──────────────────────────┐
│  React SPA (frontend/)   │  :5173 dev (Vite) / served by Axum in prod
└────────────┬─────────────┘
             │ /api/* (fetch, credentials: include)
┌────────────▼─────────────┐
│  Rust / Axum HTTP server │  :3000
│  (src/main.rs)           │
│  tokio async runtime     │
└────────────┬─────────────┘
             │ neo4rs
┌────────────▼─────────────┐
│  Neo4j (+ APOC plugin)   │  :7687 bolt  :7474 browser
│  Knowledge graph store   │
└──────────────────────────┘
  Redis (planned, not yet wired)
```

Routing in `src/main.rs`: `/api/*` routes (health, me, login, logout) return JSON; everything else falls back to `frontend/dist` (static assets, with `index.html` as the SPA fallback for client-side routes).

### Key files

| Path | Role |
|------|------|
| `src/main.rs` | Entry point — Axum router (`/api/*` JSON routes + SPA static fallback), server startup |
| `Cargo.toml` | Dependencies: axum 0.8, tokio, neo4rs 0.8, tower-http (fs), dotenvy, env_logger |
| `frontend/` | React + TypeScript SPA (Vite), built to `frontend/dist` |
| `docker-compose.yml` | Orchestrates app + Neo4j + Redis services |
| `Dockerfile` | Multi-stage build: node frontend builder → rust builder → debian bookworm-slim runtime |
| `.env.example` | Template for `NEO4J_URI`, `NEO4J_USER`, `NEO4J_PASSWORD`, `REDIS_URL`, `APP_USERNAME`, `APP_PASSWORD`, `RUST_LOG` |
| `run.sh` | Convenience wrapper around docker compose |

## Current State

- Neo4j connection via `neo4rs` is declared but not yet used in business logic.
- Redis is listed in the README as planned but has no Cargo dependency yet.
- Auth is a simple cookie session (`session=authenticated`) checked against `APP_USERNAME`/`APP_PASSWORD`; `/api/me` reports status, `/api/login` and `/api/logout` set/clear the cookie.
- The dashboard has a placeholder section reserved for future Neo4j graph visualization (`frontend/src/components/GraphViewPlaceholder.tsx`).
- No tests have been written yet.
