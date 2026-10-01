#!/usr/bin/env python3
"""Visual dataset designer with a live mockup of the game UI.

This is the visual dataset editor. It guides a new dataset
through the parts most visible in the application before exporting CSV files.
"""

import csv
import os
import re
import tempfile
import tkinter as tk
from tkinter import colorchooser, filedialog, messagebox, simpledialog, ttk

from dataset_generic import CHOICES as GENERIC_CHOICES
from dataset_generic import DEFAULT_ROWS as GENERIC_DEFAULT_ROWS
from dataset_generic import HELP as GENERIC_HELP
from dataset_generic import SECTIONS as GENERIC_SECTIONS
from dataset_generic import TUTORIAL_GUIDE


EVENT_TAGS = ("race", "track_day", "championship", "example")
EVENT_DURATIONS = ("minutes", "hours", "days", "weeks")
QUEST_TYPES = ("championship", "quest", "generic")
COST_RULE_HEADERS = [
    "id", "cost_id", "trigger_type", "trigger_ref", "amount_multiplier", "probability",
    "interval_days", "charge_mode", "resolution_mode", "pending_message", "message",
    "damage_type", "unavailable_days", "event_interval", "no_event_days",
]
COST_CONDITION_HEADERS = ["rule_id", "subject_type", "subject_ref", "operator", "value"]
COLOR_HEADERS = ("element_id", "label", "hex_color", "category", "default_hex")
HEX_COLOR = re.compile(r"^#[0-9A-Fa-f]{6}$")
DEFAULT_COLORS = [
    ("app_background", "Application background", "#0F172A", "Surface"),
    ("surface_background", "Panel and card background", "#1E293B", "Surface"),
    ("surface_border", "Panel border", "#334155", "Surface"),
    ("control_border", "Control border", "#475569", "Controls"),
    ("control_background", "Control background", "#334155", "Controls"),
    ("primary_accent", "Primary action", "#2563EB", "Controls"),
    ("primary_accent_border", "Primary action border", "#93C5FD", "Controls"),
    ("primary_text", "Primary text", "#F8FAFC", "Text"),
    ("secondary_text", "Secondary text", "#E2E8F0", "Text"),
    ("muted_text", "Muted text", "#94A3B8", "Text"),
    ("subtle_text", "Subtle text", "#CBD5E1", "Text"),
    ("link_text", "Link text", "#BFDBFE", "Text"),
    ("dark_text", "Dark text", "#0F172A", "Text"),
    ("white_text", "White text", "#FFFFFF", "Text"),
    ("success_text", "Success text", "#86EFAC", "Status"),
    ("success_background", "Success background", "#166534", "Status"),
    ("warning_text", "Warning text", "#FBBF24", "Status"),
    ("warning_background", "Warning background", "#854D0E", "Status"),
    ("error_text", "Error text", "#FCA5A5", "Status"),
    ("error_background", "Error background", "#7F1D1D", "Status"),
    ("error_border", "Error border", "#EF4444", "Status"),
    ("error_light_text", "Error banner text", "#FEE2E2", "Status"),
    ("info_background", "Information banner", "#1E3A8A", "Status"),
    ("info_border", "Information border", "#3B82F6", "Status"),
    ("info_text", "Information text", "#BFDBFE", "Status"),
    ("progress_background", "Progress track", "#334155", "Charts"),
    ("progress_high", "Progress high", "#22C55E", "Charts"),
    ("progress_medium", "Progress medium", "#EAB308", "Charts"),
    ("progress_low", "Progress low", "#EF4444", "Charts"),
    ("modal_overlay", "Modal overlay", "#020617", "Overlays"),
    ("modal_overlay_dark", "Dark modal overlay", "#000000", "Overlays"),
    ("light_surface", "Light content surface", "#F8FAFC", "Surface"),
    ("light_frame", "Light frame", "#FFFFFF", "Surface"),
    ("danger_action", "Danger action", "#EF4444", "Controls"),
    ("attention_text", "Attention text", "#F59E0B", "Status"),
    ("danger_text", "Danger text", "#B91C1C", "Status"),
    ("speed_normal_text", "Normal speed text", "#BBF7D0", "Status"),
    ("speed_fast_text", "Fast speed text", "#FEF08A", "Status"),
    ("speed_fastest_text", "Fastest speed text", "#FED7AA", "Status"),
]


THEME = {
    "bg": "#0F172A",
    "surface": "#1E293B",
    "border": "#334155",
    "accent": "#2563EB",
    "accent_active": "#3B82F6",
    "text": "#F8FAFC",
    "muted": "#94A3B8",
    "field": "#334155",
}

STEP_HELP = {
    "dashboard": "Names and labels the player sees first: the game title, the currency symbol and what your items and events are called.",
    "inventory": "Group the things a player owns into tabs. Each tab shows the object types you list for it.",
    "dealer": "The shop. Define the items players can buy, their price, and the recurring costs attached to them.",
    "events": "Scheduled happenings on the calendar: entry fee, reward, duration and any requirements to take part.",
    "quests": "Longer storylines or championships that bundle several events and award points.",
    "colors": "The shared colour theme used by this editor's mockup and by the real application.",
    "advanced": "Rules that charge money automatically, plus the conditions that decide when they fire.",
    "tables": "Direct access to every raw CSV table of the dataset, with per-column explanations.",
    "export": "Write every table to disk as CSV files inside the dataset folder.",
}

FIELD_HELP = {
    "application_name": "Title shown in the window and on the dashboard.",
    "currency_symbol": "Prefix used in front of every money amount, e.g. $ or €.",
    "inventory_name": "Label of the tab where the player sees what they own.",
    "dealer_name": "Label of the shop tab where things are bought.",
    "object_name": "What one purchasable thing is called (car, sword, tool...).",
    "object_plural": "Plural form of the above, used in lists and headings.",
    "event_name": "What one scheduled happening is called (race, mission...).",
    "event_plural": "Plural form, used for the events tab and counters.",
    "tab_id": "Short technical key, lowercase, no spaces. Stored in config.csv.",
    "tab_name": "Name of the tab as the player sees it.",
    "tab_types": "Object types shown in this tab, separated by semicolons.",
    "item_id": "Unique technical key referenced by events, quests and rules.",
    "item_type": "Category of the item; inventory tabs filter on this value.",
    "item_name": "Display name shown in the shop and inventory.",
    "item_price": "Purchase price in your currency. Whole number.",
    "cost_id": "Short unique key used to reference this cost elsewhere.",
    "cost_name": "Readable name shown when the cost is charged.",
    "cost_amount": "Money deducted each time this cost applies.",
    "event_id": "Unique technical key for this event.",
    "event_day": "Day of the in-game year (1-365) when the event happens.",
    "event_fee": "Money the player pays to take part.",
    "event_reward": "Money pool paid out on success.",
    "event_duration": "How long the event lasts, counted in the unit below.",
    "event_unit": "Time unit for the duration value.",
    "event_tag": "Free classification used for filtering and rules.",
    "event_license": "Optional licence item the player must own to enter.",
    "event_required": "Optional item the player must own to enter.",
    "event_quest": "Optional quest this event belongs to.",
    "quest_id": "Unique technical key for this quest.",
    "quest_type": "Championship, quest or a generic storyline.",
    "quest_name": "Name shown to the player.",
    "quest_points": "Points awarded when the quest succeeds.",
    "quest_fee": "Money required to join the quest.",
    "quest_license": "Optional licence item required to join.",
    "activity_jobs_columns": "Semicolon-separated Jobs table columns. Supported values: name, pay, frequency.",
    "activity_ventures_columns": "Semicolon-separated Ventures table columns. Supported values: name, cost, success, return.",
    "activity_pay_name": "Jobs payout column heading.",
    "activity_frequency_name": "Jobs payout frequency column heading.",
    "activity_cost_name": "Ventures cost column heading.",
    "activity_success_name": "Ventures success-rate column heading.",
    "activity_return_name": "Ventures successful outcome column heading.",
}


def write_csv(path, headers, rows):
    directory = os.path.dirname(path) or "."
    os.makedirs(directory, exist_ok=True)
    fd, temporary_path = tempfile.mkstemp(prefix=".dataset-", suffix=".csv", dir=directory, text=True)
    try:
        with os.fdopen(fd, "w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=headers)
            writer.writeheader()
            writer.writerows({header: row.get(header, "") for header in headers} for row in rows)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary_path, path)
    except Exception:
        if os.path.exists(temporary_path):
            os.unlink(temporary_path)
        raise


def default_color_rows():
    return [
        {
            "element_id": element_id,
            "label": label,
            "hex_color": color,
            "category": category,
            "default_hex": color,
        }
        for element_id, label, color, category in DEFAULT_COLORS
    ]


def load_color_rows(path, warn=True):
    if not os.path.exists(path):
        return default_color_rows()
    try:
        with open(path, newline="", encoding="utf-8") as handle:
            rows = list(csv.DictReader(handle))
        if not rows or any(
            not row.get("element_id", "").strip()
            or not HEX_COLOR.fullmatch(row.get("hex_color", "").strip())
            or not HEX_COLOR.fullmatch(row.get("default_hex", "").strip())
            for row in rows
        ):
            raise ValueError("colors.csv contains a missing field or invalid #RRGGBB value")
        return rows
    except (OSError, csv.Error, ValueError) as error:
        if warn:
            messagebox.showwarning(
                "Colors",
                f"Could not load colors.csv ({error}). Defaults will be used in memory; save explicitly to persist them.",
            )
        return default_color_rows()


