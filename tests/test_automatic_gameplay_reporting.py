import csv
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).parents[1]


class AutomaticGameplayReportingTests(unittest.TestCase):
    def test_windows_archive_uses_requested_base_name(self):
        makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
        self.assertIn("WINDOWS_ARCHIVE ?= gtr2career_win.zip", makefile)

    def test_progress_export_contains_progress_defining_actions(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            output = Path(directory) / "progress.csv"
            report.write_text(json.dumps({
                "trace": [
                    {
                        "day": 1,
                        "action": "wait",
                        "budget": 100,
                        "stamina": 100,
                        "status": "Ongoing",
                        "reward_components": {},
                        "error": None,
                    },
                    {
                        "day": 2,
                        "action": "buy_object:scirocco",
                        "budget": 50,
                        "stamina": 100,
                        "status": "Ongoing",
                        "reward_components": {"progress": 1},
                        "error": None,
                    },
                ]
            }), encoding="utf-8")
            subprocess.run(
                [
                    sys.executable,
                    str(ROOT / "scripts/export_automatic_gameplay_csv.py"),
                    str(report),
                    str(output),
                ],
                check=True,
            )
            with output.open(newline="", encoding="utf-8") as handle:
                rows = list(csv.DictReader(handle))
            self.assertEqual(len(rows), 1)
            self.assertEqual(rows[0]["action"], "buy_object:scirocco")


if __name__ == "__main__":
    unittest.main()
