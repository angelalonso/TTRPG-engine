# Local RL automation

The Rust `rl` module and `rl_env` binary provide the authoritative
reinforcement-learning environment. They use the same headless engine as the
application, including seeded randomness,
requirements, schedules, transactions, encounters, and goal checks.

## Current contract

- `GoalSpec` supports characteristic goals, one championship, and trophy
  count/level goals.
- `ActionDefinition` uses stable dataset IDs and a versioned catalogue.
- Every observation includes a legal-action mask.
- Invalid actions are reported in `StepInfo`; they are not silently converted
  to waiting.
- `RlEnvironment` can be cloned through its underlying headless state for
  future beam-search/MCTS work.
- The JSON-lines `rl_env` process is a dependency-free local bridge.
- `rl/gtr2career_env.py` adapts the bridge to Gymnasium.
- `rl/train.py` and `rl/evaluate.py` provide the optional masked-PPO path.

The Python trainer is intentionally optional. The engine and protocol must
remain usable without Python, Gymnasium, PyTorch, or Stable-Baselines3.

## Automatic-play status

The Makefile exposes the RL workflow for automatic play:
`playtest-prepare`, `playtest-train`, `playtest-evaluate`,
`playtest-collect`, and `playtest`. The former heuristic runner, JSON profiles,
shell wrapper, and log analyzer have been removed. `rl_env` and the Python
runner are now the only supported automatic-play path.

## Compatibility

The RL work does not change save fields, save versions, dataset revisions, or
the meaning of existing persisted values. Training observations and action
catalogues are versioned separately from save compatibility. Any future
change to those schemas must update the version and add a replay fixture.
