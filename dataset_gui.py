#!/usr/bin/env python3
"""Visual dataset designer with a live mockup of the game UI.

This is intentionally separate from dataset_editor.py. It guides a new dataset
through the parts most visible in the application before exporting CSV files.
"""

import csv
import os
import re
import tempfile
import tkinter as tk
from tkinter import colorchooser, filedialog, messagebox, ttk


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


def load_color_rows(path):
    if not os.path.exists(path):
        rows = default_color_rows()
        write_csv(path, COLOR_HEADERS, rows)
        return rows
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
        messagebox.showwarning(
            "Colors",
            f"Could not load colors.csv ({error}). Default colors will be restored.",
        )
        rows = default_color_rows()
        write_csv(path, COLOR_HEADERS, rows)
        return rows


class DatasetDesigner:
    def __init__(self, root):
        self.root = root
        self.root.title("Visual Game Dataset Designer")
        self.root.geometry("1280x780")
        self.dataset_path = os.path.abspath("dataset")
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
        }
        self.inventory_tabs = []
        self.objects = []
        self.costs = []
        self.events = []
        self.quests = []
        self.cost_rules = []
        self.cost_conditions = []
        self.colors = load_color_rows(os.path.join(self.dataset_path, "colors.csv"))
        self.color_vars = {}
        self.preview = None
        self.body = None
        self.path_var = tk.StringVar(value=self.dataset_path)
        self.show_shell()

    def clear(self, parent):
        for child in parent.winfo_children():
            child.destroy()

    def show_shell(self):
        self.clear(self.root)
        header = ttk.Frame(self.root, padding=10)
        header.pack(fill="x")
        ttk.Label(header, text="Visual Game Dataset Designer",
                  font=("TkDefaultFont", 18, "bold")).pack(side="left")
        ttk.Label(header, textvariable=self.path_var).pack(side="right")

        body = ttk.PanedWindow(self.root, orient="horizontal")
        body.pack(fill="both", expand=True, padx=10, pady=(0, 10))
        left = ttk.Frame(body, padding=10)
        right = ttk.Frame(body, padding=10)
        body.add(left, weight=1)
        body.add(right, weight=3)
        self.body = left
        self.preview = right
        self.show_steps()
        self.show_preview()

    def show_steps(self):
        self.clear(self.body)
        ttk.Label(self.body, text="Build your game", font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
        ttk.Label(
            self.body,
            text="Work from the Dashboard down. Every change is reflected in the mockup.",
            wraplength=310,
        ).pack(anchor="w", pady=(4, 14))
        for key, title in (
            ("dashboard", "1. Dashboard"),
            ("inventory", "2. Inventory tabs"),
            ("dealer", "3. Dealer and items"),
            ("events", "4. Events"),
            ("quests", "5. Quests"),
            ("colors", "6. Colors"),
            ("advanced", "7. Events and cost rules"),
            ("export", "8. Export dataset"),
        ):
            ttk.Button(self.body, text=title, command=lambda value=key: self.open_step(value)).pack(
                fill="x", pady=3
            )
        ttk.Separator(self.body).pack(fill="x", pady=12)
        ttk.Button(self.body, text="Choose output folder", command=self.choose_folder).pack(fill="x")
        ttk.Button(self.body, text="Start over", command=self.reset).pack(fill="x", pady=5)

    def open_step(self, step):
        self.step = step
        if step == "export":
            self.export()
            return
        self.show_editor()

    def show_editor(self):
        self.clear(self.body)
        ttk.Button(self.body, text="← Back to steps", command=self.show_steps).pack(anchor="w")
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

    def field(self, parent, label, value="", choices=None):
        row = ttk.Frame(parent)
        row.pack(fill="x", pady=4)
        ttk.Label(row, text=label, width=20).pack(side="left")
        variable = tk.StringVar(value=value)
        if choices:
            widget = ttk.Combobox(row, textvariable=variable, values=choices, state="readonly")
        else:
            widget = ttk.Entry(row, textvariable=variable)
        widget.pack(side="left", fill="x", expand=True)
        return variable

    def dashboard_editor(self):
        ttk.Label(self.body, text="Dashboard", font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
        ttk.Label(self.body, text="Choose the names and labels shown before adding game content.",
                  wraplength=330).pack(anchor="w", pady=(4, 12))
        variables = {
            key: self.field(self.body, label, self.config[key])
            for key, label in (
                ("application_name", "Application name"),
                ("currency_symbol", "Currency symbol"),
                ("inventory_name", "Inventory tab name"),
                ("dealer_name", "Dealer tab name"),
                ("object_name", "Singular item name"),
                ("object_plural", "Plural item name"),
                ("event_name", "Singular event name"),
                ("event_plural", "Plural event name"),
            )
        }

        def save():
            self.config.update({key: variable.get().strip() for key, variable in variables.items()})
            self.show_preview()
            messagebox.showinfo("Dashboard", "Dashboard labels updated.", parent=self.root)

        ttk.Button(self.body, text="Apply dashboard changes", command=save).pack(anchor="e", pady=14)

    def inventory_editor(self):
        ttk.Label(self.body, text="Inventory tabs", font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
        ttk.Label(self.body, text="Add a tab, choose its name, and select which object types it contains.",
                  wraplength=330).pack(anchor="w", pady=(4, 10))
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
            frame = ttk.Frame(dialog, padding=14)
            frame.pack(fill="both", expand=True)
            tab_id = self.field(frame, "Tab id")
            name = self.field(frame, "Tab name")
            types = self.field(frame, "Object types", "vehicle;equipment")

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
        ttk.Label(self.body, text=self.config["dealer_name"], font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
        ttk.Label(self.body, text="Add items, choose their category, and assign existing costs or prerequisites.",
                  wraplength=330).pack(anchor="w", pady=(4, 10))
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
        ttk.Label(self.body, text="Reusable costs", font=("TkDefaultFont", 11, "bold")).pack(anchor="w")
        cost_row = ttk.Frame(self.body)
        cost_row.pack(fill="x", pady=4)
        cost_id = self.field(cost_row, "Cost id")
        cost_name = self.field(cost_row, "Name")
        amount = self.field(cost_row, "Amount")

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
        frame = ttk.Frame(dialog, padding=14)
        frame.pack(fill="both", expand=True)
        current = self.objects[index].copy() if index is not None else {}
        item_id = self.field(frame, "Item id", current.get("id", ""))
        item_type = self.field(frame, "Item type", current.get("type", ""))
        name = self.field(frame, "Name", current.get("name", ""))
        price = self.field(frame, "Price", current.get("price", "0"))
        ttk.Label(frame, text="Service costs and intervals", font=("TkDefaultFont", 10, "bold")).pack(anchor="w", pady=(8, 2))
        cost_variables = {
            f"cost_{number}": self.field(frame, f"Cost {number}", current.get(f"cost_{number}", ""))
            for number in range(1, 16)
        }
        interval_variables = {
            f"service_{number}_interval_days": self.field(
                frame, f"Service {number} interval (days)", current.get(f"service_{number}_interval_days", "0")
            )
            for number in range(1, 16)
        }
        ttk.Label(frame, text="Acquisition and display settings", font=("TkDefaultFont", 10, "bold")).pack(anchor="w", pady=(8, 2))
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
        ttk.Label(frame, text="Required objects (choose any existing items)").pack(anchor="w", pady=(8, 2))
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

        ttk.Button(frame, text="Cancel", command=dialog.destroy).pack(side="left", pady=12)
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
        ttk.Label(self.body, text=self.config["event_plural"], font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
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
        frame = ttk.Frame(dialog, padding=14)
        frame.pack(fill="both", expand=True)
        event_id = self.field(frame, "Event id")
        name = self.field(frame, "Name")
        day = self.field(frame, "Day of year", "1")
        entry_fee = self.field(frame, "Entry fee", "0")
        reward = self.field(frame, "Reward", "0")
        duration = self.field(frame, "Duration", "1")
        duration_unit = self.field(frame, "Duration unit", "day", EVENT_DURATIONS)
        tag = self.field(frame, "Tag", "example", EVENT_TAGS)
        license_ids = [item["id"] for item in self.objects if item["type"] == "license"]
        license_id = self.field(frame, "Required license", "", [""] + license_ids)
        object_ids = [item["id"] for item in self.objects]
        required = self.field(frame, "Required object", "", [""] + object_ids)
        quest_ids = [quest["id"] for quest in self.quests]
        quest_id = self.field(frame, "Quest", "", [""] + quest_ids)

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
        ttk.Label(self.body, text="Quests and championships", font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
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
        frame = ttk.Frame(dialog, padding=14)
        frame.pack(fill="both", expand=True)
        quest_id = self.field(frame, "Quest id")
        quest_type = self.field(frame, "Type", "championship", QUEST_TYPES)
        name = self.field(frame, "Name")
        points = self.field(frame, "Success points", "10")
        join_fee = self.field(frame, "Join fee", "0")
        license_ids = [item["id"] for item in self.objects if item["type"] == "license"]
        license_id = self.field(frame, "Required license", "", [""] + license_ids)

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
        ttk.Label(self.body, text="Events and cost rules", font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
        ttk.Label(
            self.body,
            text="These sections expose the generic engine configuration instead of assuming a racing game.",
            wraplength=330,
        ).pack(anchor="w", pady=(4, 10))
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
        ttk.Label(self.body, text=title, font=("TkDefaultFont", 11, "bold")).pack(anchor="w", pady=(8, 2))
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
        frame = ttk.Frame(dialog, padding=14)
        frame.pack(fill="both", expand=True)
        current = collection[index].copy() if index is not None else {}
        variables = {
            header: self.field(frame, header.replace("_", " ").title(), current.get(header, ""), choices.get(header))
            for header in headers
        }

        def accept():
            row = {header: variables[header].get().strip() for header in headers}
            if not row["id"]:
                messagebox.showerror(title, "The id field is required.", parent=dialog)
                return
            if index is None:
                collection.append(row)
            else:
                collection[index] = row
            dialog.destroy()
            self.show_editor()

        ttk.Button(frame, text="Cancel", command=dialog.destroy).pack(side="left", pady=12)
        ttk.Button(frame, text="Save", command=accept).pack(side="right", pady=12)

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
        ttk.Label(self.body, text="Colors", font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
        ttk.Label(
            self.body,
            text="Edit the theme shared by the Python preview and the Tauri application. Values must be #RRGGBB.",
            wraplength=330,
        ).pack(anchor="w", pady=(4, 10))
        canvas = tk.Canvas(self.body, highlightthickness=0)
        scrollbar = ttk.Scrollbar(self.body, orient="vertical", command=canvas.yview)
        content = ttk.Frame(canvas)
        content.bind("<Configure>", lambda event: canvas.configure(scrollregion=canvas.bbox("all")))
        canvas.create_window((0, 0), window=content, anchor="nw")
        canvas.configure(yscrollcommand=scrollbar.set)
        canvas.pack(side="left", fill="both", expand=True)
        scrollbar.pack(side="right", fill="y")

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
        self.clear(self.preview)
        ttk.Label(self.preview, text="Live application mockup",
                  font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
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

    def choose_folder(self):
        selected = filedialog.askdirectory(initialdir=self.dataset_path)
        if selected:
            self.dataset_path = selected
            self.path_var.set(selected)
            self.colors = load_color_rows(os.path.join(self.dataset_path, "colors.csv"))
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
            self.show_shell()

    def export(self):
        config_rows = [{"variable": key, "value": value} for key, value in self.config.items()]
        for tab in self.inventory_tabs:
            config_rows.extend([
                {"variable": f"inventory_tab_{tab['id']}_name", "value": tab["name"]},
                {"variable": f"inventory_tab_{tab['id']}_types", "value": tab["types"]},
            ])
        write_csv(os.path.join(self.dataset_path, "config.csv"), ["variable", "value"], config_rows)
        object_headers = [
            "id", "type", "name", "price", "cost_1", "cost_2", "cost_3", "cost_4",
            "service_1_interval_days", "service_2_interval_days", "service_3_interval_days",
            "service_4_interval_days", "resale_initial_percent", "resale_annual_percent",
            "resale_min_percent", "description_html", "license_level", "license_previous_id",
            "requires_object_ids", "license_fee", "lifetime_days", "availability_days", "image_path",
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
        for filename, title in (("object.html", "Object"), ("event.html", "Event"), ("quest.html", "Quest")):
            path = os.path.join(self.dataset_path, "html", filename)
            if not os.path.exists(path):
                with open(path, "w", encoding="utf-8") as handle:
                    handle.write(f"<h1>{title}</h1><p>Describe this entry here.</p>\n")
        messagebox.showinfo("Dataset exported", f"Dataset files written to:\n{self.dataset_path}", parent=self.root)


if __name__ == "__main__":
    root = tk.Tk()
    DatasetDesigner(root)
    root.mainloop()
