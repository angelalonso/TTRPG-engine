#!/usr/bin/env python3
"""Guided, game-agnostic editor for dataset CSV files."""

import csv
import os
import tkinter as tk
from tkinter import filedialog, messagebox, ttk


SECTIONS = [
    ("Game", "config.csv", ["variable", "value"],
     "Set the game's visible names, calendar settings, currency, and other labels."),
    ("Main character", "player.csv", ["id", "name", "value", "min_value", "max_value"],
     "Define numeric characteristics such as budget, charisma, health, or reputation."),
    ("Objects", "objects.csv",
     ["id", "type", "name", "price"] + [f"cost_{index}" for index in range(1, 16)] +
      [f"service_{index}_interval_days" for index in range(1, 16)] +
      ["resale_initial_percent", "resale_annual_percent", "resale_min_percent", "description_html",
       "license_level", "license_previous_id", "requires_object_ids", "license_fee", "lifetime_days",
       "availability_days", "image_path"],
     "Define things the player can acquire. The engine does not assume what an object represents."),
    ("Events", "events.csv",
     ["id", "name", "day_of_year", "entry_fee", "reward_pool", "charisma_reward",
      "duration_value", "duration_unit", "tags", "description_html", "required_license_id",
      "required_object_ids", "quest_id"],
     "Define scheduled activities. Rewards are granted when the recorded result is successful."),
    ("Quests", "quests.csv",
     ["id", "type", "name", "success_points", "failure_points", "join_fee", "required_license_id", "description_html"],
     "Group events into quests. A quest with type=championship can charge join_fee and require a licence before its events become available."),
    ("Actions", "actions.csv",
     ["id", "name", "type", "base_cost", "stamina_cost", "risk_factor", "success_rate", "payout",
      "payout_freq_type", "payout_freq", "payout_freq_unit", "description_html"],
     "Define one-time or recurring actions available to the player."),
    ("Costs", "costs.csv", ["id", "name", "amount"],
     "Create reusable monetary costs referenced by objects and cost rules."),
    ("Cost rules", "cost_rules.csv",
     ["id", "cost_id", "trigger_type", "trigger_ref", "amount_multiplier",
     "probability", "interval_days", "charge_mode", "resolution_mode", "pending_message",
     "message", "damage_type", "unavailable_days", "event_interval", "no_event_days"],
     "Connect costs to elapsed time, events, actions, or object acquisition. Resolution mode can charge money or create an object service requirement."),
    ("Cost rule conditions", "cost_rule_conditions.csv",
     ["rule_id", "subject_type", "subject_ref", "operator", "value"],
     "Limit a cost rule to matching event, action, object, or player facts."),
]

CHOICES = {
    "duration_unit": ["minutes", "hours", "days", "weeks"],
    "payout_freq_type": ["once", "recurring"],
    "payout_freq_unit": ["day", "week", "month", "year"],
    "trigger_type": ["day_elapsed", "event_completed", "action_completed", "object_acquired"],
    "charge_mode": ["immediate", "pending"],
    "resolution_mode": ["charge", "object_service"],
    "subject_type": ["event", "action", "object", "player"],
    "operator": ["equals", "contains", "greater_than", "less_than"],
}

