#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

args=(
  --build
  --seed "${SEED:-1}"
  --goal "${GOAL:-all-championships}"
  --max-days "${MAX_DAYS:-3650}"
  --max-steps "${MAX_STEPS:-10000}"
  --fast-forward-days "${FAST_FORWARD_DAYS:-20}"
  --output "${OUTPUT:-playthrough/automatic-gameplay-seed1.json}"
)

if [[ -n "${CHECKPOINT:-}" ]]; then
  args+=(--checkpoint "$CHECKPOINT")
fi
if [[ -n "${CHECKPOINT_EVERY:-}" ]]; then
  args+=(--checkpoint-every "$CHECKPOINT_EVERY")
fi
if [[ -n "${RESUME:-}" ]]; then
  args+=(--resume "$RESUME")
fi

python3 playthrough/heuristic_play.py "${args[@]}"
