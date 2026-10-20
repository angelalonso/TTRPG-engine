"""Collect masked random trajectories for imitation/debugging experiments."""

from __future__ import annotations

import argparse
import json
import random
from pathlib import Path

from gtr2career_env import Gtr2CareerEnv, RustEnvConfig


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--episodes", type=int, default=10)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--output", default="rl/data/random.jsonl")
    parser.add_argument("--dataset", default="gtr2career")
    parser.add_argument("--goal", default="charisma>=100")
    parser.add_argument("--binary", default="src-tauri/target/release/rl_env")
    args = parser.parse_args()

    environment = Gtr2CareerEnv(
        RustEnvConfig(
            dataset=args.dataset,
            seed=args.seed,
            goal=args.goal,
            binary=args.binary,
        )
    )
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    rng = random.Random(args.seed)
    with output.open("w", encoding="utf-8") as stream:
        for episode in range(args.episodes):
            observation, info = environment.reset(seed=args.seed + episode)
            done = False
            while not done:
                legal = [
                    index
                    for index, allowed in enumerate(observation["action_mask"])
                    if allowed
                ]
                if not legal:
                    break
                action = rng.choice(legal)
                next_observation, reward, terminated, truncated, step_info = (
                    environment.step(action)
                )
                stream.write(
                    json.dumps(
                        {
                            "episode": episode,
                            "seed": info["seed"],
                            "observation": {
                                key: value.tolist()
                                for key, value in observation.items()
                            },
                            "action": action,
                            "reward": reward,
                            "next_observation": {
                                key: value.tolist()
                                for key, value in next_observation.items()
                            },
                            "terminated": terminated,
                            "truncated": truncated,
                            "info": step_info,
                        }
                    )
                    + "\n"
                )
                observation = next_observation
                done = terminated or truncated
    environment.close()


if __name__ == "__main__":
    main()