HELP = {
    "id": "Stable identifier used by other dataset files. Use lowercase letters and underscores.",
    "name": "Human-readable name shown in the application.",
    "value": "Numeric starting value for a player characteristic, or text for a configuration label.",
    "type": "Optional dataset-defined category for grouping or rule conditions.",
    "price": "Purchase price in the dataset currency.",
    "day_of_year": "Calendar day on which this event is available.",
    "entry_fee": "Amount charged when the event is entered.",
    "reward_pool": "Budget reward granted after a successful result.",
    "charisma_reward": "Charisma reward granted after a successful result.",
    "duration_value": "Length of the event. Whole calendar days advance the calendar; hours and minutes do not.",
    "duration_unit": "Choose minutes, hours, days, or weeks.",
    "success_rate": "Probability from 0 to 1 that an action succeeds.",
    "payout_freq_type": "Use once for an immediate payout or recurring for an active action.",
    "payout_freq_unit": "Unit used by a recurring payout.",
    "probability": "Probability from 0 to 1 that a matching cost rule is applied.",
    "charge_mode": "Immediate charges now when possible; pending records an amount for later payment.",
    "resolution_mode": "Use charge for monetary costs, or object_service to leave the cost unpaid and require the mapped object service in the garage.",
    "pending_message": "Optional message shown when this rule creates a pending cost. Supports {cost_name}, {object_id}, {currency}, and {amount}.",
    "message": "Optional message shown when this cost occurs. Supports {cost_name}, {object_id}, {currency}, and {amount}.",
    "damage_type": "Optional result choice that activates this rule after an event.",
    "unavailable_days": "Number of calendar days an affected object cannot be used.",
    "event_interval": "Minimum race count interval for an automatic event damage rule.",
    "no_event_days": "Trigger a daily rule after this many days without a completed race.",
    "lifetime_days": "Optional object lifetime. The object expires after this many calendar days.",
    "required_license_id": "License object ID required before entering this event.",
    "required_object_ids": "Optional semicolon-separated object IDs. An event can require one of these specific objects.",
    "quest_id": "Optional quest ID that groups this event with other events.",
    "join_fee": "Budget charged once when the player joins a quest.",
    "license_level": "Numeric level used to identify the highest owned license.",
    "license_previous_id": "Object ID of the previous license required before acquisition.",
    "requires_object_ids": "Semicolon-separated object IDs required before acquisition.",
    "license_fee": "Dataset-defined license fee. The object price remains the acquisition charge.",
    "trigger_type": "What causes the cost rule to be evaluated.",
    "subject_type": "Kind of fact checked by a condition.",
    "operator": "How the actual value is compared with the configured value.",
    "description_html": "Dataset-relative HTML file displayed in the detail popup.",
    "image_path": "Optional dataset-relative image path used as a market thumbnail.",
    "availability_days": "Days after purchase before this object can be used.",
    "stamina_cost": "Stamina consumed when the action is started. Recurring actions also consume this each day.",
    "resale_initial_percent": "Fraction of the original price retained immediately after purchase (0.75 means 25% depreciation).",
    "resale_annual_percent": "Fraction retained at each anniversary after the initial depreciation.",
    "resale_min_percent": "Lowest fraction of the original price the resale value may reach.",
}

