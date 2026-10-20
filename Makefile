# ==============================================================================
# Commands & Tooling
# ==============================================================================
CARGO     ?= cargo
TAURI     ?= cargo tauri
TAURI_DIR ?= src-tauri
WINDOWS_TARGET ?= x86_64-pc-windows-gnu
ANDROID_SDK ?= $(or $(ANDROID_HOME),$(ANDROID_SDK_ROOT))
WINDOWS_BUNDLES ?= none
WINDOWS_PACKAGE_DIR ?= ttrpg-engine-windows
WINDOWS_ARCHIVE ?= ttrpg-engine-windows.zip
WINDOWS_ARCHIVE_SPLIT ?= 50m
WINDOWS_PYTHON_DIR ?=
WINDOWS_WEBVIEW2_VERSION ?= 154.0.4258.62
WINDOWS_WEBVIEW2_CAB_URL ?= https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/b92cd7d9-6976-4f34-9708-47e80937c287/Microsoft.WebView2.FixedVersionRuntime.$(WINDOWS_WEBVIEW2_VERSION).x64.cab
WINDOWS_WEBVIEW2_CACHE ?= .cache/webview2/$(WINDOWS_WEBVIEW2_VERSION)
WINDOWS_WEBVIEW2_DIR ?= src-tauri/webview2-runtime
DATASET_PATH ?= ./gtr2career
RL_MODEL ?= rl/models/ppo-baseline
RL_EPISODES ?= 20
RL_SEED ?= 20000
RL_GOAL ?= championships:level=1,count=1
RL_OUTPUT ?=
RL_TRACE ?= actions
RL_COLLECT_OUTPUT ?= rl/data/random.jsonl
RL_VENV ?= .venv
RL_PYTHON ?= $(RL_VENV)/bin/python
RL_TIMESTEPS ?= 100000
RL_N_STEPS ?= 2048
RL_BATCH_SIZE ?= 64
RL_PROGRESS ?= 0
RL_PROGRESS_INTERVAL ?= 1000
PERF_DATASET ?= $(DATASET_PATH)
PERF_ITERATIONS ?= 10
PERF_WARMUP ?= 2
PERF_DAYS ?= 30

.PHONY: all help check platform-check fmt-check lint test frontend-build editor-check editor-tests dataset-check dataset-capabilities preview-check proof-fixtures run playtest playtest-prepare playtest-train playtest-evaluate playtest-collect perf rl-build rl-smoke build build-desktop build-linux build-windows prepare-windows-webview2 build-android build-all setup-windows setup-android clean

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
	@echo "  make playtest          Run the trained RL player"
	@echo "  make playtest-prepare  Create the local RL environment and dependencies"
	@echo "  make playtest-train    Train a local masked-PPO player"
	@echo "  make playtest-evaluate Evaluate a trained RL player"
	@echo "  make playtest-collect  Collect local RL trajectories"
	@echo "  make perf              Run the standalone performance tester"
	@echo "  make rl-build          Build the local RL environment process"
	@echo "  make rl-smoke          Run one dependency-free RL protocol smoke test"
	@echo "  make build             Build the desktop application"
	@echo "  make build-windows     Build a self-contained Windows GNU staging package"
	@echo "                         (WINDOWS_PYTHON_DIR=... adds an embedded Python runtime)"
	@echo "  make build-android     Build the Android APK/AAB from Linux"
	@echo "  make setup-windows     Install the Windows Rust target"
	@echo "  make setup-android     Generate the Android Tauri project"
	@echo "  make clean             Remove generated build artifacts"
	@echo ""
	@echo "Automatic-play variables:"
	@echo "  RL_MODEL=... RL_EPISODES=20 RL_SEED=20000 RL_GOAL=... RL_TRACE=actions|observations|decisions"
	@echo "  RL_VENV=.venv RL_TIMESTEPS=100000 RL_OUTPUT=..."
	@echo "  RL_N_STEPS=2048 RL_BATCH_SIZE=64"
	@echo "  RL_PROGRESS=1 RL_PROGRESS_INTERVAL=1000"
	@echo "  RL_COLLECT_OUTPUT=rl/data/random.jsonl"

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
	if rust_target "$(WINDOWS_TARGET)" && has_cmd x86_64-w64-mingw32-gcc; then \
		echo "  Windows GNU package: available"; \
	else \
		echo "  Windows GNU package: unavailable (requires $(WINDOWS_TARGET) Rust target and MinGW cross-compiler)"; \
	fi; \
	sdk="$${ANDROID_HOME:-$${ANDROID_SDK_ROOT:-}}"; \
	sdkmanager="$$(command -v sdkmanager 2>/dev/null || printf '%s/cmdline-tools/latest/bin/sdkmanager' "$$sdk")"; \
	if has_cmd cargo-tauri && has_cmd java && has_cmd adb && \
		[ -x "$$sdkmanager" ] && \
		[ -n "$$sdk" ] && [ -d "$$sdk" ] && [ -d "$(TAURI_DIR)/gen/android" ] && \
		[ -d "$$sdk/ndk" ]; then \
		echo "  Android package: available"; \
	else \
		echo "  Android package: unavailable (requires Android SDK/NDK, Java, adb, sdkmanager, cargo-tauri, and generated project)"; \
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

