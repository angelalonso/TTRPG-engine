# ==============================================================================
# Commands & Tooling
# ==============================================================================
CARGO     ?= cargo
TAURI     ?= cargo tauri
TAURI_DIR ?= src-tauri
DATASET_PATH ?= ./dataset
PLAYTEST_CONFIG ?= playtest.json
PLAYTEST_RUNS ?=
PLAYTEST_LOG ?= playtest.out
PERF_DATASET ?= $(DATASET_PATH)
PERF_ITERATIONS ?= 10
PERF_WARMUP ?= 2
PERF_DAYS ?= 30

.PHONY: all help check platform-check fmt-check lint test frontend-build editor-check editor-tests dataset-check dataset-capabilities preview-check proof-fixtures run playtest playtest-deterministic playtest-top-k playtest-diverse playtest-required playtest-analysis perf build build-desktop build-linux build-windows build-android build-all clean

# Default target
all: check

# Show available development and playtest commands.
help:
	@echo "Available targets:"
	@echo "  make check             Run linting and tests"
	@echo "  make platform-check    Report available desktop/cross-build prerequisites"
	@echo "  make fmt-check         Check Rust formatting"
	@echo "  make frontend-build    Type-check and build the frontend"
	@echo "  make editor-check      Compile Python dataset editors"
	@echo "  make editor-tests      Run lossless dataset-editor regression tests"
	@echo "  make dataset-check     Validate dataset references and assets"
	@echo "  make dataset-capabilities  Print engine-owned authoring metadata"
	@echo "  make preview-check     Check deterministic preview JSON and staged replay"
	@echo "  make proof-fixtures    Validate the non-racing proof datasets"
	@echo "  make run               Launch the Tauri application"
	@echo "  make playtest          Run the playtest configured in the JSON file"
	@echo "  make playtest-deterministic  Run the reproducible legacy baseline"
	@echo "  make playtest-top-k    Run seeded top-K action selection"
	@echo "  make playtest-diverse  Run batch-diverse action selection"
	@echo "  make playtest-required Run the required-actions baseline"
	@echo "  make playtest-analysis Analyze a playtest .out log"
	@echo "  make perf              Run the standalone performance tester"
	@echo "  make build             Build the desktop application"
	@echo "  make clean             Remove generated build artifacts"
	@echo ""
	@echo "Playtest variables:"
	@echo "  PLAYTEST_CONFIG=playtest.json  Playtest configuration file"
	@echo "  PLAYTEST_RUNS=N               Override the configured number of runs"
	@echo "  PERF_ITERATIONS=10 PERF_WARMUP=2 PERF_DAYS=30"

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

editor-check:
	@echo "--> Checking Python dataset editors..."
	python3 -m py_compile dataset_editor.py dataset_generic.py

editor-tests:
	@echo "--> Running dataset-editor regression tests..."
	python3 -m unittest discover -s tests -v

dataset-check:
	@echo "--> Validating dataset..."
	$(CARGO) run --manifest-path $(TAURI_DIR)/Cargo.toml --bin validate_dataset -- "$(DATASET_PATH)"

dataset-capabilities:
	@echo "--> Printing engine authoring capabilities..."
	$(CARGO) run --quiet --manifest-path $(TAURI_DIR)/Cargo.toml --bin validate_dataset -- --capabilities

preview-check:
	@echo "--> Checking deterministic dataset-preview JSON..."
	python3 scripts/check_preview_json.py

proof-fixtures:
	@echo "--> Validating non-racing proof fixtures..."
	$(CARGO) run --quiet --manifest-path $(TAURI_DIR)/Cargo.toml --bin validate_dataset -- tests/fixtures/pony_stable
	$(CARGO) run --quiet --manifest-path $(TAURI_DIR)/Cargo.toml --bin validate_dataset -- tests/fixtures/cooking

platform-check:
	@echo "--> Checking platform build capabilities (informational; unavailable optional targets do not fail)..."
	@host="$$(rustc -vV 2>/dev/null | sed -n 's/^host: //p')"; \
	rust_target() { rustup target list --installed 2>/dev/null | grep -qx "$$1"; }; \
	has_cmd() { command -v "$$1" >/dev/null 2>&1; }; \
	if [ -n "$$host" ] && rust_target "$$host" && has_cmd cargo-tauri; then \
		echo "  Linux desktop/package: available ($$host, cargo-tauri)"; \
	else \
		echo "  Linux desktop/package: unavailable (requires host Rust target and cargo-tauri)"; \
	fi; \
	if rust_target x86_64-pc-windows-gnu && has_cmd x86_64-w64-mingw32-gcc; then \
		echo "  Windows GNU package: available"; \
	else \
		echo "  Windows GNU package: unavailable (requires Rust target and MinGW cross-compiler)"; \
	fi; \
	sdk="$${ANDROID_HOME:-$${ANDROID_SDK_ROOT:-}}"; \
	if has_cmd cargo-tauri && has_cmd java && has_cmd adb && has_cmd sdkmanager && \
		[ -n "$$sdk" ] && [ -d "$$sdk" ] && [ -d "$(TAURI_DIR)/gen/android" ] && \
		[ -d "$$sdk/ndk" ]; then \
		echo "  Android package: available"; \
	else \
		echo "  Android package: unavailable (requires Android SDK/NDK, Java, cargo-tauri, and generated project)"; \
	fi

check: platform-check fmt-check lint test frontend-build editor-check editor-tests dataset-check preview-check proof-fixtures
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
		--config "$(PLAYTEST_CONFIG)" \
		$(if $(PLAYTEST_RUNS),--runs "$(PLAYTEST_RUNS)")

playtest-deterministic:
	$(MAKE) playtest PLAYTEST_CONFIG=playtest.deterministic.json

playtest-top-k:
	$(MAKE) playtest PLAYTEST_CONFIG=playtest.top-k.json

playtest-diverse:
	$(MAKE) playtest PLAYTEST_CONFIG=playtest.diverse.json

playtest-required:
	$(MAKE) playtest PLAYTEST_CONFIG=playtest.required.json

playtest-analysis:
	@echo "--> Analyzing playtest log..."
	node scripts/playtest_analysis.js "$(PLAYTEST_LOG)"

perf:
	@echo "--> Running standalone performance tester..."
	$(CARGO) run --manifest-path $(TAURI_DIR)/Cargo.toml --release --bin performance -- \
		--dataset "$(PERF_DATASET)" \
		--iterations "$(PERF_ITERATIONS)" \
		--warmup "$(PERF_WARMUP)" \
		--days "$(PERF_DAYS)"

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
