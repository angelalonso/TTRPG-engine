"""Evaluate a saved masked PPO policy on independent seeds."""

from __future__ import annotations

import argparse
import statistics

from gtr2career_env import Gtr2CareerEnv, RustEnvConfig


def main() -> None:
    try:
        from sb3_contrib import MaskablePPO
    except ImportError as error:
        raise SystemExit(
            "Install rl/requirements.txt before evaluating the PPO policy"
        ) from error

    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--episodes", type=int, default=20)
    parser.add_argument("--seed", type=int, default=10_000)
    parser.add_argument("--dataset", default="gtr2career")
    parser.add_argument("--goal", default="charisma>=100")
    parser.add_argument("--binary", default="src-tauri/target/release/rl_env")
    args = parser.parse_args()

    env = Gtr2CareerEnv(
        RustEnvConfig(
            dataset=args.dataset,
            seed=args.seed,
            goal=args.goal,
            binary=args.binary,
        )
    )
    model = MaskablePPO.load(args.model, env=env, device="cpu")
    outcomes: list[str] = []
    days: list[int] = []
    for offset in range(args.episodes):
        observation, _ = env.reset(seed=args.seed + offset)
        done = False
        info = {}
        current_day = 1
        while not done:
            action, _ = model.predict(
                observation,
                action_masks=env.action_masks(),
                deterministic=True,
            )
            observation, _, terminated, truncated, info = env.step(int(action))
            current_day = int(observation["numeric"][-4])
            done = terminated or truncated
        outcomes.append(info.get("status", "unknown"))
        days.append(max(0, current_day - 1))
    print(
        {
            "episodes": len(outcomes),
            "outcomes": {outcome: outcomes.count(outcome) for outcome in sorted(set(outcomes))},
            "mean_days": statistics.mean(days) if days else 0,
        }
    )
    env.close()


if __name__ == "__main__":
    main()
