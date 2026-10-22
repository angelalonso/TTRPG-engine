import importlib.util
import json
import os
from pathlib import Path
import queue
import sys
import tempfile
import threading
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "rl"))

from gui_workflow import Phase, Runner, Settings, build_commands, validate_rewards, write_configuration


class GuiConfigurationTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "game").mkdir()
        for name in ("python", "simulator"):
            (self.root / name).touch()
        self.settings = Settings(dataset="game", python="python", binary="simulator")

    def test_default_workflow_uses_shared_cli_and_identical_environment(self):
        self.settings.rewards = {"step": -0.05}
        self.settings.validate(self.root)
        phases = build_commands(self.settings, self.root / "output")
        self.assertEqual([phase.key for phase in phases], list(self.settings.phases))
        for phase in phases[:-1]:
            command = phase.command
            self.assertEqual(command[command.index("--goal") + 1], self.settings.goal)
            self.assertEqual(command[command.index("--max-days") + 1], "7300")
            self.assertEqual(command[command.index("--reward-config") + 1], str(self.root / "output/rewards.json"))
        train = next(phase.command for phase in phases if phase.key == "train")
        self.assertIn("--verbose-steps", train)
        self.assertIn("--progress", train)
        self.assertEqual(phases[0].command[phases[0].command.index("--seed") + 1], "10000")
        self.assertEqual(phases[2].command[phases[2].command.index("--seed") + 1], "10000")
        self.assertEqual(phases[-1].command[-3], phases[-2].artifact)

    def test_settings_errors_are_explicit_before_spawning(self):
        for key, value in (
            ("phases", ()), ("episodes", 0), ("max_days", -1),
            ("gamma", float("nan")), ("gamma", 1.0), ("ent_coef", float("inf")),
            ("n_steps", 7), ("batch_size", 1), ("train_seed", -1),
            ("eval_seed", 1), ("play_seed", 10000),
            ("resume", "missing.zip"), ("phases", ("evaluate", "train")),
        ):
            with self.subTest(key=key, value=value):
                settings = Settings(**{**vars(self.settings), key: value})
                with self.assertRaises(ValueError):
                    settings.validate(self.root)
        self.settings.phases = ("evaluate",)
        with self.assertRaisesRegex(ValueError, "existing model"):
            self.settings.validate(self.root)
        self.settings.phases = ("report",)
        with self.assertRaisesRegex(ValueError, "report input"):
            self.settings.validate(self.root)

    def test_rewards_reject_unknown_positive_penalties_and_fractional_days(self):
        for values in (
            {"typo": -1}, {"goal": 0}, {"day": 1}, {"step": float("nan")},
            {"progress_scale": -1}, {"low_budget_start_day": 0.5},
            {"low_budget_start_day": 2**32},
        ):
            with self.subTest(values=values), self.assertRaises(ValueError):
                validate_rewards(values)
        validate_rewards({"goal": 1000, "step": -0.05, "low_budget_start_day": 0})

    def test_paths_and_goals_are_individual_arguments_not_shell_commands(self):
        self.settings.goal = "score>=10; echo NOT_A_COMMAND"
        self.settings.model = "models/one with spaces"
        phases = build_commands(self.settings, Path("outputs/with spaces"))
        train = next(phase.command for phase in phases if phase.key == "train")
        self.assertIn(self.settings.goal, train)
        self.assertIn(self.settings.model, train)
        self.settings.phases = ("prepare",)
        self.settings.python = ".venv/bin/python"
        self.settings.binary = "src-tauri/target/release/rl_env"
        self.assertEqual(build_commands(self.settings, Path("out"))[0].command[0], "make")
        self.settings.python = "$(touch unsafe)/bin/python"
        with self.assertRaisesRegex(ValueError, "plain letters"):
            build_commands(self.settings, Path("out"))

    def test_snapshot_and_rewards_are_versioned_diagnostics_not_game_saves(self):
        self.settings.rewards = {"low_budget_start_day": 30, "step": -0.1}
        output = self.root / "experiment"
        write_configuration(self.settings, output)
        snapshot = json.loads((output / "configuration.json").read_text())
        self.assertEqual(snapshot["format_version"], 1)
        self.assertEqual(snapshot["settings"]["dataset"], "game")
        self.assertEqual(json.loads((output / "rewards.json").read_text()), self.settings.rewards)
        with self.assertRaises(FileExistsError):
            write_configuration(self.settings, output)

    def test_manual_uses_existing_runner_and_exports_one_episode(self):
        self.settings.phases = ("manual", "report")
        phases = build_commands(self.settings, Path("output"))
        self.assertIn("--interface-json", phases[0].command)
        self.assertEqual(phases[0].command[phases[0].command.index("--episodes") + 1], "1")
        self.assertIn(phases[0].artifact, phases[1].command)


class GuiRunnerTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.output = Path(self.directory.name)
        self.events = queue.Queue()
        self.runner = Runner(ROOT, self.events)

    def events_list(self):
        result = []
        while not self.events.empty():
            result.append(self.events.get_nowait())
        return result

    def test_failed_phase_blocks_later_phases_and_keeps_stderr(self):
        self.runner.run([
            Phase("baseline", (sys.executable, "-u", "-c", "import sys; print('specific failure', file=sys.stderr); sys.exit(3)")),
            Phase("train", (sys.executable, "-c", "raise AssertionError('must not run')")),
        ], self.output)
        events = self.events_list()
        self.assertEqual([value for kind, value in events if kind == "phase"], ["baseline"])
        self.assertIn("specific failure", (self.output / "session.log").read_text())
        self.assertIn(("finished", "failed"), events)
        self.assertTrue(any("code 3" in value for kind, value in events if kind == "error"))

    def test_missing_executable_is_not_success_shaped(self):
        self.runner.run([Phase("baseline", (str(self.output / "missing"),))], self.output)
        events = self.events_list()
        self.assertIn(("finished", "failed"), events)
        self.assertTrue(any(kind == "error" for kind, _ in events))

    def test_manual_state_is_structured_and_stdin_receives_the_choice(self):
        code = "import json; print('[manual-state] '+json.dumps({'actions': [], 'observation': {}}), flush=True); print('choice='+input(), flush=True)"
        worker = threading.Thread(target=self.runner.run, args=([Phase("manual", (sys.executable, "-u", "-c", code))], self.output))
        worker.start()
        self.addCleanup(lambda: self.runner.cancel() if worker.is_alive() else None)
        while True:
            kind, value = self.events.get(timeout=10)
            if kind == "manual":
                self.assertEqual(value["actions"], [])
                break
            self.assertNotEqual(kind, "finished")
        self.runner.send("7")
        worker.join(timeout=10)
        self.assertFalse(worker.is_alive())
        events = self.events_list()
        self.assertIn(("finished", "completed"), events)
        self.assertIn("choice=7", (self.output / "session.log").read_text())
        with self.assertRaises(ValueError):
            self.runner.send("8")

    @unittest.skipIf(os.name == "nt", "POSIX signal behavior")
    def test_stop_allows_checkpoint_cleanup_and_prevents_the_next_phase(self):
        code = (
            "import signal,sys,time\n"
            "def stop(*_):\n"
            " print('checkpoint saved', flush=True)\n"
            " sys.exit(130)\n"
            "signal.signal(signal.SIGINT, stop)\n"
            "print('READY', flush=True)\n"
            "time.sleep(60)\n"
        )
        worker = threading.Thread(target=self.runner.run, args=([
            Phase("train", (sys.executable, "-u", "-c", code)),
            Phase("evaluate", (sys.executable, "-c", "raise AssertionError('must not run')")),
        ], self.output))
        worker.start()
        self.addCleanup(lambda: self.runner.cancel() if worker.is_alive() else None)
        while True:
            kind, value = self.events.get(timeout=10)
            if kind == "log" and "READY" in value:
                break
            self.assertNotEqual(kind, "finished")
        self.runner.cancel()
        worker.join(timeout=10)
        self.assertFalse(worker.is_alive())
        events = self.events_list()
        self.assertIn(("finished", "cancelled"), events)
        self.assertNotIn(("phase", "evaluate"), events)
        self.assertIn("checkpoint saved", (self.output / "session.log").read_text())