TUTORIAL_GUIDE = """DATASET EDITOR GUIDE

This editor creates a complete dataset folder for the generic game engine.
Game behavior is compiled into the Rust/React application; game-specific
names, values, schedules, descriptions, and images belong in these CSV files.
Choose a folder, complete each section, and use Save and continue. The default
folder is dataset_tutorial. When finished, the editor creates starter HTML
description files in an html subfolder.

RUNNING A DATASET
Start the editor with: python3 dataset_editor.py
After creating a dataset, launch the game with DATASET_PATH=/path/to/dataset
and the normal Tauri development or production command.

CONFIGURATION (config.csv)
This is a variable/value table for labels and general settings. Use
inventory_tab_<id>_name and inventory_tab_<id>_types to create inventory tabs;
types are semicolon-separated. Use currency_symbol, days_per_year, event
labels, market_default_sort, and sickness settings for game-wide behavior.
The player starts at the age row in player.csv, not in config.csv.

PLAYER (player.csv)
Each row is id,name,value,min_value,max_value. Values are numeric starting
characteristics such as age, budget, stamina, health, mana, or reputation.
Finite max_value values produce dashboard bars. The age row is interpreted as
years at game start and is then advanced daily. Use min_value and max_value
when a characteristic must be bounded.

OBJECTS AND INVENTORY (objects.csv)
Objects have an id, type, name, price, costs, service intervals, resale
settings, description_html, prerequisites, lifetime_days, availability_days,
and optional image_path. Types are free-form and determine which inventory tab
shows an object. availability_days is set on each object; for example, put 5
on every vehicle to make a purchased car usable after five days. Use
requires_object_ids for semicolon-separated prerequisites and lifetime_days
for temporary objects. Images and HTML paths are relative to the dataset.

EVENTS (events.csv)
Events define id, name, day_of_year, entry_fee, reward_pool, charisma_reward,
duration_value, duration_unit, tags, description_html, requirements, and an
optional quest_id. Racing events should be scheduled on calendar days 5, 6,
or 7 of each week. Several events may share a day. required_object_ids means
one of the listed objects; required_license_id requires a licence object.
The result popup records a player's position and any selected damage/result.

QUESTS AND CHAMPIONSHIPS (quests.csv)
Use type=championship for a multi-race championship. Set success_points,
failure_points, join_fee, required_license_id, and description_html, then put
the same quest_id on its events. The game has a separate Championships tab
where the player can enter their position and the point-scoring competitors
after each race; standings are calculated from those results.

ACTIONS AND JOBS (actions.csv)
Actions define id, name, type, base_cost, stamina_cost, risk_factor,
success_rate, payout, payout_freq_type, payout_freq, payout_freq_unit, and
description_html. Use once for one-off work or recurring for an income source.
Recurring salary intervals can be day, week, month, or year. Work actions are
jobs and only one job can be active at a time. Put the full explanation in the
linked HTML file, including salary timing and sickness behavior.

COSTS AND COST RULES
costs.csv contains reusable id,name,amount definitions. cost_rules.csv
connects costs to day_elapsed, event_completed, action_completed, or
object_acquired triggers. probability controls how often a rule applies;
interval_days limits repeated triggers. charge_mode can be immediate or
pending. resolution_mode=object_service creates a service requirement instead
of a direct charge, and unavailable_days blocks the object temporarily.
cost_rule_conditions.csv can restrict a rule to an event, action, object, or
player value using equals, contains, greater_than, or less_than.

DESCRIPTIONS, IMAGES, AND RELOADING
HTML descriptions are dataset-relative paths such as ./html/event.html.
Market thumbnails use an image_path on the object row and are optional.
Keep the CSV headers stable and use lowercase underscore IDs. Save the dataset,
then reload the game or create a new game so changed catalog data is used;
existing save files retain their own player progress.
"""

