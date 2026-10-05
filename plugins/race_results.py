#!/usr/bin/env python3
"""Race-results plugin for TTRPG Engine.

The protocol is one JSON request on stdin and one JSON response on stdout.
The plugin is deliberately stateless: Rust supplies the current race and the
previous championship results for every invocation.
"""

import json
import logging
import os
import re
import sys
from pathlib import Path
from collections import defaultdict

from plugin_logging import configure_logging


LOGGER = logging.getLogger("race_results")


SUCCESS_RESULTS = {"success", "successful", "win", "won", "1", "yes", "true"}


def normalize_result(value):
    return value.strip()


def is_success(value):
    return normalize_result(value).lower() in SUCCESS_RESULTS


def prize_positions(event):
    rewards = event.get("position_rewards", "")
    positions = []
    for entry in rewards.split(";"):
        match = re.match(r"\s*(\d+)\s*:", entry)
        if match:
            positions.append(int(match.group(1)))
    return max(positions, default=3)


def score(event, position, race_count):
    if position <= 0 or position > prize_positions(event):
        return 0
    return max(1, race_count - position + 1)


def validate_positions(player_position, competitors):
    positions = set()
    if player_position and player_position > 0:
        positions.add(player_position)
    normalized = []
    for competitor in competitors:
        name = str(competitor.get("name", "")).strip()
        position = int(competitor.get("position", 0) or 0)
        if not name or position <= 0:
            continue
        if position in positions:
            raise ValueError(f"Championship finishing position {position} is already assigned")
        positions.add(position)
        normalized.append({"name": name, "position": position})
    normalized.sort(key=lambda entry: entry["position"])
    return normalized


