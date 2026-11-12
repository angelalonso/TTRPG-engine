import unittest
from pathlib import Path


class DatasetEditorBlankScreenTests(unittest.TestCase):
    def test_dataset_editor_is_only_the_blank_screen(self):
        source = (
            Path(__file__).parents[1] / "src/components/DatasetEditor.tsx"
        ).read_text(encoding="utf-8")

        self.assertIn('data-testid="dataset-editor-blank"', source)
        self.assertNotIn("listDatasetTables", source)
        self.assertNotIn("saveDatasetTable", source)
        self.assertNotIn("DatasetTable", source)
