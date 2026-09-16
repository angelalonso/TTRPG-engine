#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ $# -eq 0 ]]; then
  set -- \
    --dataset "${PLAYTEST_DATASET:-dataset}" \
    --max-days "${PLAYTEST_MAX_DAYS:-365}" \
    --runs "${PLAYTEST_RUNS:-1}" \
    --seed "${PLAYTEST_SEED:-}" \
    --strategy "${PLAYTEST_STRATEGY:-greedy}" \
    --goal "${PLAYTEST_GOAL:-charisma>=100}" \
    --verbosity "${PLAYTEST_VERBOSITY:-summary}"
fi

exec cargo run --manifest-path src-tauri/Cargo.toml --bin playtest -- "$@"
