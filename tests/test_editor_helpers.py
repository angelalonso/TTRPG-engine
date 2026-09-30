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


if __name__ == "__main__":
    unittest.main()
