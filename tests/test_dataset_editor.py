import unittest
from pathlib import Path


ROOT = Path(__file__).parents[1]


class DatasetEditorTests(unittest.TestCase):
    def test_dataset_editor_exposes_native_car_authoring_controls(self):
        source = (ROOT / "src/components/DatasetEditor.tsx").read_text(encoding="utf-8")

        self.assertIn('data-testid="dataset-editor"', source)
        self.assertIn("Add car", source)
        self.assertIn("Edit car", source)
        self.assertIn("saveDatasetCar", source)
        self.assertIn("getDatasetEditorCatalog", source)
        self.assertIn("imagePair", source)
        self.assertIn("Calculated maintenance costs", source)
        self.assertIn("maintenance_costs", source)
        self.assertIn("dataset-editor-car-section", source)
        self.assertIn("dataset-editor-future-section", source)
        self.assertIn("Add race", source)
        self.assertIn("Edit race", source)
        self.assertIn("Add championship", source)
        self.assertIn("Edit championship", source)
        self.assertIn("Add position reward", source)
        self.assertIn("Add reward", source)
        self.assertIn("Organiser picture", source)

    def test_dataset_editor_uses_black_authoring_surface_without_racing_chrome(self):
        stylesheet = (ROOT / "public/plugin.css").read_text(encoding="utf-8")

        self.assertIn("body:has(.dataset-editor-screen)::before", stylesheet)
        self.assertIn("body:has(.dataset-editor-screen)::after", stylesheet)
        self.assertIn("body:has(.dataset-editor-screen)", stylesheet)
        self.assertIn(".dataset-editor-screen", stylesheet)
        self.assertIn("background: #000 !important", stylesheet)

    def test_native_authoring_commands_are_registered(self):
        source = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")

        self.assertIn("fn get_dataset_editor_catalog", source)
        self.assertIn("fn save_dataset_car", source)
        self.assertIn("fn save_dataset_race", source)
        self.assertIn("fn save_dataset_championship", source)
        self.assertIn("get_dataset_editor_catalog,", source)
        self.assertIn("save_dataset_car,", source)
        self.assertIn("save_dataset_race,", source)
        self.assertIn("save_dataset_championship,", source)

    def test_race_and_championship_reward_and_organizer_fields_are_wired(self):
        frontend = (ROOT / "src/services/tauriApi.ts").read_text(encoding="utf-8")
        backend = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")

        for source in (frontend, backend):
            self.assertIn("position_rewards", source)
            self.assertIn("organizer_image", source)
        self.assertIn("rewards_html", backend)

    def test_entity_html_templates_are_present_and_used(self):
        source = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")

        for template in ("car.html", "race.html", "championship.html"):
            self.assertTrue((ROOT / "gtr2career/html/templates" / template).is_file())
            self.assertIn(template, source)
        self.assertIn("load_editor_html_template", source)
        self.assertIn("render_editor_html_template", source)

    def test_editor_uses_csv_backed_requirement_and_composition_controls(self):
        frontend = (ROOT / "src/components/DatasetEditor.tsx").read_text(encoding="utf-8")
        backend = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
        self.assertIn("requirement_options", frontend)
        self.assertIn("multiple", frontend)
        self.assertIn("event_options", frontend)
        self.assertIn("Add event", frontend)
        self.assertIn("rounds", frontend)
        self.assertIn("requirement_options", backend)
        self.assertIn("event_options", backend)
        self.assertIn("request.rounds", backend)

    def test_save_import_export_controls_are_present_and_validated(self):
        modal = (ROOT / "src/components/SaveSlotsModal.tsx").read_text(encoding="utf-8")
        api = (ROOT / "src/services/tauriApi.ts").read_text(encoding="utf-8")
        backend = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
        self.assertIn("Export selected save", modal)
        self.assertIn("Import save file", modal)
        self.assertIn("exportSaveGame", api)
        self.assertIn("importSaveGame", api)
        self.assertIn("different dataset family", backend)
        self.assertIn("newer unsupported dataset revision", backend)
        self.assertIn("Load", modal)
        self.assertIn("selectedSlotButton", modal)

    def test_championship_event_edit_returns_to_composition_editor(self):
        source = (ROOT / "src/components/DatasetEditor.tsx").read_text(encoding="utf-8")

        self.assertIn("returnToChampionship", source)
        self.assertIn("openEditRace(round.event_id, true)", source)
        self.assertIn("setRacingDialog('championship')", source)

    def test_reward_pool_remains_supported(self):
        frontend = (ROOT / "src/components/DatasetEditor.tsx").read_text(encoding="utf-8")
        backend = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
        events_header = (ROOT / "gtr2career/events.csv").read_text(encoding="utf-8").splitlines()[0]
        self.assertIn("reward_pool", frontend)
        self.assertIn("reward_pool", backend)
        self.assertIn("reward_pool", events_header)
