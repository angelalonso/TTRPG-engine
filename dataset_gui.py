#!/usr/bin/env python3
"""Visual dataset designer with a live mockup of the game UI.

This is intentionally separate from dataset_editor.py. It guides a new dataset
through the parts most visible in the application before exporting CSV files.
"""

import csv
import os
import tkinter as tk
from tkinter import filedialog, messagebox, ttk


OBJECT_TYPES = ("vehicle", "equipment", "license", "item")
EVENT_TAGS = ("race", "track_day", "championship", "example")
EVENT_DURATIONS = ("minutes", "hours", "days", "weeks")
QUEST_TYPES = ("championship", "quest", "generic")


def write_csv(path, headers, rows):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=headers)
        writer.writeheader()
        writer.writerows({header: row.get(header, "") for header in headers} for row in rows)


class DatasetDesigner:
    def __init__(self, root):
        self.root = root
        self.root.title("Visual Game Dataset Designer")
        self.root.geometry("1280x780")
        self.dataset_path = os.path.abspath("dataset_tutorial_gui")
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
            ("export", "6. Export dataset"),
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
            ttk.Label(frame, text="Object types").pack(anchor="w", pady=(8, 2))
            selected = {}
            for object_type in OBJECT_TYPES:
                selected[object_type] = tk.BooleanVar()
                ttk.Checkbutton(frame, text=object_type, variable=selected[object_type]).pack(anchor="w")

            def accept():
                types = [value for value, variable in selected.items() if variable.get()]
                if not tab_id.get().strip() or not name.get().strip() or not types:
                    messagebox.showerror("Inventory tab", "Id, name, and at least one object type are required.", parent=dialog)
                    return
                self.inventory_tabs.append({"id": tab_id.get().strip(), "name": name.get().strip(), "types": ";".join(types)})
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
        item_type = self.field(frame, "Item type", current.get("type", "item"), OBJECT_TYPES)
        name = self.field(frame, "Name", current.get("name", ""))
        price = self.field(frame, "Price", current.get("price", "0"))
        cost_choices = [cost["id"] for cost in self.costs] or ["(add a cost first)"]
        cost = self.field(frame, "Service cost", current.get("cost_1", ""), cost_choices)
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
                "price": price.get().strip(), "cost_1": "" if cost.get().startswith("(") else cost.get(),
                "requires_object_ids": ";".join(selected),
                "description_html": "./html/object.html",
                "resale_initial_percent": "0.75", "resale_annual_percent": "0.9",
                "resale_min_percent": "0.1",
            }
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

    def show_preview(self):
        if not self.preview:
            return
        self.clear(self.preview)
        ttk.Label(self.preview, text="Live application mockup",
                  font=("TkDefaultFont", 15, "bold")).pack(anchor="w")
        mock = ttk.Frame(self.preview, relief="groove", borderwidth=2)
        mock.pack(fill="both", expand=True, pady=(8, 0))
        ttk.Label(mock, text=self.config["application_name"],
                  font=("TkDefaultFont", 18, "bold")).pack(anchor="w", padx=14, pady=(12, 2))
        ttk.Label(mock, text=f"Day 1    {self.config['currency_symbol']}20,000").pack(anchor="w", padx=14)
        notebook = ttk.Notebook(mock)
        notebook.pack(fill="both", expand=True, padx=10, pady=10)
        dashboard = ttk.Frame(notebook, padding=12)
        notebook.add(dashboard, text="Dashboard")
        ttk.Label(dashboard, text="Overview", font=("TkDefaultFont", 13, "bold")).pack(anchor="w")
        ttk.Label(dashboard, text=f"Budget: {self.config['currency_symbol']}20,000").pack(anchor="w", pady=4)
        ttk.Label(dashboard, text=f"Inventory: {len(self.objects)} items").pack(anchor="w", pady=4)
        ttk.Label(dashboard, text=f"Event log: {len(self.events)} events configured").pack(anchor="w", pady=4)
        for tab in self.inventory_tabs:
            page = ttk.Frame(notebook, padding=12)
            notebook.add(page, text=tab["name"] or tab["id"])
            ttk.Label(page, text=f"{tab['name']} ({tab['types']})",
                      font=("TkDefaultFont", 13, "bold")).pack(anchor="w")
            matching = [item for item in self.objects if item["type"] in tab["types"].split(";")]
            ttk.Label(page, text=", ".join(item["name"] for item in matching) or "No items yet").pack(anchor="w", pady=6)
        dealer = ttk.Frame(notebook, padding=12)
        notebook.add(dealer, text=self.config["dealer_name"])
        ttk.Label(dealer, text=self.config["dealer_name"], font=("TkDefaultFont", 13, "bold")).pack(anchor="w")
        for item in self.objects:
            ttk.Label(dealer, text=f"{item['name']}  {self.config['currency_symbol']}{item['price']}  [{item['type']}]").pack(anchor="w", pady=2)
        events = ttk.Frame(notebook, padding=12)
        notebook.add(events, text=self.config["event_plural"])
        ttk.Label(events, text=f"{len(self.events)} scheduled {self.config['event_plural'].lower()}").pack(anchor="w")
        quests = ttk.Frame(notebook, padding=12)
        notebook.add(quests, text="Quests")
        ttk.Label(quests, text=f"{len(self.quests)} quests or championships").pack(anchor="w")

    def choose_folder(self):
        selected = filedialog.askdirectory(initialdir=self.dataset_path)
        if selected:
            self.dataset_path = selected
            self.path_var.set(selected)

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
            "required_license_id", "required_object_ids", "quest_id",
        ]
        write_csv(os.path.join(self.dataset_path, "events.csv"), event_headers, self.events)
        quest_headers = [
            "id", "type", "name", "success_points", "failure_points", "join_fee",
            "required_license_id", "description_html",
        ]
        write_csv(os.path.join(self.dataset_path, "quests.csv"), quest_headers, self.quests)
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
