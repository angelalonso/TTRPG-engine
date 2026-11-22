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

    def test_crypto_trade_has_negative_expected_return(self):
        action = self.events["act_trade"]
        expected_net = (
            float(action["success_rate"]) * float(action["payout"])
            - float(action["entry_fee"])
        )
        self.assertLess(expected_net, 0)

    def test_higher_tier_sponsors_require_manager_levels(self):
        self.assertEqual(self.events["act_sponsor_fia_ngt"]["required_object_ids"], "manager_level_4")
        self.assertEqual(self.events["act_sponsor_fia_gt"]["required_object_ids"], "manager_level_5")

    def test_loans_have_bounded_repayment_obligations(self):
        with (ROOT / "gtr2career" / "obligations.csv").open(newline="", encoding="utf-8") as handle:
            obligations = {row["event_id"]: row for row in csv.DictReader(handle)}
        for event_id in ("act_bank_loan", "act_personal_loan"):
            self.assertGreater(float(obligations[event_id]["amount"]), 0)
            self.assertGreater(int(obligations[event_id]["interval"]), 0)
            self.assertGreater(int(obligations[event_id]["max_payments"]), 0)


if __name__ == "__main__":
    unittest.main()
