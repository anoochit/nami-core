.PHONY: all webui nami clean run build desktop desktop-dev check-docs

# Default target
all: build

# Build everything
build: nami

# Build WebUI assets
webui:
	@echo "Building WebUI..."
	cd webui && pnpm install && pnpm run build

deps:
	@echo "Installing WebUI dependencies..."
	cd webui && pnpm install

# Build Rust binary (depends on WebUI assets for embedding)
nami: webui
	@echo "Building Rust application..."
ifdef TARGET
	cargo build --release --target $(TARGET)
else
	cargo build --release
endif

# Clean artifacts
clean:
	@echo "Cleaning artifacts..."
	cargo clean
	rm -rf webui/dist

# Run the application in browse mode
run:
	cargo run -- browse

# Run automated evaluations
eval:
	cargo run -- eval

# Run all tests (unit and integration)
test:
	cargo test

# Generate technical documentation
docs:
	cargo doc --no-deps --target-dir docs/reference

# Detect OS and set default TAURI_TARGET if not provided
ifeq ($(OS),Windows_NT)
	DEFAULT_TAURI_TARGET := x86_64-pc-windows-msvc
else ifeq ($(shell uname -s),Linux)
	DEFAULT_TAURI_TARGET := x86_64-unknown-linux-gnu
else ifeq ($(shell uname -s),Darwin)
	DEFAULT_TAURI_TARGET := x86_64-apple-darwin
else
	DEFAULT_TAURI_TARGET := unknown
endif

TAURI_TARGET ?= $(DEFAULT_TAURI_TARGET)

# Build for Desktop (Tauri)
desktop: webui
	@echo "Building Desktop application (Tauri)..."
	npm exec -- @tauri-apps/cli build $(if $(TAURI_TARGET),--target $(TAURI_TARGET),)

# Run Desktop in development mode
desktop-dev: deps
	@echo "Starting Desktop in development mode..."
	@cmd /c start "" pnpm -C webui dev
	@set CI=true && npx @tauri-apps/cli dev $(if $(TAURI_TARGET),--target $(TAURI_TARGET),)

# Check for missing module README.md files
check-docs:
	@find src -type d ! -path "src" ! -path "src/utils" ! -path "src/modes/ui_utils" ! -exec test -e "{}/README.md" \; -print