# Run the trained local RL player.
playtest: playtest-prepare
	@test -n "$(RL_MODEL)" || { echo "ERROR: RL_MODEL must point to a trained PPO model." >&2; exit 1; }
	"$(RL_PYTHON)" rl/playtest.py \
		--model "$(RL_MODEL)" \
		--dataset "$(DATASET_PATH)" \
		--episodes "$(RL_EPISODES)" \
		--seed "$(RL_SEED)" \
		--goal "$(RL_GOAL)" \
		--trace "$(RL_TRACE)" \
		$(if $(RL_OUTPUT),--output "$(RL_OUTPUT)")

playtest-prepare: $(RL_VENV)/.rl-prepared rl-build

$(RL_VENV)/.rl-prepared: rl/requirements.txt
	python3 -m venv "$(RL_VENV)"
	"$(RL_PYTHON)" -m pip install --upgrade pip
	"$(RL_PYTHON)" -m pip install -r rl/requirements.txt
	touch "$@"

playtest-train: playtest-prepare
	"$(RL_PYTHON)" rl/train.py \
		--dataset "$(DATASET_PATH)" \
		--goal "$(RL_GOAL)" \
		--timesteps "$(RL_TIMESTEPS)" \
		--n-steps "$(RL_N_STEPS)" \
		--batch-size "$(RL_BATCH_SIZE)" \
		--progress-interval "$(RL_PROGRESS_INTERVAL)" \
		$(if $(filter 1 true yes,$(RL_PROGRESS)),--progress,) \
		--output "$(RL_MODEL)"

playtest-evaluate: playtest-prepare
	@test -n "$(RL_MODEL)" || { echo "ERROR: RL_MODEL must point to a trained PPO model." >&2; exit 1; }
	"$(RL_PYTHON)" rl/evaluate.py \
		--model "$(RL_MODEL)" \
		--dataset "$(DATASET_PATH)" \
		--goal "$(RL_GOAL)" \
		--episodes "$(RL_EPISODES)" \
		--seed "$(RL_SEED)"

playtest-collect: playtest-prepare
	"$(RL_PYTHON)" rl/collect.py \
		--dataset "$(DATASET_PATH)" \
		--episodes "$(RL_EPISODES)" \
		--output "$(if $(RL_OUTPUT),$(RL_OUTPUT),$(RL_COLLECT_OUTPUT))"

perf:
	@echo "--> Running standalone performance tester..."
	$(CARGO) run --manifest-path $(TAURI_DIR)/Cargo.toml --release --bin performance -- \
		--dataset "$(PERF_DATASET)" \
		--iterations "$(PERF_ITERATIONS)" \
		--warmup "$(PERF_WARMUP)" \
		--days "$(PERF_DAYS)"

rl-build:
	@echo "--> Building local RL environment..."
	$(CARGO) build --manifest-path $(TAURI_DIR)/Cargo.toml --release --bin rl_env

