import csv
import tempfile
import unittest
from pathlib import Path

import dataset_editor


class EditorHelperTests(unittest.TestCase):
    def test_csv_round_trip_preserves_headers_and_values(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "objects.csv"
            headers = ["id", "name", "extension_field"]
            rows = [{"id": "dish", "name": "Soup", "extension_field": "kept"}]

            dataset_editor.write_csv(path, headers, rows)

            with path.open(newline="", encoding="utf-8") as handle:
                reader = csv.DictReader(handle)
                self.assertEqual(reader.fieldnames, headers)
                self.assertEqual(next(reader), rows[0])

    def test_missing_colors_use_explicit_default_rows(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "colors.csv"

            rows = dataset_editor.load_color_rows(path, warn=False)

            self.assertEqual(rows, dataset_editor.default_color_rows())
            self.assertFalse(path.exists())

    def test_invalid_colors_are_not_repaired_implicitly(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "colors.csv"
            path.write_text("element_id,hex_color\nbroken,not-a-color\n", encoding="utf-8")
            original = path.read_bytes()

            rows = dataset_editor.load_color_rows(path, warn=False)

            self.assertEqual(rows, dataset_editor.default_color_rows())
            self.assertEqual(path.read_bytes(), original)

    def test_missing_references_can_be_cleaned(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "objects.csv").write_text(
                "id,name,requires_object_ids\ncar,Car,missing;car\n", encoding="utf-8"
            )
            (root / "events.csv").write_text(
                "id,name,required_object_ids\nrace,Race,missing;car\n", encoding="utf-8"
            )
            (root / "quests.csv").write_text("id,name\nquest,Quest\n", encoding="utf-8")

            missing = dataset_editor.find_missing_references(root)

            self.assertEqual(len(missing), 2)
            self.assertEqual(dataset_editor.cleanup_missing_references(root), 2)
            self.assertIn("car", (root / "objects.csv").read_text(encoding="utf-8"))
            self.assertNotIn("missing", (root / "objects.csv").read_text(encoding="utf-8"))
            self.assertNotIn("missing", (root / "events.csv").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
