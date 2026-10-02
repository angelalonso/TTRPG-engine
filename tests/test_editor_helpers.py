import csv
import shutil
import subprocess
import unittest
from unittest import mock
from pathlib import Path

import dataset_editor


class EditorHelperTests(unittest.TestCase):
    def setUp(self):
        self.test_root = Path(".editor-test-fixtures")
        if self.test_root.exists():
            shutil.rmtree(self.test_root)
        self.test_root.mkdir()

    def tearDown(self):
        shutil.rmtree(self.test_root, ignore_errors=True)

    def test_csv_round_trip_preserves_headers_and_values(self):
        path = self.test_root / "objects.csv"
        headers = ["id", "name", "extension_field"]
        rows = [{"id": "dish", "name": "Soup", "extension_field": "kept"}]

        dataset_editor.write_csv(path, headers, rows)

        with path.open(newline="", encoding="utf-8") as handle:
            reader = csv.DictReader(handle)
            self.assertEqual(reader.fieldnames, headers)
            self.assertEqual(next(reader), rows[0])

    def test_missing_colors_use_explicit_default_rows(self):
        path = self.test_root / "colors.csv"

        rows = dataset_editor.load_color_rows(path, warn=False)

        self.assertEqual(rows, dataset_editor.default_color_rows())
        self.assertFalse(path.exists())

    def test_invalid_colors_are_not_repaired_implicitly(self):
        path = self.test_root / "colors.csv"
        path.write_text("element_id,hex_color\nbroken,not-a-color\n", encoding="utf-8")
        original = path.read_bytes()

        rows = dataset_editor.load_color_rows(path, warn=False)

        self.assertEqual(rows, dataset_editor.default_color_rows())
        self.assertEqual(path.read_bytes(), original)

    def test_missing_references_can_be_cleaned(self):
        root = self.test_root
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

    def test_dataset_document_preserves_unknown_tables_and_columns(self):
        root = self.test_root
        (root / "objects.csv").write_text(
            "id,name,engine_extension\nsoup,Soup,keep-me\n", encoding="utf-8"
        )
        (root / "future_table.csv").write_text(
            "key,unknown_value\nalpha,untouched\n", encoding="utf-8"
        )

        document = dataset_editor.DatasetDocument.load(root)
        document.replace_rows("objects.csv", ["id", "name"], [{"id": "soup", "name": "Stew"}])
        document.export(root)

        with (root / "objects.csv").open(newline="", encoding="utf-8") as handle:
            objects = csv.DictReader(handle)
            self.assertEqual(objects.fieldnames, ["id", "name", "engine_extension"])
            self.assertEqual(next(objects)["engine_extension"], "keep-me")
        with (root / "future_table.csv").open(newline="", encoding="utf-8") as handle:
            future = csv.DictReader(handle)
            self.assertEqual(future.fieldnames, ["key", "unknown_value"])
            self.assertEqual(next(future)["unknown_value"], "untouched")

    def test_dataset_document_uses_declared_compound_identity(self):
        metadata = {
            "tables": [
                {"file": "results.csv", "identity_fields": ["event_id", "result_id"]}
            ]
        }
        document = dataset_editor.DatasetDocument(
            {
                "results.csv": {
                    "headers": ["event_id", "result_id", "label", "extension"],
                    "rows": [
                        {
                            "event_id": "event",
                            "result_id": "win",
                            "label": "Old",
                            "extension": "keep",
                        }
                    ],
                }
            },
            metadata,
        )

        document.replace_rows(
            "results.csv",
            ["event_id", "result_id", "label"],
            [{"event_id": "event", "result_id": "win", "label": "New"}],
        )

        self.assertEqual(document.tables["results.csv"]["rows"], [{
            "event_id": "event",
            "result_id": "win",
            "label": "New",
            "extension": "keep",
        }])

    def test_dataset_document_export_rolls_back_all_files_after_replace_failure(self):
        root = self.test_root
        objects = root / "objects.csv"
        future = root / "future.csv"
        objects.write_text("id,name,extension\nsoup,Soup,keep\n", encoding="utf-8")
        future.write_text("key,value\nalpha,old\n", encoding="utf-8")
        document = dataset_editor.DatasetDocument.load(root)
        document.tables["objects.csv"]["rows"][0]["name"] = "Stew"
        document.tables["future.csv"]["rows"][0]["value"] = "new"
        original_objects = objects.read_bytes()
        original_future = future.read_bytes()

        real_replace = dataset_editor.os.replace
        failed = False

        def fail_future_install(source, destination):
            nonlocal failed
            if Path(destination).resolve() == future.resolve():
                if not failed:
                    failed = True
                    raise OSError("simulated install failure")
            real_replace(source, destination)

        with mock.patch.object(dataset_editor.os, "replace", side_effect=fail_future_install):
            with self.assertRaises(OSError):
                document.export(root)

        self.assertEqual(objects.read_bytes(), original_objects)
        self.assertEqual(future.read_bytes(), original_future)
        self.assertFalse(any(path.name.startswith(".original-") for path in root.iterdir()))

    def test_staged_workspace_is_non_mutating_and_cleaned_up(self):
        root = self.test_root
        source = root / "source"
        source.mkdir()
        csv_path = source / "objects.csv"
        csv_path.write_text("id,name\nsoup,Soup\n", encoding="utf-8")
        document = dataset_editor.DatasetDocument.load(source)
        document.tables["objects.csv"]["rows"][0]["name"] = "Stew"
        original = csv_path.read_bytes()

        with document.staged_workspace(source) as workspace:
            staged = Path(workspace) / "objects.csv"
            self.assertEqual(staged.read_text(encoding="utf-8").splitlines()[1], "soup,Stew")
            self.assertEqual(csv_path.read_bytes(), original)
            self.assertTrue(Path(workspace).exists())
        self.assertFalse(Path(workspace).exists())
        self.assertEqual(csv_path.read_bytes(), original)

    def test_staged_validation_returns_response_and_cleans_workspace(self):
        document = dataset_editor.DatasetDocument(
            {"objects.csv": {"headers": ["id", "name"], "rows": [{"id": "soup", "name": "Soup"}]}}
        )
        seen = {}

        def run(command, **kwargs):
            seen["command"] = command
            seen["workspace"] = command[command.index("--dataset") + 1]
            self.assertTrue(Path(seen["workspace"]).is_dir())
            return subprocess.CompletedProcess(
                command, 0,
                stdout='{"ok": true, "mode": "validate", "errors": [], "warnings": []}',
                stderr="",
            )

        with mock.patch.object(dataset_editor.subprocess, "run", side_effect=run):
            response = dataset_editor.validate_staged_document(document, self.test_root, ["validator"])

        self.assertTrue(response["ok"])
        self.assertEqual(seen["command"][:1], ["validator"])
        self.assertIn("--validate", seen["command"])
        self.assertFalse(Path(seen["workspace"]).exists())

    def test_staged_validation_reports_validator_errors_and_cleans_workspace(self):
        document = dataset_editor.DatasetDocument(
            {"objects.csv": {"headers": ["id"], "rows": [{"id": "broken"}]}}
        )
        seen = {}

        def run(command, **kwargs):
            seen["workspace"] = command[command.index("--dataset") + 1]
            return subprocess.CompletedProcess(
                command, 1,
                stdout='{"ok": false, "errors": ["objects.csv row 2: invalid object"]}',
                stderr="",
            )

        with mock.patch.object(dataset_editor.subprocess, "run", side_effect=run):
            with self.assertRaisesRegex(dataset_editor.DatasetPreviewError, "invalid object"):
                dataset_editor.validate_staged_document(document, self.test_root, ["validator"])

        self.assertFalse(Path(seen["workspace"]).exists())

    def test_unsaved_visual_changes_validate_and_export_losslessly(self):
        root = self.test_root
        (root / "config.csv").write_text(
            "variable,value,editor_extension\n"
            "application_name,Old Game,keep-config\n",
            encoding="utf-8",
        )
        (root / "objects.csv").write_text(
            "id,type,name,price,engine_extension\n"
            "soup,food,Old Soup,3,keep-object\n",
            encoding="utf-8",
        )
        (root / "colors.csv").write_text(
            "element_id,label,hex_color,category,default_hex,visual_extension\n"
            "app_background,Background,#FFFFFF,base,#FFFFFF,keep-color\n",
            encoding="utf-8",
        )
        (root / "future_table.csv").write_text(
            "key,unknown_value\nalpha,untouched\n", encoding="utf-8"
        )
        asset = root / "art.bin"
        asset.write_bytes(b"asset-bytes")

        designer = dataset_editor.DatasetDesigner.__new__(dataset_editor.DatasetDesigner)
        designer.document = dataset_editor.DatasetDocument.load(root)
        designer.metadata = {
            "tables": [{"file": "colors.csv", "identity_fields": ["element_id"]}]
        }
        designer.extra_config_rows = []
        designer.config = {"application_name": "Unsaved Game"}
        designer.inventory_tabs = []
        designer.objects = [{"id": "soup", "type": "food", "name": "Unsaved Soup", "price": "9"}]
        designer.costs = []
        designer.events = []
        designer.quests = []
        designer.cost_rules = []
        designer.cost_conditions = []
        designer.colors = [{
            "element_id": "app_background",
            "label": "Background",
            "hex_color": "#123456",
            "category": "base",
            "default_hex": "#FFFFFF",
        }]

        original_config = (root / "config.csv").read_bytes()
        original_objects = (root / "objects.csv").read_bytes()
        original_colors = (root / "colors.csv").read_bytes()

        seen = {}

        def run(command, **kwargs):
            workspace = Path(command[command.index("--dataset") + 1])
            seen["workspace"] = workspace
            self.assertEqual(
                (workspace / "config.csv").read_text(encoding="utf-8").splitlines()[1],
                "application_name,Unsaved Game,keep-config",
            )
            self.assertIn("soup,food,Unsaved Soup,9,keep-object",
                          (workspace / "objects.csv").read_text(encoding="utf-8"))
            self.assertIn("app_background,Background,#123456,base,#FFFFFF,keep-color",
                          (workspace / "colors.csv").read_text(encoding="utf-8"))
            self.assertEqual(
                (workspace / "future_table.csv").read_text(encoding="utf-8"),
                "key,unknown_value\nalpha,untouched\n",
            )
            self.assertEqual((workspace / "art.bin").read_bytes(), b"asset-bytes")
            return subprocess.CompletedProcess(
                command, 0,
                stdout='{"ok": true, "mode": "validate", "errors": [], "warnings": []}',
                stderr="",
            )

        with mock.patch.object(dataset_editor.subprocess, "run", side_effect=run):
            response = dataset_editor.validate_staged_document(
                designer.current_document(), root, ["validator"]
            )

        self.assertTrue(response["ok"])
        self.assertFalse(seen["workspace"].exists())
        self.assertEqual((root / "config.csv").read_bytes(), original_config)
        self.assertEqual((root / "objects.csv").read_bytes(), original_objects)
        self.assertEqual((root / "colors.csv").read_bytes(), original_colors)

        designer.current_document().export(root)

        self.assertIn("application_name,Unsaved Game,keep-config",
                      (root / "config.csv").read_text(encoding="utf-8"))
        self.assertIn("soup,food,Unsaved Soup,9,keep-object",
                      (root / "objects.csv").read_text(encoding="utf-8"))
        self.assertIn("app_background,Background,#123456,base,#FFFFFF,keep-color",
                      (root / "colors.csv").read_text(encoding="utf-8"))
        self.assertEqual(
            (root / "future_table.csv").read_text(encoding="utf-8"),
            "key,unknown_value\nalpha,untouched\n",
        )
        self.assertEqual(asset.read_bytes(), b"asset-bytes")

    def test_validation_details_explicitly_formats_errors_and_warnings(self):
        details = dataset_editor.DatasetDesigner.validation_details({
            "errors": ["objects.csv row 2: invalid type"],
            "warnings": ["events.csv row 3: legacy field"],
        })
        self.assertIn("ERROR: objects.csv row 2: invalid type", details)
        self.assertIn("WARNING: events.csv row 3: legacy field", details)

    def test_metadata_field_helpers_supply_defaults_enums_and_references(self):
        metadata = {
            "tables": [{
                "file": "events.csv",
                "fields": [
                    {
                        "name": "duration_unit",
                        "default_value": "days",
                        "enum_values": ["minutes", "days"],
                        "references": [],
                        "description": "Time unit",
                    },
                    {
                        "name": "quest_id",
                        "default_value": None,
                        "enum_values": [],
                        "references": ["quests.csv"],
                    },
                ],
            }]
        }

        duration = dataset_editor.metadata_field(metadata, "events.csv", "duration_unit")
        quest = dataset_editor.metadata_field(metadata, "events.csv", "quest_id")

        self.assertEqual(duration["enum_values"], ["minutes", "days"])
        self.assertEqual(
            dataset_editor.metadata_default_row(
                metadata, "events.csv", ["duration_unit", "quest_id", "future"]
            ),
            {"duration_unit": "days", "quest_id": "", "future": ""},
        )
        self.assertEqual(quest["references"], ["quests.csv"])

    def test_event_authoring_builds_metadata_defaults_and_validates_references(self):
        metadata = {
            "tables": [{
                "file": "events.csv",
                "identity_fields": ["id"],
                "fields": [
                    {"name": "id", "type": "identifier", "required": True},
                    {"name": "name", "type": "string", "required": True},
                    {"name": "day_of_year", "type": "integer", "default_value": "0"},
                    {"name": "duration_unit", "type": "enum", "enum_values": ["days"]},
                    {"name": "quest_id", "type": "identifier", "references": ["quests.csv"]},
                ],
            }],
        }
        row = dataset_editor.build_event_row(
            metadata, {"id": "festival", "name": "Festival", "quest_id": "missing"}
        )
        self.assertEqual(row["day_of_year"], "0")
        errors = dataset_editor.validate_event_row(
            metadata, row, {"quests.csv": [{"id": "story"}]}
        )
        self.assertEqual(errors[0]["code"], "missing_reference")
        self.assertEqual(errors[0]["field"], "quest_id")

    def test_event_authoring_upsert_rejects_duplicates_and_preserves_atomicity(self):
        metadata = {"tables": [{
            "file": "events.csv",
            "identity_fields": ["id"],
            "fields": [
                {"name": "id", "type": "identifier", "required": True},
                {"name": "name", "type": "string", "required": True},
            ],
        }]}
        rows = [{"id": "race", "name": "Race"}]
        original = [dict(row) for row in rows]
        _, errors = dataset_editor.upsert_event_row(
            metadata, rows, {"id": "race", "name": "Other"}
        )
        self.assertEqual(errors[0]["code"], "duplicate")
        self.assertEqual(rows, original)

    def test_requirement_group_composer_uses_metadata_and_preserves_extensions(self):
        metadata = {
            "tables": [
                {
                    "file": "condition_groups.csv",
                    "identity_fields": ["id"],
                    "fields": [
                        {"name": "id", "value_type": "identifier", "required": True},
                        {
                            "name": "operator",
                            "value_type": "enum",
                            "required": True,
                            "enum_values": ["all", "any", "not"],
                        },
                        {
                            "name": "children",
                            "value_type": "identifier_list",
                            "references": ["condition_groups.csv"],
                        },
                    ],
                },
                {
                    "file": "conditions.csv",
                    "identity_fields": ["id"],
                    "fields": [
                        {"name": "id", "value_type": "identifier", "required": True},
                        {
                            "name": "group_id",
                            "value_type": "identifier",
                            "required": True,
                            "references": ["condition_groups.csv"],
                        },
                        {
                            "name": "subject_type",
                            "value_type": "enum",
                            "required": True,
                            "enum_values": ["characteristic", "fact"],
                        },
                        {"name": "subject_ref", "value_type": "identifier", "required": True},
                        {
                            "name": "operator",
                            "value_type": "enum",
                            "required": True,
                            "enum_values": ["equals", "greater_than"],
                        },
                        {"name": "value", "value_type": "string"},
                    ],
                },
            ]
        }
        composed = dataset_editor.build_requirement_group(
            metadata,
            {"id": "eligible", "operator": "all"},
            [{"id": "level", "group_id": "eligible", "subject_type": "fact",
              "subject_ref": "level", "operator": "equals", "value": "3"}],
            {"id": "eligible", "operator": "all", "future_group_field": "keep"},
            [{"id": "level", "group_id": "eligible", "subject_type": "fact",
              "subject_ref": "old", "operator": "equals", "value": "2",
              "future_condition_field": "keep"}],
        )
        self.assertEqual(composed["group"]["future_group_field"], "keep")
        self.assertEqual(composed["conditions"][0]["future_condition_field"], "keep")
        self.assertEqual(
            dataset_editor.validate_requirement_group(
                metadata,
                composed["group"],
                composed["conditions"],
                {"condition_groups.csv": [{"id": "eligible"}]},
            ),
            [],
        )

    def test_requirement_group_validator_checks_operator_subject_references_and_nesting(self):
        metadata = {
            "tables": [
                {
                    "file": "condition_groups.csv",
                    "identity_fields": ["id"],
                    "fields": [
                        {"name": "id", "value_type": "identifier", "required": True},
                        {"name": "operator", "value_type": "enum", "required": True,
                         "enum_values": ["all", "any", "not"]},
                        {"name": "children", "value_type": "identifier_list",
                         "references": ["condition_groups.csv"]},
                    ],
                },
                {
                    "file": "conditions.csv",
                    "identity_fields": ["id"],
                    "fields": [
                        {"name": "id", "value_type": "identifier", "required": True},
                        {"name": "group_id", "value_type": "identifier", "required": True,
                         "references": ["condition_groups.csv"]},
                        {"name": "subject_type", "value_type": "enum", "required": True,
                         "enum_values": ["fact"]},
                        {"name": "subject_ref", "value_type": "identifier", "required": True},
                        {"name": "operator", "value_type": "enum", "required": True,
                         "enum_values": ["equals"]},
                    ],
                },
            ]
        }
        errors = dataset_editor.validate_requirement_group(
            metadata,
            {"id": "root", "operator": "xor", "children": "root;missing"},
            [{"id": "check", "group_id": "other", "subject_type": "unknown",
              "subject_ref": "x", "operator": "bad"}],
            {"condition_groups.csv": [{"id": "root"}]},
        )
        codes = {error["code"] for error in errors}
        self.assertTrue({"invalid_choice", "cycle", "missing_reference", "group_mismatch"} <= codes)

    def test_requirement_group_upsert_is_atomic_across_group_and_conditions(self):
        metadata = {
            "tables": [
                {"file": "condition_groups.csv", "identity_fields": ["id"], "fields": [
                    {"name": "id", "value_type": "identifier", "required": True},
                    {"name": "operator", "value_type": "enum", "required": True,
                     "enum_values": ["all", "any"]},
                ]},
                {"file": "conditions.csv", "identity_fields": ["id"], "fields": [
                    {"name": "id", "value_type": "identifier", "required": True},
                    {"name": "group_id", "value_type": "identifier", "required": True},
                    {"name": "operator", "value_type": "enum", "required": True,
                     "enum_values": ["equals"]},
                ]},
            ]
        }
        groups = [{"id": "root", "operator": "all", "future": "keep"}]
        conditions = [{"id": "old", "group_id": "root", "operator": "equals"}]
        original_groups = [dict(row) for row in groups]
        original_conditions = [dict(row) for row in conditions]
        _, errors = dataset_editor.upsert_requirement_group(
            metadata, groups, conditions,
            {"id": "root", "operator": "invalid"},
            [{"id": "new", "group_id": "root", "operator": "equals"}],
        )
        self.assertEqual(errors[0]["code"], "invalid_choice")
        self.assertEqual(groups, original_groups)
        self.assertEqual(conditions, original_conditions)

    def test_generic_metadata_row_helper_builds_effect_modifier_defaults_and_extensions(self):
        metadata = {
            "tables": [{
                "file": "numeric_modifiers.csv",
                "identity_fields": ["target", "operation"],
                "fields": [
                    {"name": "target", "value_type": "identifier", "required": True},
                    {"name": "operation", "value_type": "enum", "required": True,
                     "enum_values": ["add", "multiply"]},
                    {"name": "effect_id", "value_type": "identifier",
                     "references": ["effects.csv"]},
                    {"name": "priority", "value_type": "integer", "default_value": 0},
                ],
            }],
        }
        row = dataset_editor.build_metadata_row(
            metadata,
            "numeric_modifiers.csv",
            {"target": "speed", "operation": "add", "effect_id": "boost"},
            {"target": "speed", "operation": "add", "future": "keep"},
        )

        self.assertEqual(row["priority"], "0")
        self.assertEqual(row["future"], "keep")
        self.assertEqual(
            dataset_editor.validate_metadata_row(
                metadata,
                "numeric_modifiers.csv",
                row,
                {"effects.csv": [{"id": "boost"}]},
            ),
            [],
        )

    def test_generic_metadata_row_upsert_validates_references_and_is_atomic(self):
        metadata = {
            "tables": [{
                "file": "effect_bindings.csv",
                "identity_fields": ["effect_id", "object_id"],
                "fields": [
                    {"name": "effect_id", "value_type": "identifier", "required": True,
                     "references": ["effects.csv"]},
                    {"name": "object_id", "value_type": "identifier", "required": True,
                     "references": ["objects.csv"]},
                ],
            }],
        }
        rows = [{"effect_id": "old", "object_id": "sword", "future": "keep"}]
        original = [dict(row) for row in rows]

        _, errors = dataset_editor.upsert_metadata_row(
            metadata,
            "effect_bindings.csv",
            rows,
            {"effect_id": "missing", "object_id": "sword"},
            {"effects.csv": [{"id": "old"}], "objects.csv": [{"id": "sword"}]},
        )

        self.assertEqual(errors[0]["code"], "missing_reference")
        self.assertEqual(rows, original)

        row, errors = dataset_editor.upsert_metadata_row(
            metadata,
            "effect_bindings.csv",
            rows,
            {"effect_id": "old", "object_id": "sword"},
            {"effects.csv": [{"id": "old"}], "objects.csv": [{"id": "sword"}]},
        )
        self.assertEqual(errors[0]["code"], "duplicate")
        self.assertEqual(rows, original)

    def test_document_diagnostics_reports_parse_and_duplicate_identity_locations(self):
        root = self.test_root
        (root / "objects.csv").write_text(
            "id,name\nsoup,Soup\nsoup,Stew\nbroken,Oops,extra\n", encoding="utf-8"
        )
        document = dataset_editor.DatasetDocument.load(root)
        metadata = {
            "tables": [{
                "file": "objects.csv",
                "identity_fields": ["id"],
                "fields": [{"name": "id", "required": True}],
            }]
        }
        document.metadata = metadata

        diagnostics = dataset_editor.document_diagnostics(document)

        self.assertEqual(
            [(item["code"], item.get("row")) for item in diagnostics],
            [("parse_error", 4), ("duplicate_identity", 3)],
        )

    def test_document_diagnostics_marks_unsupported_fields_without_rejecting_them(self):
        metadata = {
            "tables": [{
                "file": "events.csv",
                "identity_fields": ["id"],
                "fields": [
                    {"name": "id", "required": True},
                    {"name": "legacy_script", "supported": False},
                ],
            }]
        }
        document = dataset_editor.DatasetDocument(
            {"events.csv": {
                "headers": ["id", "legacy_script"],
                "rows": [{"id": "event", "legacy_script": "keep"}],
            }},
            metadata,
        )

        diagnostics = dataset_editor.document_diagnostics(document)

        self.assertEqual(diagnostics[0]["code"], "unsupported_field")
        self.assertEqual(diagnostics[0]["table"], "events.csv")
        self.assertEqual(diagnostics[0]["row"], 2)
        self.assertEqual(diagnostics[0]["field"], "legacy_script")
        self.assertEqual(
            dataset_editor.validate_metadata_row(
                metadata, "events.csv", document.tables["events.csv"]["rows"][0]
            ),
            [],
        )

    def test_metadata_diagnostics_identifies_missing_reference_table_for_navigation(self):
        metadata = {
            "tables": [{
                "file": "events.csv",
                "identity_fields": ["id"],
                "fields": [{
                    "name": "id", "required": True,
                }, {
                    "name": "quest_id", "references": ["quests.csv"],
                }],
            }]
        }

        diagnostics = dataset_editor.diagnose_metadata_row(
            metadata, "events.csv", {"id": "festival", "quest_id": "story"},
            row_number=7,
        )

        self.assertEqual(diagnostics[0]["code"], "missing_reference_table")
        self.assertEqual(diagnostics[0]["table"], "events.csv")
        self.assertEqual(diagnostics[0]["row"], 7)
        self.assertEqual(diagnostics[0]["field"], "quest_id")


if __name__ == "__main__":
    unittest.main()
