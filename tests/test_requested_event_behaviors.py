import csv
import unittest
from pathlib import Path


ROOT = Path(__file__).parents[1]


class RequestedEventBehaviorTests(unittest.TestCase):
    def test_social_events_have_no_object_requirements_or_random_results(self):
        with (ROOT / "gtr2career/events.csv").open(newline="", encoding="utf-8") as handle:
            events = list(csv.DictReader(handle))
        social = [row for row in events if "social" in row["tags"].split(";")]
        self.assertTrue(social)
        for event in social:
            self.assertEqual(event["required_license_id"], "")
            expected_objects = "honda_civic" if event["id"] == "honda_civic_enthusiasts_meet" else ""
            self.assertEqual(event["required_object_ids"], expected_objects)

        with (ROOT / "gtr2career/event_results.csv").open(newline="", encoding="utf-8") as handle:
            results = list(csv.DictReader(handle))
        self.assertFalse(any("social" in row["event_tags"].split(";") for row in results))

    def test_race_entry_and_result_ui_contracts_are_wired(self):
        with (ROOT / "gtr2career/events.csv").open(newline="", encoding="utf-8") as handle:
            race_events = [row for row in csv.DictReader(handle) if "race" in row["tags"].split(";")]
        for event in race_events:
            self.assertTrue((ROOT / "gtr2career/html" / f"{event['id']}_gtr2.html").is_file())
        app = (ROOT / "src/App.tsx").read_text(encoding="utf-8")
        modal = (ROOT / "src/components/ResultPromptModal.tsx").read_text(encoding="utf-8")
        sponsor = (ROOT / "src/components/SponsorNegotiationModal.tsx").read_text(encoding="utf-8")
        self.assertIn("GTR2 instructions", app)
        self.assertIn("Continue", app)
        self.assertIn("gtr2InstructionsModal", app)
        self.assertIn("aria-labelledby=\"gtr2-instructions-title\"", app)
        self.assertIn("placeItems: 'center'", app)
        self.assertIn("pendingResult", app)
        self.assertIn("Enter result", app)
        self.assertIn("Track read from results", modal)
        self.assertIn("applies from next year", sponsor)

    def test_sponsor_activation_is_persisted_and_migrated(self):
        backend = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
        save = (ROOT / "src-tauri/src/engine/save.rs").read_text(encoding="utf-8")
        self.assertIn("starts_day", backend)
        self.assertIn("CURRENT_SCHEMA_VERSION: u32 = 5", save)
        self.assertIn("save_schema_version_4_sponsor_contract.json", save)
