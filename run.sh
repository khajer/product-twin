#!/bin/bash

# =============================================================================
# Docker Compose Run Script for Product Twin - Neo4j Database
# =============================================================================
# Usage: ./run.sh [command]
# Commands:
#   start     Start the Neo4j database (default)
#   stop      Stop the Neo4j database
#   restart   Restart the Neo4j database
#   status    Show status of containers
#   logs      Tail logs from containers
#   reset     Stop containers and remove volumes (WARNING: deletes all data)
#   build     Rebuild and start containers
#   shell     Open a bash shell inside the Neo4j container
#   help      Show this help message
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

COMPOSE_FILE="docker-compose.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi

    if ! docker info &> /dev/null; then
        print_error "Docker daemon is not running. Please start Docker."
        exit 1
    fi
}

check_docker_compose() {
    if docker compose version &> /dev/null 2>&1; then
        COMPOSE_CMD="docker compose"
    elif command -v docker-compose &> /dev/null; then
        COMPOSE_CMD="docker-compose"
    else
        print_error "Docker Compose is not installed. Please install Docker Compose."
        exit 1
    fi
}

wait_for_neo4j() {
    print_info "Waiting for Neo4j to be healthy..."
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        local status
        status=$(docker inspect --format='{{.State.Health.Status}}' neo4j-db 2>/dev/null || echo "not found")

        if [ "$status" = "healthy" ]; then
            print_success "Neo4j is healthy and ready!"
            return 0
        fi

        attempt=$((attempt + 1))
        echo -ne "\r${BLUE}[INFO]${NC} Waiting... (attempt $attempt/$max_attempts - status: $status)"
        sleep 5
    done

    echo ""
    print_warning "Neo4j did not become healthy within the expected time."
    print_info "You can check logs with: ./run.sh logs"
    return 1
}

cmd_start() {
    check_docker
    check_docker_compose

    print_info "Starting Neo4j database..."
    $COMPOSE_CMD -f "$COMPOSE_FILE" up -d

    print_success "Containers started!"
    echo ""
    echo "─────────────────────────────────────────────"
    echo "  Neo4j Database"
    echo "─────────────────────────────────────────────"
    echo "  HTTP Browser:  http://localhost:7474"
    echo "  Bolt Protocol: bolt://localhost:7687"
    echo "  Username:      neo4j"
    echo "  Password:      password123"
    echo "─────────────────────────────────────────────"
    echo ""

    wait_for_neo4j
}

cmd_stop() {
    check_docker
    check_docker_compose

    print_info "Stopping containers..."
    $COMPOSE_CMD -f "$COMPOSE_FILE" down
    print_success "Containers stopped."
}

cmd_restart() {
    cmd_stop
    echo ""
    cmd_start
}

cmd_status() {
    check_docker
    check_docker_compose

    echo "Container Status:"
    echo "─────────────────────────────────────────────"
    $COMPOSE_CMD -f "$COMPOSE_FILE" ps
    echo ""
    echo "Volume Usage:"
    echo "─────────────────────────────────────────────"
    docker volume ls --filter "name=product-twin" --format "table {{.Name}}\t{{.Driver}}"
}

cmd_logs() {
    check_docker
    check_docker_compose

    local service="${1:-}"
    local lines="${2:-100}"

    if [ -n "$service" ]; then
        $COMPOSE_CMD -f "$COMPOSE_FILE" logs --tail="$lines" -f "$service"
    else
        $COMPOSE_CMD -f "$COMPOSE_FILE" logs --tail="$lines" -f
    fi
}

cmd_reset() {
    check_docker
    check_docker_compose

    print_warning "This will STOP all containers and DELETE all data volumes!"
    read -rp "Are you sure? (yes/no): " confirm

    if [ "$confirm" = "yes" ]; then
        print_info "Stopping containers and removing volumes..."
        $COMPOSE_CMD -f "$COMPOSE_FILE" down -v
        print_success "Containers stopped and volumes removed."
    else
        print_info "Reset cancelled."
    fi
}

cmd_build() {
    check_docker
    check_docker_compose

    print_info "Building and starting containers..."
    $COMPOSE_CMD -f "$COMPOSE_FILE" up -d --build
    print_success "Build complete and containers started!"
}

cmd_shell() {
    check_docker

    print_info "Opening bash shell in Neo4j container..."
    docker exec -it neo4j-db bash
}

cmd_help() {
    echo "Product Twin - Docker Compose Run Script"
    echo ""
    echo "Usage: ./run.sh [command]"
    echo ""
    echo "Commands:"
    echo "  start     Start the Neo4j database (default)"
    echo "  stop      Stop the Neo4j database"
    echo "  restart   Restart the Neo4j database"
    echo "  status    Show status of containers"
    echo "  logs      Tail logs from containers (optional: service name, line count)"
    echo "            Example: ./run.sh logs neo4j 50"
    echo "  reset     Stop containers and remove volumes (WARNING: deletes all data)"
    echo "  build     Rebuild and start containers"
    echo "  shell     Open a bash shell inside the Neo4j container"
    echo "  help      Show this help message"
    echo ""
}

# Main
case "${1:-start}" in
    start)
        cmd_start
        ;;
    stop)
        cmd_stop
        ;;
    restart)
        cmd_restart
        ;;
    status)
        cmd_status
        ;;
    logs)
        cmd_logs "$2" "$3"
        ;;
    reset)
        cmd_reset
        ;;
    build)
        cmd_build
        ;;
    shell)
        cmd_shell
        ;;
    help|--help|-h)
        cmd_help
        ;;
    *)
        print_error "Unknown command: $1"
        cmd_help
        exit 1
        ;;
esac
