#!/usr/bin/env python3
"""Run a deterministic, inspectable starter strategy through the RL bridge."""

from __future__ import annotations

import argparse
import csv
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).parents[1]


def send(process: subprocess.Popen[str], request: dict) -> dict:
    process.stdin.write(json.dumps(request) + "\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        raise RuntimeError("RL bridge exited before returning a response")
    response = json.loads(line)
    if response["type"] == "error":
        raise RuntimeError(response["error"])
    return response


def dataset_objects() -> dict[str, dict[str, str]]:
    with (ROOT / "gtr2career" / "objects.csv").open(newline="", encoding="utf-8") as handle:
        return {row["id"]: row for row in csv.DictReader(handle)}


def championship_requirements() -> dict[str, list[str]]:
    requirements: dict[str, list[str]] = {}
    with (ROOT / "gtr2career" / "events.csv").open(newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            quest_id = row["quest_id"]
            if not quest_id or "championship" not in row["tags"].split(";"):
                continue
            required = [
                item.strip() for item in row["required_object_ids"].split(";")
                if item.strip() and item.strip() != "helmet"
            ]
            requirements.setdefault(quest_id, required)
    return requirements


def choose_action(
    actions: list[dict],
    observation: dict,
    objects: dict[str, dict[str, str]],
    joined_quests: dict[str, int],
) -> dict:
    legal = [
        action for action, allowed in zip(actions, observation["legal_action_mask"])
        if allowed
    ]
    keys = {action["key"]: action for action in legal}
    owned_ids = set(observation["inventory_ids"])
    completed = {
        quest_id for quest_id in (
            "amateur_championship",
            "fiesta_st_cup",
            "honda_civic_championship",
            "hh200_championship",
            "formula_ford_championship",
        )
        if any(f"trophy_{quest_id}" in item for item in owned_ids)
    }
    championship_order = (
        "amateur_championship",
        "fiesta_st_cup",
        "honda_civic_championship",
        "hh200_championship",
        "formula_ford_championship",
    )
    target_championship = next(
        (quest_id for quest_id in championship_order if quest_id not in completed),
        championship_order[-1],
    )

    def owned(object_id: str) -> bool:
        return object_id in owned_ids or any(
            item.startswith(f"{object_id}_") for item in owned_ids
        )

    pending = observation["pending_event_ids"]
    for event_id in pending:
        preferred = f"resolve_pending_event:{event_id}:success:1"
        if preferred in keys:
            return keys[preferred]
        fallback = next(
            (action for action in legal if action["key"].startswith(
                f"resolve_pending_event:{event_id}:success:"
            )),
            None,
        )
        if fallback:
            return fallback

    # Service any vehicle or equipment that the engine marks as needing it
    # before attempting another event.
    service_actions = [
        action for action in legal if action["key"].startswith("service_object:")
    ]
    if service_actions:
        return service_actions[0]

    # Keep a job active from the beginning so living costs are affordable.
    for job in ("act_work_b", "act_work", "act_part_time", "act_work_temp"):
        action = keys.get(f"start_activity:{job}")
        if action and not observation["active_event_ids"] and observation["characteristics"].get("budget", 0) < 2500:
            return action

    join = keys.get(f"join_quest:{target_championship}")
    required_vehicle = next(
        (
            object_id for object_id in championship_requirements().get(
                target_championship, []
            )
            if objects.get(object_id, {}).get("type") == "vehicle"
        ),
        None,
    )
    if (
        join
        and target_championship not in observation["quest_ids"]
        and (required_vehicle is None or owned(required_vehicle))
        and observation["current_day"]
        // observation["days_per_year"]
        > joined_quests.get(target_championship, -1) // observation["days_per_year"]
    ):
        return join

    if not any(item.startswith("ford_fiesta_st150_") for item in owned_ids):
        loan = keys.get("start_activity:act_personal_loan")
        if loan and observation["characteristics"].get("budget", 0) < 1000:
            return loan
        bank_loan = keys.get("start_activity:act_bank_loan")
        if bank_loan and observation["characteristics"].get("budget", 0) < 18000:
            return bank_loan

    # Buy the starter route prerequisites and the next championship's licence/car
    # as soon as the engine says they are legal.
    required_license = {
        "fiesta_st_cup": "license_national_b",
        "honda_civic_championship": "license_national_b",
        "hh200_championship": "license_national_b",
        "formula_ford_championship": "license_national_b",
    }.get(target_championship)
    desired_objects = ["helmet", "insurance_third_party"]
    if required_license:
        desired_objects.extend(["tracksuit", "gloves", "shoes"])
        desired_objects.append(required_license)
    desired_objects.extend(championship_requirements().get(target_championship, []))
    for object_id in desired_objects:
        action = keys.get(f"buy_object:{object_id}")
        if action and not owned(object_id):
            definition = objects.get(object_id, {})
            if definition.get("type") == "vehicle":
                price = float(definition.get("price", 0) or 0)
                if observation["characteristics"].get("budget", 0) < price + 5000:
                    continue
            return action

    # Enter the first legal race for the current championship.
    race_entries = [
        action for action in legal
        if action["key"].startswith("enter_event:")
        and target_championship in action["key"]
        and action["key"].rsplit(":", 1)[-1]
    ]
    if race_entries:
        return race_entries[0]

    if observation["active_event_ids"]:
        return keys.get("wait") or legal[0]

    return keys.get("wait") or legal[0]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--max-days", type=int, default=730)
    parser.add_argument("--max-steps", type=int, default=2500)
    parser.add_argument("--goal", default="championships:level=1,count=1")
    parser.add_argument("--output", default=str(ROOT / "playthrough" / "heuristic-route.json"))
    args = parser.parse_args()

    process = subprocess.Popen(
        [
            str(ROOT / "src-tauri" / "target" / "release" / "rl_env"),
            "--dataset", "gtr2career",
            "--goal", args.goal,
            "--max-days", str(args.max_days),
            "--max-steps", str(args.max_steps),
            "--seed", str(args.seed),
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
    )
    try:
        actions = send(process, {"op": "actions"})["actions"]
        observation_response = send(process, {"op": "observation"})
        observation = observation_response["observation"]
        objects = dataset_objects()
        trace = []
        joined_quests: dict[str, int] = {}
        while True:
            action = choose_action(actions, observation, objects, joined_quests)
            if action["key"].startswith("join_quest:"):
                joined_quests[action["key"].split(":", 1)[1]] = observation["current_day"]
            result = send(process, {"op": "step", "action_id": action["id"]})["result"]
            info = result["info"]
            trace.append({
                "step": len(trace) + 1,
                "action": action["key"],
                "day": info["current_day"],
                "days_elapsed": info["days_elapsed"],
                "budget": info["budget"],
                "stamina": info["stamina"],
                "status": info["status"],
                "error": info["error"],
                "reward_components": info["reward_components"],
            })
            observation = result["observation"]
            if result["terminated"] or result["truncated"]:
                break
        report = {
            "seed": args.seed,
            "goal": args.goal,
            "max_days": args.max_days,
            "max_steps": args.max_steps,
            "observation_schema_version": observation["schema_version"],
            "action_catalogue_size": len(actions),
            "final_observation": observation,
            "trace": trace,
        }
        Path(args.output).write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({
            "output": args.output,
            "steps": len(trace),
            "status": trace[-1]["status"] if trace else "none",
            "day": observation["current_day"],
            "budget": observation["characteristics"].get("budget"),
            "inventory": observation["inventory_ids"],
            "quests": observation["quest_ids"],
            "pending": observation["pending_event_ids"],
        }, indent=2))
    finally:
        process.terminate()
        process.wait(timeout=5)


if __name__ == "__main__":
    main()
