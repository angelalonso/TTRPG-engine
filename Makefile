# ==============================================================================
# Commands & Tooling
# ==============================================================================
CARGO     ?= cargo
TAURI     ?= cargo tauri
TAURI_DIR ?= src-tauri
DATASET_PATH ?= ./dataset
PLAYTEST_DATASET ?= $(DATASET_PATH)
PLAYTEST_SEED ?= 42
PLAYTEST_RUNS ?= 1
PLAYTEST_MAX_DAYS ?= 7300
PLAYTEST_MAX_TURNS ?= 0
PLAYTEST_STRATEGY ?= greedy
PLAYTEST_GOAL ?= charisma>=100
PLAYTEST_VERBOSITY ?= summary
PLAYTEST_SPEED ?= max
PLAYTEST_PACE_MS ?= 250
PLAYTEST_TOO_EASY_BELOW_DAYS ?= 730
PLAYTEST_HARD_ABOVE_DAYS ?= 5475
PLAYTEST_NEAR_IMPOSSIBLE_ABOVE_DAYS ?= 7300
PLAYTEST_OUTPUT ?= playtest-results.json

.PHONY: all help check lint test run playtest playtest-trace playtest-batch build build-desktop build-linux build-windows build-android build-all clean

# Default target
all: check

# Show available development and playtest commands.
help:
	@echo "Available targets:"
	@echo "  make check             Run linting and tests"
	@echo "  make run               Launch the Tauri application"
	@echo "  make playtest          Run one headless seeded playtest"
	@echo "  make playtest-trace    Run one paced playtest with trace logging"
	@echo "  make playtest-batch    Run a batch and write per-run JSON results"
	@echo "  make build             Build the desktop application"
	@echo "  make clean             Remove generated build artifacts"
	@echo ""
	@echo "Playtest variables:"
	@echo "  PLAYTEST_DATASET=./dataset   Dataset directory"
	@echo "  PLAYTEST_SEED=42              Starting seed"
	@echo "  PLAYTEST_RUNS=1               Number of runs for 'playtest'"
	@echo "  PLAYTEST_BATCH_RUNS=1000      Number of runs for 'playtest-batch'"
	@echo "  PLAYTEST_MAX_DAYS=7300        Per-run day limit"
	@echo "  PLAYTEST_MAX_TURNS=0          Turn limit; 0 uses the default"
	@echo "  PLAYTEST_STRATEGY=greedy      random, greedy, or required-only"
	@echo "  PLAYTEST_GOAL='charisma>=100' Goal condition"
	@echo "  PLAYTEST_VERBOSITY=summary    summary, run, or trace"
	@echo "  PLAYTEST_TOO_EASY_BELOW_DAYS=730   Too-easy p50 threshold"
	@echo "  PLAYTEST_HARD_ABOVE_DAYS=5475       Hard p50 threshold"
	@echo "  PLAYTEST_NEAR_IMPOSSIBLE_ABOVE_DAYS=7300  Near-impossible p90 threshold"
	@echo "  PLAYTEST_OUTPUT=playtest-results.json  Batch JSON output path"

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
	DATASET_PATH=$(DATASET_PATH) $(TAURI) dev

# Run one reproducible headless playtest.
playtest:
	@echo "--> Running headless playtest..."
	$(CARGO) run --manifest-path $(TAURI_DIR)/Cargo.toml --bin playtest -- \
		--dataset "$(PLAYTEST_DATASET)" \
		--seed "$(PLAYTEST_SEED)" \
		--runs "$(PLAYTEST_RUNS)" \
		--max-days "$(PLAYTEST_MAX_DAYS)" \
		--max-turns "$(PLAYTEST_MAX_TURNS)" \
		--strategy "$(PLAYTEST_STRATEGY)" \
		--goal "$(PLAYTEST_GOAL)" \
		--verbosity "$(PLAYTEST_VERBOSITY)" \
		--speed "$(PLAYTEST_SPEED)" \
		--pace-ms "$(PLAYTEST_PACE_MS)" \
		--too-easy-below-days "$(PLAYTEST_TOO_EASY_BELOW_DAYS)" \
		--hard-above-days "$(PLAYTEST_HARD_ABOVE_DAYS)" \
		--near-impossible-above-days "$(PLAYTEST_NEAR_IMPOSSIBLE_ABOVE_DAYS)"

# Run one paced trace for visually inspecting a seeded run.
playtest-trace:
	@echo "--> Running paced playtest trace..."
	$(MAKE) playtest \
		PLAYTEST_VERBOSITY=trace \
		PLAYTEST_RUNS=1 \
		PLAYTEST_MAX_TURNS=0 \
		PLAYTEST_SPEED=paced

# Run a larger reproducible batch and save per-run JSON results.
playtest-batch:
	@echo "--> Running playtest batch..."
	$(CARGO) run --manifest-path $(TAURI_DIR)/Cargo.toml --bin playtest -- \
		--dataset "$(PLAYTEST_DATASET)" \
		--seed "$(PLAYTEST_SEED)" \
		--runs "$${PLAYTEST_BATCH_RUNS:-1000}" \
		--max-days "$(PLAYTEST_MAX_DAYS)" \
		--max-turns "$(PLAYTEST_MAX_TURNS)" \
		--strategy "$(PLAYTEST_STRATEGY)" \
		--goal "$(PLAYTEST_GOAL)" \
		--verbosity "$(PLAYTEST_VERBOSITY)" \
		--too-easy-below-days "$(PLAYTEST_TOO_EASY_BELOW_DAYS)" \
		--hard-above-days "$(PLAYTEST_HARD_ABOVE_DAYS)" \
		--near-impossible-above-days "$(PLAYTEST_NEAR_IMPOSSIBLE_ABOVE_DAYS)" \
		--output "$(PLAYTEST_OUTPUT)"

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
