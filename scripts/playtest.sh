#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ $# -eq 0 ]]; then
  set -- \
    --dataset "${PLAYTEST_DATASET:-dataset}" \
    --max-days "${PLAYTEST_MAX_DAYS:-365}" \
    --beam-width "${PLAYTEST_BEAM_WIDTH:-64}" \
    --object-limit "${PLAYTEST_OBJECT_LIMIT:-12}" \
    --plans "${PLAYTEST_PLANS:-5}" \
    --goal "${PLAYTEST_GOAL:-budget}" \
    --target "${PLAYTEST_TARGET:-100000}"
fi

exec cargo run --manifest-path src-tauri/Cargo.toml --bin playtest -- "$@"
