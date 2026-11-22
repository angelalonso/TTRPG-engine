import csv
from pathlib import Path
import unittest


ROOT = Path(__file__).parents[1]
DATASET = ROOT / "gtr2career"


class JobDatasetTests(unittest.TestCase):
    def test_all_live_obligation_event_ids_exist_in_catalog(self):
        with (DATASET / "events.csv").open(newline="", encoding="utf-8") as handle:
            event_ids = {row["id"] for row in csv.DictReader(handle)}
        with (DATASET / "obligations.csv").open(newline="", encoding="utf-8") as handle:
            obligation_event_ids = {row["event_id"] for row in csv.DictReader(handle)}
        self.assertTrue(obligation_event_ids.issubset(event_ids))

    def test_job_images_have_one_csv_source_for_html_and_table_thumbnail(self):
        with (DATASET / "events.csv").open(newline="", encoding="utf-8") as handle:
            jobs = {row["id"]: row for row in csv.DictReader(handle) if row["type"] == "work"}
        self.assertTrue(jobs)
        for job_id, row in jobs.items():
            source = DATASET / row["image_path"]
            logo = DATASET / row["logo_image_path"]
            thumbnail = source.with_name(f"thumb_{source.stem}.jpeg")
            html = (DATASET / row["description_html"]).read_text(encoding="utf-8")
            with self.subTest(job_id=job_id):
                self.assertTrue(source.is_file())
                self.assertTrue(logo.is_file())
                self.assertTrue(thumbnail.is_file())
                self.assertIn(f"../img/{source.name}", html)
                self.assertIn(f"../img/{logo.name}", html)

    def test_job_attempt_and_workday_stamina_are_distinct_dataset_values(self):
        with (DATASET / "events.csv").open(newline="", encoding="utf-8") as handle:
            jobs = {row["id"]: row for row in csv.DictReader(handle) if row["type"] == "work"}
        with (DATASET / "obligations.csv").open(newline="", encoding="utf-8") as handle:
            obligations = {row["event_id"]: row for row in csv.DictReader(handle)}
        work_obligations = {
            event_id: obligation for event_id, obligation in obligations.items() if event_id in jobs
        }
        for event_id, obligation in work_obligations.items():
            with self.subTest(event_id=event_id):
                self.assertIn(event_id, jobs)
                self.assertGreaterEqual(float(jobs[event_id]["stamina_cost"]), 0)
                self.assertGreaterEqual(float(obligation["amount"]), 0)

        self.assertTrue(set(work_obligations).issubset(jobs))

    def test_old_job_ids_and_paths_are_not_runtime_references(self):
        for path in (DATASET / "events.csv", DATASET / "obligations.csv", ROOT / "gtr2career_moneymakers.csv"):
            text = path.read_text(encoding="utf-8")
            for old_id in ("act_work", "act_work_b", "act_part_time", "act_event_staffing",
                           "act_delivery_courier", "act_repair_service", "act_work_temp"):
                self.assertNotIn(old_id, text)

    def test_trade_ventures_have_editable_html_and_image_sources(self):
        with (DATASET / "events.csv").open(newline="", encoding="utf-8") as handle:
            ventures = [row for row in csv.DictReader(handle) if row["type"] == "trade"]
        self.assertTrue(ventures)
        for venture in ventures:
            with self.subTest(venture_id=venture["id"]):
                image = DATASET / venture["image_path"]
                html = DATASET / venture["description_html"]
                self.assertTrue(image.is_file())
                self.assertTrue(html.is_file())
                self.assertIn(f"../img/{image.name}", html.read_text(encoding="utf-8"))

    def test_finance_loans_are_venture_editor_entries(self):
        with (DATASET / "events.csv").open(newline="", encoding="utf-8") as handle:
            finance_ids = {
                row["id"] for row in csv.DictReader(handle) if row["type"] == "finance"
            }
        self.assertIn("act_bank_loan", finance_ids)
        self.assertIn("act_personal_loan", finance_ids)


if __name__ == "__main__":
    unittest.main()
