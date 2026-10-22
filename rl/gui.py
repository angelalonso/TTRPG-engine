"""Optional beginner-friendly Tk launcher; all simulation/training stays in the CLI."""

from __future__ import annotations

import argparse
import csv
import json
import queue
import shlex
import threading
import tkinter as tk
from datetime import datetime
from pathlib import Path
from tkinter import filedialog, messagebox, scrolledtext, ttk

from gui_workflow import PHASES, REWARDS, Runner, Settings, build_commands, model_file, validate_rewards, write_configuration
from translate_playtest import action_name


ROOT = Path(__file__).resolve().parents[1]

FIELDS = {
    "dataset": ("Dataset folder", "Rules and content to simulate. Changing the dataset or goal usually requires a new model.", "directory"),
    "goal": ("Goal", "An engine goal, e.g. charisma>=10 or championships:level=1,count=1. Use stable characteristic IDs, not display names.", ""),
    "model": ("Model file", "Training writes this .zip model; evaluation reads it. Use a new name for a new experiment.", "save"),
    "resume": ("Resume checkpoint (optional)", "Continue this compatible model. Timesteps below count additional training; blank means learn from scratch.", "file"),
    "episodes": ("Evaluation episodes", "20 is a first check; use 100+ for a promising policy. Compare success rate before winning duration.", ""),
    "max_days": ("Game-day limit per episode", "A time limit is not proof of a dead end. Long calendar waits may be necessary.", ""),
    "max_steps": ("Action limit per episode", "Stops endless action loops. One chosen action is not necessarily one game day.", ""),
    "timesteps": ("Training actions", "Total practice actions requested. PPO finishes whole rollouts and may slightly exceed this number. More practice is not a cure for impossible goals.", ""),
    "gamma": ("Future reward discount", "0.999 values distant goals more than 0.99. It discounts per action, not per day; very long plans may need tuning.", ""),
    "ent_coef": ("Exploration bonus", "0.01 encourages trying alternatives during training. Increase modestly for repetitive policies; too much prevents settling on a route.", ""),
    "n_steps": ("Actions per rollout", "Practice collected before a PPO update. Must be divisible by batch size. Default 2048 is a starting point, not an optimum.", ""),
    "batch_size": ("Learning batch size", "How many rollout examples are used per update batch. Default 64; must divide the rollout size.", ""),
    "checkpoint_interval": ("Checkpoint every N actions", "Saves recoverable model snapshots. Stop also requests an interrupted checkpoint once a model exists.", ""),
    "stagnation_window": ("No-progress warning after N actions", "Warn only: sparse goals and waiting can legitimately show no progress. Inspect the route, not just the warning.", ""),
    "train_seed": ("Training seed start", "Repeatable initial random sequence. Training automatically varies later episode seeds.", ""),
    "eval_seed": ("Evaluation seed start", "Random baseline and evaluation share this seed range for comparison. Keep it separate from training.", ""),
    "play_seed": ("Manual / inspection seed", "Use a known successful seed to replay a candidate route. One good seed is not proof of reliability.", ""),
    "python": ("RL Python executable", "Normally .venv/bin/python, with rl/requirements.txt installed. The GUI itself only needs Python and Tk.", "file"),
    "binary": ("Rust simulator executable", "Normally src-tauri/target/release/rl_env. Prepare tools builds this release binary.", "file"),
    "trace_input": ("Existing report input (optional)", "For report-only use: choose a playtest JSON or collected JSONL trace. Otherwise the latest selected trace is used.", "file"),
}

INT_FIELDS = {
    "episodes", "max_days", "max_steps", "timesteps", "n_steps", "batch_size",
    "checkpoint_interval", "stagnation_window", "train_seed", "eval_seed", "play_seed",
}


