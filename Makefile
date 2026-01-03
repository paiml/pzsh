# pzsh Makefile
# Performance-first shell framework with sub-10ms startup
# Following Toyota Way principles for fast feedback loops

.PHONY: all build test coverage coverage-quick coverage-full coverage-ci coverage-open coverage-clean bench lint fmt clean help

# Default target
all: fmt lint test

# Build
build: ## Build the project
	cargo build --release

# Test
test: ## Run all tests
	cargo test

# Coverage targets (bashrs style)
# Exclude main.rs (CLI glue code) from coverage
COVERAGE_EXCLUDE = --ignore-filename-regex 'main\.rs'

coverage: ## Generate HTML coverage report (cold: ~2min, warm: <30s)
	@echo "📊 Running fast coverage analysis..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@which cargo-nextest > /dev/null 2>&1 || (echo "📦 Installing cargo-nextest..." && cargo install cargo-nextest --locked)
	@mkdir -p target/coverage
	@echo "🧪 Running tests with instrumentation..."
	@cargo llvm-cov nextest \
		--all-features \
		--html --output-dir target/coverage/html \
		$(COVERAGE_EXCLUDE)
	@echo ""
	@echo "📊 Coverage Summary:"
	@cargo llvm-cov report --summary-only $(COVERAGE_EXCLUDE)
	@echo ""
	@echo "💡 HTML report: target/coverage/html/index.html"

coverage-quick: ## Quick coverage check (core tests only)
	@echo "⚡ Running quick coverage..."
	@cargo llvm-cov nextest --summary-only

coverage-full: ## Full coverage including slow tests
	@echo "🔬 Running full coverage analysis..."
	@cargo llvm-cov nextest \
		--all-features \
		--html --output-dir target/coverage/html
	@cargo llvm-cov report

coverage-ci: ## Coverage for CI (LCOV output)
	@echo "🤖 Running CI coverage..."
	@cargo llvm-cov nextest \
		--all-features \
		--lcov --output-path target/coverage/lcov.info
	@cargo llvm-cov report --summary-only

coverage-open: coverage ## Generate and open coverage report
	@open target/coverage/html/index.html 2>/dev/null || xdg-open target/coverage/html/index.html 2>/dev/null || echo "Open target/coverage/html/index.html"

coverage-clean: ## Clean coverage artifacts
	@rm -rf target/coverage
	@cargo llvm-cov clean --workspace

# Benchmarks
bench: ## Run benchmarks
	cargo bench

bench-startup: ## Run startup benchmark only
	cargo bench --bench startup

# Quality
lint: ## Run clippy lints
	cargo clippy --all-targets --all-features

fmt: ## Format code
	cargo fmt

fmt-check: ## Check formatting
	cargo fmt --check

# Audit
audit: ## Security audit
	@which cargo-audit > /dev/null 2>&1 || cargo install cargo-audit
	cargo audit

# Clean
clean: ## Clean build artifacts
	cargo clean

# pzsh specific
profile: build ## Profile startup time
	./target/release/pzsh profile

benchmark: build ## Run pzsh benchmark
	./target/release/pzsh bench --iterations 100

status: build ## Show pzsh status
	./target/release/pzsh status

# Book
book: ## Build the mdbook
	@which mdbook > /dev/null 2>&1 || cargo install mdbook
	cd book && mdbook build

book-serve: ## Serve the book locally
	@which mdbook > /dev/null 2>&1 || cargo install mdbook
	cd book && mdbook serve --open

# Examples
examples: ## Run all examples
	cargo run --example basic_config
	cargo run --example benchmark
	cargo run --example lint_config
	cargo run --example parser
	cargo run --example prompt

# Help
help: ## Show this help
	@echo "pzsh - Performance-first shell framework"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
