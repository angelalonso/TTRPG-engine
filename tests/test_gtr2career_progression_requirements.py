import csv
import unittest
from pathlib import Path


DATASET = Path(__file__).resolve().parents[1] / "gtr2career"


def rows(filename):
    with (DATASET / filename).open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


class Gtr2CareerProgressionRequirementTests(unittest.TestCase):
    def test_configured_references_and_assets_are_resolvable(self):
        objects = {row["id"]: row for row in rows("objects.csv")}
        quests = {row["id"]: row for row in rows("quests.csv")}

        for row in objects.values():
            with self.subTest(kind="object", id=row["id"]):
                if row["image_path"]:
                    self.assertTrue((DATASET / row["image_path"]).is_file())
                if row["description_html"]:
                    self.assertTrue((DATASET / row["description_html"]).is_file())
                for required_id in filter(None, row["requires_object_ids"].split(";")):
                    self.assertIn(required_id, objects)

        for row in rows("events.csv"):
            with self.subTest(kind="event", id=row["id"]):
                self.assertIn(row["quest_id"], quests | {"": None})
                if row["image_path"]:
                    self.assertTrue((DATASET / row["image_path"]).is_file())
                if row["description_html"]:
                    self.assertTrue((DATASET / row["description_html"]).is_file())
                for required_id in filter(None, row["required_object_ids"].split(";")):
                    self.assertIn(required_id, objects)
                if row["required_license_id"]:
                    self.assertIn(row["required_license_id"], objects)

        for row in rows("sponsors.csv"):
            with self.subTest(kind="sponsor", id=row["id"]):
                if row["required_license_id"]:
                    self.assertIn(row["required_license_id"], objects)

    def test_licences_form_a_staged_chain_without_skipping_levels(self):
        licences = {
            row["id"]: row for row in rows("objects.csv") if row["type"] == "license"
        }
        for row in licences.values():
            with self.subTest(id=row["id"]):
                previous = row["license_previous_id"]
                if previous:
                    self.assertIn(previous, licences)
                    self.assertLess(
                        int(licences[previous]["license_level"]),
                        int(row["license_level"]),
                    )

    def test_each_championship_has_a_calendar_and_explicit_entry_gate(self):
        events = rows("events.csv")
        for quest in rows("quests.csv"):
            quest_events = [event for event in events if event["quest_id"] == quest["id"]]
            with self.subTest(id=quest["id"]):
                self.assertTrue(quest_events)
                self.assertTrue(all(int(event["day_of_year"]) > 0 for event in quest_events))
                if quest["level"] != "1" or quest["id"] != "amateur_championship":
                    self.assertTrue(quest["required_license_id"])

    def test_sponsor_activities_reference_real_campaign_content(self):
        objects = {row["id"] for row in rows("objects.csv")}
        quests = {row["id"] for row in rows("quests.csv")}
        sponsor_events = [row for row in rows("events.csv") if row["type"] == "sponsor"]
        self.assertTrue(sponsor_events)
        for row in sponsor_events:
            with self.subTest(id=row["id"]):
                self.assertIn(row["sponsor_quest_id"], quests)
                self.assertIn(row["sponsor_object_id"], objects)
                for object_id in filter(None, row["sponsor_equipment_ids"].split(";")):
                    self.assertIn(object_id, objects)

    def test_professional_sponsor_activities_require_manager_progression(self):
        events = {row["id"]: row for row in rows("events.csv")}
        self.assertEqual(events["act_sponsor_fia_ngt"]["required_license_id"], "license_international_c")
        self.assertEqual(events["act_sponsor_fia_ngt"]["required_object_ids"], "manager_level_4")
        self.assertEqual(events["act_sponsor_fia_gt"]["required_license_id"], "license_international_c")
        self.assertEqual(events["act_sponsor_fia_gt"]["required_object_ids"], "manager_level_5")

    def test_event_csv_rows_have_no_unmapped_columns(self):
        with (DATASET / "events.csv").open(encoding="utf-8", newline="") as handle:
            reader = csv.reader(handle)
            width = len(next(reader))
            for line_number, row in enumerate(reader, start=2):
                self.assertEqual(len(row), width, f"events.csv line {line_number}")


if __name__ == "__main__":
    unittest.main()