REFERENCE_RULES = {
    "objects.csv": {
        "license_previous_id": ("objects.csv", False),
        "requires_object_ids": ("objects.csv", True),
    },
    "events.csv": {
        "required_license_id": ("objects.csv", False),
        "required_object_ids": ("objects.csv", True),
        "quest_id": ("quests.csv", False),
        "sponsor_quest_id": ("quests.csv", False),
        "sponsor_object_id": ("objects.csv", False),
        "sponsor_equipment_ids": ("objects.csv", True),
        "encounter_id": ("encounter_config.csv", False),
    },
    "quests.csv": {"required_license_id": ("objects.csv", False)},
    "obligations.csv": {"event_id": ("events.csv", False)},
    "cost_rules.csv": {
        "cost_id": ("costs.csv", False),
        "trigger_ref": ("events.csv", False),
    },
    "cost_rule_conditions.csv": {"rule_id": ("cost_rules.csv", False)},
}

MANDATORY_REFERENCE_FIELDS = {
    ("obligations.csv", "event_id"),
    ("cost_rules.csv", "cost_id"),
    ("cost_rule_conditions.csv", "rule_id"),
}


def _csv_rows(path):
    if not os.path.exists(path):
        return [], []
    with open(path, newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        return list(reader), list(reader.fieldnames or [])


def find_missing_references(dataset_path):
    """Return invalid cross-CSV references grouped by source file and row."""
    datasets = {}
    for filename in {source for rules in REFERENCE_RULES.values() for source, _ in rules.values()} | set(REFERENCE_RULES):
        rows, headers = _csv_rows(os.path.join(dataset_path, filename))
        datasets[filename] = (rows, headers)
    missing = []
    for source_file, rules in REFERENCE_RULES.items():
        rows, _ = datasets[source_file]
        for row_number, row in enumerate(rows, start=2):
            for field, (target_file, many) in rules.items():
                values = row.get(field, "").split(";") if many else [row.get(field, "")]
                invalid = [value.strip() for value in values if value.strip() and not any(
                    target.get("id", "") == value.strip() for target in datasets[target_file][0]
                )]
                if invalid:
                    missing.append({
                        "source_file": source_file, "row_number": row_number, "field": field,
                        "values": invalid, "target_file": target_file,
                    })
    return missing


def cleanup_missing_references(dataset_path):
    """Remove invalid references and rewrite only affected CSV files."""
    missing = find_missing_references(dataset_path)
    if not missing:
        return 0
    grouped = {}
    for item in missing:
        grouped.setdefault(item["source_file"], []).append(item)
    changed = 0
    for source_file, issues in grouped.items():
        path = os.path.join(dataset_path, source_file)
        rows, headers = _csv_rows(path)
        for issue in issues:
            row = rows[issue["row_number"] - 2]
            allowed = set(issue["values"])
            current = row.get(issue["field"], "").split(";")
            row[issue["field"]] = ";".join(value for value in current if value.strip() not in allowed)
        write_csv(path, headers, rows)
        changed += len(issues)
    return changed
class DatasetDesigner:
    def __init__(self, root):
        self.root = root
        self.root.title("Visual Game Dataset Designer")
        self.root.geometry("1280x780")
        self.config_path = os.path.abspath("cfg.yml")
        self.settings = self.load_settings()
        self.dataset_path = ""
        self.step = "dashboard"
        self.config = {
            "application_name": "My Game",
            "currency_symbol": "$",
            "inventory_name": "Inventory",
            "dealer_name": "Market",
            "object_name": "Item",
            "object_plural": "Items",
            "event_name": "Event",
            "event_plural": "Events",
            "activity_jobs_columns": "name;pay;frequency",
            "activity_ventures_columns": "name;cost;success;return",
            "activity_pay_name": "Pay",
            "activity_frequency_name": "Frequency",
            "activity_cost_name": "Cost",
            "activity_success_name": "Success",
            "activity_return_name": "Return",
        }
        self.inventory_tabs = []
        self.objects = []
        self.costs = []
        self.events = []
        self.quests = []
        self.cost_rules = []
        self.cost_conditions = []
        self.extra_config_rows = []
        self.colors = default_color_rows()
        self.color_vars = {}
        self.preview = None
        self.body = None
        self.context_fields = []
        self.path_var = tk.StringVar(value="")
        self.fullscreen_var = tk.BooleanVar(value=self.settings.get("fullscreen", False))
        self.root.protocol("WM_DELETE_WINDOW", self.close)
        self.root.attributes("-fullscreen", self.fullscreen_var.get())
        self.apply_theme()
        self.show_start()

    def apply_theme(self):
        style = ttk.Style(self.root)
        try:
            style.theme_use("clam")
        except tk.TclError:
            pass
        self.root.configure(background=THEME["bg"])
        style.configure(".", background=THEME["bg"], foreground=THEME["text"], font=("TkDefaultFont", 10))
        style.configure("TFrame", background=THEME["bg"])
        style.configure("TPanedwindow", background=THEME["bg"])
        style.configure("TLabel", background=THEME["bg"], foreground=THEME["text"])
        style.configure("Title.TLabel", font=("TkDefaultFont", 18, "bold"), foreground=THEME["text"])
        style.configure("Heading.TLabel", font=("TkDefaultFont", 14, "bold"), foreground=THEME["text"])
        style.configure("Sub.TLabel", font=("TkDefaultFont", 11, "bold"), foreground=THEME["text"])
        style.configure("Hint.TLabel", foreground=THEME["muted"], font=("TkDefaultFont", 9))
        style.configure("TCheckbutton", background=THEME["bg"], foreground=THEME["text"])
        style.map("TCheckbutton", background=[("active", THEME["bg"])])
        style.configure("TSeparator", background=THEME["border"])
        style.configure("TLabelframe", background=THEME["bg"], bordercolor=THEME["border"])
        style.configure("TLabelframe.Label", background=THEME["bg"], foreground=THEME["muted"])
        style.configure(
            "TButton", background=THEME["surface"], foreground=THEME["text"],
            bordercolor=THEME["border"], focuscolor=THEME["accent"], padding=6, relief="flat",
        )
        style.map("TButton", background=[("active", THEME["accent_active"]), ("pressed", THEME["accent"])])
        style.configure(
            "Step.TButton", background=THEME["surface"], foreground=THEME["text"],
            anchor="w", padding=(10, 7),
        )
        style.map("Step.TButton", background=[("active", THEME["accent_active"])])
        style.configure(
            "Accent.TButton", background=THEME["accent"], foreground="#FFFFFF", padding=(12, 7),
        )
        style.map("Accent.TButton", background=[("active", THEME["accent_active"])])
        for widget in ("TEntry", "TCombobox"):
            style.configure(
                widget, fieldbackground=THEME["field"], background=THEME["field"],
                foreground=THEME["text"], bordercolor=THEME["border"],
                insertcolor=THEME["text"], arrowcolor=THEME["text"], padding=4,
            )
        style.map(
            "TCombobox",
            fieldbackground=[("readonly", THEME["field"])],
            foreground=[("readonly", THEME["text"])],
        )
        style.configure(
            "Treeview", background=THEME["surface"], fieldbackground=THEME["surface"],
            foreground=THEME["text"], bordercolor=THEME["border"], rowheight=24,
        )
        style.configure(
            "Treeview.Heading", background=THEME["border"], foreground=THEME["text"],
            relief="flat", font=("TkDefaultFont", 9, "bold"),
        )
        style.map("Treeview", background=[("selected", THEME["accent"])], foreground=[("selected", "#FFFFFF")])
        style.configure("TScrollbar", background=THEME["surface"], troughcolor=THEME["bg"],
                        bordercolor=THEME["border"], arrowcolor=THEME["text"])

    def dialog_frame(self, dialog, title, explanation):
        """Create a themed dialog body with a title and an explanation line."""
        dialog.configure(background=THEME["bg"])
        frame = ttk.Frame(dialog, padding=16)
        frame.pack(fill="both", expand=True)
        ttk.Label(frame, text=title, style="Heading.TLabel").pack(anchor="w")
        ttk.Label(frame, text=explanation, style="Hint.TLabel", wraplength=460).pack(anchor="w", pady=(2, 10))
        return frame

    def scrollable_dialog_frame(self, dialog, title, explanation):
        """Create a dialog body whose long form can be scrolled vertically."""
        dialog.configure(background=THEME["bg"])
        outer = ttk.Frame(dialog, padding=10)
        outer.pack(fill="both", expand=True)
        canvas = tk.Canvas(outer, highlightthickness=0, background=THEME["bg"])
        scrollbar = ttk.Scrollbar(outer, orient="vertical", command=canvas.yview)
        frame = ttk.Frame(canvas, padding=6)
        window = canvas.create_window((0, 0), window=frame, anchor="nw")
        frame.bind("<Configure>", lambda _event: canvas.configure(scrollregion=canvas.bbox("all")))
        canvas.bind("<Configure>", lambda event: canvas.itemconfigure(window, width=event.width))
        canvas.configure(yscrollcommand=scrollbar.set)
        canvas.pack(side="left", fill="both", expand=True)
        scrollbar.pack(side="right", fill="y")
        ttk.Label(frame, text=title, style="Heading.TLabel").pack(anchor="w")
        ttk.Label(frame, text=explanation, style="Hint.TLabel", wraplength=460).pack(
            anchor="w", pady=(2, 10)
        )

        def scroll(event):
            canvas.yview_scroll(-1 * (event.delta // 120 or 1), "units")

        canvas.bind_all("<MouseWheel>", scroll, add="+")
        def close():
            canvas.unbind_all("<MouseWheel>")
            dialog.destroy()

        dialog._dataset_close = close
        dialog.protocol("WM_DELETE_WINDOW", close)
        return frame

    def section(self, title, explanation):
        ttk.Label(self.body, text=title, style="Heading.TLabel").pack(anchor="w", pady=(6, 0))
        ttk.Label(self.body, text=explanation, style="Hint.TLabel", wraplength=330).pack(anchor="w", pady=(4, 12))

    def load_settings(self):
        settings = {"dataset_path": "", "fullscreen": False}
        try:
            with open(self.config_path, encoding="utf-8") as handle:
                for line in handle:
                    key, separator, value = line.partition(":")
                    if not separator:
                        continue
                    value = value.strip()
                    if key.strip() == "fullscreen":
                        settings["fullscreen"] = value.lower() in ("true", "yes", "1")
                    elif key.strip() == "dataset_path":
                        settings["dataset_path"] = value.strip().strip("'\"")
        except OSError:
            pass
        return settings

    def save_settings(self):
        with open(self.config_path, "w", encoding="utf-8") as handle:
            handle.write(f"dataset_path: '{self.dataset_path}'\n")
            handle.write(f"fullscreen: {'true' if self.fullscreen_var.get() else 'false'}\n")

    def close(self):
        self.save_settings()
        self.root.destroy()

    def show_start(self):
        self.clear(self.root)
        frame = ttk.Frame(self.root, padding=30)
        frame.pack(fill="both", expand=True)
        ttk.Label(frame, text="Visual Game Dataset Designer", style="Title.TLabel").pack(anchor="w")
        ttk.Label(
            frame,
            text="A dataset is a folder of CSV files that describes your whole game: its wording, the items players "
                 "buy, the events they attend and the colours of the interface. Pick a folder to edit, or create a "
                 "new empty one to start from scratch.",
            style="Hint.TLabel",
            wraplength=900,
        ).pack(anchor="w", pady=(10, 18))
        path_row = ttk.Frame(frame)
        path_row.pack(fill="x")
        ttk.Label(path_row, text="Dataset folder").pack(side="left")
        ttk.Entry(path_row, textvariable=self.path_var).pack(side="left", fill="x", expand=True, padx=8)
        ttk.Button(path_row, text="Browse", command=self.choose_start_folder).pack(side="left")
        ttk.Checkbutton(
            frame,
            text="Start in fullscreen mode",
            variable=self.fullscreen_var,
            command=lambda: self.root.attributes("-fullscreen", self.fullscreen_var.get()),
        ).pack(anchor="w", pady=(14, 4))
        buttons = ttk.Frame(frame)
        buttons.pack(anchor="e", pady=18)
        ttk.Button(buttons, text="Create new empty dataset", command=self.create_empty_dataset).pack(side="left", padx=5)
        ttk.Button(buttons, text="Edit selected dataset", style="Accent.TButton", command=self.start_dataset).pack(side="left", padx=5)
        if self.settings.get("dataset_path") and not self.path_var.get():
            self.path_var.set(self.settings["dataset_path"])
        ttk.Label(frame, text="Existing datasets in this project:", style="Sub.TLabel").pack(anchor="w", pady=(14, 0))
        ttk.Label(frame, text="Folders next to this editor that already contain a config.csv. Click one to select it.",
                  style="Hint.TLabel").pack(anchor="w", pady=(2, 6))
        for name in sorted(entry for entry in os.listdir(".") if os.path.isdir(entry) and os.path.exists(os.path.join(entry, "config.csv"))):
            ttk.Button(frame, text=name, command=lambda value=name: self.select_dataset(value)).pack(anchor="w", pady=2)

    def select_dataset(self, path):
        self.path_var.set(os.path.abspath(path))

    def choose_start_folder(self):
        selected = filedialog.askdirectory(initialdir=self.path_var.get() or os.getcwd())
        if selected:
            self.path_var.set(selected)

    def create_empty_dataset(self):
        selected = filedialog.askdirectory(initialdir=os.getcwd(), title="Choose parent folder for new dataset")
        if not selected:
            return
        name = simpledialog.askstring("New dataset", "Dataset folder name:", parent=self.root)
        if not name or not name.strip():
            return
        path = os.path.join(selected, name.strip())
        try:
            os.makedirs(path, exist_ok=False)
        except OSError as error:
            messagebox.showerror("New dataset", f"Could not create dataset folder:\n{error}", parent=self.root)
            return
        self.path_var.set(path)
        self.start_dataset()

    def start_dataset(self):
        path = os.path.abspath(self.path_var.get().strip())
        if not path:
            messagebox.showerror("Dataset", "Choose or create a dataset folder first.", parent=self.root)
            return
        os.makedirs(path, exist_ok=True)
        self.dataset_path = path
        self.settings["dataset_path"] = path
        self.load_visual_data()
        self.save_settings()
        self.show_shell()

    def load_visual_data(self):
        def rows(filename):
            path = os.path.join(self.dataset_path, filename)
            if not os.path.exists(path):
                return []
            with open(path, newline="", encoding="utf-8") as handle:
                return list(csv.DictReader(handle))

        config_rows = rows("config.csv")
        config_values = {row.get("variable", ""): row.get("value", "") for row in config_rows}
        defaults = {
            "application_name": "My Game", "currency_symbol": "$", "inventory_name": "Inventory",
            "dealer_name": "Market", "object_name": "Item", "object_plural": "Items",
            "event_name": "Event", "event_plural": "Events",
            "activity_jobs_columns": "name;pay;frequency",
            "activity_ventures_columns": "name;cost;success;return",
            "activity_pay_name": "Pay",
            "activity_frequency_name": "Frequency",
            "activity_cost_name": "Cost",
            "activity_success_name": "Success",
            "activity_return_name": "Return",
        }
        self.config.update(defaults)
        for key in self.config:
            if key in config_values:
                self.config[key] = config_values[key]
        self.extra_config_rows = [
            row for row in config_rows if row.get("variable", "") not in self.config
            and not row.get("variable", "").startswith("inventory_tab_")
        ]
        self.inventory_tabs = [
            {"id": key.removeprefix("inventory_tab_").removesuffix("_name"), "name": value,
             "types": config_values.get(key.replace("_name", "_types"), "")}
            for key, value in config_values.items()
            if key.startswith("inventory_tab_") and key.endswith("_name")
        ]
        self.objects = rows("objects.csv")
        self.costs = rows("costs.csv")
        self.events = rows("events.csv")
        self.quests = rows("quests.csv")
        self.cost_rules = rows("cost_rules.csv")
        self.cost_conditions = rows("cost_rule_conditions.csv")
        self.colors = load_color_rows(os.path.join(self.dataset_path, "colors.csv"))

    def clear(self, parent):
        for child in parent.winfo_children():
            child.destroy()

    def show_shell(self):
        self.clear(self.root)
        header = ttk.Frame(self.root, padding=10)
        header.pack(fill="x")
        ttk.Label(header, text="Visual Game Dataset Designer", style="Title.TLabel").pack(side="left")
        ttk.Label(header, textvariable=self.path_var, style="Hint.TLabel").pack(side="right")
        ttk.Label(header, text="Editing dataset:", style="Hint.TLabel").pack(side="right", padx=(0, 6))

        body = ttk.PanedWindow(self.root, orient="horizontal")
        body.pack(fill="both", expand=True, padx=10, pady=(0, 10))
        left = ttk.Frame(body, padding=10)
        right = ttk.Frame(body, padding=10)
        body.add(left, weight=1)
        body.add(right, weight=3)
        canvas = tk.Canvas(left, highlightthickness=0, background=THEME["bg"])
        scrollbar = ttk.Scrollbar(left, orient="vertical", command=canvas.yview)
        inner = ttk.Frame(canvas)
        window = canvas.create_window((0, 0), window=inner, anchor="nw")
        inner.bind("<Configure>", lambda event: canvas.configure(scrollregion=canvas.bbox("all")))
        canvas.bind("<Configure>", lambda event: canvas.itemconfigure(window, width=event.width))
        canvas.configure(yscrollcommand=scrollbar.set)
        canvas.pack(side="left", fill="both", expand=True)
        scrollbar.pack(side="right", fill="y")
        self.body = inner
        self.preview = right
        self.show_steps()
        self.show_preview()

    def show_steps(self):
        self.clear(self.body)
        ttk.Label(self.body, text="Build your game", style="Heading.TLabel").pack(anchor="w")
        ttk.Label(
            self.body,
            text="Work from step 1 downwards. Each step explains what it changes, and every change is reflected in the live mockup on the right.",
            style="Hint.TLabel",
            wraplength=310,
        ).pack(anchor="w", pady=(4, 12))
        for key, title in (
            ("dashboard", "1. Dashboard"),
            ("inventory", "2. Inventory tabs"),
            ("dealer", "3. Dealer and items"),
            ("events", "4. Events"),
            ("quests", "5. Quests"),
            ("colors", "6. Colors"),
            ("advanced", "7. Events and cost rules"),
            ("tables", "8. All dataset tables"),
            ("export", "9. Export dataset"),
        ):
            ttk.Button(
                self.body, text=title, style="Accent.TButton" if key == "export" else "Step.TButton",
                command=lambda value=key: self.open_step(value),
            ).pack(fill="x", pady=(6, 0))
            ttk.Label(self.body, text=STEP_HELP[key], style="Hint.TLabel", wraplength=300).pack(
                anchor="w", pady=(1, 4)
            )
        ttk.Separator(self.body).pack(fill="x", pady=12)
        ttk.Button(self.body, text="Choose output folder", command=self.choose_folder).pack(fill="x")
        ttk.Label(
            self.body,
            text="Pick the folder the CSV files are read from and written to.",
            style="Hint.TLabel", wraplength=300,
        ).pack(anchor="w", pady=(1, 6))
        ttk.Button(self.body, text="Start over", command=self.reset).pack(fill="x")
        ttk.Label(
            self.body,
            text="Clears the design held in memory. Files on disk stay until you export again.",
            style="Hint.TLabel", wraplength=300,
        ).pack(anchor="w", pady=(1, 4))
        ttk.Button(self.body, text="Clean missing CSV references", command=self.clean_missing_references).pack(
            fill="x", pady=(10, 0)
        )
        ttk.Label(
            self.body,
            text="Find references to IDs that do not exist in their target CSV and remove them after confirmation.",
            style="Hint.TLabel", wraplength=300,
        ).pack(anchor="w", pady=(1, 4))

    def clean_missing_references(self):
        missing = find_missing_references(self.dataset_path)
        if not missing:
            messagebox.showinfo("CSV references", "No missing CSV references were found.", parent=self.root)
            return
        preview = "\n".join(
            f"{item['source_file']} row {item['row_number']}: {item['field']} -> "
            f"{', '.join(item['values'])} (missing from {item['target_file']})"
            for item in missing[:20]
        )
        suffix = "" if len(missing) <= 20 else f"\n…and {len(missing) - 20} more."
        if not messagebox.askyesno(
            "Clean missing CSV references",
            f"Remove {len(missing)} missing reference(s) from the dataset?\n\n{preview}{suffix}",
            parent=self.root,
        ):
            return
        cleanup_missing_references(self.dataset_path)
        self.load_visual_data()
        self.show_editor()
        self.show_preview()
        messagebox.showinfo("CSV references", f"Removed {len(missing)} missing reference(s).", parent=self.root)

    def open_step(self, step):
        self.step = step
        if step == "export":
            self.export()
            return
        self.show_editor()

    def show_editor(self):
        self.clear(self.body)
        self.context_fields = []
        ttk.Button(self.body, text="← Back to steps", command=self.show_steps).pack(anchor="w", pady=(0, 6))
        if self.step == "dashboard":
            self.dashboard_editor()
        elif self.step == "inventory":
            self.inventory_editor()
        elif self.step == "dealer":
            self.dealer_editor()
        elif self.step == "events":
            self.events_editor()
        elif self.step == "quests":
            self.quests_editor()
        elif self.step == "colors":
            self.colors_editor()
        elif self.step == "advanced":
            self.advanced_editor()
        elif self.step == "tables":
            self.tables_editor()
        self.show_preview()

    def reference_values(self, target_file):
        collections = {
            "objects.csv": self.objects,
            "costs.csv": self.costs,
            "events.csv": self.events,
            "quests.csv": self.quests,
            "cost_rules.csv": self.cost_rules,
        }
        if target_file in collections:
            return [row.get("id", "") for row in collections[target_file] if row.get("id")]
        path = os.path.join(self.dataset_path, target_file)
        if not os.path.exists(path):
            return []
        try:
            with open(path, newline="", encoding="utf-8") as handle:
                return [row.get("id", "") for row in csv.DictReader(handle) if row.get("id")]
        except (OSError, csv.Error):
            return []

    def open_reference_editor(self, target_file):
        step_by_file = {
            "objects.csv": "dealer",
            "costs.csv": "dealer",
            "events.csv": "events",
            "quests.csv": "quests",
            "cost_rules.csv": "advanced",
        }
        step = step_by_file.get(target_file)
        if step is None:
            messagebox.showinfo(
                "Reference editor",
                f"Use the All dataset tables step to add entries to {target_file}.",
                parent=self.root,
            )
            return
        self.step = step
        self.show_editor()

    def field(
        self,
        parent,
        label,
        value="",
        choices=None,
        hint=None,
        reference_file=None,
        required=False,
    ):
        holder = ttk.Frame(parent)
        holder.pack(fill="x", pady=(4, 2))
        row = ttk.Frame(holder)
        row.pack(fill="x")
        ttk.Label(row, text=label, width=22).pack(side="left")
        variable = tk.StringVar(value=value)
        if reference_file:
            choices = ([""] if not required else []) + self.reference_values(reference_file)
        if choices:
            widget = ttk.Combobox(
                row,
                textvariable=variable,
                values=choices,
                state="readonly" if reference_file else "normal",
            )
        else:
            widget = ttk.Entry(row, textvariable=variable)
        widget.pack(side="left", fill="x", expand=True)
        if reference_file:
            ttk.Button(
                row,
                text="Add new...",
                command=lambda target=reference_file: self.open_reference_editor(target),
            ).pack(side="left", padx=(4, 0))
        if hint:
            ttk.Label(holder, text=hint, style="Hint.TLabel", wraplength=420).pack(
                anchor="w", padx=(22 * 7, 0)
            )
        return variable

    def dashboard_editor(self):
        self.section(
            "1. Dashboard",
            "This step contains no game content yet - only the wording. It renames the application, the currency "
            "and the tabs, so the rest of the editor speaks your game's language.",
        )
        variables = {
            key: self.field(self.body, label, self.config[key], hint=FIELD_HELP[key])
            for key, label in (
                ("application_name", "Application name"),
                ("currency_symbol", "Currency symbol"),
                ("inventory_name", "Inventory tab name"),
                ("dealer_name", "Dealer tab name"),
                ("object_name", "Singular item name"),
                ("object_plural", "Plural item name"),
                ("event_name", "Singular event name"),
                ("event_plural", "Plural event name"),
                ("activity_jobs_columns", "Jobs table columns"),
                ("activity_ventures_columns", "Ventures table columns"),
                ("activity_pay_name", "Jobs pay heading"),
                ("activity_frequency_name", "Jobs frequency heading"),
                ("activity_cost_name", "Ventures cost heading"),
                ("activity_success_name", "Ventures success heading"),
                ("activity_return_name", "Ventures return heading"),
            )
        }

        def save():
            self.config.update({key: variable.get().strip() for key, variable in variables.items()})
            self.show_preview()
            messagebox.showinfo("Dashboard", "Dashboard labels updated.", parent=self.root)

        ttk.Button(self.body, text="Apply dashboard changes", command=save).pack(anchor="e", pady=14)

    def inventory_editor(self):
        self.section(
            "2. Inventory tabs",
            "Split what the player owns into tabs. A tab lists every object whose type appears in its type list, "
            "so 'garage' with types 'vehicle' shows all vehicles the player bought.",
        )
        table = ttk.Treeview(self.body, columns=("id", "name", "types"), show="headings", height=7)
        for column in ("id", "name", "types"):
            table.heading(column, text=column.title())
            table.column(column, width=95)
        table.pack(fill="x")
        for tab in self.inventory_tabs:
            table.insert("", "end", values=(tab["id"], tab["name"], tab["types"]))

        def add_tab():
            dialog = tk.Toplevel(self.root)
            dialog.title("Add inventory tab")
            dialog.transient(self.root)
            dialog.grab_set()
            frame = self.dialog_frame(
                dialog, "Add inventory tab",
                "A tab groups owned objects by their type. All three fields are required.",
            )
            tab_id = self.field(frame, "Tab id", hint=FIELD_HELP["tab_id"])
            name = self.field(frame, "Tab name", hint=FIELD_HELP["tab_name"])
            types = self.field(frame, "Object types", "vehicle;equipment", hint=FIELD_HELP["tab_types"])

            def accept():
                type_values = ";".join(value.strip() for value in types.get().split(";") if value.strip())
                if not tab_id.get().strip() or not name.get().strip() or not type_values:
                    messagebox.showerror("Inventory tab", "Id, name, and at least one object type are required.", parent=dialog)
                    return
                self.inventory_tabs.append({"id": tab_id.get().strip(), "name": name.get().strip(), "types": type_values})
                dialog.destroy()
                self.show_editor()
                self.show_preview()

            ttk.Button(frame, text="Cancel", command=dialog.destroy).pack(side="left", pady=12)
            ttk.Button(frame, text="Add tab", command=accept).pack(side="right", pady=12)

        def remove_tab():
            selected = table.selection()
            if selected:
                del self.inventory_tabs[table.index(selected[0])]
                self.show_editor()
                self.show_preview()

        buttons = ttk.Frame(self.body)
        buttons.pack(fill="x", pady=10)
        ttk.Button(buttons, text="Add tab", command=add_tab).pack(side="left")
        ttk.Button(buttons, text="Remove selected", command=remove_tab).pack(side="left", padx=6)

    def dealer_editor(self):
        self.section(
            f"3. {self.config['dealer_name']}",
            f"Everything the player can buy. Each entry becomes one row of objects.csv: an id, a type used by the "
            f"inventory tabs, a price in {self.config['currency_symbol']} and optional recurring costs.",
        )
        table = ttk.Treeview(self.body, columns=("id", "type", "name", "price", "cost"), show="headings", height=8)
        for column in ("id", "type", "name", "price", "cost"):
            table.heading(column, text=column.title())
            table.column(column, width=70 if column != "name" else 120)
        table.pack(fill="x")
        for item in self.objects:
            table.insert("", "end", values=(item["id"], item["type"], item["name"], item["price"], item.get("cost_1", "")))

        buttons = ttk.Frame(self.body)
        buttons.pack(fill="x", pady=10)
        ttk.Button(buttons, text="Add item", command=self.item_dialog).pack(side="left")
        ttk.Button(buttons, text="Edit selected", command=lambda: self.edit_item(table)).pack(side="left", padx=6)
        ttk.Button(buttons, text="Remove selected", command=lambda: self.remove_item(table)).pack(side="left")
        ttk.Separator(self.body).pack(fill="x", pady=8)
        ttk.Label(self.body, text="Reusable costs", style="Sub.TLabel").pack(anchor="w")
        ttk.Label(
            self.body,
            text="A cost is a named amount of money that items can charge repeatedly, for example maintenance or "
                 "insurance. Define it once here, then pick it in an item's cost slots.",
            style="Hint.TLabel", wraplength=330,
        ).pack(anchor="w", pady=(2, 6))
        cost_row = ttk.Frame(self.body)
        cost_row.pack(fill="x", pady=4)
        cost_id = self.field(cost_row, "Cost id", hint=FIELD_HELP["cost_id"])
        cost_name = self.field(cost_row, "Name", hint=FIELD_HELP["cost_name"])
        amount = self.field(cost_row, "Amount", hint=FIELD_HELP["cost_amount"])

        def add_cost():
            if not cost_id.get().strip() or not cost_name.get().strip():
                messagebox.showerror("Cost", "Cost id and name are required.", parent=self.root)
                return
            self.costs.append({"id": cost_id.get().strip(), "name": cost_name.get().strip(), "amount": amount.get().strip()})
            self.show_editor()
            self.show_preview()

        ttk.Button(self.body, text="Add cost", command=add_cost).pack(anchor="e")
        if self.costs:
            ttk.Label(self.body, text="Existing costs: " + ", ".join(f"{cost['id']} ({cost['amount']})" for cost in self.costs),
                      wraplength=330).pack(anchor="w", pady=8)
            edit_cost_id = self.field(self.body, "Modify cost", "", [cost["id"] for cost in self.costs])
            edit_amount = self.field(self.body, "New amount")

            def modify_cost():
                selected = next((cost for cost in self.costs if cost["id"] == edit_cost_id.get()), None)
                if selected is None:
                    messagebox.showerror("Cost", "Choose an existing cost to modify.", parent=self.root)
                    return
                selected["amount"] = edit_amount.get().strip()
                self.show_editor()
                self.show_preview()

            ttk.Button(self.body, text="Modify selected cost", command=modify_cost).pack(anchor="e", pady=(4, 0))

    def item_dialog(self, index=None):
        dialog = tk.Toplevel(self.root)
        dialog.title("Edit item")
        dialog.transient(self.root)
        dialog.grab_set()
        dialog.geometry("620x760")
        frame = self.scrollable_dialog_frame(
            dialog, "Item",
            "One purchasable object. The id is referenced by events, quests and rules, the type decides which "
            "inventory tab shows it.",
        )
        current = self.objects[index].copy() if index is not None else {}
        item_id = self.field(frame, "Item id", current.get("id", ""), hint=FIELD_HELP["item_id"])
        item_type = self.field(frame, "Item type", current.get("type", ""), hint=FIELD_HELP["item_type"])
        name = self.field(frame, "Name", current.get("name", ""), hint=FIELD_HELP["item_name"])
        price = self.field(frame, "Price", current.get("price", "0"), hint=FIELD_HELP["item_price"])
        ttk.Label(frame, text="Service costs and intervals", style="Sub.TLabel").pack(anchor="w", pady=(10, 2))
        ttk.Label(
            frame,
            text="Slots 1-15 attach reusable costs to this item. The matching interval says how many days pass "
                 "between two charges; leave 0 to disable the slot.",
            style="Hint.TLabel", wraplength=460,
        ).pack(anchor="w", pady=(0, 4))
        cost_variables = {
            f"cost_{number}": self.field(
                frame,
                f"Cost {number}",
                current.get(f"cost_{number}", ""),
                reference_file="costs.csv",
            )
            for number in range(1, 16)
        }
        interval_variables = {
            f"service_{number}_interval_days": self.field(
                frame, f"Service {number} interval (days)", current.get(f"service_{number}_interval_days", "0")
            )
            for number in range(1, 16)
        }
        ttk.Label(frame, text="Acquisition and display settings", style="Sub.TLabel").pack(anchor="w", pady=(10, 2))
        ttk.Label(
            frame,
            text="How the item is presented and how it loses value: resale percentages, the licence level needed "
                 "to own it, how long it lasts and which HTML description and image it uses.",
            style="Hint.TLabel", wraplength=460,
        ).pack(anchor="w", pady=(0, 4))
        advanced_variables = {
            key: self.field(frame, key.replace("_", " ").title(), current.get(key, default))
            for key, default in (
                ("description_html", "./html/object.html"),
                ("resale_initial_percent", "0.75"),
                ("resale_annual_percent", "0.9"),
                ("resale_min_percent", "0.1"),
                ("license_level", "0"),
                ("license_previous_id", ""),
                ("license_fee", "0"),
                ("lifetime_days", "0"),
                ("availability_days", "0"),
                ("image_path", ""),
            )
        }
        ttk.Label(frame, text="Required objects", style="Sub.TLabel").pack(anchor="w", pady=(10, 2))
        ttk.Label(
            frame,
            text="The player must already own every item selected here before this one can be bought.",
            style="Hint.TLabel", wraplength=460,
        ).pack(anchor="w", pady=(0, 4))
        required = tk.Listbox(frame, selectmode="multiple", height=5, exportselection=False)
        existing_ids = [item["id"] for item in self.objects if item is not current]
        for object_id in existing_ids:
            required.insert("end", object_id)
            if object_id in current.get("requires_object_ids", "").split(";"):
                required.selection_set(required.size() - 1)
        required.pack(fill="x")

        def accept():
            if not item_id.get().strip() or not name.get().strip():
                messagebox.showerror("Item", "Item id and name are required.", parent=dialog)
                return
            selected = [required.get(position) for position in required.curselection()]
            entry = {
                "id": item_id.get().strip(), "type": item_type.get(), "name": name.get().strip(),
                "price": price.get().strip(),
                "requires_object_ids": ";".join(selected),
            }
            entry.update({key: variable.get().strip() for key, variable in cost_variables.items()})
            entry.update({key: variable.get().strip() for key, variable in interval_variables.items()})
            entry.update({key: variable.get().strip() for key, variable in advanced_variables.items()})
            if index is None:
                self.objects.append(entry)
            else:
                self.objects[index] = entry
            dialog.destroy()
            self.show_editor()
            self.show_preview()

        ttk.Button(frame, text="Cancel", command=dialog._dataset_close).pack(side="left", pady=12)
        ttk.Button(frame, text="Save item", command=accept).pack(side="right", pady=12)

    def edit_item(self, table):
        selected = table.selection()
        if selected:
            self.item_dialog(table.index(selected[0]))

    def remove_item(self, table):
        selected = table.selection()
        if selected:
            del self.objects[table.index(selected[0])]
            self.show_editor()
            self.show_preview()

    def events_editor(self):
        self.section(
            f"4. {self.config['event_plural']}",
            "Things that happen on a given day of the in-game year. The player pays an entry fee, spends the "
            "duration on it and can win the reward pool. Requirements decide who may take part.",
        )
        table = ttk.Treeview(self.body, columns=("id", "name", "day", "reward", "quest"), show="headings", height=8)
        for column in ("id", "name", "day", "reward", "quest"):
            table.heading(column, text=column.title())
            table.column(column, width=85 if column != "name" else 135)
        table.pack(fill="x")
        for event in self.events:
            table.insert("", "end", values=(event["id"], event["name"], event["day_of_year"], event["reward_pool"], event.get("quest_id", "")))
        ttk.Button(self.body, text="Add event", command=self.event_dialog).pack(anchor="w", pady=10)

    def event_dialog(self):
        dialog = tk.Toplevel(self.root)
        dialog.title("Add event")
        dialog.transient(self.root)
        dialog.grab_set()
        frame = self.dialog_frame(
            dialog, f"Add {self.config['event_name'].lower()}",
            "One scheduled occasion on the calendar. Only id and name are mandatory; the rest tunes cost, payout "
            "and who is allowed to join.",
        )
        event_id = self.field(frame, "Event id", hint=FIELD_HELP["event_id"])
        name = self.field(frame, "Name", hint="Display name shown in the events list.")
        day = self.field(frame, "Day of year", "1", hint=FIELD_HELP["event_day"])
        entry_fee = self.field(frame, "Entry fee", "0", hint=FIELD_HELP["event_fee"])
        reward = self.field(frame, "Reward", "0", hint=FIELD_HELP["event_reward"])
        duration = self.field(frame, "Duration", "1", hint=FIELD_HELP["event_duration"])
        duration_unit = self.field(frame, "Duration unit", "day", EVENT_DURATIONS, hint=FIELD_HELP["event_unit"])
        tag = self.field(frame, "Tag", "example", EVENT_TAGS, hint=FIELD_HELP["event_tag"])
        license_id = self.field(
            frame, "Required license", "", hint=FIELD_HELP["event_license"], reference_file="objects.csv"
        )
        required = self.field(
            frame, "Required object", "", hint=FIELD_HELP["event_required"], reference_file="objects.csv"
        )
        quest_id = self.field(
            frame, "Quest", "", hint=FIELD_HELP["event_quest"], reference_file="quests.csv"
        )

        def accept():
            if not event_id.get().strip() or not name.get().strip():
                messagebox.showerror("Event", "Event id and name are required.", parent=dialog)
                return
            self.events.append({
                "id": event_id.get().strip(), "name": name.get().strip(), "day_of_year": day.get(),
                "entry_fee": entry_fee.get(), "reward_pool": reward.get(), "charisma_reward": "0",
                "duration_value": duration.get(), "duration_unit": duration_unit.get(),
                "tags": tag.get(), "description_html": "./html/event.html",
                "required_license_id": license_id.get(), "required_object_ids": required.get(),
                "quest_id": quest_id.get(),
            })
            dialog.destroy()
            self.show_editor()
            self.show_preview()

        ttk.Button(frame, text="Cancel", command=dialog.destroy).pack(side="left", pady=12)
        ttk.Button(frame, text="Save event", command=accept).pack(side="right", pady=12)

    def quests_editor(self):
        self.section(
            "5. Quests and championships",
            "A quest bundles several events into a storyline the player joins once. Championships additionally "
            "collect points across their events.",
        )
        table = ttk.Treeview(self.body, columns=("id", "type", "name", "join_fee"), show="headings", height=8)
        for column in ("id", "type", "name", "join_fee"):
            table.heading(column, text=column.title())
            table.column(column, width=95 if column != "name" else 145)
        table.pack(fill="x")
        for quest in self.quests:
            table.insert("", "end", values=(quest["id"], quest["type"], quest["name"], quest["join_fee"]))
        ttk.Button(self.body, text="Add quest", command=self.quest_dialog).pack(anchor="w", pady=10)

    def quest_dialog(self):
        dialog = tk.Toplevel(self.root)
        dialog.title("Add quest")
        dialog.transient(self.root)
        dialog.grab_set()
        frame = self.dialog_frame(
            dialog, "Add quest",
            "Link events together into one storyline. Events reference this quest through their Quest field.",
        )
        quest_id = self.field(frame, "Quest id", hint=FIELD_HELP["quest_id"])
        quest_type = self.field(frame, "Type", "championship", QUEST_TYPES, hint=FIELD_HELP["quest_type"])
        name = self.field(frame, "Name", hint=FIELD_HELP["quest_name"])
        points = self.field(frame, "Success points", "10", hint=FIELD_HELP["quest_points"])
        join_fee = self.field(frame, "Join fee", "0", hint=FIELD_HELP["quest_fee"])
        license_id = self.field(
            frame, "Required license", "", hint=FIELD_HELP["quest_license"], reference_file="objects.csv"
        )

        def accept():
            if not quest_id.get().strip() or not name.get().strip():
                messagebox.showerror("Quest", "Quest id and name are required.", parent=dialog)
                return
            self.quests.append({
                "id": quest_id.get().strip(), "type": quest_type.get(), "name": name.get().strip(),
                "success_points": points.get(), "failure_points": "0", "join_fee": join_fee.get(),
                "required_license_id": license_id.get(), "description_html": "./html/quest.html",
            })
            dialog.destroy()
            self.show_editor()
            self.show_preview()

        ttk.Button(frame, text="Cancel", command=dialog.destroy).pack(side="left", pady=12)
        ttk.Button(frame, text="Save quest", command=accept).pack(side="right", pady=12)

    def advanced_editor(self):
        self.section(
            "7. Events and cost rules",
            "Automatic money rules. A cost rule fires on a trigger (a day passing, an event finishing, an object "
            "being acquired) and charges the linked cost. Conditions narrow down when a rule applies.",
        )
        self.collection_editor(
            "Cost rules",
            self.cost_rules,
            COST_RULE_HEADERS,
            {
                "trigger_type": ("day_elapsed", "event_completed", "object_acquired"),
                "charge_mode": ("immediate", "pending"),
                "resolution_mode": ("charge", "object_service"),
            },
        )
        self.collection_editor(
            "Cost rule conditions",
            self.cost_conditions,
            COST_CONDITION_HEADERS,
            {
                "subject_type": ("event", "event", "object", "player"),
                "operator": ("equals", "contains", "greater_than", "less_than"),
            },
        )

    def collection_editor(self, title, collection, headers, choices):
        ttk.Label(self.body, text=title, style="Sub.TLabel").pack(anchor="w", pady=(10, 2))
        table = ttk.Treeview(self.body, columns=headers[:5], show="headings", height=3)
        for header in headers[:5]:
            table.heading(header, text=header)
            table.column(header, width=100)
        table.pack(fill="x")
        for row in collection:
            table.insert("", "end", values=tuple(row.get(header, "") for header in headers[:5]))
        buttons = ttk.Frame(self.body)
        buttons.pack(fill="x", pady=(3, 8))
        ttk.Button(buttons, text=f"Add {title[:-1] if title.endswith('s') else title}", command=lambda: self.collection_dialog(
            collection, headers, title, choices,
        )).pack(side="left")

        def edit():
            selected = table.selection()
            if selected:
                self.collection_dialog(collection, headers, title, choices, table.index(selected[0]))

        def remove():
            selected = table.selection()
            if selected:
                del collection[table.index(selected[0])]
                self.show_editor()

        ttk.Button(buttons, text="Edit selected", command=edit).pack(side="left", padx=4)
        ttk.Button(buttons, text="Remove selected", command=remove).pack(side="left")

    def collection_dialog(self, collection, headers, title, choices, index=None):
        dialog = tk.Toplevel(self.root)
        dialog.title(f"Edit {title[:-1] if title.endswith('s') else title}")
        dialog.transient(self.root)
        dialog.grab_set()
        frame = self.dialog_frame(
            dialog, title,
            "Fill in the columns of this table. Only the first column is mandatory; leave anything you do not "
            "need empty.",
        )
        current = collection[index].copy() if index is not None else {}
        reference_fields = {
            "Cost rules": {
                "cost_id": ("costs.csv", True),
                "trigger_ref": ("events.csv", False),
            },
            "Cost rule conditions": {
                "rule_id": ("cost_rules.csv", True),
            },
        }.get(title, {})
        variables = {
            header: self.field(
                frame,
                header.replace("_", " ").title(),
                current.get(header, ""),
                choices.get(header),
                reference_file=reference_fields.get(header, (None, False))[0],
                required=reference_fields.get(header, (None, False))[1],
            )
            for header in headers
        }

        def accept():
            row = {header: variables[header].get().strip() for header in headers}
            identity = "id" if "id" in headers else headers[0]
            if not row[identity]:
                messagebox.showerror(title, f"The {identity} field is required.", parent=dialog)
                return
            if index is None:
                collection.append(row)
            else:
                collection[index] = row
            dialog.destroy()
            self.show_editor()

        ttk.Button(frame, text="Cancel", command=dialog.destroy).pack(side="left", pady=12)
        ttk.Button(frame, text="Save", command=accept).pack(side="right", pady=12)

    def tables_editor(self):
        self.section(
            "8. All dataset tables",
            "Raw access to every CSV the engine reads, including tables the guided steps do not cover such as "
            "player characteristics and obligations. Each column shows its own explanation while editing.",
        )
        ttk.Button(self.body, text="Read dataset tutorial", command=self.show_tutorial).pack(anchor="w", pady=(0, 8))
        for index, (title, filename, _, _) in enumerate(GENERIC_SECTIONS):
            ttk.Button(
                self.body,
                text=f"{index + 1}. {title} ({filename})",
                command=lambda value=index: self.generic_table_editor(value),
            ).pack(fill="x", pady=2)

    def generic_table_editor(self, section_index):
        title, filename, default_headers, explanation = GENERIC_SECTIONS[section_index]
        path = os.path.join(self.dataset_path, filename)
        headers = list(default_headers)
        rows = []
        if os.path.exists(path):
            with open(path, newline="", encoding="utf-8") as handle:
                reader = csv.DictReader(handle)
                if reader.fieldnames:
                    headers = list(reader.fieldnames)
                rows = [{header: row.get(header, "") for header in headers} for row in reader]
        if not rows:
            rows = [
                {header: row.get(header, "") for header in headers}
                for row in GENERIC_DEFAULT_ROWS.get(filename, [{}])
            ]
        self.context_fields = [
            (
                header,
                GENERIC_HELP.get(header, "Dataset-defined value."),
                REFERENCE_RULES.get(filename, {}).get(header, (None, False))[0],
            )
            for header in headers
        ]

        self.clear(self.body)
        ttk.Button(self.body, text="← Back to tables", command=self.tables_editor).pack(anchor="w")
        ttk.Label(self.body, text=title, style="Heading.TLabel").pack(anchor="w", pady=(8, 0))
        ttk.Label(self.body, text=f"{explanation}\n\nEdits are written to {filename} as soon as you save an entry.",
                  style="Hint.TLabel", wraplength=330).pack(anchor="w", pady=(4, 10))
        table = ttk.Treeview(self.body, columns=headers, show="headings", height=12)
        for header in headers:
            table.heading(header, text=header)
            table.column(header, width=max(90, min(180, len(header) * 9)))
        table.pack(fill="both", expand=True)
        for row in rows:
            table.insert("", "end", values=[row.get(header, "") for header in headers])

        def save_rows(show_message=True):
            write_csv(path, headers, rows)
            if show_message:
                messagebox.showinfo("Saved", f"Saved {filename}.", parent=self.root)

        def edit_row(index=None):
            dialog = tk.Toplevel(self.root)
            dialog.title(f"Edit {title}")
            dialog.transient(self.root)
            dialog.grab_set()
            dialog.configure(background=THEME["bg"])
            frame = ttk.Frame(dialog, padding=14)
            frame.pack(fill="both", expand=True)
            ttk.Label(frame, text=f"{title} entry", style="Heading.TLabel").grid(
                row=0, column=0, columnspan=3, sticky="w", pady=(0, 2))
            ttk.Label(frame, text="Left: column name. Middle: your value. Right: what the engine does with it.",
                      style="Hint.TLabel").grid(row=1, column=0, columnspan=3, sticky="w", pady=(0, 8))
            current = rows[index].copy() if index is not None else {header: "" for header in headers}
            variables = {}
            for row_number, header in enumerate(headers):
                ttk.Label(frame, text=header).grid(row=row_number + 2, column=0, sticky="nw", padx=(0, 10), pady=3)
                choices = GENERIC_CHOICES.get(header)
                reference = REFERENCE_RULES.get(filename, {}).get(header)
                required_reference = (filename, header) in MANDATORY_REFERENCE_FIELDS
                if header.startswith("cost_"):
                    choices = [cost["id"] for cost in self.costs if cost.get("id")]
                variable = tk.StringVar(value=current.get(header, ""))
                variables[header] = variable
                if reference:
                    target_file = reference[0]
                    values = self.reference_values(target_file)
                    if not required_reference:
                        values = [""] + values
                    widget = ttk.Combobox(
                        frame,
                        textvariable=variable,
                        values=values,
                        state="readonly",
                        width=38,
                    )
                    ttk.Button(
                        frame,
                        text="Add new...",
                        command=lambda target=target_file: self.open_reference_editor(target),
                    ).grid(row=row_number + 2, column=3, padx=(4, 0), pady=3)
                else:
                    widget = ttk.Combobox(frame, textvariable=variable, values=choices, width=38) if choices else ttk.Entry(
                        frame, textvariable=variable, width=42
                    )
                widget.grid(row=row_number + 2, column=1, sticky="ew", pady=3)
                ttk.Label(
                    frame,
                    text=GENERIC_HELP.get(header, "Dataset-defined value."),
                    wraplength=360,
                    style="Hint.TLabel",
                ).grid(row=row_number + 2, column=2, sticky="w", padx=(10, 0), pady=3)
            frame.columnconfigure(1, weight=1)

            def accept():
                entry = {header: variables[header].get().strip() for header in headers}
                identity = "variable" if filename == "config.csv" else "id"
                if not entry.get(identity):
                    messagebox.showerror("Entry", f"{identity} is required.", parent=dialog)
                    return
                if index is None:
                    rows.append(entry)
                else:
                    rows[index] = entry
                save_rows(False)
                dialog.destroy()
                self.generic_table_editor(section_index)

            ttk.Button(frame, text="Cancel", command=dialog.destroy).grid(
                row=len(headers) + 2, column=1, sticky="e", pady=(10, 0)
            )
            ttk.Button(frame, text="Save entry", command=accept).grid(
                row=len(headers) + 2, column=2, sticky="e", pady=(10, 0)
            )

        def selected_index():
            selected = table.selection()
            return table.index(selected[0]) if selected else None

        buttons = ttk.Frame(self.body)
        buttons.pack(fill="x", pady=8)
        ttk.Button(buttons, text="Add", command=lambda: edit_row()).pack(side="left")
        ttk.Button(buttons, text="Edit", command=lambda: edit_row(selected_index()) if selected_index() is not None else None).pack(side="left", padx=5)

        def delete_row():
            index = selected_index()
            if index is None:
                return
            if messagebox.askyesno("Delete", "Delete the selected entry?", parent=self.root):
                del rows[index]
                save_rows(False)
                self.generic_table_editor(section_index)

        ttk.Button(buttons, text="Delete", command=delete_row).pack(side="left")
        ttk.Button(buttons, text="Save", command=save_rows).pack(side="right")
        self.show_context_panel()

    def color_value(self, element_id):
        return next(row["hex_color"] for row in self.colors if row["element_id"] == element_id)

    def update_color(self, element_id, value, refresh=True):
        value = value.strip().upper()
        if not HEX_COLOR.fullmatch(value):
            return
        for row in self.colors:
            if row["element_id"] == element_id:
                row["hex_color"] = value
                break
        if refresh:
            self.show_preview()

    def choose_color(self, element_id):
        selected = colorchooser.askcolor(color=self.color_value(element_id), parent=self.root)[1]
        if selected:
            self.color_vars[element_id].set(selected.upper())
            self.update_color(element_id, selected)

    def reset_color(self, element_id):
        default = next(row["default_hex"] for row in self.colors if row["element_id"] == element_id)
        self.color_vars[element_id].set(default)
        self.update_color(element_id, default)

    def save_colors(self):
        for element_id, variable in self.color_vars.items():
            value = variable.get().strip().upper()
            if not HEX_COLOR.fullmatch(value):
                messagebox.showerror("Colors", f"{element_id} must use #RRGGBB.", parent=self.root)
                return
            self.update_color(element_id, value, refresh=False)
        write_csv(os.path.join(self.dataset_path, "colors.csv"), COLOR_HEADERS, self.colors)
        messagebox.showinfo("Colors", f"Colors saved to:\n{self.dataset_path}", parent=self.root)

    def reset_colors(self):
        if not messagebox.askyesno("Colors", "Reset every color to its default value?", parent=self.root):
            return
        for row in self.colors:
            row["hex_color"] = row["default_hex"]
            if row["element_id"] in self.color_vars:
                self.color_vars[row["element_id"]].set(row["default_hex"])
        self.show_preview()

    def colors_editor(self):
        self.color_vars.clear()
        self.section(
            "Colors",
            "Every colour of the game interface. The same values are used by the mockup on the right and by the "
            "real Tauri application, so what you see here is what players get. Values must be #RRGGBB.",
        )
        content = ttk.Frame(self.body)
        content.pack(fill="both", expand=True)

        categories = {}
        for row in self.colors:
            categories.setdefault(row.get("category", "Other"), []).append(row)
        for category, rows in categories.items():
            group = ttk.LabelFrame(content, text=category, padding=6)
            group.pack(fill="x", pady=4)
            for row in rows:
                element_id = row["element_id"]
                line = ttk.Frame(group)
                line.pack(fill="x", pady=2)
                ttk.Label(line, text=row["label"], width=22).pack(side="left")
                swatch = tk.Canvas(line, width=28, height=20, highlightthickness=1, highlightbackground="#777")
                swatch.pack(side="left", padx=4)
                variable = tk.StringVar(value=row["hex_color"])
                self.color_vars[element_id] = variable
                entry = ttk.Entry(line, textvariable=variable, width=10)
                entry.pack(side="left")
                variable.trace_add("write", lambda *_args, key=element_id, canvas=swatch, value=variable: self._color_changed(key, canvas, value))
                ttk.Button(line, text="Pick", command=lambda key=element_id: self.choose_color(key)).pack(side="left", padx=3)
                ttk.Button(line, text="Reset", command=lambda key=element_id: self.reset_color(key)).pack(side="left")
                self._color_changed(element_id, swatch, variable)

        buttons = ttk.Frame(self.body)
        buttons.pack(fill="x", pady=8)
        ttk.Button(buttons, text="Reset all defaults", command=self.reset_colors).pack(side="left")
        ttk.Button(buttons, text="Save colors", command=self.save_colors).pack(side="right")

    def _color_changed(self, element_id, swatch, variable):
        value = variable.get().strip().upper()
        if HEX_COLOR.fullmatch(value):
            swatch.configure(background=value)
            self.update_color(element_id, value)

    def show_preview(self):
        if not self.preview:
            return
        if self.step not in ("dashboard", "colors"):
            self.show_context_panel()
            return
        self.clear(self.preview)
        ttk.Label(self.preview, text="Live application mockup", style="Heading.TLabel").pack(anchor="w")
        ttk.Label(
            self.preview,
            text="An approximation of how the running game will look with your current settings and colours.",
            style="Hint.TLabel",
        ).pack(anchor="w", pady=(2, 0))
        mock = tk.Frame(
            self.preview,
            relief="groove",
            borderwidth=2,
            background=self.color_value("app_background"),
        )
        mock.pack(fill="both", expand=True, pady=(8, 0))
        tk.Label(mock, text=self.config["application_name"], font=("TkDefaultFont", 18, "bold"),
                 background=self.color_value("app_background"), foreground=self.color_value("primary_text")).pack(anchor="w", padx=14, pady=(12, 2))
        tk.Label(mock, text=f"Day 1    {self.config['currency_symbol']}20,000",
                 background=self.color_value("app_background"), foreground=self.color_value("muted_text")).pack(anchor="w", padx=14)
        tabs = tk.Frame(mock, background=self.color_value("app_background"))
        tabs.pack(fill="x", padx=10, pady=10)
        for title in ("Dashboard", self.config["inventory_name"], self.config["dealer_name"], self.config["event_plural"]):
            tk.Label(tabs, text=title, background=self.color_value("primary_accent"),
                     foreground=self.color_value("white_text"), padx=10, pady=5).pack(side="left", padx=2)
        dashboard = tk.Frame(mock, background=self.color_value("surface_background"), padx=12, pady=12)
        dashboard.pack(fill="both", expand=True, padx=10, pady=(0, 10))
        tk.Label(dashboard, text="Overview", font=("TkDefaultFont", 13, "bold"),
                 background=self.color_value("surface_background"), foreground=self.color_value("primary_text")).pack(anchor="w")
        for text in (
            f"Budget: {self.config['currency_symbol']}20,000",
            f"Inventory: {len(self.objects)} items",
            f"Event log: {len(self.events)} events configured",
        ):
            tk.Label(dashboard, text=text, background=self.color_value("surface_background"),
                     foreground=self.color_value("secondary_text")).pack(anchor="w", pady=4)
        for tab in self.inventory_tabs:
            page = tk.Frame(dashboard, background=self.color_value("surface_background"))
            matching = [item for item in self.objects if item["type"] in tab["types"].split(";")]
            tk.Label(page, text=f"{tab['name']} ({tab['types']})",
                     background=self.color_value("surface_background"), foreground=self.color_value("primary_text"),
                     font=("TkDefaultFont", 13, "bold")).pack(anchor="w")
            tk.Label(page, text=", ".join(item["name"] for item in matching) or "No items yet",
                     background=self.color_value("surface_background"), foreground=self.color_value("secondary_text")).pack(anchor="w", pady=6)
        dealer = tk.Frame(dashboard, background=self.color_value("surface_background"))
        tk.Label(dealer, text=self.config["dealer_name"], font=("TkDefaultFont", 13, "bold"),
                 background=self.color_value("surface_background"), foreground=self.color_value("primary_text")).pack(anchor="w")
        for item in self.objects:
            tk.Label(dealer, text=f"{item['name']}  {self.config['currency_symbol']}{item['price']}  [{item['type']}]",
                     background=self.color_value("surface_background"), foreground=self.color_value("secondary_text")).pack(anchor="w", pady=2)
        tk.Label(dashboard, text=f"{len(self.events)} scheduled {self.config['event_plural'].lower()}",
                 background=self.color_value("info_background"), foreground=self.color_value("white_text"),
                 padx=8, pady=5).pack(anchor="w", pady=8)
        tk.Label(dashboard, text=f"{len(self.quests)} quests or championships",
                 background=self.color_value("success_background"), foreground=self.color_value("white_text"),
                 padx=8, pady=5).pack(anchor="w")

    def show_context_panel(self):
        self.clear(self.preview)
        ttk.Label(self.preview, text="Field navigator", style="Heading.TLabel").pack(anchor="w")
        ttk.Label(
            self.preview,
            text="Select a variable to see its meaning, linked records, and current values available "
                 "in this dataset.",
            style="Hint.TLabel",
            wraplength=420,
        ).pack(anchor="w", pady=(2, 8))
        fields_by_step = {
            "inventory": [
                ("tab_id", FIELD_HELP["tab_id"], None),
                ("types", FIELD_HELP["tab_types"], None),
            ],
            "dealer": [
                ("id", FIELD_HELP["item_id"], None),
                ("type", FIELD_HELP["item_type"], None),
                ("price", FIELD_HELP["item_price"], None),
                ("cost_id", FIELD_HELP["cost_id"], "costs.csv"),
            ],
            "events": [
                ("id", FIELD_HELP["event_id"], None),
                ("required_license_id", FIELD_HELP["event_license"], "objects.csv"),
                ("required_object_ids", FIELD_HELP["event_required"], "objects.csv"),
                ("quest_id", FIELD_HELP["event_quest"], "quests.csv"),
            ],
            "quests": [
                ("id", FIELD_HELP["quest_id"], None),
                ("required_license_id", FIELD_HELP["quest_license"], "objects.csv"),
            ],
            "advanced": [
                ("cost_id", FIELD_HELP["cost_id"], "costs.csv"),
                ("trigger_ref", "Optional event referenced by this rule.", "events.csv"),
                ("rule_id", "Cost rule containing this condition.", "cost_rules.csv"),
            ],
        }
        fields = self.context_fields or fields_by_step.get(self.step, [])
        if not fields:
            fields = [("field", "Choose a table or guided step on the left.", None)]
        table = ttk.Treeview(self.preview, columns=("field", "linked"), show="headings", height=7)
        table.heading("field", text="Variable")
        table.heading("linked", text="Linked records")
        table.column("field", width=170)
        table.column("linked", width=170)
        table.pack(fill="x")
        for name, _description, reference in fields:
            table.insert("", "end", values=(name, reference or "free value"))
        details = ttk.Frame(self.preview)
        details.pack(fill="both", expand=True, pady=(8, 0))

        def show_field(_event=None):
            self.clear(details)
            selected = table.selection()
            if not selected:
                return
            name, description, reference = fields[table.index(selected[0])]
            ttk.Label(details, text=name, style="Sub.TLabel").pack(anchor="w")
            ttk.Label(details, text=description, style="Hint.TLabel", wraplength=420).pack(
                anchor="w", pady=(3, 8)
            )
            if not reference:
                ttk.Label(
                    details,
                    text="This is a dataset-defined value. Edit it using the form on the left.",
                    style="Hint.TLabel",
                    wraplength=420,
                ).pack(anchor="w")
                return
            values = self.reference_values(reference)
            ttk.Label(
                details,
                text=f"Current {reference} values ({len(values)}):",
                style="Sub.TLabel",
            ).pack(anchor="w")
            values_box = tk.Listbox(
                details,
                height=min(7, max(3, len(values))),
                background=THEME["field"],
                foreground=THEME["text"],
                relief="flat",
            )
            values_box.pack(fill="both", expand=True, pady=(4, 8))
            for value in values:
                values_box.insert("end", value)
            ttk.Button(
                details,
                text=f"Add new {reference[:-4]}",
                command=lambda target=reference: self.open_reference_editor(target),
            ).pack(anchor="e")

        table.bind("<<TreeviewSelect>>", show_field)
        first = table.get_children()[0]
        table.selection_set(first)
        table.focus(first)
        show_field()

    def choose_folder(self):
        selected = filedialog.askdirectory(initialdir=self.dataset_path)
        if selected:
            self.dataset_path = selected
            self.path_var.set(selected)
            self.load_visual_data()
            self.save_settings()
            self.show_preview()

    def reset(self):
        if messagebox.askyesno("Start over", "Clear the current visual design?"):
            self.config.update({
                "application_name": "My Game", "currency_symbol": "$", "inventory_name": "Inventory",
                "dealer_name": "Market", "object_name": "Item", "object_plural": "Items",
                "event_name": "Event", "event_plural": "Events",
            })
            self.inventory_tabs.clear()
            self.objects.clear()
            self.costs.clear()
            self.events.clear()
            self.quests.clear()
            self.cost_rules.clear()
            self.cost_conditions.clear()
            self.extra_config_rows.clear()
            self.show_shell()

    def show_tutorial(self):
        dialog = tk.Toplevel(self.root)
        dialog.title("Dataset tutorial")
        dialog.geometry("950x700")
        text = tk.Text(dialog, wrap="word")
        text.pack(side="left", fill="both", expand=True)
        scrollbar = ttk.Scrollbar(dialog, orient="vertical", command=text.yview)
        scrollbar.pack(side="right", fill="y")
        text.configure(yscrollcommand=scrollbar.set)
        text.insert("1.0", TUTORIAL_GUIDE)
        text.configure(state="disabled")

    def export(self):
        config_rows = list(self.extra_config_rows)
        config_rows.extend({"variable": key, "value": value} for key, value in self.config.items())
        for tab in self.inventory_tabs:
            config_rows.extend([
                {"variable": f"inventory_tab_{tab['id']}_name", "value": tab["name"]},
                {"variable": f"inventory_tab_{tab['id']}_types", "value": tab["types"]},
            ])
        write_csv(os.path.join(self.dataset_path, "config.csv"), ["variable", "value"], config_rows)
        object_headers = [
            "id", "type", "name", "price",
            *[f"cost_{number}" for number in range(1, 16)],
            *[f"service_{number}_interval_days" for number in range(1, 16)],
            "resale_initial_percent", "resale_annual_percent", "resale_min_percent",
            "description_html", "license_level", "license_previous_id", "requires_object_ids",
            "license_fee", "lifetime_days", "availability_days", "image_path",
            "requirement_group", "paddock_cred_bonus", "trophy_championship", "trophy_position", "trophy_level",
        ]
        write_csv(os.path.join(self.dataset_path, "objects.csv"), object_headers, self.objects)
        write_csv(os.path.join(self.dataset_path, "costs.csv"), ["id", "name", "amount"], self.costs)
        event_headers = [
            "id", "name", "day_of_year", "entry_fee", "reward_pool", "charisma_reward",
            "duration_value", "duration_unit", "tags", "description_html",
            "required_license_id", "required_object_ids", "quest_id", "position_rewards",
            "type", "resolution_method", "success_rate", "encounter_id", "base_cost",
            "stamina_cost", "risk_factor", "payout", "payout_freq_type", "payout_freq",
            "payout_freq_unit", "sponsor_quest_id", "sponsor_object_id", "sponsor_payouts",
            "sponsor_equipment_ids",
        ]
        write_csv(os.path.join(self.dataset_path, "events.csv"), event_headers, self.events)
        quest_headers = [
            "id", "type", "name", "success_points", "failure_points", "join_fee",
            "required_license_id", "description_html",
        ]
        write_csv(os.path.join(self.dataset_path, "quests.csv"), quest_headers, self.quests)
        write_csv(os.path.join(self.dataset_path, "cost_rules.csv"), COST_RULE_HEADERS, self.cost_rules)
        write_csv(
            os.path.join(self.dataset_path, "cost_rule_conditions.csv"),
            COST_CONDITION_HEADERS,
            self.cost_conditions,
        )
        write_csv(os.path.join(self.dataset_path, "colors.csv"), COLOR_HEADERS, self.colors)
        os.makedirs(os.path.join(self.dataset_path, "html"), exist_ok=True)
        for filename, title in (
            ("object.html", "Object"),
            ("event.html", "Event"),
            ("event_activity.html", "Event activity"),
            ("quest.html", "Quest"),
        ):
            path = os.path.join(self.dataset_path, "html", filename)
            if not os.path.exists(path):
                with open(path, "w", encoding="utf-8") as handle:
                    handle.write(f"<h1>{title}</h1><p>Describe this entry here.</p>\n")
        messagebox.showinfo("Dataset exported", f"Dataset files written to:\n{self.dataset_path}", parent=self.root)


if __name__ == "__main__":
    root = tk.Tk()
    DatasetDesigner(root)
    root.mainloop()
