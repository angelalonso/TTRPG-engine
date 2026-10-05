import json
import subprocess
import sys
import unittest
from pathlib import Path


PLUGIN = Path(__file__).parents[1] / "plugins" / "plugin_template.py"


class PluginTemplateTests(unittest.TestCase):
    def run_plugin(self, request):
        return subprocess.run(
            [sys.executable, str(PLUGIN), "--json"],
            input=json.dumps(request),
            text=True,
            capture_output=True,
            check=False,
        )

    def test_input_list_is_structured_json(self):
        result = subprocess.run(
            [sys.executable, str(PLUGIN), "-inputlist"],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0)
        self.assertEqual(
            json.loads(result.stdout),
            {
                "inputs": [
                    {
                        "key": "player_name",
                        "type": "string",
                        "label": "Player Name",
                        "required": True,
                    }
                ]
            },
        )

    def test_hello_world_uses_plugin_protocol(self):
        result = self.run_plugin(
            {
                "protocol_version": 1,
                "plugin_id": "plugin_template",
                "operation": "provide_result",
                "payload": {"player_name": "Ada"},
            }
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["results"][0]["value"], "Hello, Ada!")

    def test_player_name_is_required(self):
        result = self.run_plugin(
            {
                "protocol_version": 1,
                "plugin_id": "plugin_template",
                "operation": "provide_result",
                "payload": {},
            }
        )
        self.assertEqual(result.returncode, 1)
        self.assertEqual(json.loads(result.stdout), {"error": "player_name is required"})


if __name__ == "__main__":
    unittest.main()
