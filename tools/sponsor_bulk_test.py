#!/usr/bin/env python3
"""Run repeatable bulk tests against the sponsor negotiation plugin."""

from __future__ import annotations

import argparse
import csv
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path
from typing import Any


TIERS = ("local", "regional", "national", "continental", "world")
DEFAULT_PLUGIN = Path(__file__).parents[1] / "dataset" / "plugins" / "sponsor_negotiator.py"
DEFAULT_DATASET = Path(__file__).parents[1] / "dataset"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plugin", type=Path, default=DEFAULT_PLUGIN)
    parser.add_argument("--dataset", type=Path, default=DEFAULT_DATASET)
    parser.add_argument("--runs", type=int, default=100)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--results", type=int, default=60)
    parser.add_argument("--exp", type=int, default=100)
    parser.add_argument("--charisma", type=int, default=5)
    parser.add_argument("--podiums", type=int, default=3)
    parser.add_argument("--wins", type=int, default=1)
    parser.add_argument("--poles", type=int, default=0)
    parser.add_argument("--championships-won", type=int, default=0)
    parser.add_argument("--race-tier", choices=TIERS, default="local")
    parser.add_argument("--scope", choices=("race", "quest", "championship", "year"), default="race")
    parser.add_argument("--manager-level", type=int, default=0)
    parser.add_argument("--strategy", choices=("accept", "logic"), default="accept")
    parser.add_argument(
        "--matrix",
        action="store_true",
        help="run no-manager plus manager levels 2, 5, and 8 across every race tier",
    )
    parser.add_argument("--format", choices=("table", "csv", "json"), default="table")
    return parser


def parse_messages(stdout: str) -> list[dict[str, Any]]:
    messages: list[dict[str, Any]] = []
    for line in stdout.splitlines():
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise RuntimeError(f"plugin emitted non-JSON output: {line!r}") from error
        if not isinstance(value, dict):
            raise RuntimeError("plugin emitted a JSON value that was not an object")
        messages.append(value)
    return messages


def run_once(args: argparse.Namespace, manager_level: int, race_tier: str, run_number: int) -> dict[str, Any]:
    command = [
        sys.executable,
        str(args.plugin),
        "--json",
        "--dataset-path",
        str(args.dataset),
        "--seed",
        str(args.seed + run_number),
        "--results",
        str(args.results),
        "--exp",
        str(args.exp),
        "--charisma",
        str(args.charisma),
        "--podiums",
        str(args.podiums),
        "--wins",
        str(args.wins),
        "--poles",
        str(args.poles),
        "--championships-won",
        str(args.championships_won),
        "--race-tier",
        race_tier,
        "--scope",
        args.scope,
    ]
    if manager_level > 0:
        command.extend(["--has-agent", "--agent-level", str(manager_level)])

    if args.strategy == "logic":
        actions = [{"action_id": "logic_rebuttal"}] * 8
    else:
        actions = [{"action_id": "accept_proposal"}]

    completed = subprocess.run(
        command,
        input="\n".join(json.dumps(action) for action in actions) + "\n",
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        error_message = completed.stderr.strip() or "plugin rejected the scenario"
        try:
            error_message = json.loads(completed.stdout).get("error", error_message)
        except (json.JSONDecodeError, AttributeError):
            pass
        return {
            "run": run_number,
            "manager_level": manager_level,
            "race_tier": race_tier,
            "cold_call": False,
            "sponsor": "",
            "sponsor_tier": "",
            "attraction": 0,
            "proposal_expires": "",
            "status": "ERROR",
            "rounds": 0,
            "error": error_message,
        }
    messages = parse_messages(completed.stdout)
    if not messages:
        raise RuntimeError(f"run {run_number} emitted no messages")
    initial = messages[0]
    terminal = next(
        (message for message in reversed(messages) if message.get("status") in {"SIGNED", "REJECTED", "BANNED"}),
        messages[-1],
    )
    return {
        "run": run_number,
        "manager_level": manager_level,
        "race_tier": race_tier,
        "cold_call": bool(initial.get("cold_call", False)),
        "sponsor": initial.get("sponsor_name", ""),
        "sponsor_tier": initial.get("sponsor_tier", ""),
        "attraction": initial.get("attraction_score", 0),
        "proposal_expires": initial.get("proposal_expires") or "",
        "status": terminal.get("status", "ERROR"),
        "rounds": terminal.get("round", len(messages) - 1),
        "error": terminal.get("error", ""),
    }


def scenarios(args: argparse.Namespace) -> list[tuple[int, str]]:
    if args.matrix:
        return [
            (manager_level, race_tier)
            for manager_level in (0, 2, 5, 8)
            for race_tier in TIERS
        ]
    return [(args.manager_level, args.race_tier)]


def render(results: list[dict[str, Any]], output_format: str) -> None:
    if output_format == "json":
        print(json.dumps(results, indent=2))
        return
    if output_format == "csv":
        writer = csv.DictWriter(sys.stdout, fieldnames=results[0].keys())
        writer.writeheader()
        writer.writerows(results)
        return

    headers = ("manager", "race tier", "runs", "signed", "rejected", "banned", "errors", "cold calls", "avg attraction")
    print(" | ".join(headers))
    print("-+-".join("-" * len(header) for header in headers))
    grouped: dict[tuple[int, str], list[dict[str, Any]]] = {}
    for result in results:
        grouped.setdefault((result["manager_level"], result["race_tier"]), []).append(result)
    for (manager, tier), group in grouped.items():
        statuses = Counter(result["status"] for result in group)
        attraction = sum(float(result["attraction"]) for result in group) / len(group)
        print(
            f"{manager:7} | {tier:9} | {len(group):4} | "
            f"{statuses['SIGNED']:6} | {statuses['REJECTED']:8} | {statuses['BANNED']:6} | "
            f"{statuses['ERROR']:6} | {sum(result['cold_call'] for result in group):10} | {attraction:14.2f}"
        )


def main() -> int:
    args = build_parser().parse_args()
    if args.runs < 1:
        raise SystemExit("--runs must be at least 1")
    if not args.plugin.is_file():
        raise SystemExit(f"plugin not found: {args.plugin}")
    if not args.dataset.is_dir():
        raise SystemExit(f"dataset directory not found: {args.dataset}")

    results: list[dict[str, Any]] = []
    run_number = 0
    for manager_level, race_tier in scenarios(args):
        for _ in range(args.runs):
            results.append(run_once(args, manager_level, race_tier, run_number))
            run_number += 1
    render(results, args.format)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
