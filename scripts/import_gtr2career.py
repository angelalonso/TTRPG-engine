#!/usr/bin/env python3
"""Import one catalog export into a dataset after explicit confirmation."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

from catalog_tools import read_csv, write_csv


def confirm(path: Path, targets: list[Path], assume_yes: bool) -> None:
    existing = [target for target in targets if target.exists()]
    if not existing:
        return
    if assume_yes:
        return
    answer = input(
        f"Import {path} and overwrite {', '.join(str(item) for item in existing)}? [y/N] "
    ).strip().lower()
    if answer not in {"y", "yes"}:
        raise SystemExit("Import cancelled; no files were changed.")


def update_table(
    path: Path,
    incoming: list[dict[str, str]],
    identity: str = "id",
) -> None:
    headers, current = read_csv(path)
    by_id = {row.get(identity, ""): row for row in current}
    for row in incoming:
        key = row.get(identity, "")
        if not key or key not in by_id:
            raise SystemExit(f"Cannot import unknown {identity} {key!r} into {path}")
        by_id[key].update({key_: value for key_, value in row.items() if key_ in headers})
    write_csv(path, headers, current)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("csv_file", type=Path)
    parser.add_argument("--dataset", type=Path, default=Path("gtr2career"))
    parser.add_argument("--yes", action="store_true", help="skip the overwrite prompt")
    args = parser.parse_args()

    headers, rows = read_csv(args.csv_file)
    kind = args.csv_file.name
    if kind == "gtr2career_vehicles.csv":
        confirm(args.csv_file, [args.dataset / "objects.csv"], args.yes)
        for row in rows:
            for index in range(1, 16):
                configured_id = f"cost_{index}_configured_id"
                if configured_id in row:
                    row[f"cost_{index}"] = row[configured_id]
        update_table(args.dataset / "objects.csv", rows)
    elif kind == "gtr2career_moneymakers.csv":
        confirm(args.csv_file, [args.dataset / "events.csv"], args.yes)
        update_table(args.dataset / "events.csv", rows)
    elif kind == "gtr2career_races_championships.csv":
        event_rows = [row for row in rows if row.get("record_type") == "race"]
        quest_rows = [row for row in rows if row.get("record_type") == "championship"]
        targets = [args.dataset / "events.csv", args.dataset / "quests.csv"]
        confirm(args.csv_file, targets, args.yes)
        if event_rows:
            update_table(args.dataset / "events.csv", event_rows)
        if quest_rows:
            update_table(args.dataset / "quests.csv", quest_rows)
    else:
        raise SystemExit(
            "CSV filename must be gtr2career_vehicles.csv, "
            "gtr2career_races_championships.csv, or gtr2career_moneymakers.csv"
        )


if __name__ == "__main__":
    main()
