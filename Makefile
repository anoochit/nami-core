.PHONY: all webui nami clean run build

# Default target
all: build

# Build everything
build: nami

# Build WebUI assets
webui:
	@echo "Building WebUI..."
	cd webui && npm install && npm run build

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