class PlaytestGui:
    def __init__(self, root: tk.Tk, defaults: Settings):
        self.root = root
        self.events: queue.Queue = queue.Queue()
        self.runner: Runner | None = None
        self.running = False
        self.closing = False
        self.output: Path | None = None
        self.actions: list[dict] = []
        self.manual_catalog: dict[str, dict[str, dict[str, str]]] = {}
        self.manual_ready = False
        self.inputs: list[tuple[tk.Widget, str]] = []
        self.values = {key: tk.StringVar(value=str(getattr(defaults, key))) for key in FIELDS}
        self.phases = {key: tk.BooleanVar(value=key in defaults.phases) for key in PHASES}
        self.reward_enabled = {key: tk.BooleanVar(value=False) for key in REWARDS}
        self.reward_values = {key: tk.StringVar(value=default) for key, (_, default, _) in REWARDS.items()}
        self.trace = tk.StringVar(value=defaults.trace)
        self.stochastic = tk.BooleanVar(value=defaults.stochastic)
        self.verbose_training = tk.BooleanVar(value=defaults.verbose_training)
        self.auto_scroll = tk.BooleanVar(value=True)
        self.status = tk.StringVar(value="Ready. Choose your phases, review settings, then preview the commands.")
        self.root.title("Playtest lab - learn and inspect game routes")
        self.root.geometry("1180x850")
        self.root.minsize(850, 620)
        self.root.protocol("WM_DELETE_WINDOW", self.close)
        self.root.bind("<Configure>", self.resize_labels)
        header = ttk.Frame(root, padding=12)
        header.pack(fill="x")
        ttk.Label(header, text="Playtest lab", font=("", 18, "bold")).pack(anchor="w")
        ttk.Label(header, text="Play first, measure a baseline, then train and evaluate. Success rate comes before speed.\n"
                  "Runs start fresh simulated games, not your saves. Simulation results are not proof of real-world event performance.",
                  wraplength=1100).pack(anchor="w", pady=(4, 0))
        self.tabs = ttk.Notebook(root)
        self.tabs.pack(fill="both", expand=True, padx=12)
        workflow = self.page("1. Workflow")
        settings = self.page("2. Settings")
        rewards = self.page("3. Rewards & penalties")
        self.manual_tab = ttk.Frame(self.tabs, padding=10)
        self.tabs.add(self.manual_tab, text="4. Manual play")
        self.log_tab = ttk.Frame(self.tabs, padding=8)
        self.tabs.add(self.log_tab, text="5. Live log")
        self.workflow_page(workflow)
        self.settings_page(settings)
        self.rewards_page(rewards)
        self.manual_page()
        self.log = scrolledtext.ScrolledText(self.log_tab, wrap="word", state="disabled", font=("TkFixedFont", 10))
        self.log.pack(fill="both", expand=True)
        ttk.Checkbutton(self.log_tab, text="Follow live output (full output is also saved to session.log)", variable=self.auto_scroll).pack(anchor="w")
        footer = ttk.Frame(root, padding=12)
        footer.pack(fill="x")
        self.preview_button = ttk.Button(footer, text="Preview commands", command=self.preview)
        self.preview_button.pack(side="left")
        self.start_button = ttk.Button(footer, text="Start selected phases", command=self.start)
        self.start_button.pack(side="left", padx=8)
        self.stop_button = ttk.Button(footer, text="Stop safely", command=self.stop, state="disabled")
        self.stop_button.pack(side="left")
        ttk.Button(footer, text="Save visible log...", command=self.save_log).pack(side="right")
        ttk.Label(root, textvariable=self.status, wraplength=1100, padding=(12, 0, 12, 10)).pack(fill="x")
        self.root.after(60, self.poll)

    def track(self, widget: tk.Widget, normal: str = "normal") -> tk.Widget:
        self.inputs.append((widget, normal))
        return widget

    def resize_labels(self, event: tk.Event) -> None:
        if event.widget is not self.root:
            return
        def resize(widget: tk.Widget) -> None:
            for child in widget.winfo_children():
                if isinstance(child, ttk.Label) and int(child.cget("wraplength") or 0):
                    child.configure(wraplength=max(200, event.width - 90))
                resize(child)
        resize(self.root)

    def page(self, title: str) -> ttk.Frame:
        outer = ttk.Frame(self.tabs)
        self.tabs.add(outer, text=title)
        canvas = tk.Canvas(outer, highlightthickness=0)
        scroll = ttk.Scrollbar(outer, orient="vertical", command=canvas.yview)
        frame = ttk.Frame(canvas, padding=12)
        window = canvas.create_window((0, 0), window=frame, anchor="nw")
        frame.bind("<Configure>", lambda _: canvas.configure(scrollregion=canvas.bbox("all")))
        canvas.bind("<Configure>", lambda event: canvas.itemconfigure(window, width=event.width))
        canvas.configure(yscrollcommand=scroll.set)
        canvas.pack(side="left", fill="both", expand=True)
        scroll.pack(side="right", fill="y")
        canvas.bind("<Enter>", lambda _: canvas.focus_set())
        canvas.bind("<MouseWheel>", lambda event: canvas.yview_scroll(-int(event.delta / 120), "units"))
        canvas.bind("<Button-4>", lambda _: canvas.yview_scroll(-1, "units"))
        canvas.bind("<Button-5>", lambda _: canvas.yview_scroll(1, "units"))
        return frame

    def workflow_page(self, frame: ttk.Frame) -> None:
        ttk.Label(frame, text="Select phases - they always run top to bottom", font=("", 12, "bold")).pack(anchor="w")
        for key, (title, help_text) in PHASES.items():
            box = ttk.Frame(frame, padding=(0, 8))
            box.pack(fill="x")
            self.track(ttk.Checkbutton(box, text=title, variable=self.phases[key])).pack(anchor="w")
            ttk.Label(box, text=help_text, wraplength=1000, padding=(24, 2)).pack(anchor="w")
        buttons = ttk.Frame(frame)
        buttons.pack(fill="x", pady=8)
        for title, keys in (
            ("Just play manually", ("manual",)),
            ("First experiment", ("baseline", "train", "evaluate", "inspect", "report")),
            ("Evaluate existing model", ("evaluate", "inspect", "report")),
            ("Select all", tuple(PHASES)), ("Deselect all", ()),
        ):
            self.track(ttk.Button(buttons, text=title, command=lambda selected=keys: self.select_phases(selected))).pack(side="left", padx=(0, 6))
        ttk.Label(frame, text="If a phase fails or you stop, later phases do not run. Prepare tools is opt-in and may download dependencies.\n"
                  "Manual play uses the legal-action chooser, not a training demonstration. Finish it to continue to the next selected phase.\n"
                  "Start with a reachable milestone. If no wins occur, inspect failures and prerequisites before increasing training time.",
                  wraplength=1000).pack(anchor="w", pady=12)
        self.track(ttk.Button(frame, text="Open beginner guide", command=self.open_guide)).pack(anchor="w")

    def select_phases(self, keys: tuple[str, ...]) -> None:
        for key, variable in self.phases.items():
            variable.set(key in keys)

    def add_fields(self, frame: ttk.Frame, title: str, keys: tuple[str, ...]) -> None:
        group = ttk.LabelFrame(frame, text=title, padding=12)
        group.pack(fill="x", pady=8)
        group.columnconfigure(1, weight=1)
        for index, key in enumerate(keys):
            label, explanation, chooser = FIELDS[key]
            row = index * 2
            ttk.Label(group, text=label).grid(row=row, column=0, sticky="w", padx=(0, 12))
            self.track(ttk.Entry(group, textvariable=self.values[key])).grid(row=row, column=1, sticky="ew")
            if chooser:
                self.track(ttk.Button(group, text="Browse...", command=lambda k=key, kind=chooser: self.browse(k, kind))).grid(row=row, column=2, padx=(8, 0))
            ttk.Label(group, text=explanation, wraplength=960).grid(row=row + 1, column=0, columnspan=3, sticky="w", pady=(3, 12))

    def settings_page(self, frame: ttk.Frame) -> None:
        self.add_fields(frame, "Game and goal", ("dataset", "goal", "model", "resume"))
        self.track(ttk.Button(frame, text="Show this dataset's characteristic IDs", command=self.show_characteristics)).pack(anchor="w")
        self.add_fields(frame, "How much practice and testing?", ("timesteps", "episodes", "max_days", "max_steps"))
        self.add_fields(frame, "Learning controls", ("gamma", "ent_coef", "n_steps", "batch_size", "checkpoint_interval", "stagnation_window"))
        self.track(ttk.Checkbutton(frame, text="Log every training action and its reward components (very verbose; can slow training)", variable=self.verbose_training)).pack(anchor="w")
        self.add_fields(frame, "Separate random sequences", ("train_seed", "eval_seed", "play_seed"))
        group = ttk.LabelFrame(frame, text="Trace and evaluation behavior", padding=12)
        group.pack(fill="x", pady=8)
        self.track(ttk.Combobox(group, textvariable=self.trace, values=("actions", "observations", "decisions"), state="readonly"), "readonly").pack(anchor="w")
        ttk.Label(group, text="Inspection: actions = choices; observations = before/after state; decisions = also policy probabilities.\n"
                  "Baseline/evaluation always retain state changes; all runs log each chosen action live.", wraplength=1000).pack(anchor="w", pady=5)
        self.track(ttk.Checkbutton(group, text="Sample policy choices during evaluation/inspection", variable=self.stochastic)).pack(anchor="w")
        ttk.Label(group, text="Off: choose the policy's most likely legal action. On: sample alternatives. Keep this consistent when comparing experiments.", wraplength=1000).pack(anchor="w")
        self.add_fields(frame, "Existing tools and report", ("python", "binary", "trace_input"))

    def rewards_page(self, frame: ttk.Frame) -> None:
        ttk.Label(frame, text="Unchecked = use the Rust engine's defaults. Checked = explicitly override for this experiment.\n"
                  "Rewards teach the player; they do not change the game's actual prices, prerequisites, bills or terminal rules.\n"
                  "Change one thing at a time and compare the same evaluation seeds. Excessive penalties can teach the player to avoid playing.",
                  wraplength=1000).pack(anchor="w", pady=(0, 12))
        buttons = ttk.Frame(frame)
        buttons.pack(fill="x")
        self.track(ttk.Button(buttons, text="Use engine defaults", command=lambda: self.set_rewards({}))).pack(side="left")
        self.track(ttk.Button(buttons, text="Gentle safety + small action cost", command=lambda: self.set_rewards({
            "low_stamina_penalty": -1, "low_budget_penalty": -1, "low_budget_threshold": 500, "step": -0.05,
        }))).pack(side="left", padx=8)
        self.track(ttk.Button(buttons, text="Import reward JSON...", command=self.import_rewards)).pack(side="left")
        self.track(ttk.Button(frame, text="Discourage trade loops (action cost only)",
                              command=lambda: self.load_rewards(ROOT / "rl/rewards.no-trade-loops.json"))).pack(anchor="w", pady=8)
        ttk.Label(frame, text="Charges every choice equally, with no extra day, progress or safety shaping. "
                  "This tests whether free trades are distracting the policy; it optimizes actions, not calendar days. "
                  "Game costs and death rules still apply. Train a new model to compare.",
                  wraplength=1000).pack(anchor="w")
        for key, (label, _, explanation) in REWARDS.items():
            group = ttk.Frame(frame, padding=(0, 8))
            group.pack(fill="x")
            self.track(ttk.Checkbutton(group, text=label, variable=self.reward_enabled[key])).pack(side="left")
            self.track(ttk.Entry(group, textvariable=self.reward_values[key], width=14)).pack(side="right")
            ttk.Label(frame, text=explanation, wraplength=1000, padding=(24, 0)).pack(anchor="w")

    def set_rewards(self, values: dict[str, float]) -> None:
        for key in REWARDS:
            self.reward_enabled[key].set(key in values)
            if key in values:
                self.reward_values[key].set(str(values[key]))

    def import_rewards(self) -> None:
        path = filedialog.askopenfilename(title="Import reward overrides", filetypes=[("JSON", "*.json")])
        if not path:
            return
        self.load_rewards(Path(path))

    def load_rewards(self, path: Path) -> None:
        try:
            values = json.loads(path.read_text(encoding="utf-8"))
            if not isinstance(values, dict) or any(type(value) not in (int, float) for value in values.values()):
                raise ValueError("Reward JSON must be an object of numeric overrides")
            validate_rewards(values)
            self.set_rewards(values)
        except (OSError, ValueError) as error:
            messagebox.showerror("Cannot import rewards", str(error))

    def manual_page(self) -> None:
        frame = self.manual_tab
        ttk.Label(frame, text="Select Play yourself in Workflow, then Start. Choose an action below; the engine controls legality.\n"
                  "Inspect the live log for rewards and consequences. This does not teach PPO from your choices.", wraplength=1050).pack(anchor="w")
        self.manual_state = scrolledtext.ScrolledText(frame, height=9, wrap="word", state="disabled")
        self.manual_state.pack(fill="x", pady=8)
        self.filter = tk.StringVar()
        self.filter.trace_add("write", lambda *_: self.refresh_actions())
        ttk.Label(frame, text="Filter actions by name, ID or stable key:").pack(anchor="w")
        ttk.Entry(frame, textvariable=self.filter).pack(fill="x", pady=4)
        self.action_help = tk.StringVar(value="Select an action to see its purpose and dataset terms before performing it.")
        ttk.Label(frame, textvariable=self.action_help, wraplength=1050).pack(fill="x", pady=6)
        self.action_list = ttk.Treeview(frame, columns=("id", "name", "key"), show="headings", selectmode="browse")
        for key, title, width in (("id", "ID", 70), ("name", "Legal action", 380), ("key", "Stable key / parameters", 500)):
            self.action_list.heading(key, text=title)
            self.action_list.column(key, width=width)
        scrollbar = ttk.Scrollbar(frame, command=self.action_list.yview)
        self.action_list.configure(yscrollcommand=scrollbar.set)
        actions = ttk.Frame(frame)
        actions.pack(side="bottom", fill="x", pady=8)
        self.choose_button = ttk.Button(actions, text="Perform selected action", command=self.choose_action, state="disabled")
        self.choose_button.pack(side="left")
        self.finish_button = ttk.Button(actions, text="Finish manual phase and export", command=lambda: self.send_manual("q"), state="disabled")
        self.finish_button.pack(side="left", padx=8)
        self.action_list.pack(side="left", fill="both", expand=True)
        scrollbar.pack(side="right", fill="y")
        self.action_list.bind("<Double-1>", lambda _: self.choose_action())
        self.action_list.bind("<<TreeviewSelect>>", lambda _: self.explain_action())

    def load_manual_catalog(self, dataset: str) -> None:
        self.manual_catalog = {}
        for kind in ("objects", "events", "quests"):
            path = ROOT / dataset / f"{kind}.csv"
            if path.is_file():
                with path.open(encoding="utf-8-sig", newline="") as source:
                    reader = csv.DictReader(source)
                    if not reader.fieldnames or "id" not in reader.fieldnames:
                        raise ValueError(f"{path} needs an id column")
                    self.manual_catalog[kind] = {row["id"]: row for row in reader}

    def action_row(self, key: str) -> dict[str, str]:
        parts = key.split(":")
        kind = {"start_activity": "events", "enter_event": "events", "buy_object": "objects",
                "sell_object": "objects", "join_quest": "quests", "resolve_pending_event": "events"}.get(parts[0], "")
        return self.manual_catalog.get(kind, {}).get(parts[1], {}) if len(parts) > 1 else {}

    def explain_action(self) -> None:
        selected = self.action_list.selection()
        if not selected:
            return
        action = next(entry for entry in self.actions if str(entry["id"]) == selected[0])
        key = action["key"]
        explanation = {
            "wait": "Advance one day. Existing bills, work, recovery and scheduled opportunities still apply.",
            "start_activity": "Start or attempt this activity. Recurring activities can add future obligations; inspect the resulting state and log.",
            "buy_object": "Acquire this object and pay its current engine-calculated price. Prerequisites and modifiers are enforced.",
            "sell_object": "Sell this owned object. The actual resale value and any restrictions are engine-controlled.",
            "join_quest": "Enrol in this quest. Enrolment is not a guarantee that you can enter every event.",
            "enter_event": "Enter this scheduled event using the selected object. Entry costs and duration apply; a result may still be pending.",
            "resolve_pending_event": "Submit this simulated result, not a prediction of external performance. Do not treat an assumed win as proven playing skill.",
            "resolve_encounter": "Choose this encounter move. The engine determines its costs and outcome.",
        }.get(key.split(":")[0], "The engine will validate this currently legal choice.")
        row = self.action_row(key)
        terms = "; ".join(f"{field.replace('_', ' ')}: {row[field]}" for field in
                          ("price", "join_fee", "entry_fee", "base_cost", "stamina_cost", "payout", "payout_freq_type", "payout_freq", "payout_freq_unit")
                          if row.get(field) not in (None, "", "0"))
        self.action_help.set(f"{row.get('name', action_name(key))}: {explanation}"
                             + (f"\nDataset base terms (modifiers may apply): {terms}" if terms else ""))

    def refresh_actions(self) -> None:
        query = self.filter.get().strip().lower()
        self.action_list.delete(*self.action_list.get_children())
        for action in self.actions:
            label = action_name(action["key"])
            row = self.action_row(action["key"])
            if row.get("name"):
                label = f"{action_name(action['key'].split(':')[0])}: {row['name']}"
                parameters = action["key"].split(":")[2:]
                if parameters:
                    label += " (" + ", ".join(self.manual_catalog.get("objects", {}).get(value, {}).get("name", value) for value in parameters) + ")"
            if query in f"{action['id']} {action['key']} {label}".lower():
                self.action_list.insert("", "end", iid=str(action["id"]), values=(action["id"], label, action["key"]))

    def choose_action(self) -> None:
        selected = self.action_list.selection()
        if selected and self.manual_ready:
            self.send_manual(selected[0])

    def send_manual(self, value: str) -> None:
        if not self.runner or not self.manual_ready:
            return
        try:
            self.runner.send(value)
            self.append_log(f"\n[manual choice] {value}\n")
            self.set_manual_ready(False)
        except (OSError, ValueError) as error:
            messagebox.showerror("Cannot send choice", str(error))

    def set_manual_ready(self, ready: bool) -> None:
        self.manual_ready = ready
        state = "normal" if ready else "disabled"
        self.choose_button.configure(state=state)
        self.finish_button.configure(state=state)

    def browse(self, key: str, kind: str) -> None:
        if kind == "directory":
            value = filedialog.askdirectory(title=FIELDS[key][0])
        elif kind == "save":
            value = filedialog.asksaveasfilename(title=FIELDS[key][0], defaultextension=".zip")
        else:
            value = filedialog.askopenfilename(title=FIELDS[key][0])
        if value:
            self.values[key].set(value)

    def read_settings(self) -> Settings:
        values = {key: variable.get().strip() for key, variable in self.values.items()}
        for key in INT_FIELDS:
            try:
                values[key] = int(values[key])
            except ValueError as error:
                raise ValueError(f"{FIELDS[key][0]} must be an integer") from error
        for key in ("gamma", "ent_coef"):
            values[key] = float(values[key])
        rewards = {key: float(self.reward_values[key].get()) for key in REWARDS if self.reward_enabled[key].get()}
        if "low_budget_start_day" in rewards and rewards["low_budget_start_day"].is_integer():
            rewards["low_budget_start_day"] = int(rewards["low_budget_start_day"])
        settings = Settings(**values, trace=self.trace.get(), stochastic=self.stochastic.get(),
                            verbose_training=self.verbose_training.get(),
                            phases=tuple(key for key in PHASES if self.phases[key].get()), rewards=rewards)
        settings.validate(ROOT)
        return settings

    def preview(self) -> None:
        try:
            settings = self.read_settings()
            if "manual" in settings.phases:
                self.load_manual_catalog(settings.dataset)
            phases = build_commands(settings, ROOT / "rl/data/gui/PREVIEW")
            self.append_log("\nCommand preview (nothing executed):\n")
            for phase in phases:
                self.append_log(f"\n{PHASES[phase.key][0]}: {PHASES[phase.key][1]}\n{shlex.join(phase.command)}\n")
            self.tabs.select(self.log_tab)
        except (OSError, ValueError) as error:
            messagebox.showerror("Review settings", str(error))

    def start(self) -> None:
        if self.running:
            return
        try:
            settings = self.read_settings()
            if "manual" in settings.phases:
                self.load_manual_catalog(settings.dataset)
            output = ROOT / "rl/data/gui" / datetime.now().strftime("%Y%m%d-%H%M%S-%f")
            phases = build_commands(settings, output)
            warning = "\n\nPrepare may download packages and compile Rust." if "prepare" in settings.phases else ""
            if "train" in settings.phases and model_file(ROOT / settings.model).exists():
                warning += "\n\nThe chosen model output already exists and will be overwritten. Choose a new model name to preserve it."
            if not messagebox.askokcancel("Start this experiment?", f"Phases: {', '.join(PHASES[key][0] for key in settings.phases)}\n"
                                         f"Goal: {settings.goal}\nOutputs: {output}\n"
                                         "This may take a long time. Stop requests an interrupted training checkpoint." + warning):
                return
            write_configuration(settings, output)
            self.output = output
            self.runner = Runner(ROOT, self.events)
            self.running = True
            self.actions = []
            self.refresh_actions()
            for widget, _ in self.inputs:
                widget.configure(state="disabled")
            self.start_button.configure(state="disabled")
            self.preview_button.configure(state="disabled")
            self.stop_button.configure(state="normal")
            self.append_log(f"\nExperiment: {output}\nConfiguration:\n{json.dumps(json.loads((output / 'configuration.json').read_text()), indent=2)}\n")
            for phase in phases:
                self.append_log(f"[command] {shlex.join(phase.command)}\n")
            self.tabs.select(self.log_tab)
            threading.Thread(target=self.runner.run, args=(phases, output), daemon=True).start()
        except (OSError, ValueError) as error:
            messagebox.showerror("Cannot start", str(error))

    def stop(self) -> None:
        if self.runner and self.running:
            self.status.set("Stopping. Waiting for checkpoint/cleanup; remaining phases will not run.")
            self.stop_button.configure(state="disabled")
            self.set_manual_ready(False)
            self.runner.cancel()

    def poll(self) -> None:
        for _ in range(250):
            try:
                kind, value = self.events.get_nowait()
            except queue.Empty:
                break
            if kind == "log":
                self.append_log(value)
            elif kind == "phase":
                self.set_manual_ready(False)
                self.status.set(f"Running: {PHASES[value][0]} - {PHASES[value][1]}")
            elif kind == "phase_done":
                self.append_log(f"\n[completed] {PHASES[value][0]}\n")
            elif kind == "manual":
                self.actions = value["actions"]
                self.manual_state.configure(state="normal")
                self.manual_state.delete("1.0", "end")
                observation = {key: entry for key, entry in value["observation"].items() if key != "legal_action_mask"}
                self.manual_state.insert("end", json.dumps(observation, indent=2))
                self.manual_state.configure(state="disabled")
                self.refresh_actions()
                self.set_manual_ready(True)
                self.tabs.select(self.manual_tab)
                self.status.set(f"Manual game: {len(self.actions)} legal actions. Choose one, or finish and export.")
            elif kind == "error":
                self.append_log(f"\nERROR: {value}\n")
                self.tabs.select(self.log_tab)
            elif kind == "finished":
                self.running = False
                self.set_manual_ready(False)
                self.stop_button.configure(state="disabled")
                self.start_button.configure(state="normal")
                self.preview_button.configure(state="normal")
                for widget, normal in self.inputs:
                    widget.configure(state=normal)
                self.status.set(f"Experiment {value}. Outputs: {self.output}")
                self.append_log(f"\n[experiment {value}] Outputs: {self.output}\n")
                if self.closing:
                    self.root.destroy()
                    return
        self.root.after(60, self.poll)

    def append_log(self, text: str) -> None:
        self.log.configure(state="normal")
        self.log.insert("end", text)
        lines = int(self.log.index("end-1c").split(".")[0])
        if lines > 12000:
            self.log.delete("1.0", f"{lines - 12000 + 1}.0")
        if self.auto_scroll.get():
            self.log.see("end")
        self.log.configure(state="disabled")

    def save_log(self) -> None:
        path = filedialog.asksaveasfilename(title="Save visible output", defaultextension=".txt")
        if path:
            try:
                Path(path).write_text(self.log.get("1.0", "end-1c"), encoding="utf-8")
            except OSError as error:
                messagebox.showerror("Cannot save log", str(error))

    def open_guide(self) -> None:
        window = tk.Toplevel(self.root)
        window.title("Beginner guide - docs/000_rl.md")
        text = scrolledtext.ScrolledText(window, wrap="word", width=100, height=35)
        text.pack(fill="both", expand=True)
        try:
            text.insert("end", (ROOT / "docs/000_rl.md").read_text(encoding="utf-8"))
        except OSError as error:
            text.insert("end", f"Cannot open guide: {error}")
        text.configure(state="disabled")

    def show_characteristics(self) -> None:
        path = ROOT / self.values["dataset"].get() / "player.csv"
        try:
            with path.open(encoding="utf-8-sig", newline="") as source:
                entries = list(csv.DictReader(source))
            messagebox.showinfo("Goal IDs", "\n".join(f"{entry['id']}: {entry.get('name', '')}" for entry in entries)
                                + "\n\nExample syntax: stable_id>=10\nChoose a target appropriate to the dataset's range.")
        except (OSError, KeyError) as error:
            messagebox.showerror("Cannot read characteristics", str(error))

    def close(self) -> None:
        if self.running:
            if not messagebox.askyesno("Stop and close?", "Request a safe stop and wait for checkpoint/cleanup before closing?"):
                return
            self.closing = True
            self.stop()
        else:
            self.root.destroy()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset", default="gtr2career")
    parser.add_argument("--goal", default="championships:level=1,count=1")
    parser.add_argument("--python", default=".venv/bin/python")
    parser.add_argument("--binary", default="src-tauri/target/release/rl_env")
    args = parser.parse_args()
    try:
        root = tk.Tk()
    except tk.TclError as error:
        raise SystemExit(f"A graphical display and Python Tk support are required: {error}") from error
    PlaytestGui(root, Settings(dataset=args.dataset, goal=args.goal, python=args.python, binary=args.binary))
    root.mainloop()


if __name__ == "__main__":
    main()
