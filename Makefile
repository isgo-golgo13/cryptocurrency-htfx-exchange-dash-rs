# ==============================================================================
# BTC Exchange Dashboard - Production Makefile
# ==============================================================================
#
# Usage:
#   make help          - Show this help
#   make dev           - Start development environment
#   make build         - Build everything (debug)
#   make release       - Build optimized release
#   make test          - Run all tests
#   make clean         - Clean build artifacts
#
# Requirements:
#   - Rust 1.82+ (rustup.rs)
#   - wasm32-unknown-unknown target
#   - trunk (cargo install trunk)
#
# ==============================================================================

.PHONY: help dev build release test clean fmt lint check \
        install-deps install-trunk install-wasm \
        server frontend docker firecracker \
        build-server build-frontend build-static \
        watch-server watch-frontend

# ------------------------------------------------------------------------------
# Configuration
# ------------------------------------------------------------------------------

CARGO := cargo
TRUNK := trunk
RUSTUP := rustup

# Directories
PROJECT_ROOT := $(shell pwd)
SERVER_DIR := server/dash-server
FRONTEND_DIR := crates/dash-app
DIST_DIR := $(FRONTEND_DIR)/dist
STATIC_DIR := static

# Build targets
WASM_TARGET := wasm32-unknown-unknown
MUSL_TARGET := x86_64-unknown-linux-musl

# Colors
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m

# ------------------------------------------------------------------------------
# Default & Help
# ------------------------------------------------------------------------------

.DEFAULT_GOAL := help

help: ## Show this help
	@echo ""
	@echo "$(CYAN)BTC Exchange Dashboard$(NC)"
	@echo "$(CYAN)======================$(NC)"
	@echo ""
	@echo "$(YELLOW)Setup:$(NC)"
	@grep -E '^install-[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Development:$(NC)"
	@grep -E '^(dev|watch|server|frontend):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Build:$(NC)"
	@grep -E '^(build|release|build-)[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Quality:$(NC)"
	@grep -E '^(test|fmt|lint|check|audit):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Deploy:$(NC)"
	@grep -E '^(docker|firecracker)[a-zA-Z_-]*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Cleanup:$(NC)"
	@grep -E '^clean[a-zA-Z_-]*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""

# ------------------------------------------------------------------------------
# Setup & Dependencies
# ------------------------------------------------------------------------------

install-deps: install-wasm install-trunk ## Install all dependencies
	@echo "$(GREEN)✓ All dependencies installed$(NC)"

install-wasm: ## Install WASM target
	@echo "$(CYAN)Installing wasm32-unknown-unknown target...$(NC)"
	$(RUSTUP) target add $(WASM_TARGET)

install-trunk: ## Install trunk bundler
	@echo "$(CYAN)Installing trunk...$(NC)"
	$(CARGO) install trunk wasm-bindgen-cli

install-musl: ## Install musl target (for static binaries)
	@echo "$(CYAN)Installing musl target...$(NC)"
	$(RUSTUP) target add $(MUSL_TARGET)

# ------------------------------------------------------------------------------
# Development
# ------------------------------------------------------------------------------

dev: ## Start development environment (server + frontend)
	@echo "$(CYAN)Starting development environment...$(NC)"
	@echo "$(YELLOW)Run these in separate terminals:$(NC)"
	@echo "  Terminal 1: make watch-server"
	@echo "  Terminal 2: make watch-frontend"
	@echo ""
	@echo "$(GREEN)Dashboard: http://127.0.0.1:8080$(NC)"
	@echo "$(GREEN)Server:    http://127.0.0.1:3001$(NC)"

server: build-server ## Run backend server
	@echo "$(CYAN)Starting server on :3001...$(NC)"
	cd $(SERVER_DIR) && $(CARGO) run

frontend: ## Run frontend dev server
	@echo "$(CYAN)Starting frontend on :8080...$(NC)"
	cd $(FRONTEND_DIR) && $(TRUNK) serve --open

watch-server: ## Run server with auto-reload
	@echo "$(CYAN)Starting server with watch...$(NC)"
	cd $(SERVER_DIR) && $(CARGO) watch -x run

watch-frontend: ## Run frontend with hot reload
	@echo "$(CYAN)Starting frontend with hot reload...$(NC)"
	cd $(FRONTEND_DIR) && $(TRUNK) serve

# ------------------------------------------------------------------------------
# Build
# ------------------------------------------------------------------------------

build: ## Build all crates (debug)
	@echo "$(CYAN)Building workspace (debug)...$(NC)"
	$(CARGO) build --workspace
	@echo "$(GREEN)✓ Build complete$(NC)"

build-server: ## Build server only
	@echo "$(CYAN)Building server...$(NC)"
	$(CARGO) build -p dash-server

build-frontend: ## Build WASM frontend
	@echo "$(CYAN)Building WASM frontend...$(NC)"
	cd $(FRONTEND_DIR) && $(TRUNK) build
	@echo "$(GREEN)✓ Frontend built: $(DIST_DIR)/$(NC)"

release: release-server release-frontend ## Build optimized release
	@echo "$(GREEN)✓ Release build complete$(NC)"

release-server: ## Build optimized server
	@echo "$(CYAN)Building server (release)...$(NC)"
	$(CARGO) build --release -p dash-server
	@ls -lh target/release/dash-server

