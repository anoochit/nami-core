.PHONY: all webui nami clean run build

# Default target
all: build

# Build everything
build: nami

# Build WebUI assets
webui:
	@echo "Building WebUI..."
	cd webui && pnpm install && pnpm run build

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

# Build for Desktop (Tauri)
desktop:
	@echo "Building Desktop application (Tauri)..."
	npx @tauri-apps/cli build

# Run Desktop in development mode
desktop-dev:
	@echo "Starting Desktop in development mode..."
	npx @tauri-apps/cli dev

# Check for missing module README.md files
check-docs:
	@find src -type d ! -path "src" ! -path "src/utils" ! -path "src/modes/ui_utils" ! -exec test -e "{}/README.md" \; -print
