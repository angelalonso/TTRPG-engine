"""Run a trained masked-PPO policy as a local automatic player.

The Rust environment remains authoritative for transitions and legality. This
runner only selects actions, records the resulting path, and summarizes runs.
"""

from __future__ import annotations

import argparse
import json
import statistics
from pathlib import Path
from typing import Any

from gtr2career_env import Gtr2CareerEnv, RustEnvConfig


def run_episode(
    env: Gtr2CareerEnv,
    model: Any,
    seed: int,
    deterministic: bool,
) -> dict[str, Any]:
    observation, _ = env.reset(seed=seed)
    path: list[dict[str, Any]] = []
    terminated = False
    truncated = False
    info: dict[str, Any] = {}

    while not terminated and not truncated:
        action, _ = model.predict(
            observation,
            action_masks=env.action_masks(),
            deterministic=deterministic,
        )
        action_id = int(action)
        action_definition = env.action_definitions[action_id]
        observation, reward, terminated, truncated, info = env.step(action_id)
        path.append(
            {
                "action_id": action_id,
                "action_key": action_definition["key"],
                "reward": reward,
                "days_elapsed": info.get("days_elapsed", 0),
                "status": info.get("status", "ongoing"),
            }
        )

    return {
        "seed": seed,
        "status": info.get("status", "unknown"),
        "terminated": terminated,
        "truncated": truncated,
        "current_day": int(observation["numeric"][-4]),
        "budget": float(info.get("budget", 0.0)),
        "stamina": float(info.get("stamina", 0.0)),
        "steps": len(path),
        "path": path,
    }


def main() -> None:
    try:
        from sb3_contrib import MaskablePPO
    except ImportError as error:
        raise SystemExit(
            "Install rl/requirements.txt before running the trained policy"
        ) from error

    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--episodes", type=int, default=20)
    parser.add_argument("--seed", type=int, default=20_000)
    parser.add_argument("--dataset", default="gtr2career")
    parser.add_argument("--goal", default="charisma>=100")
    parser.add_argument("--max-days", type=int, default=7300)
    parser.add_argument("--max-steps", type=int, default=14600)
    parser.add_argument("--binary", default="src-tauri/target/release/rl_env")
    parser.add_argument("--stochastic", action="store_true")
    parser.add_argument("--output")
    args = parser.parse_args()

    env = Gtr2CareerEnv(
        RustEnvConfig(
            dataset=args.dataset,
            seed=args.seed,
            goal=args.goal,
            max_days=args.max_days,
            max_steps=args.max_steps,
            binary=args.binary,
        )
    )
    try:
        model = MaskablePPO.load(args.model, env=env, device="cpu")
        episodes = [
            run_episode(
                env,
                model,
                args.seed + offset,
                deterministic=not args.stochastic,
            )
            for offset in range(max(1, args.episodes))
        ]
    finally:
        env.close()

    statuses = [episode["status"] for episode in episodes]
    summary = {
        "episodes": len(episodes),
        "goal_reached": statuses.count("goal_reached"),
        "statuses": {status: statuses.count(status) for status in sorted(set(statuses))},
        "mean_days": statistics.mean(
            episode["current_day"] - 1 for episode in episodes
        ),
        "mean_steps": statistics.mean(episode["steps"] for episode in episodes),
    }
    result = {"summary": summary, "episodes": episodes}
    if args.output:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    main()
