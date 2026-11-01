#!/usr/bin/env python3
"""Export one selected section of the gtr2career catalog."""

from __future__ import annotations

import argparse
from pathlib import Path

from catalog_tools import MONEYMAKER_TYPES, read_csv, write_csv


def union_headers(*groups: list[str]) -> list[str]:
    result: list[str] = []
    for group in groups:
        for header in group:
            if header not in result:
                result.append(header)
    return result


def car_export_rows(
    objects: list[dict[str, str]],
    costs: dict[str, dict[str, str]],
) -> tuple[list[str], list[dict[str, str]]]:
    direct_headers = [
        header for header in objects[0]
        if header not in {"type"}
        and not header.startswith("cost_")
        and not header.startswith("service_")
    ]
    cost_headers: list[str] = []
    for index in range(1, 16):
        cost_headers.extend(
            [
                f"cost_{index}_configured_id",
                f"cost_{index}_name",
                f"cost_{index}_amount",
                f"cost_{index}_event_tags",
                f"service_{index}_interval_days",
            ]
        )
    headers = direct_headers + cost_headers
    result = []
    for source in objects:
        if source.get("type") != "vehicle":
            continue
        row = dict(source)
        weight = float(source.get("weight_kg") or 0)
        power = float(source.get("max_power_hp") or 0)
        row["power_to_weight_hp_per_tonne"] = (
            f"{power / weight * 1000:.2f}" if weight else ""
        )
        for index in range(1, 16):
            cost_id = source.get(f"cost_{index}", "")
            cost = costs.get(cost_id, {})
            row[f"cost_{index}_configured_id"] = cost_id
            row[f"cost_{index}_name"] = cost.get("name", "")
            row[f"cost_{index}_amount"] = cost.get("amount", "")
            row[f"cost_{index}_event_tags"] = cost.get("event_tags", "")
            row[f"service_{index}_interval_days"] = source.get(
                f"service_{index}_interval_days", ""
            )
        result.append(row)
    if "power_to_weight_hp_per_tonne" not in headers:
        headers.insert(headers.index("description_html"), "power_to_weight_hp_per_tonne")
    return headers, result


def championship_export_row(
    row: dict[str, str],
    rounds: list[dict[str, str]],
) -> dict[str, str]:
    related = [item for item in rounds if item.get("championship_id") == row.get("id")]
    output = {"record_type": "championship", **row}
    output.update(
        {
            "round_count": str(len(related)),
            "round_event_ids": "|".join(item.get("event_id", "") for item in related),
            "round_names": "|".join(item.get("event_name", "") for item in related),
            "round_days": "|".join(item.get("day_of_year", "") for item in related),
            "round_entry_fees": "|".join(item.get("entry_fee", "") for item in related),
            "round_reward_pools": "|".join(item.get("reward_pool", "") for item in related),
            "round_cars": "|".join(item.get("cars", "") for item in related),
            "round_points_awarded": "|".join(item.get("points_awarded", "") for item in related),
        }
    )
    return output


def object_export_rows(
    objects: list[dict[str, str]],
    object_type: str,
) -> tuple[list[str], list[dict[str, str]]]:
    rows = [row for row in objects if row.get("type") == object_type]
    headers: list[str] = []
    for row in objects[:1]:
        headers.extend(row.keys())
    return headers, rows


def choose_export() -> str:
    options = [
        ("cars", "Cars"),
        ("race_gear", "Race gear"),
        ("races_championships", "Races and championships"),
        ("moneymakers", "Money-making items"),
    ]
    print("What would you like to export?")
    for index, (_, label) in enumerate(options, start=1):
        print(f"{index}. {label}")
    while True:
        answer = input("Choose 1-4: ").strip().lower()
        if answer in {key for key, _ in options}:
            return answer
        if answer.isdigit() and 1 <= int(answer) <= len(options):
            return options[int(answer) - 1][0]
        print("Please choose 1, 2, 3, or 4.")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset", type=Path, default=Path("gtr2career"))
    parser.add_argument("--output-dir", type=Path, default=Path("."))
    parser.add_argument(
        "--export",
        choices=("cars", "race_gear", "races_championships", "moneymakers"),
        help="Skip the prompt and export this section.",
    )
    args = parser.parse_args()
    export_kind = args.export or choose_export()

    object_headers, objects = read_csv(args.dataset / "objects.csv")
    _, costs = read_csv(args.dataset / "costs.csv")
    event_headers, events = read_csv(args.dataset / "events.csv")
    quest_headers, quests = read_csv(args.dataset / "quests.csv")
    championship_headers, championships = read_csv(args.dataset / "championships.csv")

    cost_map = {row["id"]: row for row in costs}
    if export_kind == "cars":
        car_headers, car_rows = car_export_rows(objects, cost_map)
        write_csv(args.output_dir / "gtr2career_vehicles.csv", car_headers, car_rows)
        return

    if export_kind == "race_gear":
        gear_headers, gear_rows = object_export_rows(objects, "equipment")
        write_csv(args.output_dir / "gtr2career_race_gear.csv", gear_headers, gear_rows)
        return

    combined_headers = union_headers(
        ["record_type"],
        event_headers,
        quest_headers,
        championship_headers,
    )
    race_rows = [
        {"record_type": "race", **row}
        for row in events
        if row.get("type") == "race"
    ]
    championship_rows = []
    for row in quests:
        if row.get("type") != "championship":
            continue
        championship_rows.append(championship_export_row(row, championships))
    if export_kind == "races_championships":
        write_csv(
            args.output_dir / "gtr2career_races_championships.csv",
            combined_headers,
            race_rows + championship_rows,
        )
        return

    money_rows = [
        {"record_type": "moneymaker", **row}
        for row in events
        if row.get("type") in MONEYMAKER_TYPES
    ]
    write_csv(
        args.output_dir / "gtr2career_moneymakers.csv",
        union_headers(["record_type"], event_headers),
        money_rows,
    )


if __name__ == "__main__":
    main()
