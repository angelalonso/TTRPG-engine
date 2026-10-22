"""Run a trained masked-PPO policy as a local automatic player.

The Rust environment remains authoritative for transitions and legality. This
runner only selects actions, records the resulting path, and summarizes runs.
"""

from __future__ import annotations

import argparse
import json
import os
import statistics
from pathlib import Path
from typing import Any

import numpy as np

from gtr2career_env import Gtr2CareerEnv
from workflow import add_environment_arguments, check_model_contract, environment_config, positive_int
from translate_playtest import action_name


TRACE_LEVELS = ("actions", "observations", "decisions")


def manual_action(env: Gtr2CareerEnv, mask: np.ndarray) -> int | None:
    legal = [item for item, allowed in zip(env.action_definitions, mask) if allowed]
    matches = legal
    while True:
        for item in matches[:30]:
            print(f"  {item['id']:>5}: {action_name(item['key'])} [{item['key']}]")
        if len(matches) > 30:
            print(f"  Showing 30 of {len(matches)}; narrow the list with /text.")
        try:
            choice = input("Action ID, /search, ? to reset search, or q to stop: ").strip()
        except EOFError:
            return None
        if choice.lower() == "q":
            return None
        if choice == "?":
            matches = legal
        elif choice.startswith("/"):
            matches = [item for item in legal if choice[1:].lower() in item["key"].lower()]
        elif choice.isdigit() and any(item["id"] == int(choice) for item in legal):
            return int(choice)
        else:
            print("Choose a currently legal action ID (not its row number).")


def diagnose_episode(path: list[dict[str, Any]], status: str, window: int) -> dict[str, Any]:
    best_progress = path[0].get("progress_before", 0.0) if path else 0.0
    stagnant = longest = 0
    totals: dict[str, float] = {}
    errors = []
    for event in path:
        progress = event.get("goal_progress", 0.0)
        stagnant = 0 if progress > best_progress + 1e-9 else stagnant + 1
        best_progress = max(best_progress, progress)
        longest = max(longest, stagnant)
        for key, value in event.get("reward_components", {}).items():
            totals[key] = totals.get(key, 0.0) + value
        if event.get("error"):
            errors.append({"step": event["step"], "error": event["error"]})
    warnings = []
    if longest >= window:
        warnings.append(
            f"No new best goal progress for {longest} actions. Inspect the path for "
            "waiting or buy/sell cycles; championship goals have sparse progress, "
            "so this is not proof of an unreachable goal."
        )
    if status in ("max_days", "max_steps"):
        warnings.append(
            f"Run hit {status}, not a confirmed dead end. Check prerequisites and "
            "calendar waits manually before increasing limits."
        )
    if status in ("dead", "dead_money", "dead_stamina"):
        warnings.append(
            f"Run ended with {status}. Inspect the last safe state and resource costs; "
            "try a small safety penalty or an easier goal, not just more training."
        )
    if errors:
        warnings.append("A masked action failed: inspect the engine error before tuning rewards.")
    if any(event["action_key"].startswith("resolve_pending_event:") for event in path):
        warnings.append(
            "This route submits event results through the simulator. It is a planning "
            "experiment, not evidence the player can achieve those results in an external race."
        )
    if abs(totals.get("low_budget", 0) + totals.get("low_stamina", 0)) > 1000:
        warnings.append(
            "Safety penalties exceeded the default win bonus in magnitude. Check "
            "whether necessary purchases/recovery are being punished too heavily."
        )
    return {
        "best_goal_progress": best_progress,
        "longest_no_progress_steps": longest,
        "reward_totals": totals,
        "action_errors": errors,
        "warnings": warnings,
    }


