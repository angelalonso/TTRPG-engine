"""Display-independent configuration and subprocess execution for the optional GUI."""

from __future__ import annotations

import json
import math
import os
import queue
import re
import signal
import shlex
import subprocess
import threading
from dataclasses import asdict, dataclass, field
from pathlib import Path


PHASES = {
    "prepare": ("Prepare tools", "Run Make's existing setup/build. May download Python packages and compile Rust; only select when needed."),
    "manual": ("Play yourself", "Choose legal actions in the Manual play tab. Check that a route exists; your choices do not train PPO."),
    "baseline": ("Random baseline", "Try untrained random legal choices on the evaluation seeds. This is a reference, not a competent player."),
    "train": ("Train / resume", "Learn from repeated fresh games. Watch wins, failures, successful days and checkpoints; high reward alone is not success."),
    "evaluate": ("Evaluate", "Freeze the learned policy and measure success and winning duration on other seeds. No learning happens here."),
    "inspect": ("Inspect a route", "Run one learned episode and log its actions, state and optional decision probabilities."),
    "report": ("Explain the route", "Translate the latest selected trace (or an existing export) into readable rewards, state changes and warnings."),
}

# Values here label the current CLI defaults; omitted overrides remain engine-owned.
REWARDS = {
    "goal": ("Win bonus", "1000", "Positive reward for reaching the actual goal. Keep this large compared with accumulated shaping penalties."),
    "death": ("Terminal failure penalty", "-1000", "Non-positive penalty for a terminal failure. This cannot make an unreachable goal reachable."),
    "day": ("Cost per elapsed day", "-1", "Prefer faster successful routes. A huge cost can make immediate failure more attractive than a long win."),
    "step": ("Cost per chosen action", "0", "A small negative value discourages same-day loops. Too much punishes legitimate preparation."),
    "invalid_action": ("Invalid action penalty", "-10", "A masked action failing is an engine/data issue to investigate, not something to solve with a bigger penalty."),
    "progress_scale": ("Progress shaping", "5", "Non-negative encouragement for goal progress. Check that repeated actions cannot farm it without winning."),
    "low_stamina_threshold": ("Recovery safety threshold", "10", "The engine's recovery resource, not always stamina. A threshold, not a penalty."),
    "low_stamina_penalty": ("Low recovery penalty", "-10", "Applied per action below the threshold. Necessary activities may temporarily reduce recovery."),
    "low_budget_threshold": ("Currency safety threshold", "950", "The engine's currency resource. A high threshold may discourage essential purchases."),
    "low_budget_penalty": ("Low currency penalty", "-10", "Applied per action below the threshold after the grace period. Try -1 if fear of spending blocks progress."),
    "low_budget_start_day": ("Currency safety starts on day", "30", "Integer grace period before the low-currency penalty applies. This does not postpone actual bills."),
}


def validate_rewards(values: dict[str, float]) -> None:
    for key, value in values.items():
        if key not in REWARDS:
            raise ValueError(f"Unknown reward setting: {key}")
        if not math.isfinite(value):
            raise ValueError(f"{key} must be finite")
        if key == "goal" and value <= 0:
            raise ValueError("Win bonus must be positive")
        if key == "progress_scale" and value < 0:
            raise ValueError("Progress shaping cannot be negative")
        if key in {"death", "day", "step", "invalid_action", "low_stamina_penalty", "low_budget_penalty"} and value > 0:
            raise ValueError(f"{key} must be zero or negative")
        if key == "low_budget_start_day" and (value < 0 or int(value) != value or value > 2**32 - 1):
            raise ValueError("Currency safety start day must be an integer from 0 to 4294967295")


