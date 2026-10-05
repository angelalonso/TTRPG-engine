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


def _race_time_seconds(value):
    match = re.fullmatch(r"\s*(?:(\d+):)?(\d+):(\d+(?:\.\d+)?)\s*", value)
    if not match:
        return None
    hours = int(match.group(1) or 0)
    minutes = int(match.group(2))
    seconds = float(match.group(3))
    return hours * 3600 + minutes * 60 + seconds


def _parse_gtr2_race(text):
    race_fields = {}
    race_match = re.search(r"(?ms)^\[Race\]\s*(.*?)(?=^\[|\Z)", text)
    if race_match:
        for line in race_match.group(1).splitlines():
            key, separator, value = line.partition("=")
            if separator:
                race_fields[key.strip()] = value.strip()

    racers = []
    for block_match in re.finditer(r"(?ms)^\[Slot(\d+)\]\s*(.*?)(?=^\[Slot\d+\]|\Z)", text):
        fields = {}
        for line in block_match.group(2).splitlines():
            key, separator, value = line.partition("=")
            if separator:
                fields[key.strip()] = value.strip()
        driver = fields.get("Driver", "").strip()
        if not driver:
            continue
        race_time = _race_time_seconds(fields.get("RaceTime", ""))
        try:
            laps = int(fields.get("Laps", "0") or 0)
        except ValueError:
            laps = 0
        racers.append({
            "slot": block_match.group(1),
            "position": 0,
            "Driver": driver,
            "Vehicle": fields.get("Vehicle", ""),
            "VehicleNumber": fields.get("VehicleNumber", ""),
            "Team": fields.get("Team", ""),
            "Penalty": fields.get("Penalty", "0"),
            "Laps": fields.get("Laps", "0"),
            "LapDistanceTravelled": fields.get("LapDistanceTravelled", ""),
            "BestLap": fields.get("BestLap", ""),
            "RaceTime": fields.get("RaceTime", ""),
            "QualTime": fields.get("QualTime", ""),
            "Reason": fields.get("Reason", ""),
            "_race_time_seconds": race_time,
            "_laps": laps,
        })

    racers.sort(
        key=lambda racer: (
            racer["_race_time_seconds"] is None,
            -racer["_laps"],
            racer["_race_time_seconds"] if racer["_race_time_seconds"] is not None else float("inf"),
            racer["slot"],
        )
    )
    for position, racer in enumerate(racers, start=1):
        racer["position"] = position
        racer.pop("_race_time_seconds", None)
        racer.pop("_laps", None)
    return {
        "track_id": race_fields.get("Scene", ""),
        "aidb": race_fields.get("AIDB", ""),
        "racers": racers,
    }


def _parse_gtr2_results(text):
    return [
        {"name": racer["Driver"], "position": racer["position"]}
        for racer in _parse_gtr2_race(text)["racers"]
    ]


def _results_directory(request):
    configured = str(request.get("results_directory", "")).strip()
    if configured:
        return Path(configured)
    game_directory = str(request.get("game_directory", "")).strip()
    return Path(game_directory) / "UserData" / "Log" / "Results"


