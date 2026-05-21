# Product Twin

A proof-of-concept **Digital Twin** and **Knowledge Graph** system built with Rust, Neo4j, and Redis.

The system uses:
- **Rust & Axum**: High-performance asynchronous HTTP server.
- **Neo4j**: Graph database for modeling product relationships, hierarchies, and knowledge representation.
- **Redis**: Fast, in-memory store for session caching and rapid lookups (integration planned).
- **Docker Compose**: Pre-configured multi-container setup for the database backend services.

---

## Architecture Overview

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
  Redis (in-memory caching)
```

---

## Features & Endpoints

- **`/` (`GET`)**: Hello World welcome screen.
- **`/health` (`GET`)**: Service health check returning service status, runtime epoch timestamp, and semantic version.
- **`/login` (`GET`/`POST`)**: Simple Form-based Authentication.
- **`/logout` (`POST`)**: Clear session cookie and redirect to login.
- **`/landing` (`GET`)**: Secure authenticated dashboard.

---

## Getting Started

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (2024 edition)
- [Docker & Docker Compose](https://www.docker.com/)

### 1. Environment Setup

Copy `.env.example` to `.env` and adjust the variables if necessary:

```bash
cp .env.example .env
```

Default variables inside `.env`:
* `NEO4J_URI=bolt://localhost:7687`
* `NEO4J_USER=neo4j`
* `NEO4J_PASSWORD=password123`
* `REDIS_URL=redis://localhost:6379`
* `RUST_LOG=info`
* `APP_USERNAME=admin`
* `APP_PASSWORD=changeme`

### 2. Start Backend Services (Docker)

Use the helper script `run.sh` to spin up Neo4j and Redis:

```bash
# Start infrastructure containers
./run.sh start
```

This command spins up:
- **Neo4j Browser** on [http://localhost:7474](http://localhost:7474) (credentials: `neo4j` / `password123`)
- **Neo4j Bolt** on `bolt://localhost:7687`
- **Redis** on `localhost:6379`

### 3. Run the Rust Application

To run the Axum web server locally with hot logging:

```bash
RUST_LOG=info cargo run
```

The application will bind to [http://localhost:3000](http://localhost:3000).

---

## Helper Script Commands (`./run.sh`)

A convenient wrapper script is provided to manage the environment:

| Command | Action |
|:---|:---|
| `./run.sh start` | Spin up Neo4j and Redis containers (detached) |
| `./run.sh stop` | Stop and remove the containers |
| `./run.sh restart` | Stop and restart all backend services |
| `./run.sh status` | Show status of running Docker containers |
| `./run.sh logs [svc]` | View or tail logs (optionally pass `neo4j` or `redis`) |
| `./run.sh build` | Rebuild and run containers from scratch |
| `./run.sh reset` | Stop containers and wipe **all data volumes** |
| `./run.sh shell` | Open a bash shell inside the Neo4j container |

---

## Development

Check lint issues, format code, and run tests before pushing changes:

```bash
# Code formatter checks
cargo fmt --check

# Linter analysis
cargo clippy --all-targets --all-features -- -D warnings

# Run the test suite
cargo test
```
