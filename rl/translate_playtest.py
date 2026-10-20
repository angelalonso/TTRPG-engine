"""Translate a playtest JSON export into a readable event log."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Iterable


def display_name(value: str) -> str:
    return value.replace("_", " ").replace("-", " ").strip().capitalize()


def action_name(action_key: str) -> str:
    parts = action_key.split(":")
    action = display_name(parts[0])
    details = [display_name(part) for part in parts[1:] if part and part != "none"]
    return f"{action}: {', '.join(details)}" if details else action


def changed_values(
    before: dict[str, Any], after: dict[str, Any], key: str
) -> list[str]:
    before_values = before.get(key, [])
    after_values = after.get(key, [])
    added = sorted(set(after_values) - set(before_values))
    removed = sorted(set(before_values) - set(after_values))
    changes = [f"+{value}" for value in added]
    changes.extend(f"-{value}" for value in removed)
    return changes


def observation_changes(
    before: dict[str, Any] | None, after: dict[str, Any] | None
) -> list[str]:
    if not before or not after:
        return []
    changes: list[str] = []
    for key in ("inventory_ids", "quest_ids", "active_event_ids", "pending_event_ids"):
        values = changed_values(before, after, key)
        if values:
            changes.append(f"{display_name(key)} [{', '.join(values)}]")

    characteristic_changes = []
    before_characteristics = before.get("characteristics", {})
    after_characteristics = after.get("characteristics", {})
    for key in sorted(set(before_characteristics) | set(after_characteristics)):
        old = before_characteristics.get(key)
        new = after_characteristics.get(key)
        if old != new:
            characteristic_changes.append(f"{display_name(key)} {old:g}->{new:g}")
    if characteristic_changes:
        changes.append(f"Characteristics [{', '.join(characteristic_changes)}]")

    if before.get("current_day") != after.get("current_day"):
        changes.append(
            f"Day {before.get('current_day')}->{after.get('current_day')}"
        )
    return changes


def format_policy(decision: dict[str, Any]) -> str:
    selected = decision.get("selected_probability")
    selected_text = f"{selected:.3f}" if isinstance(selected, (int, float)) else "?"
    alternatives = decision.get("legal_actions", [])
    alternative_text = ", ".join(
        f"{action_name(item['action_key'])} ({item['probability']:.3f})"
        for item in alternatives[:5]
    )
    suffix = f"; alternatives: {alternative_text}" if alternative_text else ""
    return f"Policy {decision.get('selection', 'unknown')}, selected p={selected_text}{suffix}"


def episode_lines(episode: dict[str, Any], episode_number: int) -> Iterable[str]:
    yield (
        f"Episode {episode_number} | seed {episode.get('seed')} | "
        f"{episode.get('status', 'unknown')} | "
        f"{episode.get('steps', 0)} steps | day {episode.get('current_day', '?')}"
    )
    for event in episode.get("path", []):
        reward = event.get("reward", 0)
        reward_text = f"{reward:+.3f}" if isinstance(reward, (int, float)) else str(reward)
        line = (
            f"  {event.get('step', '?'):>4}  Day {event.get('day', '?'):<5} "
            f"{action_name(event.get('action_key', 'unknown'))} "
            f"(reward {reward_text}, +{event.get('days_elapsed', 0)} days)"
        )
        yield line

        changes = observation_changes(
            event.get("observation_before"), event.get("observation_after")
        )
        if changes:
            yield f"         Changes: {'; '.join(changes)}"
        if event.get("decision"):
            yield f"         {format_policy(event['decision'])}"
        if event.get("status") != "ongoing":
            yield f"         Result: {event.get('status')}"


def jsonl_payload(records: list[dict[str, Any]]) -> dict[str, Any]:
    grouped: dict[int, dict[str, Any]] = {}
    for record in records:
        episode_number = int(record["episode"])
        episode = grouped.setdefault(
            episode_number,
            {
                "seed": record.get("seed"),
                "status": "unknown",
                "current_day": "?",
                "steps": 0,
                "path": [],
            },
        )
        next_numeric = record.get("next_observation", {}).get("numeric", [])
        current_day = int(next_numeric[-4]) if len(next_numeric) >= 4 else "?"
        info = record.get("info", {})
        episode["status"] = str(info.get("status", "unknown")).lower()
        episode["current_day"] = current_day
        episode["steps"] += 1
        episode["path"].append(
            {
                "step": episode["steps"],
                "day": (
                    int(record["observation"]["numeric"][-4])
                    if len(record.get("observation", {}).get("numeric", [])) >= 4
                    else "?"
                ),
                "action_id": record.get("action"),
                "action_key": info.get(
                    "action_key", f"action_id:{record.get('action', '?')}"
                ),
                "reward": record.get("reward", 0),
                "days_elapsed": info.get("days_elapsed", 0),
                "status": str(info.get("status", "ongoing")).lower(),
            }
        )
    return {"episodes": [grouped[index] for index in sorted(grouped)]}


def load_payload(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        records = [
            json.loads(line)
            for line in text.splitlines()
            if line.strip()
        ]
        if not records or not all("episode" in record for record in records):
            raise SystemExit(f"{path} is neither playtest JSON nor trajectory JSONL")
        return jsonl_payload(records)
    if not isinstance(payload, dict) or not isinstance(payload.get("episodes"), list):
        raise SystemExit(f"{path} does not contain a playtest episode export")
    return payload


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Translate an rl/playtest.py JSON export into a readable event log."
    )
    parser.add_argument("input", type=Path)
    parser.add_argument(
        "--episode",
        type=int,
        help="Translate only this zero-based episode (default: all episodes)",
    )
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    payload = load_payload(args.input)
    episodes = payload.get("episodes", [])
    if args.episode is not None:
        if args.episode < 0 or args.episode >= len(episodes):
            raise SystemExit(f"episode index {args.episode} is outside 0..{len(episodes) - 1}")
        episodes = [episodes[args.episode]]

    lines: list[str] = []
    for index, episode in enumerate(episodes):
        if index:
            lines.append("")
        lines.extend(episode_lines(episode, args.episode if args.episode is not None else index))

    result = "\n".join(lines) + "\n"
    if args.output:
        args.output.write_text(result, encoding="utf-8")
    else:
        print(result, end="")


if __name__ == "__main__":
    main()