DEFAULT_ROWS = {
    "config.csv": [
        {"variable": "application_name", "value": "My Game"},
        {"variable": "object_name", "value": "Object"},
        {"variable": "object_plural", "value": "Objects"},
        {"variable": "inventory_name", "value": "Inventory"},
        {"variable": "dealer_name", "value": "Racing Market"},
        {"variable": "event_name", "value": "Event"},
        {"variable": "event_plural", "value": "Events"},
        {"variable": "action_name", "value": "Actions"},
        {"variable": "currency_symbol", "value": "$"},
        {"variable": "days_per_year", "value": "365"},
        {"variable": "speed_icon_paused", "value": "/img/pause.svg"},
        {"variable": "speed_icon_normal", "value": "/img/normal.svg"},
        {"variable": "speed_icon_fast", "value": "/img/fast.svg"},
        {"variable": "speed_icon_fastest", "value": "/img/fastest.svg"},
        {"variable": "settings_icon", "value": "/img/settings.svg"},
        {"variable": "save_icon", "value": "/img/save.svg"},
        {"variable": "load_icon", "value": "/img/load.svg"},
        {"variable": "income_sources_name", "value": "Income sources"},
        {"variable": "inventory_service_bay_name", "value": "Service Bay"},
        {"variable": "inventory_tab_drivers_room_name", "value": "Driver's room"},
        {"variable": "inventory_tab_drivers_room_types", "value": "equipment;license"},
        {"variable": "market_default_sort", "value": "price"},
        {"variable": "sickness_daily_probability", "value": "0.001111111"},
        {"variable": "sickness_recovery_stamina", "value": "50"},
        {"variable": "sickness_final_recovery", "value": "50"},
        {"variable": "sickness_event_name", "value": "Sickness"},
        {"variable": "sickness_event_message", "value": "You are sick. Stamina is 0 for four days, then recovers to 50 for two days. Jobs do not pay during this sickness week."},
    ],
    "player.csv": [
        {"id": "age", "name": "Age", "value": "18", "min_value": "0", "max_value": ""},
        {"id": "budget", "name": "Budget", "value": "20000", "min_value": "0", "max_value": ""},
        {"id": "charisma", "name": "Charisma", "value": "1", "min_value": "0", "max_value": ""},
        {"id": "stamina", "name": "Stamina", "value": "100", "min_value": "0", "max_value": "100"},
    ],
    "objects.csv": [
        {
            "id": "starter_object", "type": "item", "name": "Starter Object", "price": "1000",
            "cost_1": "starter_service", "cost_2": "", "cost_3": "", "cost_4": "",
            "service_1_interval_days": "0", "service_2_interval_days": "0", "service_3_interval_days": "0",
            "service_4_interval_days": "0", "description_html": "./html/object.html",
            "resale_initial_percent": "0.75", "resale_annual_percent": "0.9", "resale_min_percent": "0.1",
            "availability_days": "", "image_path": "",
        },
    ],
    "events.csv": [
        {
            "id": "starter_event", "name": "Starter Event", "day_of_year": "30",
            "entry_fee": "50", "reward_pool": "250", "charisma_reward": "1",
            "duration_value": "1", "duration_unit": "day",
            "tags": "example", "description_html": "./html/event.html",
        },
    ],
    "actions.csv": [
        {
            "id": "starter_action", "name": "Starter Action", "type": "general",
            "base_cost": "0", "stamina_cost": "1", "risk_factor": "0", "success_rate": "1",
            "payout": "100", "payout_freq_type": "once", "payout_freq": "0",
            "payout_freq_unit": "day", "description_html": "./html/action.html",
        },
    ],
    "costs.csv": [
        {"id": "starter_service", "name": "Starter Service", "amount": "100"},
    ],
    "cost_rules.csv": [
        {
            "id": "starter_daily_cost", "cost_id": "starter_service",
            "trigger_type": "day_elapsed", "trigger_ref": "",
            "amount_multiplier": "1", "probability": "1",
            "interval_days": "30", "charge_mode": "immediate", "resolution_mode": "charge",
            "pending_message": "",
        },
    ],
    "cost_rule_conditions.csv": [
        {
            "rule_id": "starter_daily_cost", "subject_type": "player",
            "subject_ref": "budget", "operator": "greater_than", "value": "0",
        },
    ],
}


