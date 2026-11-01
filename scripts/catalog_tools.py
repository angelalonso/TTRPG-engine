#!/usr/bin/env python3
"""Shared CSV and HTML helpers for the gtr2career catalog tools."""

from __future__ import annotations

import csv
import html
from pathlib import Path
from typing import Iterable


MONEYMAKER_TYPES = {"work", "trade", "finance", "sponsor"}


def read_csv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        return list(reader.fieldnames or []), list(reader)


def write_csv(path: Path, headers: list[str], rows: Iterable[dict[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=headers,
            extrasaction="ignore",
            lineterminator="\n",
        )
        writer.writeheader()
        writer.writerows({header: row.get(header, "") for header in headers} for row in rows)


def money(value: str) -> str:
    try:
        number = float(value)
    except (TypeError, ValueError):
        return value or "—"
    if number.is_integer():
        return f"£{int(number):,}"
    return f"£{number:,.2f}"


def esc(value: str) -> str:
    return html.escape(value or "—")


def page(title: str, card_class: str, image: str, main: str, aside: str) -> str:
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{esc(title)}</title>
  <link rel="stylesheet" href="./gtr-theme.css">
  <link rel="stylesheet" href="./gtr-card.css">
</head>
<body class="catalog-page">
  <div class="catalog-container">
    <main class="catalog-card {card_class}">
      <div class="catalog-card-inner">
        {main}
      </div>
    </main>
    <aside class="catalog-side {card_class}">
      {aside}
    </aside>
  </div>
</body>
</html>
"""


def image_src(image_path: str, fallback: str = "./img/event_placeholder.svg") -> str:
    path = image_path or fallback
    return "../" + path[2:] if path.startswith("./") else "../" + path.lstrip("/")


def list_items(items: Iterable[tuple[str, str]]) -> str:
    return "".join(
        f'<li><span class="catalog-label">{esc(label)}</span> '
        f'<span class="catalog-value">{esc(value)}</span></li>'
        for label, value in items
    )


def event_html(row: dict[str, str], card_class: str = "silver") -> str:
    event_type = row.get("type", "").upper()
    payout = row.get("payout", "")
    if payout and row.get("payout_freq_type"):
        payout = f"{payout} / {row.get('payout_freq', '')} {row.get('payout_freq_unit', '')}"
    details = [
        ("TYPE", row.get("type", "")),
        ("DURATION", f"{row.get('duration_value', '')} {row.get('duration_unit', '')}".strip()),
        ("ENTRY", money(row.get("entry_fee", ""))),
        ("REWARD POOL", money(row.get("reward_pool", ""))),
        ("CHARISMA", row.get("charisma_reward", "")),
        ("TAGS", row.get("tags", "")),
    ]
    if row.get("position_rewards"):
        details.append(("POSITION REWARDS", row["position_rewards"]))
    if row.get("required_object_ids"):
        details.append(("REQUIRED OBJECTS", row["required_object_ids"]))
    if row.get("stamina_cost"):
        details.append(("STAMINA", row["stamina_cost"]))
    if payout:
        details.append(("PAYOUT", money(payout.split(" / ", 1)[0]) + (payout[payout.find(" / "):] if " / " in payout else "")))
    if row.get("success_rate"):
        details.append(("SUCCESS", f"{float(row['success_rate']) * 100:g}%"))
    image = image_src(row.get("image_path", ""))
    main = (
        f'<div class="catalog-image"><img src="{esc(image)}" alt="{esc(row["name"])}"></div>'
        '<section class="catalog-details"><h2>EVENT DETAILS</h2>'
        f"<ul>{list_items(details)}</ul></section>"
    )
    aside = (
        '<div class="catalog-logo-row"><img class="catalog-logo" '
        'src="../img/organizer_180_motor_club.svg" alt="180 Motor Club logo">'
        '<div><h2>180 Motor Club</h2><p>Organizer</p></div></div>'
        f'<h1>{esc(row["name"])}</h1>'
        "<p>This activity is organized by 180 Motor Club. Review the current "
        "entry details, requirements, and payout before committing to it.</p>"
    )
    return page(row["name"], card_class, image, main, aside)


def car_html(row: dict[str, str], costs: dict[str, dict[str, str]]) -> str:
    details = []
    for index in range(1, 16):
        cost_id = row.get(f"cost_{index}", "")
        if cost_id and cost_id in costs:
            cost = costs[cost_id]
            details.append((cost["name"].upper(), money(cost["amount"])))
    stats = (
        f'<section class="catalog-stats">'
        f'<div class="catalog-stat"><span class="catalog-label">PRICE</span>'
        f'<span class="catalog-value">{money(row["price"])}</span></div>'
        f'<div class="catalog-stat"><span class="catalog-label">WEIGHT</span>'
        f'<span class="catalog-value">{esc(row.get("weight_kg", ""))} kg</span></div>'
        f'<div class="catalog-stat"><span class="catalog-label">POWER</span>'
        f'<span class="catalog-value">{esc(row.get("max_power_hp", ""))} hp</span></div></section>'
    )
    main = (
        f'<div class="catalog-image"><img src="{esc(image_src(row.get("image_path", ""), "./img/" + row["id"] + ".jpeg"))}" '
        f'alt="{esc(row["name"])}"></div>{stats}'
        '<section class="catalog-details"><h2>MAINTENANCE COSTS</h2>'
        f"<ul>{list_items(details)}</ul></section>"
    )
    requirement = row.get("requires_object_ids") or "None"
    aside = f"<h1>{esc(row['name'])}</h1><p><strong>Required equipment:</strong> {esc(requirement)}</p>"
    return page(row["name"], "red", "", main, aside)


def object_html(row: dict[str, str], objects: list[dict[str, str]]) -> str:
    object_type = row.get("type", "").lower()
    if object_type == "license":
        requirements = row.get("requires_object_ids", "").replace(";", ", ")
        description = "This licence unlocks the configured events and championships at its level."
        if requirements:
            description += f" It requires: {requirements}."
        details = [
            ("LICENCE FEE", money(row.get("license_fee", ""))),
            ("LEVEL", row.get("license_level", "")),
        ]
    elif object_type == "insurance":
        insurance_terms = {
            "insurance_third_party": ("only pays for damage to other vehicles", 250, 0.015, 2000),
            "insurance_basic": ("covers damage to your vehicle up to £7,500", 500, 0.03, 7500),
            "insurance_full": ("covers damage to your vehicle with the full policy limit", 750, 0.05, 30000),
        }
        coverage, base, rate, cap = insurance_terms.get(
            row["id"], ("provides the configured insurance coverage", 0, 0, 0)
        )
        cap_text = f", capped at {money(str(cap))}" if cap else ""
        description = (
            f"{coverage.capitalize()}. The annual premium starts at {money(str(base))} "
            f"plus {rate * 100:g}% of the value of all owned cars{cap_text}."
        )
        details = [
            ("BASE PREMIUM", money(str(base))),
            ("CAR VALUE RATE", f"{rate * 100:g}%"),
            ("ANNUAL CAP", money(str(cap)) if cap else "None"),
        ]
    else:
        group = row.get("requirement_group", "").strip()
        alternatives = [
            item["name"]
            for item in objects
            if item.get("requirement_group", "").strip().lower() == group.lower()
            and item.get("type", "").lower() == "equipment"
            and item["id"] != row["id"]
        ]
        description = "This item satisfies the configured equipment requirement."
        if alternatives:
            description += f" Any {group.replace('_', ' ')} qualifies, including this item and other compatible versions."
        lifetime = int(float(row.get("lifetime_days", "0") or 0))
        if lifetime:
            description += f" Configured lifespan: {lifetime:,} days."
        cred = float(row.get("paddock_cred_bonus", "0") or 0)
        if cred:
            description += f" Provides +{cred:g} paddock credibility."
        details = [
            ("PRICE", money(row.get("price", ""))),
            ("VALID FOR", f"{lifetime:,} days" if lifetime else "Permanent"),
            ("PADDOCK CREDIBILITY", f"+{cred:g}" if cred else "None"),
        ]
    image = image_src(row.get("image_path", ""), "./img/" + row["id"] + ".jpeg")
    if object_type in {"equipment", "trailer", "insurance", "companion", "license"}:
        main = f'<div class="catalog-image"><img src="{esc(image)}" alt="{esc(row["name"])}"></div>'
        aside = (
            f"<h1>{esc(row['name'])}</h1><p>{esc(description)}</p>"
            f'<section class="catalog-details"><h2>{esc(object_type.upper())} DETAILS</h2>'
            f"<ul>{list_items(details)}</ul></section>"
        )
    else:
        main = (
            f'<div class="catalog-image"><img src="{esc(image)}" alt="{esc(row["name"])}"></div>'
            f'<section class="catalog-details"><h2>{esc(object_type.upper())} DETAILS</h2>'
            f"<ul>{list_items(details)}</ul></section>"
        )
        aside = f"<h1>{esc(row['name'])}</h1><p>{esc(description)}</p>"
    return page(row["name"], "", "", main, aside)


def championship_html(row: dict[str, str], rounds: list[dict[str, str]]) -> str:
    related = [item for item in rounds if item.get("championship_id") == row["id"]]
    details = [
        ("ENTRY FEE", money(row.get("join_fee", ""))),
        ("REWARDS", row.get("championship_rewards", "")),
        ("REQUIRED LICENCE", row.get("required_license_id", "")),
        ("ROUNDS", str(len(related))),
    ]
    round_items = "".join(
        f"<li>{esc(item.get('event_name', item.get('event_id', '')))} — result pending</li>"
        for item in related
    )
    main = (
        f'<div class="catalog-image"><img src="{esc(image_src(row.get("image_path", "")))}" '
        f'alt="{esc(row["name"])}"></div>'
        '<section class="catalog-details"><h2>ENTRY AND REWARDS</h2>'
        f"<ul>{list_items(details)}</ul></section>"
    )
    aside = (
        '<div class="catalog-logo-row"><img class="catalog-logo" '
        'src="../img/organizer_180_motor_club.svg" alt="180 Motor Club logo">'
        '<img class="catalog-logo center" src="../img/championship_placeholder.svg" alt="Championship logo"></div>'
        f'<h1>{esc(row["name"])}</h1><p>Organized by 180 Motor Club. '
        "Join the championship to unlock its configured rounds.</p>"
        f'<section class="catalog-rounds"><h2>Rounds and results</h2><ol>{round_items}</ol></section>'
        f'<section class="entry-planner" data-engine-slot="quest-plan" data-quest-id="{esc(row["id"])}">'
        "<h2>How do I enter this championship?</h2>"
        "<p>Open this page in the game for live requirements and preparation costs.</p></section>"
    )
    return page(row["name"], "gold", "", main, aside)