rl-smoke: rl-build
	@printf '%s\n' '{"op":"actions"}' '{"op":"observation"}' '{"op":"step","action_id":0}' | \
		$(TAURI_DIR)/target/release/rl_env --dataset "$(DATASET_PATH)" --seed 42

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

setup-windows:
	@echo "--> Installing the Windows Rust target ($(WINDOWS_TARGET))..."
	rustup target add $(WINDOWS_TARGET)

prepare-windows-webview2:
	@echo "--> Preparing pinned WebView2 runtime ($(WINDOWS_WEBVIEW2_VERSION))..."
	@command -v cabextract >/dev/null 2>&1 || { echo "ERROR: cabextract is required to prepare the WebView2 runtime." >&2; exit 1; }
	@mkdir -p "$(WINDOWS_WEBVIEW2_CACHE)" "$(WINDOWS_WEBVIEW2_DIR)"
	@if [ ! -f "$(WINDOWS_WEBVIEW2_CACHE)/runtime.cab" ]; then \
		curl --fail --location --retry 3 --output "$(WINDOWS_WEBVIEW2_CACHE)/runtime.cab" "$(WINDOWS_WEBVIEW2_CAB_URL)"; \
	fi
	@rm -rf "$(WINDOWS_WEBVIEW2_CACHE)/extracted" "$(WINDOWS_WEBVIEW2_DIR)"
	@mkdir -p "$(WINDOWS_WEBVIEW2_CACHE)/extracted" "$(WINDOWS_WEBVIEW2_DIR)"
	@cabextract --quiet --directory "$(WINDOWS_WEBVIEW2_CACHE)/extracted" "$(WINDOWS_WEBVIEW2_CACHE)/runtime.cab"
	@runtime_source="$$(dirname "$$(find "$(WINDOWS_WEBVIEW2_CACHE)/extracted" -type f -name msedgewebview2.exe -print -quit)")"; \
		test -f "$$runtime_source/msedgewebview2.exe" || { echo "ERROR: WebView2 runtime extraction did not produce msedgewebview2.exe." >&2; exit 1; }; \
		cp -R "$$runtime_source"/. "$(WINDOWS_WEBVIEW2_DIR)/"
	@test -f "$(WINDOWS_WEBVIEW2_DIR)/msedgewebview2.exe" || { echo "ERROR: WebView2 runtime extraction did not produce msedgewebview2.exe." >&2; exit 1; }