class DatasetEditor:
    def __init__(self, root):
        self.root = root
        self.root.title("Game Dataset Editor")
        self.root.geometry("1100x700")
        self.dataset_path = ""
        self.section_index = 0
        self.headers = []
        self.rows = []
        self.tree = None
        self.show_start()

    def clear(self):
        for child in self.root.winfo_children():
            child.destroy()

    def show_start(self):
        self.clear()
        frame = ttk.Frame(self.root, padding=30)
        frame.pack(fill="both", expand=True)
        ttk.Label(frame, text="Game Dataset Editor", font=("TkDefaultFont", 20, "bold")).pack(anchor="w")
        ttk.Label(
            frame,
            text=("This guided editor writes the same CSV dataset format used by the game engine. "
                  "It is intentionally generic: you define the game's labels, character, objects, "
                  "events, actions, and economy."),
            wraplength=850,
        ).pack(anchor="w", pady=(12, 24))
        path_row = ttk.Frame(frame)
        path_row.pack(fill="x")
        self.path_var = tk.StringVar(value="dataset_tutorial")
        ttk.Entry(path_row, textvariable=self.path_var).pack(side="left", fill="x", expand=True)
        ttk.Button(path_row, text="Choose folder", command=self.choose_folder).pack(side="left", padx=(8, 0))
        ttk.Button(frame, text="Start with game settings", command=self.start).pack(anchor="e", pady=24)
        ttk.Button(frame, text="Read dataset tutorial", command=self.show_tutorial).pack(anchor="e")

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

    def choose_folder(self):
        selected = filedialog.askdirectory(initialdir=self.path_var.get() or os.getcwd())
        if selected:
            self.path_var.set(selected)

    def start(self):
        path = os.path.abspath(self.path_var.get())
        os.makedirs(path, exist_ok=True)
        self.dataset_path = path
        self.section_index = 0
        self.show_section()

    def section(self):
        return SECTIONS[self.section_index]

    def load_rows(self):
        _, filename, default_headers, _ = self.section()
        path = os.path.join(self.dataset_path, filename)
        self.headers = list(default_headers)
        self.rows = []
        if os.path.exists(path):
            with open(path, newline="", encoding="utf-8") as handle:
                reader = csv.DictReader(handle)
                if reader.fieldnames:
                    self.headers = list(reader.fieldnames)
                self.rows = [{header: row.get(header, "") for header in self.headers} for row in reader]
        if not self.rows:
            self.rows = [
                {header: row.get(header, "") for header in self.headers}
                for row in DEFAULT_ROWS.get(filename, [{}])
            ]

    def show_section(self, reload_rows=True):
        self.clear()
        title, _, _, explanation = self.section()
        if reload_rows:
            self.load_rows()
        frame = ttk.Frame(self.root, padding=16)
        frame.pack(fill="both", expand=True)
        ttk.Label(frame, text=f"{self.section_index + 1}. {title}",
                  font=("TkDefaultFont", 18, "bold")).pack(anchor="w")
        ttk.Label(frame, text=explanation, wraplength=1000).pack(anchor="w", pady=(6, 12))

        table_frame = ttk.Frame(frame)
        table_frame.pack(fill="both", expand=True)
        self.tree = ttk.Treeview(table_frame, columns=self.headers, show="headings")
        for header in self.headers:
            self.tree.heading(header, text=header)
            self.tree.column(header, width=max(110, min(220, len(header) * 10)), anchor="w")
        scrollbar = ttk.Scrollbar(table_frame, orient="vertical", command=self.tree.yview)
        self.tree.configure(yscrollcommand=scrollbar.set)
        self.tree.pack(side="left", fill="both", expand=True)
        scrollbar.pack(side="right", fill="y")
        self.tree.bind("<Double-1>", self.edit_double_clicked)
        for row in self.rows:
            self.tree.insert("", "end", values=[row.get(header, "") for header in self.headers])

        buttons = ttk.Frame(frame)
        buttons.pack(fill="x", pady=(12, 0))
        ttk.Button(buttons, text="Add", command=lambda: self.edit_row()).pack(side="left")
        ttk.Button(buttons, text="Edit", command=self.edit_selected).pack(side="left", padx=6)
        ttk.Button(buttons, text="Delete", command=self.delete_selected).pack(side="left")
        ttk.Button(buttons, text="Save", command=self.save).pack(side="left", padx=(24, 0))
        if self.section_index:
            ttk.Button(buttons, text="Back", command=self.previous).pack(side="right", padx=6)
        next_text = "Finish" if self.section_index == len(SECTIONS) - 1 else "Save and continue"
        ttk.Button(buttons, text=next_text, command=self.next).pack(side="right")

    def edit_selected(self):
        selected = self.tree.selection()
        if not selected:
            messagebox.showinfo("Edit", "Select a row first.")
            return
        index = self.tree.index(selected[0])
        self.edit_row(index)

    def edit_double_clicked(self, event):
        row_id = self.tree.identify_row(event.y)
        if not row_id:
            return
        self.tree.selection_set(row_id)
        self.tree.focus(row_id)
        self.edit_row(self.tree.index(row_id))

    def edit_row(self, index=None):
        dialog = tk.Toplevel(self.root)
        dialog.title("Edit entry")
        dialog.transient(self.root)
        values = self.rows[index].copy() if index is not None else {header: "" for header in self.headers}
        variables = {}
        body = ttk.Frame(dialog, padding=14)
        body.pack(fill="both", expand=True)
        for row_number, header in enumerate(self.headers):
            ttk.Label(body, text=header).grid(row=row_number, column=0, sticky="nw", padx=(0, 12), pady=4)
            variable = tk.StringVar(value=values.get(header, ""))
            variables[header] = variable
            choices = CHOICES.get(header)
            if header.startswith("cost_"):
                choices = self.cost_choices()
            if choices:
                widget = ttk.Combobox(body, textvariable=variable, values=choices, state="readonly", width=38)
            else:
                widget = ttk.Entry(body, textvariable=variable, width=42)
            widget.grid(row=row_number, column=1, sticky="ew", pady=4)
            help_text = HELP.get(header, "Dataset-defined value. Use the format expected by this field.")
            ttk.Label(body, text=help_text, wraplength=360).grid(
                row=row_number, column=2, sticky="w", padx=(12, 0), pady=4
            )
        body.columnconfigure(1, weight=1)

        def accept():
            entry = {header: variables[header].get().strip() for header in self.headers}
            identity = "variable" if self.section()[1] == "config.csv" else "id"
            if not entry.get(identity):
                messagebox.showerror("Entry", f"{identity} is required.", parent=dialog)
                return
            if index is None:
                self.rows.append(entry)
            else:
                self.rows[index] = entry
            self.save(show_message=False)
            dialog.destroy()
            self.show_section(reload_rows=False)

        ttk.Button(body, text="Cancel", command=dialog.destroy).grid(
            row=len(self.headers), column=1, sticky="e", pady=(12, 0)
        )
        ttk.Button(body, text="Save entry", command=accept).grid(
            row=len(self.headers), column=2, sticky="e", pady=(12, 0)
        )
        dialog.wait_visibility()
        dialog.grab_set()
        dialog.focus_set()

    def cost_choices(self):
        path = os.path.join(self.dataset_path, "costs.csv")
        if not os.path.exists(path):
            return []
        with open(path, newline="", encoding="utf-8") as handle:
            return [row.get("id", "") for row in csv.DictReader(handle) if row.get("id", "")]

    def delete_selected(self):
        selected = self.tree.selection()
        if not selected:
            messagebox.showinfo("Delete", "Select a row first.")
            return
        if messagebox.askyesno("Delete", "Delete the selected entry?"):
            del self.rows[self.tree.index(selected[0])]
            self.save(show_message=False)
            self.show_section(reload_rows=False)

    def save(self, show_message=True):
        _, filename, _, _ = self.section()
        os.makedirs(self.dataset_path, exist_ok=True)
        path = os.path.join(self.dataset_path, filename)
        with open(path, "w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=self.headers)
            writer.writeheader()
            writer.writerows({header: row.get(header, "") for header in self.headers} for row in self.rows)
        if show_message:
            messagebox.showinfo("Saved", f"Saved {filename}.")

    def next(self):
        self.save()
        if self.section_index < len(SECTIONS) - 1:
            self.section_index += 1
            self.show_section()
        else:
            messagebox.showinfo("Complete", "The dataset setup is complete.")
            html_dir = os.path.join(self.dataset_path, "html")
            os.makedirs(html_dir, exist_ok=True)
            for filename, title in (
                ("object.html", "Object"),
                ("event.html", "Event"),
                ("action.html", "Action"),
                ("quest.html", "Quest"),
            ):
                path = os.path.join(html_dir, filename)
                if not os.path.exists(path):
                    with open(path, "w", encoding="utf-8") as handle:
                        handle.write(f"<h1>{title}</h1><p>Describe this entry here.</p>\n")
            self.show_start()

    def previous(self):
        self.save()
        self.section_index -= 1
        self.show_section()


if __name__ == "__main__":
    root = tk.Tk()
    DatasetEditor(root)
    root.mainloop()
