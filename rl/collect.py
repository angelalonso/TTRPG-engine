"""Collect masked random trajectories for imitation/debugging experiments."""

from __future__ import annotations

import argparse
import json
import random
from contextlib import closing
from pathlib import Path

from gtr2career_env import Gtr2CareerEnv
from workflow import add_environment_arguments, environment_config, positive_int


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--episodes", type=positive_int, default=10)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--output", default="rl/data/random.jsonl")
    add_environment_arguments(parser)
    args = parser.parse_args()

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    environment = Gtr2CareerEnv(environment_config(args))
    rng = random.Random(args.seed)
    with closing(environment), output.open("w", encoding="utf-8") as stream:
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
                    raise RuntimeError("No legal actions in an active episode; inspect the engine mask")
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


if __name__ == "__main__":
    main()
