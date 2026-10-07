#!/usr/bin/env python3
"""Run a readable end-to-end smoke suite for the sponsor negotiation plugin."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).parents[1]
DEFAULT_PLUGIN = ROOT / "gtr2career" / "plugins" / "sponsor_negotiator.py"
DEFAULT_DATASET = ROOT / "gtr2career"
TERMINAL = {"SIGNED", "REJECTED", "BANNED"}


def parse_messages(output: str) -> list[dict[str, Any]]:
    messages = []
    for line in output.splitlines():
        if line.strip():
            value = json.loads(line)
            if not isinstance(value, dict):
                raise AssertionError("plugin emitted a non-object JSON response")
            messages.append(value)
    return messages


def run_protocol(
    plugin: Path,
    dataset: Path,
    actions: list[dict[str, Any]],
    *,
    scope: str = "race",
    seed: int = 7,
    manager_level: int = 0,
) -> list[dict[str, Any]]:
    command = [
        sys.executable,
        str(plugin),
        "--json",
        "--dataset-path",
        str(dataset),
        "--scope",
        scope,
        "--race-tier",
        "national",
        "--seed",
        str(seed),
        "--results",
        "80",
        "--exp",
        "100",
        "--charisma",
        "5",
        "--podiums",
        "3",
        "--wins",
        "1",
        "--poles",
        "1",
    ]
    if manager_level:
        command.extend(["--has-agent", "--agent-level", str(manager_level)])
    completed = subprocess.run(
        command,
        input="\n".join(json.dumps(action) for action in actions) + "\n",
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(completed.stderr.strip() or "plugin process failed")
    messages = parse_messages(completed.stdout)
    if not messages:
        raise AssertionError("plugin emitted no JSON responses")
    return messages


def check_protocol(plugin: Path, dataset: Path) -> None:
    messages = run_protocol(plugin, dataset, [{"action_id": "accept_proposal"}])
    assert messages[0]["status"] == "ONGOING"
    assert messages[-1]["status"] == "SIGNED"
    assert messages[-1]["agreement"]["scope"] == "race"


def check_scope_terms(plugin: Path, dataset: Path) -> None:
    for scope in ("race", "championship", "year"):
        messages = run_protocol(plugin, dataset, [{"action_id": "accept_proposal"}], scope=scope)
        proposal = messages[-1]["proposal"]
        expected_monthly = 0 if scope != "year" else proposal["monthly_payment"]
        assert proposal["monthly_payment"] == expected_monthly
        assert messages[-1]["agreement"]["scope"] == scope


def check_negotiation_rounds(plugin: Path, dataset: Path) -> None:
    messages = run_protocol(
        plugin,
        dataset,
        [{"action_id": "logic_rebuttal"}] * 8,
        manager_level=4,
    )
    assert messages[0]["manager_level"] == 4
    assert messages[-1]["status"] in TERMINAL
    assert all(message["status"] != "ERROR" for message in messages)


def check_error_recovery(plugin: Path, dataset: Path) -> None:
    messages = run_protocol(
        plugin,
        dataset,
        [{"action_id": "not_an_action"}, {"action_id": "accept_proposal"}],
    )
    assert messages[1]["status"] == "ERROR"
    assert messages[-1]["status"] == "SIGNED"


def check_target_vehicle_data(plugin: Path, dataset: Path) -> None:
    sys.path.insert(0, str(plugin.parent))
    import sponsor_negotiator

    _, races, vehicles, required_by_event, _, _ = sponsor_negotiator._load_target_options(str(dataset))
    required = required_by_event["ford_fiesta_st150_national_round_1"]
    assert required == ["ford_fiesta_st150"]
    available = [vehicle_id for vehicle_id, _ in vehicles if vehicle_id in set(required)]
    assert available == ["ford_fiesta_st150"]
    assert any(event_id == "ford_fiesta_st150_national_round_1" for event_id, _ in races)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plugin", type=Path, default=DEFAULT_PLUGIN)
    parser.add_argument("--dataset", type=Path, default=DEFAULT_DATASET)
    args = parser.parse_args()
    checks = (
        ("JSON protocol and signed agreement", check_protocol),
        ("scope payment rules", check_scope_terms),
        ("multi-round negotiation", check_negotiation_rounds),
        ("error recovery", check_error_recovery),
        ("target vehicle data", check_target_vehicle_data),
    )
    failures = []
    for name, check in checks:
        try:
            check(args.plugin, args.dataset)
        except (AssertionError, KeyError, json.JSONDecodeError, OSError) as error:
            failures.append((name, str(error) or error.__class__.__name__))
            print(f"FAIL  {name}: {error}")
        else:
            print(f"PASS  {name}")
    if failures:
        print(f"\n{len(failures)} check(s) failed.")
        return 1
    print(f"\nAll {len(checks)} sponsor plugin checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
