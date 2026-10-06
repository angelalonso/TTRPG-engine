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

try:
    from plugin_logging import configure_logging
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "plugins"))
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
    requested_file = str(request.get("results_file", "")).strip()
    uploaded_text = request.get("results_text")
    if isinstance(uploaded_text, str):
        text = uploaded_text
        detected_name = str(request.get("results_name", "uploaded result"))
        LOGGER.info("autodetect parsing uploaded result %s", detected_name)
    elif requested_file:
        selected_file = Path(requested_file)
        if not selected_file.is_file():
            raise ValueError(f"Selected result file does not exist: {selected_file}")
        files = [selected_file]
        text = selected_file.read_text(encoding="utf-8", errors="replace")
        detected_name = str(selected_file)
    elif not directory.is_dir():
        raise ValueError(f"Results directory does not exist: {directory}")
    else:
        files = sorted(
            (path for path in directory.glob("*.txt") if path.is_file()),
            key=lambda path: (path.stat().st_mtime, path.name.casefold()),
            reverse=True,
        )
        if not files:
            raise ValueError(f"No result files found in {directory}")
        text = files[0].read_text(encoding="utf-8", errors="replace")
        detected_name = str(files[0])
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
    remembered_names = [
        str(name).strip().casefold()
        for name in request.get("driver_names", [])
        if str(name).strip()
    ]
    remembered_name = next(
        (
            entry["name"]
            for entry in parsed
            if any(name in entry["name"].casefold() for name in remembered_names)
        ),
        "",
    )
    if not player_name and remembered_name:
        player_name = remembered_name.casefold()
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
        raise ValueError(f"No classified results found in {detected_name}")
    return {
        **request,
        "result": "success" if player_position and player_position <= 3 else "failure",
        "player_position": player_position,
        "competitors": competitors,
        "track_id": parsed_race["track_id"],
        "aidb": parsed_race["aidb"],
        "racers": racers,
        "damage_type": "none",
        "detected_file": detected_name,
        "driver_name": next(
            (entry["name"] for entry in parsed if player_name and player_name in entry["name"].casefold()),
            "",
        ),
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
        "driver_name": str(request.get("driver_name", "")),
        **({"detected_file": request["detected_file"]} if request.get("detected_file") else {}),
    }


def _shared_theme():
    """Read the shared colour tokens from public/plugin.css when it can be found."""
    here = Path(__file__).resolve()
    for base in list(here.parents)[1:3]:
        stylesheet = base / "public" / "plugin.css"
        try:
            source = stylesheet.read_text(encoding="utf-8")
        except OSError:
            continue
        return dict(re.findall(r"--([\w-]+):\s*(#[0-9a-fA-F]{6});", source))
    return {}


RACER_FIELDS = (
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
)


