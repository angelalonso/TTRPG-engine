import os
import shutil
import sys
import tempfile
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[1] / "plugins"))

import race_results


ROOT = Path(__file__).parents[1]


class RaceResultsTests(unittest.TestCase):
    def test_parses_gtr2_slot_telemetry_and_ranks_by_race_time(self):
        text = (ROOT / "raceresults_2.txt").read_text(encoding="utf-8", errors="replace")
        parsed = race_results._parse_gtr2_results(text)

        self.assertEqual(parsed[0], {"name": "AngelAlonso", "position": 1})
        self.assertEqual(parsed[1]["name"], "Mark Fullalove")
        self.assertEqual(len(parsed), 23)

    def test_autodetect_selects_newest_file_and_extracts_player(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            first = directory / "older.txt"
            latest = directory / "latest.txt"
            shutil.copyfile(ROOT / "raceresults.txt", first)
            shutil.copyfile(ROOT / "raceresults_2.txt", latest)
            now = time.time()
            os.utime(first, (now - 10, now - 10))
            os.utime(latest, (now, now))

            response = race_results.autodetect_result({
                "results_directory": str(directory),
                "player_name": "AngelAlonso",
            })

            self.assertEqual(response["player_position"], 1)
            self.assertEqual(response["detected_file"], str(latest))
            self.assertEqual(response["competitors"][0]["name"], "Mark Fullalove")

    def test_game_directory_can_supply_results_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            game_directory = Path(temporary)
            results_directory = game_directory / "UserData" / "Log" / "Results"
            results_directory.mkdir(parents=True)
            shutil.copyfile(ROOT / "raceresults.txt", results_directory / "race.txt")

            response = race_results.autodetect_result({
                "game_directory": str(game_directory),
                "player_name": "AngelAlonso",
            })

            self.assertEqual(response["player_position"], 7)


if __name__ == "__main__":
    unittest.main()
