import csv
from pathlib import Path
import unittest


ROOT = Path(__file__).parents[1]
DATASET = ROOT / "gtr2career"
JOB_IDS = {
    "work_9_5",
    "work_9_9",
    "work_part_time",
    "work_event_staff",
    "work_delivery",
    "work_repair_service",
    "work_temporary",
}


class JobDatasetTests(unittest.TestCase):
    def test_job_images_have_one_csv_source_for_html_and_table_thumbnail(self):
        with (DATASET / "events.csv").open(newline="", encoding="utf-8") as handle:
            jobs = {row["id"]: row for row in csv.DictReader(handle) if row["type"] == "work"}
        self.assertTrue(JOB_IDS.issubset(jobs))
        for job_id in JOB_IDS:
            row = jobs[job_id]
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
            jobs = {row["id"]: row for row in csv.DictReader(handle) if row["id"] in JOB_IDS}
        with (DATASET / "obligations.csv").open(newline="", encoding="utf-8") as handle:
            obligations = {row["event_id"]: row for row in csv.DictReader(handle)}
        for job_id in JOB_IDS:
            with self.subTest(job_id=job_id):
                self.assertIn(job_id, obligations)
                self.assertGreaterEqual(float(jobs[job_id]["stamina_cost"]), 0)
                self.assertGreaterEqual(float(obligations[job_id]["amount"]), 0)

    def test_old_job_ids_and_paths_are_not_runtime_references(self):
        for path in (DATASET / "events.csv", DATASET / "obligations.csv", ROOT / "gtr2career_moneymakers.csv"):
            text = path.read_text(encoding="utf-8")
            for old_id in ("act_work", "act_work_b", "act_part_time", "act_event_staffing",
                           "act_delivery_courier", "act_repair_service", "act_work_temp"):
                self.assertNotIn(old_id, text)


if __name__ == "__main__":
    unittest.main()