build-windows: setup-windows prepare-windows-webview2
	@echo "--> Compiling Windows release..."
	@if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then \
		echo "ERROR: x86_64-w64-mingw32-gcc is required for Windows GNU cross-compilation." >&2; \
		echo "Install the MinGW cross-compiler with your system package manager." >&2; \
		exit 1; \
	fi
	$(if $(filter none,$(WINDOWS_BUNDLES)),$(TAURI) build --target $(WINDOWS_TARGET) --no-bundle,$(TAURI) build --target $(WINDOWS_TARGET) --bundles $(WINDOWS_BUNDLES))
	@if [ "$(WINDOWS_BUNDLES)" != "none" ]; then \
		test -d "$(TAURI_DIR)/target/$(WINDOWS_TARGET)/release/bundle" || { echo "ERROR: Windows bundle output was not created." >&2; exit 1; }; \
		echo "--> Windows installer output is under $(TAURI_DIR)/target/$(WINDOWS_TARGET)/release/bundle"; \
	else \
		release_dir="$(TAURI_DIR)/target/$(WINDOWS_TARGET)/release"; \
		test -f "$$release_dir/ttrpg-engine.exe" || { echo "ERROR: Windows executable was not created." >&2; exit 1; }; \
		test -f "$$release_dir/WebView2Loader.dll" || { echo "ERROR: WebView2Loader.dll was not created beside the executable." >&2; exit 1; }; \
		test -d "$(DATASET_PATH)" || { echo "ERROR: dataset directory '$(DATASET_PATH)' was not found." >&2; exit 1; }; \
		rm -rf "$(WINDOWS_PACKAGE_DIR)" "$(WINDOWS_ARCHIVE)" "$(WINDOWS_ARCHIVE:.zip=.z)"*; \
		mkdir -p "$(WINDOWS_PACKAGE_DIR)"; \
		cp "$$release_dir/ttrpg-engine.exe" "$(WINDOWS_PACKAGE_DIR)/"; \
		cp "$$release_dir/WebView2Loader.dll" "$(WINDOWS_PACKAGE_DIR)/"; \
		cp -R "$(DATASET_PATH)" "$(WINDOWS_PACKAGE_DIR)/gtr2career"; \
		mkdir -p "$(WINDOWS_PACKAGE_DIR)/plugins"; \
		cp plugins/plugin_logging.py "$(WINDOWS_PACKAGE_DIR)/plugins/"; \
		if [ -n "$(WINDOWS_PYTHON_DIR)" ]; then \
			test -f "$(WINDOWS_PYTHON_DIR)/python.exe" || { echo "ERROR: WINDOWS_PYTHON_DIR must contain python.exe." >&2; exit 1; }; \
			cp -R "$(WINDOWS_PYTHON_DIR)" "$(WINDOWS_PACKAGE_DIR)/python"; \
		else \
			echo "WARNING: no embedded Python runtime was supplied; plugin actions require Python 3 on PATH."; \
		fi; \
		cp "$(WINDOWS_PACKAGE_DIR)/ttrpg-engine.exe" ./ttrpg-engine.exe; \
		cp -R "$(TAURI_DIR)/target/$(WINDOWS_TARGET)/release/webview2-runtime" "$(WINDOWS_PACKAGE_DIR)/"; \
		command -v zip >/dev/null 2>&1 || { echo "ERROR: zip is required to create split Windows archives." >&2; exit 1; }; \
		zip -q -s "$(WINDOWS_ARCHIVE_SPLIT)" -r "$(WINDOWS_ARCHIVE)" "$(WINDOWS_PACKAGE_DIR)"; \
		rm -rf "$(WINDOWS_PACKAGE_DIR)"; \
		echo "--> Windows split archive ready: $(WINDOWS_ARCHIVE) and matching .zNN volumes (maximum $(WINDOWS_ARCHIVE_SPLIT) each)."; \
	fi

# Build Windows release bundle (.exe, .msi)
#build-windows:
#	@echo "--> Compiling release bundle for Windows..."
#	$(TAURI) build --target x86_64-pc-windows-msvc
#	@echo "--> Moving Windows executables/installers to main directory..."
#	@find $(TAURI_DIR)/target/x86_64-pc-windows-msvc/release -maxdepth 1 -type f -name "*.exe" -exec cp {} . \; 2>/dev/null || true
#	@find $(TAURI_DIR)/target/x86_64-pc-windows-msvc/release/bundle -type f \( -name "*.msi" -o -name "*.exe" \) -exec cp {} . \; 2>/dev/null || true
#	@echo "--> Windows package ready in main folder!"

setup-android:
	@echo "--> Checking Android build prerequisites..."
	@test -n "$(ANDROID_SDK)" || (echo "ERROR: ANDROID_HOME or ANDROID_SDK_ROOT must point to the Android SDK." >&2; exit 1)
	@test -d "$(ANDROID_SDK)/ndk" || (echo "ERROR: Android NDK is missing under $(ANDROID_SDK)/ndk." >&2; exit 1)
	@command -v java >/dev/null 2>&1 || (echo "ERROR: Java is required for Android builds." >&2; exit 1)
	@command -v adb >/dev/null 2>&1 || (echo "ERROR: adb is required for Android builds." >&2; exit 1)
	@command -v sdkmanager >/dev/null 2>&1 || test -x "$(ANDROID_SDK)/cmdline-tools/latest/bin/sdkmanager" || (echo "ERROR: sdkmanager is required for Android builds." >&2; exit 1)
	@if [ ! -d "$(TAURI_DIR)/gen/android" ]; then \
		echo "--> Generating the Tauri Android project..."; \
		$(TAURI) android init; \
	fi

# Build Android package (.apk / .aab)
build-android: setup-android
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
