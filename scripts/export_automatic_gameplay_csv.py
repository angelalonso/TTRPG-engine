#!/usr/bin/env python3
"""Export the progress-defining actions from an automatic-play report."""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path


SIGNIFICANT_PREFIXES = (
    "buy_object:",
    "sell_object:",
    "join_quest:",
    "enter_event:",
    "resolve_pending_event:",
    "start_activity:",
    "service_object:",
)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("report", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    report = json.loads(args.report.read_text(encoding="utf-8"))
    rows = []
    for item in report.get("trace", []):
        action = item["action"]
        if not action.startswith(SIGNIFICANT_PREFIXES):
            continue
        rows.append({
            "day": f"{item['day']:04d}",
            "action": action,
            "budget": item["budget"],
            "stamina": item["stamina"],
            "status": item["status"],
            "reward": json.dumps(item["reward_components"], sort_keys=True),
            "error": item["error"] or "",
        })

    with args.output.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0].keys() if rows else [
            "day", "action", "budget", "stamina", "status", "reward", "error"
        ])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    main()