def autodetect_result(request):
    directory = Path(str(request.get("results_directory", "")).strip())
    if not directory.is_dir():
        raise ValueError(f"Results directory does not exist: {directory}")
    files = sorted(
        (path for path in directory.glob("*.txt") if path.is_file()),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    if not files:
        raise ValueError(f"No result files found in {directory}")
    text = files[0].read_text(encoding="utf-8", errors="replace")
    LOGGER.info("autodetect selected result file %s", files[0])
    player_name = str(request.get("player_name", "")).strip().lower()
    player_position = 0
    competitors = []
    for line in text.splitlines():
        match = re.match(r"\s*(\d+)\s*[,;:\t ]+\s*(.+?)\s*$", line)
        if not match:
            match = re.match(r"\s*(.+?)\s*[,;:\t ]+\s*(\d+)\s*$", line)
            if not match:
                continue
            name, raw_position = match.group(1), match.group(2)
        else:
            raw_position, name = match.group(1), match.group(2)
        try:
            position = int(raw_position)
        except ValueError:
            continue
        name = name.strip()
        if not name or position <= 0:
            continue
        if player_name and player_name in name.lower():
            player_position = position
        else:
            competitors.append({"name": name, "position": position})
    if player_position == 0 and not competitors:
        raise ValueError(f"No classified results found in {files[0].name}")
    return {
        **request,
        "result": "success" if player_position and player_position <= 3 else "failure",
        "player_position": player_position,
        "competitors": competitors,
        "damage_type": "none",
    }


def standings(events, results, current_event, player_position, competitors):
    all_results = list(results)
    if current_event and player_position and player_position > 0:
        all_results.append(
            {
                "event_id": current_event.get("id", ""),
                "player_position": player_position,
                "competitors": competitors,
            }
        )
    event_by_id = {event.get("id", ""): event for event in events}
    championship_events = [
        event for event in events
        if event.get("quest_id", "") == current_event.get("quest_id", "")
    ] if current_event else events
    race_count = len(championship_events)
    points = defaultdict(int)
    for result in all_results:
        event = event_by_id.get(result.get("event_id", ""))
        if not event:
            continue
        points["You"] += score(event, int(result.get("player_position", 0) or 0), race_count)
        for competitor in result.get("competitors", []):
            name = str(competitor.get("name", "")).strip()
            if name:
                points[name] += score(event, int(competitor.get("position", 0) or 0), race_count)
    return [
        {"name": name, "points": total}
        for name, total in sorted(points.items(), key=lambda item: (-item[1], item[0].lower()))
    ]


def process(request):
    LOGGER.info(
        "processing result event=%s interactive=%s autodetect=%s",
        (request.get("event") or {}).get("id", ""),
        bool(request.get("interactive")),
        bool(request.get("autodetect")),
    )
    if request.get("autodetect"):
        request = autodetect_result(request)
    event = request.get("event") or {}
    championship = bool(event.get("quest_id", "").strip())
    player_position = int(request.get("player_position", 0) or 0)
    competitors = validate_positions(player_position, request.get("competitors") or [])
    result = normalize_result(request.get("result", ""))
    if championship and not is_success(result) and normalize_result(result).lower() != "failure":
        result = "success"
    elif not result:
        raise ValueError("Enter an event result before submitting")
    return {
        "result": result,
        "success": is_success(result),
        "player_position": player_position,
        "competitors": competitors,
        "damage_type": normalize_result(request.get("damage_type", "none")) or "none",
        "pole_position": bool(request.get("pole_position", False)),
        "standings": standings(
            request.get("events") or [],
            request.get("previous_results") or [],
            event if championship else None,
            player_position,
            competitors,
        ) if championship else [],
    }


def _shared_theme():
    stylesheet = Path(__file__).resolve().parents[1] / "public" / "plugin.css"
    try:
        source = stylesheet.read_text(encoding="utf-8")
    except OSError:
        return {}
    return dict(re.findall(r"--([\w-]+):\s*(#[0-9a-fA-F]{6});", source))


def interactive_process(request):
    import tkinter as tk
    from tkinter import messagebox
    from html.parser import HTMLParser

    class HtmlRenderer(HTMLParser):
        def __init__(self, widget):
            super().__init__()
            self.widget = widget

        def handle_starttag(self, tag, attrs):
            if tag in {"br", "p", "div", "li", "h1", "h2", "h3"}:
                self.widget.insert("end", "\n")
            self.widget.mark_set("html_start", "end-1c")

        def handle_endtag(self, tag):
            if tag in {"p", "div", "li", "h1", "h2", "h3"}:
                self.widget.insert("end", "\n")

        def handle_data(self, data):
            self.widget.insert("end", data)

    event = request.get("event") or {}
    championship = bool(event.get("quest_id", "").strip())
    result = {"value": "", "damage_type": "none", "player_position": 0, "competitors": [], "pole_position": False}
    window = tk.Tk()
    window.title("Race results plugin")
    colors = _shared_theme()
    background = colors.get("app-background", "#0f172a")
    surface = colors.get("surface-background", "#1e293b")
    foreground = colors.get("primary-text", "#f8fafc")
    secondary = colors.get("secondary-text", "#e2e8f0")
    control = colors.get("control-background", "#334155")
    accent = colors.get("primary-accent", "#2563eb")
    window.configure(bg=background)
    window.resizable(False, False)
    frame = tk.Frame(window, bg=surface, padx=22, pady=22, highlightbackground=colors.get("surface-border", "#475569"), highlightthickness=1)
    frame.pack()
    tk.Label(
        frame,
        text="RACE RESULTS PLUGIN",
        bg=surface,
        fg=foreground,
        font=("TkDefaultFont", 14, "bold"),
    ).pack(anchor="w")
    tk.Label(
        frame,
        text=event.get("name", "Race result"),
        bg=surface,
        fg=foreground,
        font=("TkDefaultFont", 11, "bold"),
    ).pack(anchor="w", pady=(8, 2))
    description = tk.Text(
        frame, width=64, height=8, wrap="word", bg=control, fg=secondary,
        relief="flat", padx=8, pady=6,
    )
    description.pack(anchor="w", pady=(0, 10))
    description.configure(state="normal")
    HtmlRenderer(description).feed(request.get("description") or event.get("description_html", ""))
    description.configure(state="disabled")

    if championship:
        tk.Label(frame, text="Your finishing position", bg=surface, fg=foreground).pack(anchor="w")
        position_value = tk.StringVar(value="1")
        position = tk.OptionMenu(
            frame,
            position_value,
            *[str(value) for value in range(1, max(1, int(request.get("max_reward_position", 1))) + 1)],
            "further down",
            "DNF",
        )
        position.configure(width=18, bg=control, fg=secondary, activebackground=accent)
        position.pack(anchor="w", pady=(2, 8))
        tk.Label(
            frame,
            text="Other competitors, one per line as name:position",
            bg=surface,
            fg=secondary,
            wraplength=520,
            justify="left",
        ).pack(anchor="w")
        competitor_text = tk.Text(frame, width=58, height=6)
        competitor_text.pack(anchor="w", pady=(2, 8))
        competitor_text.insert("1.0", "\n".join(
            f"{entry.get('name', '')}:{entry.get('position', '')}"
            for entry in request.get("competitors", [])
            if entry.get("name")
        ))
    else:
        tk.Label(frame, text="Result", bg=surface, fg=foreground).pack(anchor="w")
        result_entry = tk.StringVar(value="1")
        result_menu = tk.OptionMenu(
            frame,
            result_entry,
            *[str(position) for position in range(1, max(1, int(request.get("max_reward_position", 1))) + 1)],
            "further down",
            "DNF",
        )
        result_menu.configure(width=18, bg=control, fg=secondary, activebackground=accent)
        result_menu.pack(anchor="w", pady=(2, 8))
        position = None

    tk.Label(frame, text="Damage type", bg=surface, fg=foreground).pack(anchor="w")
    damage_options = request.get("damage_options") or []
    damage_values = ["none"] + [str(option.get("id", "")) for option in damage_options if option.get("id")]
    damage_names = {"none": "No additional damage"}
    damage_names.update({
        str(option.get("id")): str(option.get("name") or option.get("id"))
        for option in damage_options
    })
    damage_labels = [damage_names[value] for value in damage_values]
    damage_value = tk.StringVar(value=damage_labels[0])
    damage = tk.OptionMenu(frame, damage_value, *damage_labels)
    damage.configure(width=24, bg=control, fg=secondary, activebackground=accent)
    damage.pack(anchor="w", pady=(2, 12))
    pole_value = tk.BooleanVar(value=False)
    tk.Checkbutton(
        frame,
        text="Pole position",
        variable=pole_value,
        bg=surface,
        fg=foreground,
        selectcolor=control,
        activebackground=surface,
        activeforeground=foreground,
    ).pack(anchor="w", pady=(0, 8))

    def save():
        try:
            if championship:
                selection = position_value.get().strip()
                if selection == "DNF":
                    result["value"] = "failure"
                    result["player_position"] = 0
                elif selection == "further down":
                    result["value"] = "success"
                    result["player_position"] = int(request.get("max_reward_position", 1)) + 1
                else:
                    result["value"] = "success"
                    result["player_position"] = int(selection)
                competitors = []
                for line in competitor_text.get("1.0", "end").splitlines():
                    if not line.strip():
                        continue
                    name, separator, raw_position = line.rpartition(":")
                    if not separator or not name.strip():
                        raise ValueError("Competitors must use name:position format")
                    competitors.append({"name": name.strip(), "position": int(raw_position.strip())})
                result["competitors"] = competitors
            else:
                selection = result_entry.get().strip()
                if selection == "DNF":
                    result["value"] = "failure"
                elif selection == "further down":
                    result["value"] = "success"
                    result["player_position"] = int(request.get("max_reward_position", 1)) + 1
                else:
                    result["value"] = "success"
                    result["player_position"] = int(selection)
            result["damage_type"] = next(
                (value for value in damage_values if damage_names[value] == damage_value.get()),
                "none",
            )
            result["pole_position"] = bool(pole_value.get())
            response = process({
                **request,
                "result": result["value"],
                "player_position": result["player_position"],
                "competitors": result["competitors"],
                "damage_type": result["damage_type"],
                "pole_position": result["pole_position"],
            })
            LOGGER.info("interactive result saved event=%s", event.get("id", ""))
            print(json.dumps(response), flush=True)
            window.destroy()
        except (ValueError, TypeError) as error:
            messagebox.showerror("Invalid result", str(error), parent=window)

    tk.Button(
        frame,
        text="Save result",
        command=save,
        bg=accent,
        fg=foreground,
        activebackground=colors.get("primary-accent-border", "#60a5fa"),
        padx=12,
        pady=6,
    ).pack(anchor="e")
    window.protocol("WM_DELETE_WINDOW", window.destroy)
    window.mainloop()
    return 0 if result["value"] else 1


def main():
    global LOGGER
    try:
        request = json.load(sys.stdin)
        destination = request.get("log_destination")
        if destination is None and "TTRPG_LOG_DEST" not in os.environ:
            destination = "stderr"
        LOGGER = configure_logging("race_results", destination)
        if request.get("interactive"):
            return interactive_process(request)
        response = process(request)
        json.dump(response, sys.stdout)
        sys.stdout.write("\n")
    except (ValueError, TypeError, json.JSONDecodeError) as error:
        json.dump({"error": str(error)}, sys.stdout)
        sys.stdout.write("\n")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
