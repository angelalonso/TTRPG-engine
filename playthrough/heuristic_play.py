#!/usr/bin/env python3
"""Run a deterministic, inspectable starter strategy through the RL bridge."""

from __future__ import annotations

import argparse
import csv
import json
import signal
import subprocess
import time
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


def championship_ids() -> list[str]:
    ids: list[str] = []
    with (ROOT / "gtr2career" / "quests.csv").open(newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            quest_id = row["id"]
            if row["type"] == "championship" and quest_id not in ids:
                ids.append(quest_id)
    return ids


def championship_metadata() -> dict[str, dict[str, str]]:
    with (ROOT / "gtr2career" / "quests.csv").open(newline="", encoding="utf-8") as handle:
        return {
            row["id"]: row
            for row in csv.DictReader(handle)
            if row["type"] == "championship"
        }


def championship_first_days() -> dict[str, int]:
    first_days: dict[str, int] = {}
    with (ROOT / "gtr2career" / "events.csv").open(newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            quest_id = row["quest_id"]
            if not quest_id or "championship" not in row["tags"].split(";"):
                continue
            day = int(row["day_of_year"])
            first_days[quest_id] = min(first_days.get(quest_id, day), day)
    return first_days


def championship_event_days() -> dict[str, list[int]]:
    event_days: dict[str, list[int]] = {}
    with (ROOT / "gtr2career" / "events.csv").open(newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            quest_id = row["quest_id"]
            if not quest_id or "championship" not in row["tags"].split(";"):
                continue
            event_days.setdefault(quest_id, []).append(int(row["day_of_year"]))
    for days in event_days.values():
        days.sort()
    return event_days


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
    configured_championships = championship_ids()
    metadata = championship_metadata()
    completed = {
        quest_id for quest_id in configured_championships
        if any(f"trophy_{quest_id}" in item for item in owned_ids)
    }
    championship_order = tuple(configured_championships)
    target_championship = next(
        (quest_id for quest_id in championship_order if quest_id not in completed),
        championship_order[-1] if championship_order else "",
    )

    def owned(object_id: str) -> bool:
        return object_id in owned_ids or any(
            item.startswith(f"{object_id}_") for item in owned_ids
        )

    pending = observation["pending_event_ids"]
    first_days = championship_first_days()
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

    encounter_actions = [
        action for action in legal if action["key"].startswith("resolve_encounter:")
    ]
    if encounter_actions:
        return encounter_actions[0]

    # Keep a job active from the beginning so living costs are affordable.
    for job in (
        "9_9_harder_job",
        "9_5_job",
        "part_time_job",
        "temporary_job",
    ):
        action = keys.get(f"start_activity:{job}")
        if (
            action
            and observation["current_day"] > 1
            and observation["characteristics"].get("budget", 0) < 12000
            and not any(
                event_id not in {"act_bank_loan", "act_personal_loan"}
                for event_id in observation["active_event_ids"]
            )
        ):
            return action

    join = keys.get(f"join_quest:{target_championship}")
    season_day = observation["current_day"] % observation["days_per_year"]
    first_day = first_days.get(target_championship, 1)
    if (
        join
        and target_championship not in observation["quest_ids"]
        and first_day - 1 <= season_day <= first_day + 7
        and (
            not metadata.get(target_championship, {}).get("required_license_id")
            or owned(metadata[target_championship]["required_license_id"])
        )
    ):
        return join

    sponsor_activity = {
        "fiesta_st_cup": "act_sponsor_ford_fiesta_club",
        "honda_civic_championship": "act_sponsor_honda_civic_club",
        "hh200_championship": "act_sponsor_hot_hatch_club",
        "fia_ngt_championship": "act_sponsor_fia_ngt",
        "fia_gt_championship": "act_sponsor_fia_gt",
    }.get(target_championship)
    sponsor_vehicle = {
        "fiesta_st_cup": "ford_fiesta_st150",
        "honda_civic_championship": "honda_civic",
        "hh200_championship": "ford_fiesta",
    }.get(target_championship)
    requirements = championship_requirements().get(target_championship, [])
    vehicle_requirements = [
        object_id for object_id in requirements
        if objects.get(object_id, {}).get("type") == "vehicle"
    ]
    target_vehicle = (
        sponsor_vehicle
        if sponsor_vehicle in vehicle_requirements
        else next(iter(vehicle_requirements), None)
    )

    if target_vehicle and owned(target_vehicle):
        service_action = next(
            (
                action for action in legal
                if action["key"].startswith(f"service_object:{target_vehicle}:")
            ),
            None,
        )
        if service_action:
            return service_action

    acquisition_window = (
        target_championship in observation["quest_ids"]
        or (
            join is not None
            and first_day - 1 <= season_day <= first_day + 7
        )
    )
    target_prerequisites = {
        prerequisite
        for prerequisite in objects.get(target_vehicle, {}).get(
            "requires_object_ids", ""
        ).split(";")
        if prerequisite
    }
    if target_vehicle and not owned(target_vehicle) and acquisition_window:
        sale_candidates = sorted(
            (
                action for action in legal
                if action["key"].startswith("sell_object:")
                and objects.get(action["key"].split(":", 1)[1], {}).get("type")
                in {"vehicle", "trailer"}
                and action["key"].split(":", 1)[1] not in target_prerequisites
                and action["key"] != f"sell_object:{target_vehicle}"
            ),
            key=lambda action: float(
                objects.get(action["key"].split(":", 1)[1], {}).get("price", 0) or 0
            ),
            reverse=True,
        )
        target_price = float(objects.get(target_vehicle, {}).get("price", 0) or 0)
        purchase_reserve = max(1000.0, min(20000.0, target_price * 0.25))
        if sale_candidates:
            if observation["characteristics"].get("budget", 0) < target_price + purchase_reserve:
                return sale_candidates[0]

        if observation["characteristics"].get("budget", 0) < target_price + purchase_reserve:
            bank_loan = keys.get("start_activity:act_bank_loan")
            if bank_loan:
                return bank_loan
            personal_loan = keys.get("start_activity:act_personal_loan")
            if personal_loan:
                return personal_loan

    if not any(item.startswith("ford_fiesta_st150_") for item in owned_ids):
        loan = keys.get("start_activity:act_personal_loan")
        if loan and observation["characteristics"].get("budget", 0) < 1000:
            return loan
        bank_loan = keys.get("start_activity:act_bank_loan")
        if bank_loan and observation["characteristics"].get("budget", 0) < 18000:
            return bank_loan

    # Buy the starter route prerequisites and the next championship's licence/car
    # as soon as the engine says they are legal.
    required_license = metadata.get(target_championship, {}).get("required_license_id")
    manager_id = {
        "fia_ngt_championship": "manager_level_4",
        "fia_gt_championship": "manager_level_5",
    }.get(target_championship, "")
    desired_objects = ["helmet", "insurance_third_party"]
    if required_license:
        desired_objects.extend(["tracksuit", "gloves", "shoes"])
        desired_objects.append(required_license)
    if manager_id:
        desired_objects.append(manager_id)
    sponsor_start = keys.get(f"start_activity:{sponsor_activity}") if sponsor_activity else None
    sponsor_entry = keys.get(f"enter_event:{sponsor_activity}:") if sponsor_activity else None
    sponsor_available = sponsor_start or sponsor_entry
    desired_objects.extend(
        object_id for object_id in requirements
        if object_id == target_vehicle
        or objects.get(object_id, {}).get("type") != "vehicle"
    )
    expanded_objects: list[str] = []

    def add_with_prerequisites(object_id: str) -> None:
        definition = objects.get(object_id, {})
        prerequisites = [
            prerequisite for prerequisite in definition.get("requires_object_ids", "").split(";")
            if prerequisite
        ]
        previous_license = definition.get("license_previous_id", "")
        if previous_license:
            prerequisites.append(previous_license)
        for prerequisite in prerequisites:
            if prerequisite not in expanded_objects:
                add_with_prerequisites(prerequisite)
        if object_id not in expanded_objects:
            expanded_objects.append(object_id)

    for object_id in desired_objects:
        add_with_prerequisites(object_id)
    for object_id in expanded_objects:
        if object_id == target_vehicle and not acquisition_window:
            continue
        if object_id in target_prerequisites and (
            not acquisition_window or owned(target_vehicle)
        ):
            continue
        if object_id == target_vehicle and sponsor_available:
            continue
        action = keys.get(f"buy_object:{object_id}")
        if action and not owned(object_id):
            definition = objects.get(object_id, {})
            if definition.get("type") == "vehicle":
                price = float(definition.get("price", 0) or 0)
                if observation["characteristics"].get("budget", 0) < price + 1000:
                    continue
            return action

    if target_vehicle and not owned(target_vehicle) and sponsor_activity:
        sponsor = sponsor_start
        if sponsor and (
            not required_license
            or owned(required_license)
        ):
            return sponsor
        sponsor = sponsor_entry
        if sponsor and (
            not required_license
            or owned(required_license)
        ):
            return sponsor

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
        if "wait" in keys:
            return keys["wait"]

    if "wait" in keys:
        return keys["wait"]
    fallback = next(
        (
            action for action in legal
            if not action["key"].startswith(
                ("service_object:", "buy_object:", "sell_object:")
            )
        ),
        None,
    )
    if fallback:
        return fallback
    raise RuntimeError(
        "No safe legal fallback action is available: "
        + ", ".join(action["key"] for action in legal[:20])
        + f" (day={observation['current_day']}, "
        f"active={observation['active_event_ids']}, "
        f"pending={observation['pending_event_ids']})"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--max-days", type=int, default=730)
    parser.add_argument("--max-steps", type=int, default=2500)
    parser.add_argument(
        "--fast-forward-days",
        type=int,
        default=1,
        help="Batch consecutive wait actions by this many days",
    )
    parser.add_argument(
        "--checkpoint",
        help="Write an RL checkpoint JSON file during the run",
    )
    parser.add_argument(
        "--checkpoint-every",
        type=int,
        default=0,
        help="Checkpoint every N trace entries (requires --checkpoint)",
    )
    parser.add_argument(
        "--resume",
        help="Resume from a checkpoint JSON file",
    )
    parser.add_argument("--goal", default="championships:level=1,count=1")
    parser.add_argument(
        "--build",
        action="store_true",
        help="Build the release RL bridge before running",
    )
    parser.add_argument("--output", default=str(ROOT / "playthrough" / "heuristic-route.json"))
    args = parser.parse_args()
    started_at = time.time()
    interrupted = False

    def request_interrupt(_signum: int, _frame: object) -> None:
        nonlocal interrupted
        interrupted = True

    signal.signal(signal.SIGINT, request_interrupt)

    goal = args.goal
    binary = ROOT / "src-tauri" / "target" / "release" / "rl_env"
    if args.build:
        subprocess.run(
            ["cargo", "build", "--release", "--manifest-path", str(ROOT / "src-tauri" / "Cargo.toml")],
            check=True,
        )
    if not binary.is_file():
        raise SystemExit(f"RL bridge not found at {binary}; rerun with --build")
    dataset_revision = next(
        row["value"].strip()
        for row in csv.DictReader((ROOT / "gtr2career" / "config.csv").open(
            newline="", encoding="utf-8"
        ))
        if row["variable"].strip() == "dataset_revision"
    )
    engine_build = subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()

    process = subprocess.Popen(
        [
            str(binary),
            "--dataset", "gtr2career",
            "--goal", goal,
            "--max-days", str(args.max_days),
            "--max-steps", str(args.max_steps),
            "--seed", str(args.seed),
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        # Keep Ctrl-C in the runner from terminating the RL bridge before the
        # runner can request and persist a checkpoint.
        start_new_session=True,
    )
    try:
        actions_response = send(process, {"op": "actions"})
        actions = actions_response["actions"]
        observation_response = send(process, {"op": "observation"})
        observation = observation_response["observation"]
        objects = dataset_objects()
        configured_championships = championship_ids()
        first_days = championship_first_days()
        event_days = championship_event_days()
        trace = []
        joined_quests: dict[str, int] = {}
        if args.resume:
            checkpoint_data = json.loads(Path(args.resume).read_text(encoding="utf-8"))
            checkpoint_data["checkpoint"]["config"]["max_days"] = args.max_days
            checkpoint_data["checkpoint"]["config"]["max_steps"] = args.max_steps
            observation = send(
                process,
                {"op": "restore", "checkpoint": checkpoint_data["checkpoint"]},
            )["observation"]
            # Restoring a checkpoint rebuilds the RL action catalogue from the
            # checkpoint's persisted catalog, so refresh IDs before stepping.
            actions = send(process, {"op": "actions"})["actions"]
            trace = checkpoint_data.get("trace", [])
        while True:
            action = choose_action(actions, observation, objects, joined_quests)
            if action["key"].startswith("join_quest:"):
                joined_quests[action["key"].split(":", 1)[1]] = observation["current_day"]
            repeat = (
                max(1, args.fast_forward_days)
                if action["key"] == "wait"
                and not observation["pending_event_ids"]
                else 1
            )
            # Keep enough decision points for the heuristic to start work or
            # buy a missing prerequisite before a long wait reaches the
            # episode step limit.
            if action["key"] == "wait" and observation["current_day"] < 14:
                repeat = min(repeat, 1)
            if repeat > 1:
                owned_ids = set(observation["inventory_ids"])
                completed = {
                    quest_id for quest_id in configured_championships
                    if any(f"trophy_{quest_id}" in item for item in owned_ids)
                }
                target_championship = next(
                    (
                        quest_id for quest_id in configured_championships
                        if quest_id not in completed
                    ),
                    "",
                )
                if (
                    target_championship
                ):
                    season_day = observation["current_day"] % observation["days_per_year"]
                    if target_championship not in observation["quest_ids"]:
                        if any(
                            candidate["key"] == f"join_quest:{target_championship}"
                            for candidate, allowed in zip(
                                actions, observation["legal_action_mask"]
                            )
                            if allowed
                        ):
                            first_day = first_days.get(target_championship, 1)
                            if season_day < first_day:
                                repeat = min(repeat, first_day - season_day)
                            elif season_day > first_day + 7:
                                until_next_year = (
                                    observation["days_per_year"] - season_day + first_day
                                )
                                repeat = min(repeat, until_next_year)
                    else:
                        scheduled_days = event_days.get(target_championship, [])
                        next_event = next(
                            (day for day in scheduled_days if day > season_day),
                            None,
                        )
                        if next_event is None and scheduled_days:
                            next_event = (
                                observation["days_per_year"]
                                - season_day
                                + scheduled_days[0]
                            )
                        if next_event is not None:
                            repeat = min(
                                repeat,
                                max(1, next_event - season_day),
                            )
            result = send(
                process,
                {"op": "step", "action_id": action["id"], "repeat": repeat},
            )["result"]
            info = result["info"]
            legal = [
                candidate for candidate, allowed in zip(actions, observation["legal_action_mask"])
                if allowed
            ]
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
                "legal_sponsor_actions": [
                    candidate["key"] for candidate in legal
                    if "sponsor" in candidate["key"]
                ],
            })
            observation = result["observation"]
            actions = send(process, {"op": "actions"})["actions"]
            if interrupted:
                checkpoint = send(process, {"op": "checkpoint"})["checkpoint"]
                checkpoint_path = Path(args.checkpoint or f"{args.output}.checkpoint.json")
                checkpoint_path.write_text(
                    json.dumps({"checkpoint": checkpoint, "trace": trace}, indent=2)
                    + "\n",
                    encoding="utf-8",
                )
                break
            if (
                args.checkpoint
                and args.checkpoint_every > 0
                and len(trace) % args.checkpoint_every == 0
            ):
                checkpoint = send(process, {"op": "checkpoint"})["checkpoint"]
                Path(args.checkpoint).write_text(
                    json.dumps({"checkpoint": checkpoint, "trace": trace}, indent=2)
                    + "\n",
                    encoding="utf-8",
                )
            if result["terminated"] or result["truncated"]:
                break
        report = {
            "seed": args.seed,
            "goal": args.goal,
            "resolved_goal": goal,
            "dataset_revision": int(dataset_revision),
            "engine_build": engine_build,
            "action_catalogue_version": actions_response["action_catalogue_version"],
            "max_days": args.max_days,
            "max_steps": args.max_steps,
            "observation_schema_version": observation["schema_version"],
            "action_catalogue_size": len(actions),
            "started_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(started_at)),
            "elapsed_seconds": round(time.time() - started_at, 3),
            "interrupted": interrupted,
            "blockers": {
                "errors": {
                    error: sum(1 for item in trace if item["error"] == error)
                    for error in sorted({item["error"] for item in trace if item["error"]})
                },
                "dead_money_steps": sum(item["status"] == "DeadMoney" for item in trace),
                "dead_stamina_steps": sum(item["status"] == "DeadStamina" for item in trace),
                "low_budget_steps": sum(
                    item["budget"] < 950 for item in trace
                ),
            },
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
            "elapsed_seconds": report["elapsed_seconds"],
            "blockers": report["blockers"],
            "inventory": observation["inventory_ids"],
            "quests": observation["quest_ids"],
            "pending": observation["pending_event_ids"],
        }, indent=2))
    finally:
        process.terminate()
        process.wait(timeout=5)


if __name__ == "__main__":
    main()
