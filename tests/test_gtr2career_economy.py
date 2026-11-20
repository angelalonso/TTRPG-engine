import csv
import unittest
from pathlib import Path


ROOT = Path(__file__).parents[1]


class Gtr2CareerEconomyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with (ROOT / "gtr2career" / "events.csv").open(newline="", encoding="utf-8") as handle:
            cls.events = {row["id"]: row for row in csv.DictReader(handle)}

    def test_market_flipping_no_longer_pays_easy_mode_profit(self):
        action = self.events["act_market_flipping"]
        self.assertEqual(action["entry_fee"], "5000")
        self.assertEqual(action["payout"], "6000")
        self.assertLessEqual(
            float(action["payout"]) - float(action["entry_fee"]),
            1000,
        )

    def test_fia_sponsor_payouts_remain_above_market_shortcut(self):
        self.assertGreater(float(self.events["act_sponsor_fia_ngt"]["payout"]), 6000)
        self.assertGreater(float(self.events["act_sponsor_fia_gt"]["payout"]), 6000)


if __name__ == "__main__":
    unittest.main()