@dataclass
class Settings:
    dataset: str = "gtr2career"
    goal: str = "championships:level=1,count=1"
    python: str = ".venv/bin/python"
    binary: str = "src-tauri/target/release/rl_env"
    model: str = "rl/models/gui-policy"
    resume: str = ""
    trace_input: str = ""
    max_days: int = 7300
    max_steps: int = 14600
    episodes: int = 20
    train_seed: int = 1
    eval_seed: int = 10000
    play_seed: int = 20000
    timesteps: int = 100000
    n_steps: int = 2048
    batch_size: int = 64
    gamma: float = 0.999
    ent_coef: float = 0.01
    checkpoint_interval: int = 10000
    stagnation_window: int = 100
    trace: str = "decisions"
    stochastic: bool = False
    verbose_training: bool = True
    phases: tuple[str, ...] = ("baseline", "train", "evaluate", "inspect", "report")
    rewards: dict[str, float] = field(default_factory=dict)

    def validate(self, root: Path) -> None:
        if not self.phases or len(set(self.phases)) != len(self.phases) or any(key not in PHASES for key in self.phases):
            raise ValueError("Select at least one valid phase without duplicates")
        if tuple(key for key in PHASES if key in self.phases) != self.phases:
            raise ValueError("Phases must run in the displayed order")
        if not self.goal.strip() or not (root / self.dataset).is_dir():
            raise ValueError("Choose an existing dataset folder and a goal")
        for name in ("max_days", "max_steps", "episodes", "timesteps", "n_steps", "batch_size", "checkpoint_interval", "stagnation_window"):
            value = getattr(self, name)
            if type(value) is not int or not 0 < value <= 2**32 - 1:
                raise ValueError(f"{name} must be a positive integer within the engine's range")
        for name in ("train_seed", "eval_seed", "play_seed"):
            value = getattr(self, name)
            if type(value) is not int or not 0 <= value <= 2**64 - self.episodes:
                raise ValueError(f"{name} must be a non-negative seed within the engine's range")
        if self.train_seed == self.eval_seed or self.play_seed in (self.train_seed, self.eval_seed):
            raise ValueError("Use different training, evaluation and inspection seed starts")
        if "train" in self.phases:
            if self.batch_size < 2 or self.n_steps < 2 or self.batch_size > self.n_steps or self.n_steps % self.batch_size:
                raise ValueError("Rollout must be >= 2 and divisible by batch size (also >= 2)")
            if not 0 < self.gamma < 1 or not 0 <= self.ent_coef <= 1:
                raise ValueError("Future reward discount must be between 0 and 1; exploration between 0 and 1 inclusive")
        if self.trace not in ("actions", "observations", "decisions"):
            raise ValueError("Choose a supported trace detail")
        if not self.model.strip():
            raise ValueError("Choose a model output/input path")
        if self.resume and "train" in self.phases and not model_file(root / self.resume).is_file():
            raise ValueError("Resume checkpoint does not exist")
        if any(key in self.phases for key in ("evaluate", "inspect")) and "train" not in self.phases:
            if not model_file(root / self.model).is_file():
                raise ValueError("Select training or choose an existing model")
        if "report" in self.phases and not any(key in self.phases for key in ("manual", "baseline", "evaluate", "inspect")):
            if not self.trace_input or not (root / self.trace_input).is_file():
                raise ValueError("Select a trace-producing phase or choose an existing report input")
        if any(key in self.phases for key in ("manual", "baseline", "train", "evaluate", "inspect")) and "prepare" not in self.phases:
            if not (root / self.python).is_file() or not (root / self.binary).is_file():
                raise ValueError("RL Python or simulator is missing. Select Prepare tools, or run make playtest-prepare")
        validate_rewards(self.rewards)


def model_file(path: Path) -> Path:
    return path if path.suffix == ".zip" else Path(str(path) + ".zip")


@dataclass(frozen=True)
class Phase:
    key: str
    command: tuple[str, ...]
    artifact: str = ""


def build_commands(settings: Settings, output: Path) -> list[Phase]:
    """Build shell-free commands. Validation and output creation are separate."""
    env = [
        "--dataset", settings.dataset, "--goal", settings.goal,
        "--binary", settings.binary, "--max-days", str(settings.max_days),
        "--max-steps", str(settings.max_steps),
    ]
    if settings.rewards:
        env += ["--reward-config", str(output / "rewards.json")]
    python = [settings.python, "-u"]
    common = ["--stagnation-window", str(settings.stagnation_window), "--verbose"]
    stochastic = ["--stochastic"] if settings.stochastic else []
    commands = []
    latest = settings.trace_input
    for key in settings.phases:
        artifact = ""
        if key == "prepare":
            python_path = Path(settings.python)
            if python_path.name not in ("python", "python3") or python_path.parent.name != "bin":
                raise ValueError("Prepare uses a virtualenv/bin/python path; otherwise prepare externally and deselect this phase")
            if not all(re.fullmatch(r"[A-Za-z0-9_./-]+", path) for path in (settings.python, settings.binary)):
                raise ValueError("Make preparation paths must use plain letters, digits, '/', '.', '_' or '-'; prepare paths with spaces or shell/Make syntax externally")
            if settings.binary != "src-tauri/target/release/rl_env":
                raise ValueError("Prepare builds the default release simulator; build custom simulator paths externally")
            command = ["make", "playtest-prepare", f"RL_VENV={python_path.parent.parent}", f"RL_PYTHON={settings.python}", f"RL_BINARY={settings.binary}"]
        elif key == "train":
            command = python + ["rl/train.py"] + env + [
                "--seed", str(settings.train_seed), "--timesteps", str(settings.timesteps),
                "--n-steps", str(settings.n_steps), "--batch-size", str(settings.batch_size),
                "--gamma", str(settings.gamma), "--ent-coef", str(settings.ent_coef),
                "--checkpoint-interval", str(settings.checkpoint_interval),
                "--progress", "--progress-interval", "100", "--episode-log-interval", "1",
                "--output", settings.model,
            ]
            if settings.resume:
                command += ["--resume", settings.resume]
            if settings.verbose_training:
                command += ["--verbose-steps"]
        elif key == "report":
            artifact = str(output / "report.txt")
            command = python + ["rl/translate_playtest.py", latest, "--output", artifact]
        else:
            script = "rl/evaluate.py" if key == "evaluate" else "rl/playtest.py"
            mode = "manual" if key == "manual" else "random" if key == "baseline" else "policy"
            seed = settings.play_seed if key in ("manual", "inspect") else settings.eval_seed
            trace = settings.trace if key == "inspect" else "observations"
            artifact = str(output / f"{key}.json")
            command = python + [script] + env + common + [
                "--mode", mode, "--seed", str(seed),
                "--episodes", str(1 if key in ("manual", "inspect") else settings.episodes),
                "--trace", trace, "--output", artifact,
            ]
            if mode == "policy":
                command += ["--model", settings.model] + stochastic
            if mode == "manual":
                command += ["--interface-json"]
            latest = artifact
        commands.append(Phase(key, tuple(command), artifact))
    return commands


