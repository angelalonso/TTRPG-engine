# ==============================================================================
# Commands & Tooling
# ==============================================================================
CARGO     ?= cargo
TAURI     ?= cargo tauri
TAURI_DIR ?= src-tauri

.PHONY: all help check lint test run build build-desktop build-linux build-windows build-android build-all clean

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
	@echo "--> All lints and tests passed successfully!"

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

# Build Linux release bundle (.AppImage, .deb, .rpm)
build-linux:
	@echo "--> Compiling release bundle for Linux..."
	$(TAURI) build --target x86_64-unknown-linux-gnu
	@echo "--> Moving Linux packages to main directory..."
	@find $(TAURI_DIR)/target/x86_64-unknown-linux-gnu/release/bundle -type f \( -name "*.AppImage" -o -name "*.deb" -o -name "*.rpm" \) -exec cp {} . \; 2>/dev/null || true
	@echo "--> Linux package ready in main folder!"

# Build Windows release bundle (.exe, .msi)
build-windows:
	@echo "--> Compiling release bundle for Windows..."
	$(TAURI) build --target x86_64-pc-windows-msvc
	@echo "--> Moving Windows executables/installers to main directory..."
	@find $(TAURI_DIR)/target/x86_64-pc-windows-msvc/release -maxdepth 1 -type f -name "*.exe" -exec cp {} . \; 2>/dev/null || true
	@find $(TAURI_DIR)/target/x86_64-pc-windows-msvc/release/bundle -type f \( -name "*.msi" -o -name "*.exe" \) -exec cp {} . \; 2>/dev/null || true
	@echo "--> Windows package ready in main folder!"

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
	@cp $(TAURI_DIR)/target/release/gtr2-racewars . 2>/dev/null || cp $(TAURI_DIR)/target/release/gtr2-racewars.exe . 2>/dev/null || true
	@echo "--> Standalone executable ready in main folder: ./gtr2-racewars"

# Alias to build default native desktop binary
build: build-desktop

# Compile Linux, Windows, and Android packages sequentially
build-all: build-linux build-windows build-android

# ==============================================================================
# Cleanup
# ==============================================================================

clean:
	@echo "--> Cleaning frontend artifacts, Rust target build outputs, and root executables..."
	find src -type f \( -name "*.js" -o -name "*.js.map" -o -name "*.d.ts" \) -delete
	rm -rf dist
	cd $(TAURI_DIR) && $(CARGO) clean
	@rm -f ./*.apk ./*.aab ./*.exe ./*.msi ./*.dmg ./*.AppImage ./*.deb ./*.rpm ./gtr2-racewars