def autodetect_result(request):
    directory = _results_directory(request)
    if not directory.is_dir():
        raise ValueError(f"Results directory does not exist: {directory}")
    files = sorted(
        (path for path in directory.glob("*.txt") if path.is_file()),
        key=lambda path: (path.stat().st_mtime, path.name.casefold()),
        reverse=True,
    )
    if not files:
        raise ValueError(f"No result files found in {directory}")
    text = files[0].read_text(encoding="utf-8", errors="replace")
    LOGGER.info("autodetect selected result file %s", files[0])
    parsed_race = _parse_gtr2_race(text)
    parsed = [
        {"name": racer["Driver"], "position": racer["position"]}
        for racer in parsed_race["racers"]
    ]
    if not parsed:
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
            if name.strip() and position > 0:
                parsed.append({"name": name.strip(), "position": position})
    player_name = str(request.get("player_name", "")).strip().casefold()
    player_position = next(
        (
            entry["position"]
            for entry in parsed
            if player_name and player_name in entry["name"].casefold()
        ),
        0,
    )
    competitors = [
        entry for entry in parsed
        if not (player_name and player_name in entry["name"].casefold())
    ]
    racers = parsed_race["racers"]
    if player_position == 0 and not competitors:
        raise ValueError(f"No classified results found in {files[0].name}")
    return {
        **request,
        "result": "success" if player_position and player_position <= 3 else "failure",
        "player_position": player_position,
        "competitors": competitors,
        "track_id": parsed_race["track_id"],
        "aidb": parsed_race["aidb"],
        "racers": racers,
        "damage_type": "none",
        "detected_file": str(files[0]),
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
        "track_id": str(request.get("track_id", "")),
        "racers": request.get("racers") or [],
        **({"detected_file": request["detected_file"]} if request.get("detected_file") else {}),
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
    result = {
        "value": "",
        "damage_type": "none",
        "player_position": 0,
        "competitors": [],
        "pole_position": False,
        "track_id": "",
        "racers": [],
    }
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
    window.resizable(True, True)
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

    tk.Label(frame, text="Track ID (editable)", bg=surface, fg=foreground).pack(anchor="w")
    track_value = tk.StringVar()
    tk.Entry(frame, textvariable=track_value, width=92, bg=control, fg=secondary, insertbackground=foreground).pack(
        anchor="w", pady=(2, 8)
    )
    tk.Label(
        frame,
        text="Imported racers. Select the player on the left; every value can be corrected before saving.",
        bg=surface,
        fg=secondary,
        wraplength=1200,
        justify="left",
    ).pack(anchor="w")
    racer_canvas = tk.Canvas(frame, bg=surface, highlightthickness=0, height=360)
    racer_scrollbar = tk.Scrollbar(frame, orient="vertical", command=racer_canvas.yview)
    racer_form = tk.Frame(racer_canvas, bg=surface)
    racer_canvas.configure(yscrollcommand=racer_scrollbar.set)
    racer_canvas.pack(side="left", fill="both", expand=True, pady=(2, 8))
    racer_scrollbar.pack(side="right", fill="y", pady=(2, 8))
    racer_canvas.create_window((0, 0), window=racer_form, anchor="nw")
    racer_form.bind("<Configure>", lambda _: racer_canvas.configure(scrollregion=racer_canvas.bbox("all")))
    player_value = tk.StringVar(value="")
    racer_rows = []
    racer_fields = [
        ("position", "Pos", 6),
        ("Driver", "Driver", 22),
        ("Vehicle", "Vehicle", 22),
        ("VehicleNumber", "Number", 10),
        ("Team", "Team", 24),
        ("Penalty", "Penalty", 8),
        ("Laps", "Laps", 6),
        ("LapDistanceTravelled", "Distance", 14),
        ("BestLap", "Best lap", 11),
        ("RaceTime", "Race time", 13),
        ("QualTime", "Qual time", 11),
        ("Reason", "Reason", 8),
    ]

    def render_racers(racers):
        for child in racer_form.winfo_children():
            child.destroy()
        racer_rows.clear()
        tk.Label(racer_form, text="Player", bg=surface, fg=foreground).grid(row=0, column=0, padx=2, sticky="w")
        for column, (_, label, _) in enumerate(racer_fields, start=1):
            tk.Label(racer_form, text=label, bg=surface, fg=foreground).grid(row=0, column=column, padx=2, sticky="w")
        for row_index, racer in enumerate(racers, start=1):
            variables = {}
            tk.Radiobutton(
                racer_form,
                variable=player_value,
                value=str(row_index - 1),
                bg=surface,
                fg=foreground,
                selectcolor=control,
                activebackground=surface,
                activeforeground=foreground,
            ).grid(row=row_index, column=0, padx=2)
            for column, (field, _, width) in enumerate(racer_fields, start=1):
                variable = tk.StringVar(value=str(racer.get(field, "")))
                variables[field] = variable
                tk.Entry(
                    racer_form,
                    textvariable=variable,
                    width=width,
                    bg=control,
                    fg=secondary,
                    insertbackground=foreground,
                ).grid(row=row_index, column=column, padx=2, pady=1)
            racer_rows.append(variables)
        if racer_rows:
            player_value.set("0")
        racer_canvas.configure(scrollregion=racer_canvas.bbox("all"))
    detected_file_var = tk.StringVar()
    tk.Label(
        frame,
        textvariable=detected_file_var,
        bg=surface,
        fg=secondary,
        wraplength=520,
        justify="left",
    ).pack(anchor="w", pady=(0, 8))

    def read_latest_gtr2_result():
        try:
            detected = autodetect_result(request)
            detected_position = int(detected.get("player_position", 0) or 0)
            max_position = int(request.get("max_reward_position", 1) or 1)
            if championship:
                position_value.set(
                    "DNF"
                    if detected_position <= 0
                    else str(detected_position)
                    if detected_position <= max_position
                    else "further down"
                )
                competitor_text.delete("1.0", "end")
                competitor_text.insert(
                    "1.0",
                    "\n".join(
                        f"{entry['name']}:{entry['position']}"
                        for entry in detected.get("competitors", [])
                    ),
                )
            else:
                result_entry.set(
                    "DNF"
                    if detected_position <= 0
                    else str(detected_position)
                    if detected_position <= max_position
                    else "further down"
                )
            damage_value.set(damage_names["none"])
            pole_value.set(False)
            detected_file_var.set(
                f"Loaded {Path(detected['detected_file']).name}. Review the values, then confirm."
            )
            track_value.set(detected.get("track_id", ""))
            render_racers(detected.get("racers", []))
            detected_player = str(request.get("player_name", "")).casefold()
            for index, racer in enumerate(detected.get("racers", [])):
                if str(racer.get("Driver", "")).casefold() == detected_player:
                    player_value.set(str(index))
                    break
            LOGGER.info("loaded GTR2 result into interactive form from %s", detected["detected_file"])
        except (OSError, ValueError, TypeError) as error:
            messagebox.showerror("GTR2 import failed", str(error), parent=window)

    def save():
        try:
            if racer_rows:
                selected_index = int(player_value.get())
                if selected_index < 0 or selected_index >= len(racer_rows):
                    raise ValueError("Select which imported racer is the player")
                imported_racers = []
                for variables in racer_rows:
                    racer = {field: variable.get().strip() for field, variable in variables.items()}
                    racer["position"] = int(racer["position"] or 0)
                    imported_racers.append(racer)
                selected = imported_racers[selected_index]
                selected_is_dnf = selected.get("RaceTime", "").strip().upper() == "DNF"
                result["player_position"] = 0 if selected_is_dnf else selected["position"]
                result["value"] = "failure" if selected_is_dnf else "success"
                result["competitors"] = [
                    {"name": racer["Driver"], "position": racer["position"]}
                    for index, racer in enumerate(imported_racers)
                    if index != selected_index and racer["Driver"] and racer["position"] > 0
                ]
                result["racers"] = imported_racers
                result["track_id"] = track_value.get().strip()
            elif championship:
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
                "track_id": result["track_id"],
                "racers": result["racers"],
            })
            LOGGER.info("interactive result saved event=%s", event.get("id", ""))
            print(json.dumps(response), flush=True)
            window.destroy()
        except (ValueError, TypeError) as error:
            messagebox.showerror("Invalid result", str(error), parent=window)

    tk.Button(
        frame,
        text="Read latest GTR2 result",
        command=read_latest_gtr2_result,
        bg=control,
        fg=foreground,
        activebackground=colors.get("primary-accent-border", "#60a5fa"),
        padx=12,
        pady=6,
    ).pack(anchor="e", pady=(0, 6))
    tk.Button(
        frame,
        text="Confirm & Save",
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