def write_configuration(settings: Settings, output: Path) -> None:
    output.mkdir(parents=True, exist_ok=False)
    (output / "configuration.json").write_text(json.dumps({
        "format_version": 1, "settings": asdict(settings),
    }, indent=2) + "\n", encoding="utf-8")
    if settings.rewards:
        (output / "rewards.json").write_text(json.dumps(settings.rewards, indent=2) + "\n", encoding="utf-8")


class Runner:
    """Worker-only subprocess I/O; the GUI consumes events on its own thread."""

    def __init__(self, root: Path, events: queue.Queue):
        self.root = root
        self.events = events
        self.cancelled = threading.Event()
        self.lock = threading.Lock()
        self.process: subprocess.Popen | None = None
        self.phase = ""

    def send(self, value: str) -> None:
        with self.lock:
            if self.phase != "manual" or self.process is None or self.process.poll() is not None or self.process.stdin is None:
                raise ValueError("No manual game is waiting for input")
            self.process.stdin.write(value + "\n")
            self.process.stdin.flush()

    def cancel(self) -> None:
        self.cancelled.set()
        with self.lock:
            process = self.process
            if process is None or process.poll() is not None:
                return
            try:
                if os.name == "nt":
                    process.send_signal(signal.CTRL_BREAK_EVENT)
                elif self.phase == "prepare":
                    os.killpg(process.pid, signal.SIGINT)
                else:
                    process.send_signal(signal.SIGINT)
            except ProcessLookupError:
                return
        threading.Thread(target=self._force_stop, args=(process,), daemon=True).start()

    def _force_stop(self, process: subprocess.Popen) -> None:
        for delay, sig in ((10, signal.SIGTERM), (2, signal.SIGKILL if os.name != "nt" else signal.SIGTERM)):
            threading.Event().wait(delay)
            with self.lock:
                if self.process is not process:
                    return
                self.events.put(("log", "[stop] Grace period expired; stopping the remaining process group. A final checkpoint is not guaranteed.\n"))
                try:
                    if os.name == "nt":
                        process.kill()
                    else:
                        os.killpg(process.pid, sig)
                except ProcessLookupError:
                    return

    def run(self, phases: list[Phase], output: Path) -> None:
        status = "completed"
        try:
            with (output / "session.log").open("w", encoding="utf-8") as log:
                for phase in phases:
                    if self.cancelled.is_set():
                        break
                    self.events.put(("phase", phase.key))
                    heading = f"\n=== {PHASES[phase.key][0]} ===\n{PHASES[phase.key][1]}\n{shlex.join(phase.command)}\n"
                    log.write(heading)
                    log.flush()
                    self.events.put(("log", heading))
                    kwargs = {"start_new_session": True} if os.name != "nt" else {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP}
                    with self.lock:
                        if self.cancelled.is_set():
                            break
                        process = subprocess.Popen(
                            phase.command, cwd=self.root, stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                            text=True, encoding="utf-8", errors="replace", bufsize=1, **kwargs,
                        )
                        self.process, self.phase = process, phase.key
                    try:
                        for line in process.stdout:
                            log.write(line)
                            log.flush()
                            if line.startswith("[manual-state] "):
                                self.events.put(("manual", json.loads(line[len("[manual-state] "):])))
                            else:
                                self.events.put(("log", line))
                        code = process.wait()
                    except (OSError, ValueError):
                        if process.poll() is None:
                            if os.name == "nt":
                                process.kill()
                            else:
                                os.killpg(process.pid, signal.SIGTERM)
                            try:
                                process.wait(timeout=2)
                            except subprocess.TimeoutExpired:
                                process.kill()
                                process.wait()
                        raise
                    finally:
                        process.stdout.close()
                        process.stdin.close()
                        with self.lock:
                            self.process, self.phase = None, ""
                    if code:
                        if not self.cancelled.is_set():
                            raise ValueError(f"{PHASES[phase.key][0]} exited with code {code}. Remaining phases were not run; read the error above.")
                        break
                    if phase.key == "report":
                        report = Path(phase.artifact).read_text(encoding="utf-8")
                        log.write(report)
                        self.events.put(("log", report))
                    self.events.put(("phase_done", phase.key))
        except (OSError, ValueError) as error:
            status = "failed"
            self.events.put(("error", str(error)))
        finally:
            if self.cancelled.is_set():
                status = "cancelled"
            self.events.put(("finished", status))
