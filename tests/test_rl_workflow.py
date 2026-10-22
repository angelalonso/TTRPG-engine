import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "rl"))

import gtr2career_env as adapter
from translate_playtest import episode_lines, observation_changes


class StatusAndReportTests(unittest.TestCase):
    def test_status_normalization_matches_engine_variants(self):
        for raw, expected in (
            ("GoalReached", "goal_reached"), ("DeadMoney", "dead_money"),
            ("MaxSteps", "max_steps"), ("Ongoing", "ongoing"),
            ("goal_reached", "goal_reached"),
        ):
            self.assertEqual(adapter.normalize_status(raw), expected)

    def test_readable_report_explains_rewards_and_errors(self):
        report = "\n".join(episode_lines({
            "diagnostics": {"warnings": ["Check prerequisites"]},
            "path": [{
                "action_key": "wait", "status": "ongoing",
                "reward_components": {"time": -1}, "error": "test error",
            }],
        }, 0))
        self.assertIn("Check prerequisites", report)
        self.assertIn("Time -1.000", report)
        self.assertIn("Engine error: test error", report)
        self.assertTrue(observation_changes(
            {"characteristics": {"new": 1}},
            {"characteristics": {"old": 2}},
        ))


@unittest.skipIf(adapter.np is None or adapter.gym is None, "Optional RL dependencies not installed")
class WorkflowTests(unittest.TestCase):
    def test_reset_replaces_terminal_or_previous_episode_mask(self):
        env = adapter.Gtr2CareerEnv.__new__(adapter.Gtr2CareerEnv)
        env.config = adapter.RustEnvConfig()
        env._characteristic_keys = ["budget"]
        env._last_action_mask = adapter.np.array([0, 0], dtype=adapter.np.int8)
        raw = {
            "characteristics": {"budget": 100}, "current_day": 1,
            "days_per_year": 365, "inventory_ids": [], "quest_ids": [],
            "legal_action_mask": [True, False],
        }
        with patch.object(env, "_request", return_value={"observation": raw}):
            observation, _ = env.reset(seed=5)
        self.assertEqual(env.action_masks().tolist(), [True, False])
        self.assertEqual(observation["action_mask"].tolist(), [1, 0])
        with patch.object(env, "_request", return_value={"observation": raw}) as request:
            env.reset()
            self.assertEqual(request.call_args.args[0], {"op": "reset", "seed": 6})
            env.reset()
            self.assertEqual(request.call_args.args[0], {"op": "reset", "seed": 7})
            env.reset(seed=5)
            self.assertEqual(request.call_args.args[0], {"op": "reset", "seed": 5})

    def test_summary_does_not_confuse_fast_death_with_fast_win(self):
        from playtest import summarize_episodes
        episodes = [
            {"status": status, "current_day": days + 1, "steps": 5,
             "seed": seed, "budget": 100, "stamina": 10}
            for seed, status, days in (
                (1, "dead_money", 1), (2, "goal_reached", 10),
                (3, "goal_reached", 20),
            )
        ]
        summary = summarize_episodes(episodes)
        self.assertEqual(summary["successful_mean_days"], 15)
        self.assertEqual(summary["fastest_success_seed"], 2)
        self.assertAlmostEqual(summary["success_rate"], 2 / 3)
        self.assertIsNone(summarize_episodes(episodes[:1])["successful_mean_days"])

    def test_stagnation_is_a_warning_not_a_fabricated_failure(self):
        from playtest import diagnose_episode
        path = [
            {"step": index, "action_key": "wait", "goal_progress": 0,
             "reward_components": {"time": -1}}
            for index in range(5)
        ]
        result = diagnose_episode(path, "max_steps", 3)
        self.assertEqual(result["longest_no_progress_steps"], 5)
        self.assertEqual(result["reward_totals"]["time"], -5)
        self.assertIn("not proof", result["warnings"][0])

    def test_model_metadata_rejects_same_shape_but_different_action_order(self):
        from workflow import check_model_contract, model_contract, model_path
        class FakeEnv:
            config = adapter.RustEnvConfig()
            action_definitions = ({"key": "wait"}, {"key": "work"})

            def debug_observation(self):
                return {"schema_version": 1, "characteristics": {"budget": 1}}

        env = FakeEnv()
        contract = model_contract(env)
        with tempfile.TemporaryDirectory() as directory:
            model = Path(directory) / "policy"
            Path(str(model_path(model)) + ".json").write_text(json.dumps({"contract": contract}))
            check_model_contract(model, env)
            env.action_definitions = tuple(reversed(env.action_definitions))
            with self.assertRaisesRegex(ValueError, "action_keys"):
                check_model_contract(model, env)

    def test_manual_choices_reject_illegal_ids_and_support_search(self):
        from playtest import manual_action
        class FakeEnv:
            action_definitions = (
                {"id": 0, "key": "wait"}, {"id": 1, "key": "work"},
            )
        with patch("builtins.input", side_effect=["1", "/wait", "0"]), patch("builtins.print"):
            self.assertEqual(manual_action(FakeEnv(), adapter.np.array([True, False])), 0)
        with patch("builtins.input", return_value="q"), patch("builtins.print"):
            self.assertIsNone(manual_action(FakeEnv(), adapter.np.array([True, False])))


@unittest.skipIf(adapter.np is None or adapter.gym is None, "Optional RL dependencies not installed")
class BridgeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.binary = Path(os.environ.get("RL_TEST_BINARY", ROOT / "src-tauri/target/release/rl_env"))
        if not cls.binary.exists():
            if "RL_TEST_BINARY" in os.environ:
                raise RuntimeError(f"Configured bridge binary does not exist: {cls.binary}")
            raise unittest.SkipTest("Run make rl-build for bridge integration tests")

    def test_episode_reset_rewards_and_status_over_real_bridge(self):
        with tempfile.TemporaryDirectory() as directory:
            reward = Path(directory) / "reward.json"
            reward.write_text('{"day": -2, "step": -0.25, "progress_scale": 0, "low_stamina_penalty": 0, "low_budget_penalty": 0}')
            env = adapter.Gtr2CareerEnv(adapter.RustEnvConfig(
                binary=str(self.binary), dataset=str(ROOT / "gtr2career"),
                max_steps=1, reward_config=str(reward),
            ))
            try:
                env.reset(seed=42)
                _, reward_value, terminated, truncated, info = env.step(0)
                self.assertEqual(info["status"], "max_steps")
                self.assertEqual(reward_value, -2.25)
                self.assertEqual(sum(info["reward_components"].values()), reward_value)
                self.assertFalse(terminated)
                self.assertTrue(truncated)
                self.assertFalse(env.action_masks().any())
                env.reset(seed=42)
                self.assertTrue(env.action_masks()[0])
            finally:
                env.close()

    def test_invalid_goal_and_reward_settings_fail_explicitly(self):
        for goal in ("typo>=100", "charisma>=oops"):
            result = subprocess.run([
                str(self.binary), "--dataset", str(ROOT / "gtr2career"), "--goal", goal,
            ], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("RL configuration error", result.stderr)
            self.assertEqual(result.stdout, "")
        with tempfile.TemporaryDirectory() as directory:
            reward = Path(directory) / "bad.json"
            for payload in ('{"stepp": -1}', '{"day": 1}'):
                reward.write_text(payload)
                result = subprocess.run([
                    str(self.binary), "--reward-config", str(reward),
                ], capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("RL configuration error", result.stderr)


if __name__ == "__main__":
    unittest.main()
