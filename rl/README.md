# Local reinforcement learning

**Start with the [beginner playtest guide](../docs/000_rl.md)** for manual
play, reward tuning, fast-route comparisons and symptom-by-symptom
troubleshooting. `make playtest-manual` needs no trained model;
`make playtest-random` provides an untrained baseline.
`make playtest-gui` opens the optional Python/Tk alternative with explained
phase checkboxes, settings, reward overrides, legal-action manual play and
live logs. It launches the same CLI tools and preserves all Make workflows.
See the guide for setup, cancellation and output-file details.

This directory contains the optional Python training layer. The Rust engine
remains authoritative for state transitions, legality, seeded randomness, and
goal status.

The first bridge is a persistent JSON-lines process. It avoids one process
startup per environment step and works without changing the Tauri application.
For high-volume training, it can later be replaced by a native PyO3 bridge
without changing the Gymnasium-facing API.

## Setup

```sh
python3 -m venv .venv
. .venv/bin/activate
pip install -r rl/requirements.txt
```

Build the environment process:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --release --bin rl_env
```

The Gymnasium adapter and PPO trainer are intentionally optional until these
dependencies are installed.

## Smoke test

```sh
printf '%s\n' '{"op":"actions"}' '{"op":"observation"}' '{"op":"step","action_id":0}' \
  | src-tauri/target/release/rl_env --dataset gtr2career --seed 42
```

## Train and evaluate

After installing the optional dependencies:

```sh
python3 rl/train.py --goal "championships:level=1,count=1" \
  --timesteps 100000 --output rl/models/ppo-baseline
python3 rl/evaluate.py --model rl/models/ppo-baseline --episodes 20 \
  --goal "championships:level=1,count=1"
python3 rl/playtest.py --model rl/models/ppo-baseline --episodes 20 \
  --goal "championships:level=1,count=1" \
  --output rl/data/ppo-playtest.json
python3 rl/collect.py --episodes 20 --output rl/data/random.jsonl
```

The current trainer uses CPU-only masked PPO from `sb3-contrib`. Training
prints progress plus human-readable success rate and mean game days. Playtest
prints one result per episode; add `--verbose` for every action, budget, and
stamina update, or `--quiet` for JSON-only output. The playtest simulator makes
work and sponsor-start interactions deterministic while retaining sickness,
encounter, and other unrelated randomness.

The warning about a GPU whose compute capability is unsupported by the
installed PyTorch CUDA wheel is not a game error. These scripts select CPU and
hide CUDA devices by default, so rebuild the Rust bridge and rerun the command
rather than changing game data. Training and evaluation seeds are explicit so
results can be reproduced.

All runners accept `--max-days`, `--max-steps`, and `--reward-config` JSON
overrides. The opt-in `rewards.example.json` illustrates gentle safety
penalties and a per-action cost; it is not the default reward configuration.
Exports explain individual reward components and warn about possible
stagnation without claiming a proven dead end. Compare success rate first,
then **successful-run** days, not the average duration of failures.

Training saves periodic checkpoints and an interrupted checkpoint on
Ctrl+C/SIGTERM when possible. `--resume` continues learning. Keep each
model's `.zip.json` metadata beside it so goal, dataset and action-order
mismatches can be rejected before play.
