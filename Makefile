# ==============================================================================
# Commands & Tooling
# ==============================================================================
CARGO     ?= cargo
TAURI     ?= cargo tauri
TAURI_DIR ?= src-tauri
DATASET_PATH ?= ./dataset
PLAYTEST_CONFIG ?= playtest.json
PLAYTEST_LOG ?= playtest.out

.PHONY: all help check fmt-check lint test frontend-build dataset-check run playtest playtest-analysis build build-desktop build-linux build-windows build-android build-all clean

# Default target
all: check

# Show available development and playtest commands.
help:
	@echo "Available targets:"
	@echo "  make check             Run linting and tests"
	@echo "  make fmt-check         Check Rust formatting"
	@echo "  make frontend-build    Type-check and build the frontend"
	@echo "  make dataset-check     Validate dataset references and assets"
	@echo "  make run               Launch the Tauri application"
	@echo "  make playtest          Run the playtest configured in the JSON file"
	@echo "  make playtest-analysis Analyze a playtest .out log"
	@echo "  make build             Build the desktop application"
	@echo "  make clean             Remove generated build artifacts"
	@echo ""
	@echo "Playtest variables:"
	@echo "  PLAYTEST_CONFIG=playtest.json  Playtest configuration file"

# ==============================================================================
# Quality Assurance (Lint & Test)
# ==============================================================================

fmt-check:
	@echo "--> Checking Rust formatting..."
	$(CARGO) fmt --manifest-path $(TAURI_DIR)/Cargo.toml -- --check

# Run Clippy linter
lint:
	@echo "--> Running Clippy linter..."
	cd $(TAURI_DIR) && $(CARGO) clippy --all-targets --all-features -- -D warnings

# Run all Rust unit and integration tests
test:
	@echo "--> Running unit & integration tests..."
	cd $(TAURI_DIR) && $(CARGO) test

# Single target to perform both linting and testing
frontend-build:
	@echo "--> Building frontend..."
	npm run build

dataset-check:
	@echo "--> Validating dataset..."
	$(CARGO) run --manifest-path $(TAURI_DIR)/Cargo.toml --bin validate_dataset -- "$(DATASET_PATH)"

check: fmt-check lint test frontend-build dataset-check
	@echo "--> All lints and tests passed successfully!"

# ==============================================================================
# Execution (Dev Mode)
# ==============================================================================

# Launch application in local development mode
run:
	@echo "--> Launching application in dev mode..."
	DATASET_PATH=$(DATASET_PATH) $(TAURI) dev

# Run the configured headless playtest.
playtest:
	@echo "--> Running headless playtest..."
	$(CARGO) run --manifest-path $(TAURI_DIR)/Cargo.toml --bin playtest -- \
		--config "$(PLAYTEST_CONFIG)"

playtest-analysis:
	@echo "--> Analyzing playtest log..."
	node scripts/playtest_analysis.js "$(PLAYTEST_LOG)"

# ==============================================================================
# Compilation & Packaging
# ==============================================================================

# Build Linux release bundle (.AppImage, .deb, .rpm)
build-linux:
	@echo "--> Compiling release bundle for Linux..."
	$(TAURI) build --target x86_64-unknown-linux-gnu
	@echo "--> Moving Linux packages to main directory..."
	@find $(TAURI_DIR)/target/x86_64-unknown-linux-gnu/release/bundle -type f \( -name "*.AppImage" -o -name "*.deb" -o -name "*.rpm" \) -exec cp {} . \; 2>/dev/null || true
	@echo "--> Linux package ready in main folder!"

build-windows:
	@echo "--> Compiling release bundle for Windows..."
	$(TAURI) build --target x86_64-pc-windows-gnu
	@echo "--> Moving Windows executables/installers to main directory..."
	@find $(TAURI_DIR)/target/x86_64-pc-windows-gnu/release -maxdepth 1 -type f -name "*.exe" -exec cp {} . \; 2>/dev/null || true
	@find $(TAURI_DIR)/target/x86_64-pc-windows-gnu/release/bundle -type f \( -name "*.exe" -o -name "*.msi" \) -exec cp {} . \; 2>/dev/null || true
	@echo "--> Windows package ready in main folder!"

# Build Windows release bundle (.exe, .msi)
#build-windows:
#	@echo "--> Compiling release bundle for Windows..."
#	$(TAURI) build --target x86_64-pc-windows-msvc
#	@echo "--> Moving Windows executables/installers to main directory..."
#	@find $(TAURI_DIR)/target/x86_64-pc-windows-msvc/release -maxdepth 1 -type f -name "*.exe" -exec cp {} . \; 2>/dev/null || true
#	@find $(TAURI_DIR)/target/x86_64-pc-windows-msvc/release/bundle -type f \( -name "*.msi" -o -name "*.exe" \) -exec cp {} . \; 2>/dev/null || true
#	@echo "--> Windows package ready in main folder!"

# Build Android package (.apk / .aab)
build-android:
	@echo "--> Compiling build for Android..."
	$(TAURI) android build
	@echo "--> Moving Android APK/AAB outputs to main directory..."
	@find $(TAURI_DIR)/gen/android/app/build/outputs -type f \( -name "*.apk" -o -name "*.aab" \) -exec cp {} . \; 2>/dev/null || true
	@echo "--> Android binary ready in main folder!"

# Build native standalone executable (no packaging/bundling)
build-desktop:
	@echo "--> Compiling standalone executable..."
	$(TAURI) build --no-bundle
	@echo "--> Copying executable to main directory..."
	@cp $(TAURI_DIR)/target/release/ttrpg-engine . 2>/dev/null || cp $(TAURI_DIR)/target/release/ttrpg-engine.exe . 2>/dev/null || true
	@echo "--> Standalone executable ready in main folder: ./ttrpg-engine"

# Alias to build default native desktop binary
build: build-desktop

# Compile Linux, Windows, and Android packages sequentially
build-all: build-linux build-windows build-android

# ==============================================================================
# Cleanup
# ==============================================================================

clean:
	@echo "--> Cleaning frontend artifacts and Rust target build outputs..."
	find src -type f \( -name "*.js" -o -name "*.js.map" -o -name "*.d.ts" -o -name "*.d.ts.map" \) -delete
	rm -rf dist
	cd $(TAURI_DIR) && $(CARGO) clean
	@rm -f ./*.apk ./*.aab ./*.exe ./*.msi ./*.dmg ./*.AppImage ./*.deb ./*.rpm ./*.d.ts.map
