# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Product Twin is a proof-of-concept digital twin / knowledge graph system built with Rust, Neo4j, and Redis. The Rust service exposes an HTTP API (Axum), reads configuration from `.env`, and persists graph data in Neo4j.

## Commands

### Development

```bash
cargo build              # debug build
cargo build --release    # release build
cargo run                # run locally (requires .env)
RUST_LOG=info cargo run  # run with log output
cargo test               # run tests
cargo clippy             # lint
```

Copy `.env.example` to `.env` and fill in values before running locally.

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

### Key files

| Path | Role |
|------|------|
| `src/main.rs` | Entry point — Axum router, server startup |
| `Cargo.toml` | Dependencies: axum 0.8, tokio, neo4rs 0.8, dotenvy, env_logger |
| `docker-compose.yml` | Orchestrates app + Neo4j services |
| `Dockerfile` | Multi-stage build: rust builder → debian bookworm-slim runtime |
| `.env.example` | Template for `NEO4J_URI`, `NEO4J_USER`, `NEO4J_PASSWORD`, `RUST_LOG` |
| `run.sh` | Convenience wrapper around docker compose |

## Current State

- Neo4j connection via `neo4rs` is declared but not yet used in business logic.
- Redis is listed in the README as planned but has no Cargo dependency yet.
- Only one route exists: `GET /` → `"Hello, World!"`.
- No tests have been written yet.
