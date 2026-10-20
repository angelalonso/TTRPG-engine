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

import numpy as np

from gtr2career_env import Gtr2CareerEnv, RustEnvConfig


TRACE_LEVELS = ("actions", "observations", "decisions")


def policy_decision_trace(
    model: Any,
    observation: dict[str, np.ndarray],
    action_mask: np.ndarray,
    action_definitions: tuple[dict[str, Any], ...],
    selected_action: int,
    deterministic: bool,
) -> dict[str, Any]:
    observation_tensor, _ = model.policy.obs_to_tensor(observation)
    distribution = model.policy.get_distribution(
        observation_tensor,
        action_masks=action_mask,
    )
    probabilities = (
        distribution.distribution.probs.detach().cpu().numpy()[0].tolist()
    )
    legal_actions = [
        {
            "action_id": action_id,
            "action_key": action_definitions[action_id]["key"],
            "probability": probabilities[action_id],
        }
        for action_id, allowed in enumerate(action_mask)
        if allowed
    ]
    legal_actions.sort(key=lambda item: item["probability"], reverse=True)
    return {
        "selection": "deterministic" if deterministic else "sampled",
        "selected_probability": probabilities[selected_action],
        "legal_actions": legal_actions,
    }


def run_episode(
    env: Gtr2CareerEnv,
    model: Any,
    seed: int,
    deterministic: bool,
    trace_level: str,
) -> dict[str, Any]:
    observation, _ = env.reset(seed=seed)
    path: list[dict[str, Any]] = []
    terminated = False
    truncated = False
    info: dict[str, Any] = {}

    while not terminated and not truncated:
        action_mask = env.action_masks()
        model_observation = observation
        observation_before = (
            env.debug_observation()
            if trace_level in ("observations", "decisions")
            else None
        )
        action, _ = model.predict(
            observation,
            action_masks=action_mask,
            deterministic=deterministic,
        )
        action_id = int(action)
        action_definition = env.action_definitions[action_id]
        day_before = int(observation["numeric"][-4])
        observation, reward, terminated, truncated, info = env.step(action_id)
        event: dict[str, Any] = {
            "step": len(path) + 1,
            "day": day_before,
            "action_id": action_id,
            "action_key": action_definition["key"],
            "reward": reward,
            "days_elapsed": info.get("days_elapsed", 0),
            "status": info.get("status", "ongoing"),
        }
        if trace_level in ("observations", "decisions"):
            event["observation_before"] = observation_before
            event["observation_after"] = env.debug_observation()
        if trace_level == "decisions":
            event["decision"] = policy_decision_trace(
                model,
                model_observation,
                action_mask,
                env.action_definitions,
                action_id,
                deterministic,
            )
        path.append(event)

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
    parser.add_argument(
        "--trace",
        choices=TRACE_LEVELS,
        default="actions",
        help=(
            "Export detail level: actions records the ordered choices; "
            "observations adds state before/after each choice; decisions "
            "also adds policy probabilities for legal alternatives."
        ),
    )
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
                trace_level=args.trace,
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
