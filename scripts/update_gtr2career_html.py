#!/usr/bin/env python3
"""Regenerate catalog HTML pages from the current dataset CSV values."""

from __future__ import annotations

import argparse
from pathlib import Path

from catalog_tools import (
    MONEYMAKER_TYPES,
    car_html,
    championship_html,
    event_html,
    object_html,
    read_csv,
)


def write_page(dataset: Path, relative_path: str, content: str) -> None:
    path = dataset / relative_path.removeprefix("./")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset", type=Path, default=Path("gtr2career"))
    args = parser.parse_args()

    _, objects = read_csv(args.dataset / "objects.csv")
    _, costs = read_csv(args.dataset / "costs.csv")
    _, events = read_csv(args.dataset / "events.csv")
    _, quests = read_csv(args.dataset / "quests.csv")
    _, rounds = read_csv(args.dataset / "championships.csv")
    cost_map = {row["id"]: row for row in costs}

    for row in objects:
        if row.get("type") == "vehicle" and row.get("description_html"):
            write_page(args.dataset, row["description_html"], car_html(row, cost_map))
        elif row.get("type") in {"equipment", "trailer", "insurance", "companion", "license"} and row.get("description_html"):
            write_page(args.dataset, row["description_html"], object_html(row, objects))
    for row in events:
        if row.get("description_html") and (
            row.get("type") == "race" or row.get("type") in MONEYMAKER_TYPES
        ):
            write_page(args.dataset, row["description_html"], event_html(row))
    for row in quests:
        if row.get("type") == "championship" and row.get("description_html"):
            write_page(args.dataset, row["description_html"], championship_html(row, rounds))


if __name__ == "__main__":
    main()
