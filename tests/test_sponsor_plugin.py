import json
import subprocess
import sys
import unittest
from pathlib import Path


PLUGIN = Path(__file__).parents[1] / "gtr2career" / "plugins" / "sponsor_negotiator.py"


class SponsorPluginTests(unittest.TestCase):
    def run_plugin(self, actions, *extra):
        request = "\n".join(json.dumps({"action_id": action}) for action in actions) + "\n"
        return subprocess.run(
            [
                sys.executable,
                str(PLUGIN),
                "--json",
                "--seed",
                "7",
                "--results",
                "80",
                *extra,
            ],
            input=request,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_protocol_emits_initial_and_terminal_json(self):
        result = self.run_plugin(["logic_rebuttal"] * 8)
        self.assertEqual(result.returncode, 0, result.stderr)
        messages = [json.loads(line) for line in result.stdout.splitlines()]
        self.assertEqual(messages[0]["status"], "ONGOING")
        self.assertIn(messages[-1]["status"], {"SIGNED", "BANNED", "REJECTED"})
        self.assertIn("starting sponsor negotiation", result.stderr)

    def test_invalid_action_is_reported_without_corrupting_state(self):
        result = self.run_plugin(["not_an_option", "charm"])
        messages = [json.loads(line) for line in result.stdout.splitlines()]
        self.assertEqual(messages[1]["status"], "ERROR")
        self.assertEqual(messages[2]["status"], "ONGOING")

    def test_help_is_available(self):
        result = subprocess.run(
            [sys.executable, str(PLUGIN), "--help"],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0)
        self.assertIn("--agent-level", result.stdout)

    def test_parameter_effect_help_is_available(self):
        result = subprocess.run(
            [sys.executable, str(PLUGIN), "--help-effects"],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0)
        self.assertIn("Having an agent adds 8 attraction", result.stdout)
        self.assertIn("agent_level * 8", result.stdout)

    def test_local_sponsors_are_tier_filtered_and_cold_calls_are_conservative(self):
        result = self.run_plugin(
            [],
            "--dataset-path",
            str(PLUGIN.parents[1]),
            "--race-tier",
            "local",
            "--scope",
            "race",
        )
        state = json.loads(result.stdout.splitlines()[0])
        self.assertTrue(state["cold_call"])
        self.assertIsNone(state["proposal_expires"])
        self.assertEqual({sponsor["tier"] for sponsor in state["sponsors"]}, {"local"})
        self.assertLess(
            state["proposal"]["initial_money"],
            state["ideal_proposal"]["initial_money"],
        )

    def test_manager_proposals_expire_and_include_manager_level(self):
        result = self.run_plugin(
            [],
            "--dataset-path",
            str(PLUGIN.parents[1]),
            "--has-agent",
            "--agent-level",
            "4",
            "--approach",
            "proposal",
            "--race-tier",
            "national",
        )
        state = json.loads(result.stdout.splitlines()[0])
        self.assertFalse(state["cold_call"])
        self.assertEqual(state["manager_level"], 4)
        self.assertIsNotNone(state["proposal_expires"])


if __name__ == "__main__":
    unittest.main()