release-frontend: ## Build optimized WASM frontend
	@echo "$(CYAN)Building frontend (release)...$(NC)"
	cd $(FRONTEND_DIR) && $(TRUNK) build --release
	@echo "$(GREEN)✓ Frontend built: $(DIST_DIR)/$(NC)"
	@du -sh $(DIST_DIR)

build-static: install-musl ## Build static server binary (musl)
	@echo "$(CYAN)Building static binary (musl)...$(NC)"
	RUSTFLAGS='-C target-feature=+crt-static' \
		$(CARGO) build --release --target $(MUSL_TARGET) -p dash-server
	@ls -lh target/$(MUSL_TARGET)/release/dash-server
	@echo "$(GREEN)✓ Static binary built$(NC)"

# ------------------------------------------------------------------------------
# Quality
# ------------------------------------------------------------------------------

test: ## Run all tests
	@echo "$(CYAN)Running tests...$(NC)"
	$(CARGO) test --workspace
	@echo "$(GREEN)✓ All tests passed$(NC)"

test-verbose: ## Run tests with output
	$(CARGO) test --workspace -- --nocapture

fmt: ## Format code
	@echo "$(CYAN)Formatting code...$(NC)"
	$(CARGO) fmt --all
	@echo "$(GREEN)✓ Code formatted$(NC)"

fmt-check: ## Check formatting
	$(CARGO) fmt --all -- --check

lint: ## Run clippy lints
	@echo "$(CYAN)Running clippy...$(NC)"
	$(CARGO) clippy --workspace --all-targets -- -D warnings
	@echo "$(GREEN)✓ No lint warnings$(NC)"

check: ## Quick check (no codegen)
	@echo "$(CYAN)Checking workspace...$(NC)"
	$(CARGO) check --workspace
	@echo "$(GREEN)✓ Check passed$(NC)"

audit: ## Audit dependencies for vulnerabilities
	@echo "$(CYAN)Auditing dependencies...$(NC)"
	$(CARGO) audit
	@echo "$(GREEN)✓ No vulnerabilities found$(NC)"

# Alias for CI pipeline
ci: fmt-check lint test ## Run all CI checks
	@echo "$(GREEN)✓ CI checks passed$(NC)"

# ------------------------------------------------------------------------------
# Docker
# ------------------------------------------------------------------------------

docker: docker-build ## Build and run Docker containers
	@echo "$(CYAN)Starting Docker containers...$(NC)"
	cd deploy/docker && docker-compose up -d
	@echo "$(GREEN)✓ Dashboard: http://localhost:8080$(NC)"

docker-build: ## Build Docker images
	@echo "$(CYAN)Building Docker images...$(NC)"
	cd deploy/docker && docker-compose build

docker-down: ## Stop Docker containers
	@echo "$(CYAN)Stopping Docker containers...$(NC)"
	cd deploy/docker && docker-compose down

docker-logs: ## Follow Docker logs
	cd deploy/docker && docker-compose logs -f

docker-clean: ## Remove Docker images and volumes
	cd deploy/docker && docker-compose down -v --rmi local

# ------------------------------------------------------------------------------
# Firecracker
# ------------------------------------------------------------------------------

firecracker-prereqs: ## Check Firecracker prerequisites
	@echo "$(CYAN)Checking Firecracker prerequisites...$(NC)"
	./deploy/firecracker/setup.sh prereqs

firecracker-build: build-static release-frontend ## Build for Firecracker
	@echo "$(CYAN)Building Firecracker rootfs...$(NC)"
	./deploy/firecracker/setup.sh build

firecracker-run: ## Run Firecracker VM
	@echo "$(CYAN)Starting Firecracker VM...$(NC)"
	./deploy/firecracker/setup.sh network
	./deploy/firecracker/setup.sh run

firecracker-clean: ## Cleanup Firecracker artifacts
	./deploy/firecracker/setup.sh clean

# ------------------------------------------------------------------------------
# Cleanup
# ------------------------------------------------------------------------------

clean: ## Clean build artifacts
	@echo "$(CYAN)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	rm -rf $(DIST_DIR)
	rm -rf node_modules
	@echo "$(GREEN)✓ Clean complete$(NC)"

clean-all: clean docker-clean firecracker-clean ## Clean everything
	@echo "$(GREEN)✓ Full clean complete$(NC)"

# ------------------------------------------------------------------------------
# Utilities
# ------------------------------------------------------------------------------

loc: ## Count lines of code
	@echo "$(CYAN)Lines of code:$(NC)"
	@find crates server -name "*.rs" | xargs wc -l | tail -1
	@echo ""
	@echo "$(CYAN)By crate:$(NC)"
	@for dir in crates/*/src server/*/src; do \
		count=$$(find $$dir -name "*.rs" | xargs wc -l 2>/dev/null | tail -1 | awk '{print $$1}'); \
		printf "  %-30s %s\n" "$$dir" "$$count"; \
	done

tree: ## Show project structure
	@tree -I 'target|dist|node_modules' -L 3

deps: ## Show dependency tree
	$(CARGO) tree --workspace

outdated: ## Check for outdated dependencies
	$(CARGO) outdated --workspace

size: release ## Show binary sizes
	@echo "$(CYAN)Binary sizes:$(NC)"
	@ls -lh target/release/dash-server 2>/dev/null || echo "  Server: not built"
	@du -sh $(DIST_DIR) 2>/dev/null || echo "  Frontend: not built"

.PHONY: loc tree deps outdated size
