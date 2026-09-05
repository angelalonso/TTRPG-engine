# ==============================================================================
# Commands & Tooling
# ==============================================================================
CARGO     ?= cargo
TAURI     ?= cargo tauri
TAURI_DIR ?= src-tauri

.PHONY: all help check lint test run build build-desktop build-android build-all clean

# Default target
all: check

# ==============================================================================
# Quality Assurance (Lint & Test)
# ==============================================================================

# Run Clippy linter
lint:
	@echo "--> Running Clippy linter..."
	cd $(TAURI_DIR) && $(CARGO) clippy --all-targets --all-features -- -D warnings

# Run all Rust unit and integration tests
test:
	@echo "--> Running unit & integration tests..."
	cd $(TAURI_DIR) && $(CARGO) test

# Single target to perform both linting and testing
check: lint test
	@echo "--> ✨ All lints and tests passed successfully!"

# ==============================================================================
# Execution (Dev Mode)
# ==============================================================================

# Launch application in local development mode
run:
	@echo "--> Launching application in dev mode..."
	$(TAURI) dev

# ==============================================================================
# Compilation & Packaging
# ==============================================================================

# Build desktop binary and extract executables/installers to the root directory
build-desktop:
	@echo "--> Compiling release bundle for Desktop..."
	$(TAURI) build
	@echo "--> Moving desktop binaries to main directory..."
	@find $(TAURI_DIR)/target/release -maxdepth 1 -type f \( -executable -o -name "*.exe" \) -exec cp {} . \; 2>/dev/null || true
	@find $(TAURI_DIR)/target/release/bundle -type f \( -name "*.msi" -o -name "*.exe" -o -name "*.dmg" -o -name "*.AppImage" -o -name "*.deb" \) -exec cp {} . \; 2>/dev/null || true
	@echo "--> 📦 Desktop executable/bundle ready in main folder!"

# Build Android package (.apk / .aab) and extract to the root directory
build-android:
	@echo "--> Compiling build for Android..."
	$(TAURI) android build
	@echo "--> Moving Android APK/AAB outputs to main directory..."
	@find $(TAURI_DIR)/gen/android/app/build/outputs -type f \( -name "*.apk" -o -name "*.aab" \) -exec cp {} . \; 2>/dev/null || true
	@echo "--> 📦 Android binary ready in main folder!"

# Alias to build desktop target by default
build: build-desktop

# Compile both Desktop and Android packages sequentially
build-all: build-desktop build-android

# ==============================================================================
# Cleanup
# ==============================================================================

clean:
	@echo "--> Cleaning Rust target build outputs and root executables..."
	cd $(TAURI_DIR) && $(CARGO) clean
	@rm -f ./*.apk ./*.aab ./*.exe ./*.msi ./*.dmg ./*.AppImage ./*.deb