def _interactive_process_tk(request):
    """Native tkinter/ttk window (no web browser, no local web server)."""
    import tkinter as tk
    from tkinter import filedialog, ttk

    event = request.get("event") or {}
    championship = bool(str(event.get("quest_id", "")).strip())
    max_position = max(1, int(request.get("max_reward_position", 1) or 1))
    position_choices = [str(value) for value in range(1, max_position + 1)] + ["further down", "DNF"]

    result = {
        "value": "",
        "damage_type": "none",
        "player_position": 0,
        "competitors": [],
        "pole_position": False,
        "track_id": "",
        "racers": [],
        "driver_name": "",
    }

    colors = _shared_theme()
    background = colors.get("app-background", "#0f172a")
    surface = colors.get("surface-background", "#1e293b")
    foreground = colors.get("primary-text", "#f8fafc")
    secondary = colors.get("secondary-text", "#e2e8f0")
    subtle = colors.get("subtle-text", "#94a3b8")
    danger = colors.get("danger-text", "#f87171")
    border = colors.get("control-border", "#7b8794")
    control = colors.get("control-background", "#334155")
    accent = colors.get("primary-accent", "#2563eb")

    window = tk.Tk()
    window.title("TTRPG Engine - Race Results")
    window.geometry("1180x820")
    window.minsize(760, 600)
    window.configure(background=background)
    window.resizable(True, True)

    style = ttk.Style(window)
    style.theme_use("clam")
    style.configure("Plugin.TFrame", background=surface)
    style.configure("Plugin.TLabel", background=surface, foreground=foreground)
    style.configure("Plugin.Subtle.TLabel", background=surface, foreground=subtle)
    style.configure("Plugin.Heading.TLabel", background=surface, foreground=foreground, font=("TkDefaultFont", 12, "bold"))
    style.configure("Plugin.Title.TLabel", background=surface, foreground=foreground, font=("TkDefaultFont", 24, "bold"))
    style.configure("Plugin.Tier.TLabel", background=surface, foreground=secondary, font=("TkDefaultFont", 11))
    style.configure("Plugin.Counter.TLabel", background=surface, foreground=danger)
    style.configure("Plugin.TButton", background=control, foreground=secondary, bordercolor=border, padding=(12, 7))
    style.map("Plugin.TButton", background=[("active", accent)])
    style.configure("Plugin.Accent.TButton", background=accent, foreground=foreground, bordercolor=border, padding=(12, 7))
    style.map("Plugin.Accent.TButton", background=[("active", colors.get("primary-accent-border", "#60a5fa"))])
    style.configure("Plugin.TEntry", fieldbackground=control, foreground=foreground)
    style.configure(
        "Plugin.TCombobox",
        fieldbackground=control,
        background=control,
        foreground=foreground,
        arrowcolor=foreground,
    )
    style.map(
        "Plugin.TCombobox",
        fieldbackground=[("readonly", control), ("disabled", control)],
        background=[("readonly", control), ("active", control)],
        foreground=[("readonly", foreground), ("disabled", subtle)],
    )
    style.configure("Plugin.TCheckbutton", background=surface, foreground=foreground)
    style.map("Plugin.TCheckbutton", background=[("active", surface)], indicatorcolor=[("selected", accent)])
    style.configure("Plugin.TRadiobutton", background=surface, foreground=foreground)
    style.map("Plugin.TRadiobutton", background=[("active", surface)], indicatorcolor=[("selected", accent)])
    style.configure("Plugin.Vertical.TScrollbar", background=control, troughcolor=surface, bordercolor=border, arrowcolor=foreground)
    style.configure("Plugin.Horizontal.TScrollbar", background=control, troughcolor=surface, bordercolor=border, arrowcolor=foreground)
    window.option_add("*TCombobox*Listbox.background", control)
    window.option_add("*TCombobox*Listbox.foreground", foreground)
    window.option_add("*TCombobox*Listbox.selectBackground", accent)
    window.option_add("*TCombobox*Listbox.selectForeground", foreground)

    outer = ttk.Frame(window, style="Plugin.TFrame", padding=2)
    outer.pack(fill="both", expand=True, padx=16, pady=16)
    content = ttk.Frame(outer, style="Plugin.TFrame", padding=20)
    content.pack(fill="both", expand=True)

    status_var = tk.StringVar()
    detected_file_var = tk.StringVar()
    position_var = tk.StringVar(value="1")
    damage_var = tk.StringVar()
    pole_var = tk.BooleanVar(value=False)
    track_var = tk.StringVar()
    player_var = tk.StringVar(value="")
    racer_rows = []
    imported_result = {"loaded": False}
    saved = {"done": False}

    # ---- header -----------------------------------------------------------
    header = ttk.Frame(content, style="Plugin.TFrame")
    header.pack(fill="x")
    ttk.Label(header, text="RACE RESULTS", style="Plugin.Title.TLabel").pack(anchor="w")
    ttk.Label(header, text=event.get("name", "Race result"), style="Plugin.Heading.TLabel").pack(anchor="w", pady=(2, 0))
    ttk.Label(
        header,
        text="CHAMPIONSHIP ROUND" if championship else "RACE RESULT",
        style="Plugin.Tier.TLabel",
    ).pack(anchor="w", pady=(2, 10))

    # ---- bottom action bar (packed first so it is never pushed off-screen)
    actions = ttk.Frame(content, style="Plugin.TFrame")
    actions.pack(side="bottom", fill="x", pady=(10, 0))

    # ---- toolbar ----------------------------------------------------------
    toolbar = ttk.Frame(content, style="Plugin.TFrame")
    toolbar.pack(fill="x", pady=(0, 8))

    # ---- status line ------------------------------------------------------
    status_label = ttk.Label(
        content, textvariable=status_var, style="Plugin.Counter.TLabel", wraplength=1080, justify="left"
    )
    status_label.pack(fill="x", pady=(0, 4))

    # ---- result form ------------------------------------------------------
    form = ttk.Frame(content, style="Plugin.TFrame")
    form.pack(fill="x", pady=(4, 6))
    form.columnconfigure(0, weight=1)
    form.columnconfigure(1, weight=1)
    form.columnconfigure(2, weight=2)

    ttk.Label(
        form,
        text="Your finishing position" if championship else "Result",
        style="Plugin.TLabel",
    ).grid(row=0, column=0, sticky="w")
    ttk.Label(form, text="Damage type", style="Plugin.TLabel").grid(row=0, column=1, sticky="w", padx=(12, 0))
    ttk.Label(form, text="Track ID (double check this is the right file)", style="Plugin.TLabel").grid(
        row=0, column=2, sticky="w", padx=(12, 0)
    )

    position_menu = ttk.Combobox(
        form,
        textvariable=position_var,
        values=position_choices,
        state="readonly",
        width=18,
        style="Plugin.TCombobox",
    )
    position_menu.grid(row=1, column=0, sticky="w", pady=(2, 8))

    damage_options = request.get("damage_options") or []
    damage_values = ["none"] + [str(option.get("id", "")) for option in damage_options if option.get("id")]
    damage_names = {"none": "No additional damage"}
    damage_names.update({
        str(option.get("id")): str(option.get("name") or option.get("id"))
        for option in damage_options
        if option.get("id")
    })
    damage_labels = [damage_names[value] for value in damage_values]
    damage_var.set(damage_labels[0])
    ttk.Combobox(
        form,
        textvariable=damage_var,
        values=damage_labels,
        state="readonly",
        width=28,
        style="Plugin.TCombobox",
    ).grid(row=1, column=1, sticky="w", padx=(12, 0), pady=(2, 8))

    ttk.Entry(form, textvariable=track_var, style="Plugin.TEntry").grid(
        row=1, column=2, sticky="ew", padx=(12, 0), pady=(2, 8)
    )
    ttk.Checkbutton(form, text="Pole position", variable=pole_var, style="Plugin.TCheckbutton").grid(
        row=2, column=0, sticky="w", pady=(0, 6)
    )

    competitor_text = None
    if championship:
        ttk.Label(
            form,
            text="Other competitors, one per line as name:position",
            style="Plugin.Subtle.TLabel",
        ).grid(row=3, column=0, columnspan=3, sticky="w")
        competitor_text = tk.Text(
            form,
            width=58,
            height=5,
            background=control,
            foreground=foreground,
            insertbackground=foreground,
            relief="flat",
            highlightthickness=1,
            highlightbackground=border,
        )
        competitor_text.grid(row=4, column=0, columnspan=3, sticky="ew", pady=(2, 4))
        competitor_text.insert("1.0", "\n".join(
            f"{entry.get('name', '')}:{entry.get('position', '')}"
            for entry in request.get("competitors", [])
            if entry.get("name")
        ))

    def set_competitors(text):
        if competitor_text is None:
            return
        previous = str(competitor_text.cget("state"))
        competitor_text.configure(state="normal")
        competitor_text.delete("1.0", "end")
        competitor_text.insert("1.0", text)
        competitor_text.configure(state=previous)

    # ---- racers table -----------------------------------------------------
    ttk.Label(
        content,
        text="Imported racers. Select the player on the left; every value can be corrected before saving.",
        style="Plugin.Subtle.TLabel",
        wraplength=1080,
        justify="left",
    ).pack(anchor="w", pady=(4, 0))
    ttk.Label(content, textvariable=detected_file_var, style="Plugin.Tier.TLabel", wraplength=1080, justify="left").pack(
        anchor="w", pady=(0, 2)
    )

    table = ttk.Frame(content, style="Plugin.TFrame")
    table.pack(fill="both", expand=True, pady=(2, 4))
    table.rowconfigure(0, weight=1)
    table.columnconfigure(0, weight=1)
    racer_canvas = tk.Canvas(table, background=surface, highlightthickness=0, height=240)
    racer_vscroll = ttk.Scrollbar(table, orient="vertical", command=racer_canvas.yview, style="Plugin.Vertical.TScrollbar")
    racer_hscroll = ttk.Scrollbar(table, orient="horizontal", command=racer_canvas.xview, style="Plugin.Horizontal.TScrollbar")
    racer_canvas.configure(yscrollcommand=racer_vscroll.set, xscrollcommand=racer_hscroll.set)
    racer_canvas.grid(row=0, column=0, sticky="nsew")
    racer_vscroll.grid(row=0, column=1, sticky="ns")
    racer_hscroll.grid(row=1, column=0, sticky="ew")
    racer_form = ttk.Frame(racer_canvas, style="Plugin.TFrame")
    racer_canvas.create_window((0, 0), window=racer_form, anchor="nw")
    racer_form.bind("<Configure>", lambda _event: racer_canvas.configure(scrollregion=racer_canvas.bbox("all")))

    def _wheel(event):
        if getattr(event, "num", None) == 4:
            racer_canvas.yview_scroll(-1, "units")
        elif getattr(event, "num", None) == 5:
            racer_canvas.yview_scroll(1, "units")
        elif event.delta:
            racer_canvas.yview_scroll(-1 if event.delta > 0 else 1, "units")

    def _bind_wheel(_event):
        window.bind_all("<MouseWheel>", _wheel)
        window.bind_all("<Button-4>", _wheel)
        window.bind_all("<Button-5>", _wheel)

    def _unbind_wheel(_event):
        window.unbind_all("<MouseWheel>")
        window.unbind_all("<Button-4>")
        window.unbind_all("<Button-5>")

    racer_canvas.bind("<Enter>", _bind_wheel)
    racer_canvas.bind("<Leave>", _unbind_wheel)

    def render_racers(racers):
        for child in racer_form.winfo_children():
            child.destroy()
        racer_rows.clear()
        ttk.Label(racer_form, text="Player", style="Plugin.TLabel").grid(row=0, column=0, padx=2, sticky="w")
        for column, (_, label, _) in enumerate(RACER_FIELDS, start=1):
            ttk.Label(racer_form, text=label, style="Plugin.TLabel").grid(row=0, column=column, padx=2, sticky="w")
        for row_index, racer in enumerate(racers, start=1):
            variables = {}
            ttk.Radiobutton(
                racer_form,
                variable=player_var,
                value=str(row_index - 1),
                style="Plugin.TRadiobutton",
            ).grid(row=row_index, column=0, padx=2)
            for column, (field, _, width) in enumerate(RACER_FIELDS, start=1):
                variable = tk.StringVar(value=str(racer.get(field, "")))
                variables[field] = variable
                ttk.Entry(racer_form, textvariable=variable, width=width, style="Plugin.TEntry").grid(
                    row=row_index, column=column, padx=2, pady=1
                )
            racer_rows.append(variables)
        if racer_rows:
            player_var.set("0")
        racer_canvas.update_idletasks()
        racer_canvas.configure(scrollregion=racer_canvas.bbox("all"))

    # ---- behaviour --------------------------------------------------------
    def set_import_lock(locked):
        """While a result file is loaded, its racer table replaces the manual inputs."""
        if championship:
            position_menu.configure(state="disabled" if locked else "readonly")
            if competitor_text is not None:
                competitor_text.configure(state="disabled" if locked else "normal")

    def read_result_file(results_file=""):
        try:
            detected = autodetect_result({
                **request,
                **({"results_file": results_file} if results_file else {}),
            })
            detected_position = int(detected.get("player_position", 0) or 0)
            position_var.set(
                "DNF"
                if detected_position <= 0
                else str(detected_position)
                if detected_position <= max_position
                else "further down"
            )
            if championship:
                set_competitors("\n".join(
                    f"{entry['name']}:{entry['position']}"
                    for entry in detected.get("competitors", [])
                ))
            damage_var.set(damage_names["none"])
            pole_var.set(False)
            detected_file_var.set(
                f"Loaded {Path(detected['detected_file']).name}. Review the values, then confirm."
            )
            track_var.set(detected.get("track_id", ""))
            render_racers(detected.get("racers", []))
            detected_players = [
                str(request.get("player_name", "")).casefold(),
                *[
                    str(name).casefold()
                    for name in request.get("driver_names", [])
                    if str(name).strip()
                ],
            ]
            for index, racer in enumerate(detected.get("racers", [])):
                driver = str(racer.get("Driver", "")).casefold()
                if any(name and name in driver for name in detected_players):
                    player_var.set(str(index))
                    break
            LOGGER.info("loaded GTR2 result into interactive form from %s", detected["detected_file"])
            imported_result["loaded"] = True
            set_import_lock(True)
            status_var.set("")
        except (OSError, ValueError, TypeError) as error:
            status_var.set(f"GTR2 import failed: {error}")

    def choose_result_file():
        selected_file = filedialog.askopenfilename(
            parent=window,
            title="Choose GTR2 result file",
            initialdir=str(_results_directory(request)),
            filetypes=[
                ("GTR2 result files", "*.txt"),
                ("All files", "*.*"),
            ],
        )
        if selected_file:
            read_result_file(selected_file)

    def reset_import():
        imported_result["loaded"] = False
        result["driver_name"] = ""
        detected_file_var.set("")
        track_var.set("")
        player_var.set("")
        pole_var.set(False)
        status_var.set("")
        render_racers([])
        set_import_lock(False)
        position_var.set("1")
        set_competitors("")

    def save():
        try:
            if racer_rows:
                selected_text = player_var.get()
                if not selected_text:
                    raise ValueError("Select which imported racer is the player")
                selected_index = int(selected_text)
                if selected_index < 0 or selected_index >= len(racer_rows):
                    raise ValueError("Select which imported racer is the player")
                imported_racers = []
                for variables in racer_rows:
                    racer = {field: variable.get().strip() for field, variable in variables.items()}
                    racer["position"] = int(racer["position"] or 0)
                    imported_racers.append(racer)
                selected = imported_racers[selected_index]
                result["driver_name"] = selected.get("Driver", "").strip()
                selected_is_dnf = selected.get("RaceTime", "").strip().upper() == "DNF"
                result["player_position"] = 0 if selected_is_dnf else selected["position"]
                result["value"] = "failure" if selected_is_dnf else "success"
                result["competitors"] = [
                    {"name": racer["Driver"], "position": racer["position"]}
                    for index, racer in enumerate(imported_racers)
                    if index != selected_index and racer["Driver"] and racer["position"] > 0
                ]
                result["racers"] = imported_racers
            else:
                selection = position_var.get().strip()
                if selection == "DNF":
                    result["value"] = "failure"
                    result["player_position"] = 0
                elif selection == "further down":
                    result["value"] = "success"
                    result["player_position"] = max_position + 1
                else:
                    result["value"] = "success"
                    result["player_position"] = int(selection)
                competitors = []
                if championship and competitor_text is not None:
                    for line in competitor_text.get("1.0", "end").splitlines():
                        if not line.strip():
                            continue
                        name, separator, raw_position = line.rpartition(":")
                        if not separator or not name.strip():
                            raise ValueError("Competitors must use name:position format")
                        competitors.append({"name": name.strip(), "position": int(raw_position.strip())})
                result["competitors"] = competitors
                result["racers"] = []
            result["track_id"] = track_var.get().strip()
            result["damage_type"] = next(
                (value for value in damage_values if damage_names[value] == damage_var.get()),
                "none",
            )
            result["pole_position"] = bool(pole_var.get())
            response = process({
                **request,
                "result": result["value"],
                "player_position": result["player_position"],
                "competitors": result["competitors"],
                "damage_type": result["damage_type"],
                "pole_position": result["pole_position"],
                "track_id": result["track_id"],
                "racers": result["racers"],
                "driver_name": result.get("driver_name", ""),
            })
            LOGGER.info("interactive result saved event=%s", event.get("id", ""))
            print(json.dumps(response), flush=True)
            saved["done"] = True
            window.destroy()
        except (ValueError, TypeError) as error:
            status_var.set(f"Invalid result: {error}")

    # ---- buttons ----------------------------------------------------------
    ttk.Button(toolbar, text="Read latest GTR2 result", style="Plugin.TButton", command=read_result_file).pack(side="left")
    ttk.Button(toolbar, text="Choose result file", style="Plugin.TButton", command=choose_result_file).pack(
        side="left", padx=(8, 0)
    )
    ttk.Button(toolbar, text="Reset imported result", style="Plugin.TButton", command=reset_import).pack(
        side="left", padx=(8, 0)
    )
    ttk.Button(actions, text="Confirm & Save", style="Plugin.Accent.TButton", command=save).pack(side="right")
    ttk.Button(actions, text="Close", style="Plugin.TButton", command=window.destroy).pack(side="right", padx=(0, 8))

    render_racers([])
    window.protocol("WM_DELETE_WINDOW", window.destroy)
    window.mainloop()
    return 0 if saved["done"] else 1


def interactive_process(request):
    try:
        return _interactive_process_tk(request)
    except ImportError as error:
        raise ValueError(f"tkinter is not available, cannot open the race results window: {error}")
    except Exception as error:  # tkinter.TclError (e.g. no display) is not a ValueError
        if type(error).__name__ == "TclError":
            raise ValueError(f"could not open the race results window: {error}")
        raise


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