class GuiIntegrationTests(unittest.TestCase):
    @unittest.skipUnless(importlib.util.find_spec("numpy") and importlib.util.find_spec("gymnasium"), "Optional RL dependencies unavailable")
    def test_manual_cli_exports_gui_state_and_rewards(self):
        import subprocess
        binary = os.environ.get("RL_TEST_BINARY", str(ROOT / "src-tauri/target/release/rl_env"))
        if not Path(binary).is_file():
            self.skipTest("Release simulator not built")
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "manual.json"
            result = subprocess.run([
                sys.executable, "-u", "rl/playtest.py", "--mode", "manual", "--interface-json",
                "--binary", binary, "--goal", "charisma>=100", "--max-steps", "2",
                "--episodes", "1", "--trace", "observations", "--output", str(output),
            ], cwd=ROOT, input="0\nq\n", text=True, capture_output=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            states = [json.loads(line.removeprefix("[manual-state] ")) for line in result.stdout.splitlines() if line.startswith("[manual-state] ")]
            self.assertTrue(states)
            self.assertNotIn("Showing 30", result.stdout)
            self.assertNotIn("Action ID, /search", result.stdout)
            self.assertTrue(all(action["id"] >= 0 for action in states[0]["actions"]))
            self.assertIn("characteristics", states[0]["observation"])
            self.assertEqual(json.loads(output.read_text())["episodes"][0]["status"], "user_stopped")

    @unittest.skipUnless(importlib.util.find_spec("sb3_contrib"), "Optional PPO dependencies unavailable")
    def test_real_small_gui_pipeline_trains_evaluates_and_explains(self):
        binary = os.environ.get("RL_TEST_BINARY", str(ROOT / "src-tauri/target/release/rl_env"))
        if not Path(binary).is_file():
            self.skipTest("Release simulator not built")
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "run"
            settings = Settings(
                python=sys.executable, binary=binary, model=str(Path(directory) / "model"),
                goal="charisma>=10", timesteps=4, n_steps=4, batch_size=2,
                checkpoint_interval=2, episodes=1, max_days=3, max_steps=3,
                rewards={"step": -0.05},
            )
            settings.validate(ROOT)
            write_configuration(settings, output)
            events = queue.Queue()
            Runner(ROOT, events).run(build_commands(settings, output), output)
            results = []
            while not events.empty():
                results.append(events.get_nowait())
            self.assertIn(("finished", "completed"), results, [value for kind, value in results if kind == "error"])
            self.assertTrue(Path(settings.model + ".zip.json").is_file())
            self.assertTrue((output / "report.txt").is_file())
            self.assertIn("[practice]", (output / "session.log").read_text())
            self.assertEqual(json.loads((output / "evaluate.json").read_text())["configuration"]["seed"], 10000)

    def test_tk_window_constructs_when_a_display_is_available(self):
        try:
            import tkinter as tk
        except ImportError:
            self.skipTest("Python Tk support unavailable")
        try:
            root = tk.Tk()
        except tk.TclError as error:
            self.skipTest(f"No graphical display: {error}")
        self.addCleanup(root.destroy)
        root.withdraw()
        from gui import PlaytestGui
        gui = PlaytestGui(root, Settings())
        root.update_idletasks()
        self.assertEqual(len(gui.tabs.tabs()), 5)
        gui.select_phases(("manual",))
        self.assertTrue(gui.phases["manual"].get())
        self.assertFalse(gui.phases["train"].get())
        gui.load_manual_catalog(str(ROOT / "tests/fixtures/cooking"))
        gui.actions = [{"id": 1, "key": "buy_object:skillet"}]
        gui.refresh_actions()
        gui.action_list.selection_set("1")
        gui.explain_action()
        self.assertIn("Acquire this object", gui.action_help.get())
        self.assertIn("price:", gui.action_help.get())


if __name__ == "__main__":
    unittest.main()