def summarize_episodes(episodes: list[dict[str, Any]]) -> dict[str, Any]:
    statuses = [episode["status"] for episode in episodes]
    successes = [episode for episode in episodes if episode["status"] == "goal_reached"]
    successful_days = [episode["current_day"] - 1 for episode in successes]
    fastest = min(successes, key=lambda episode: (episode["current_day"], episode["steps"])) if successes else None
    return {
        "episodes": len(episodes),
        "goal_reached": len(successes),
        "statuses": {status: statuses.count(status) for status in sorted(set(statuses))},
        "mean_days": statistics.mean(episode["current_day"] - 1 for episode in episodes),
        "mean_steps": statistics.mean(episode["steps"] for episode in episodes),
        "success_rate": len(successes) / len(episodes),
        "mean_budget": statistics.mean(episode["budget"] for episode in episodes),
        "mean_stamina": statistics.mean(episode["stamina"] for episode in episodes),
        "successful_mean_days": statistics.mean(successful_days) if successes else None,
        "successful_median_days": statistics.median(successful_days) if successes else None,
        "successful_p90_days": float(np.percentile(successful_days, 90)) if successes else None,
        "fastest_success_seed": fastest["seed"] if fastest else None,
        "fastest_success_days": fastest["current_day"] - 1 if fastest else None,
    }


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
    verbose: bool,
    mode: str = "policy",
    stagnation_window: int = 100,
) -> dict[str, Any]:
    observation, info = env.reset(seed=seed)
    path: list[dict[str, Any]] = []
    terminated = False
    truncated = False
    rng = np.random.default_rng(seed)
    if mode == "policy":
        model.set_random_seed(seed)

    while not terminated and not truncated:
        action_mask = env.action_masks()
        if not action_mask.any():
            raise RuntimeError("No legal actions in an active episode; inspect the goal, initial state and engine mask")
        model_observation = observation
        observation_before = (
            env.debug_observation()
            if trace_level in ("observations", "decisions")
            else None
        )
        if mode == "manual":
            print(
                f"\nDay {int(observation['numeric'][-4])} | budget {info['budget']:.0f} "
                f"| stamina {info['stamina']:.1f} | goal progress {info['goal_progress']:.1%}"
            )
            state = observation_before or env.debug_observation()
            print("Characteristics:", state["characteristics"])
            print("Inventory:", ", ".join(state["inventory_ids"]) or "(empty)")
            print("Quests:", ", ".join(state["quest_ids"]) or "(none)")
            print("Pending events:", ", ".join(state["pending_event_ids"]) or "(none)")
            action = manual_action(env, action_mask)
            if action is None:
                info["status"] = "user_stopped"
                truncated = True
                break
        elif mode == "random":
            action = rng.choice(np.flatnonzero(action_mask))
        else:
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
            "budget": info.get("budget", 0.0),
            "stamina": info.get("stamina", 0.0),
            "goal_progress": info["goal_progress"],
            "progress_before": info["goal_progress_before"],
            "reward_components": info.get("reward_components", {}),
            "error": info.get("error"),
        }
        if trace_level in ("observations", "decisions"):
            event["observation_before"] = observation_before
            event["observation_after"] = env.debug_observation()
        if trace_level == "decisions" and mode == "policy":
            event["decision"] = policy_decision_trace(
                model,
                model_observation,
                action_mask,
                env.action_definitions,
                action_id,
                deterministic,
            )
        path.append(event)
        if verbose:
            print(
                f"  step={event['step']:>4} day={event['day']:>4} "
                f"{action_definition['key']} reward={reward:+.2f} "
                f"budget={event['budget']:.0f} stamina={event['stamina']:.1f} "
                f"progress={event['goal_progress']:.1%} "
                f"status={event['status']}",
                flush=True,
            )
            if mode == "manual":
                print(f"    Reward explanation: {event['reward_components']}")

    episode = {
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
    episode["diagnostics"] = diagnose_episode(path, episode["status"], stagnation_window)
    return episode


def main() -> None:
    parser = argparse.ArgumentParser(description="Play manually, measure a random baseline, or inspect a trained player.")
    parser.add_argument("--mode", choices=("policy", "manual", "random"), default="policy")
    parser.add_argument("--model")
    parser.add_argument("--episodes", type=positive_int, default=20)
    parser.add_argument("--seed", type=int, default=20_000)
    add_environment_arguments(parser)
    parser.add_argument("--stagnation-window", type=positive_int, default=100, help="Warn after this many actions without a new best goal value; never auto-terminate")
    parser.add_argument("--stochastic", action="store_true")
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="print every selected action and resulting budget/stamina",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="print only the final JSON summary",
    )
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
    if args.mode == "policy" and not args.model:
        parser.error("--model is required in policy mode; use --mode manual or random without a model")
    if args.mode == "manual" and args.quiet:
        parser.error("--quiet cannot be used with interactive manual play")
    reward_overrides = json.loads(Path(args.reward_config).read_text()) if args.reward_config else {}
    env = Gtr2CareerEnv(environment_config(args))
    try:
        model = None
        if args.mode == "policy":
            try:
                os.environ.setdefault("CUDA_VISIBLE_DEVICES", "")
                from sb3_contrib import MaskablePPO
            except ImportError as error:
                raise SystemExit("Install rl/requirements.txt before running the trained policy") from error
            check_model_contract(args.model, env)
            model = MaskablePPO.load(
                args.model, env=env, device="cpu",
                custom_objects={"verbose": 0},
            )
        episodes = []
        for offset in range(args.episodes):
            if not args.quiet:
                print(
                    f"[playtest] episode {offset + 1}/{args.episodes} "
                    f"seed={args.seed + offset}",
                    flush=True,
                )
            episode = run_episode(
                env,
                model,
                args.seed + offset,
                deterministic=not args.stochastic,
                trace_level=args.trace,
                verbose=(args.verbose or args.mode == "manual") and not args.quiet,
                mode=args.mode,
                stagnation_window=args.stagnation_window,
            )
            episodes.append(episode)
            if not args.quiet:
                print(
                    f"[playtest] result={episode['status']} "
                    f"days={episode['current_day'] - 1} steps={episode['steps']} "
                    f"budget={episode['budget']:.0f} stamina={episode['stamina']:.1f}",
                    flush=True,
                )
                for warning in episode["diagnostics"]["warnings"]:
                    print(f"  Warning: {warning}", flush=True)
            if episode["status"] == "user_stopped":
                break
    finally:
        env.close()

    summary = summarize_episodes(episodes)
    result = {
        "summary": summary, "configuration": vars(args),
        "reward_overrides": reward_overrides, "episodes": episodes,
    }
    if args.output:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(result, indent=2) + "\n")
    if not args.quiet:
        print(
            f"[playtest] success_rate={summary['success_rate']:.1%} "
            f"mean_days={summary['mean_days']:.1f} "
            f"mean_steps={summary['mean_steps']:.1f} "
            f"mean_budget={summary['mean_budget']:.0f} "
            f"mean_stamina={summary['mean_stamina']:.1f}",
            flush=True,
        )
        print(
            f"[playtest] successful_mean_days={summary['successful_mean_days'] if summary['successful_mean_days'] is not None else 'n/a'} "
            f"fastest_success_seed={summary['fastest_success_seed'] if summary['fastest_success_seed'] is not None else 'n/a'}"
        )
    print(json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    main()
