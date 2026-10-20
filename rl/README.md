# Local reinforcement learning

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
python3 rl/train.py --timesteps 100000 --output rl/models/ppo-baseline
python3 rl/evaluate.py --model rl/models/ppo-baseline --episodes 20
python3 rl/playtest.py --model rl/models/ppo-baseline --episodes 20 \
  --goal "championships:level=1,count=1" \
  --output rl/data/ppo-playtest.json
python3 rl/collect.py --episodes 20 --output rl/data/random.jsonl
```

The current trainer uses CPU-only masked PPO from `sb3-contrib`. It is a
baseline environment/trainer. `playtest.py` runs a saved policy as an
automatic player, records its action paths, and reports episode outcomes. It
is not yet the final search-assisted policy or desktop UI integration.
Training and evaluation seeds are explicit so results can be reproduced.
