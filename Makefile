.PHONY: build build-release test server desktop desktop-build docs clean db-reset lint fmt check all dev

# ─── Build ───────────────────────────────────────────────────────────
build:
	cargo build --workspace

build-release:
	cargo build --workspace --release

# ─── Test ────────────────────────────────────────────────────────────
test:
	cargo test --workspace

# ─── Lint & Format ───────────────────────────────────────────────────
lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

check:
	cargo check --workspace

# ─── Server ──────────────────────────────────────────────────────────
server:
	cargo run -p ciab-cli -- server start --config config.toml

server-release:
	cargo run -p ciab-cli --release -- server start --config config.toml

# ─── Desktop App ─────────────────────────────────────────────────────
desktop-install:
	cd desktop && npm install

desktop:
	cd desktop && npm run tauri dev

desktop-build:
	cd desktop && npm run tauri build

# ─── Docs ────────────────────────────────────────────────────────────
docs-install:
	cd docs && pip install -r requirements.txt

docs:
	cd docs && mkdocs serve

docs-build:
	cd docs && mkdocs build

# ─── Database ────────────────────────────────────────────────────────
db-reset:
	rm -f ciab.db && echo "Database reset. Will be recreated on next server start."

# ─── Clean ───────────────────────────────────────────────────────────
clean:
	cargo clean
	rm -rf desktop/dist desktop/src-tauri/target

# ─── Development (server + desktop) ─────────────────────────────────
# Run server in background, then start desktop app
dev:
	@echo "Starting CIAB server + desktop app..."
	@echo "Step 1: Starting backend server on :9090..."
	@cargo run -p ciab-cli -- server start --config config.toml &
	@sleep 3
	@echo "Step 2: Starting desktop app..."
	@cd desktop && npm run tauri dev

# ─── All (build + test) ─────────────────────────────────────────────
all: build test
