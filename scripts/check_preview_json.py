#!/usr/bin/env python3
"""Run deterministic authoring-preview smoke checks against a proof fixture."""

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "src-tauri" / "Cargo.toml"
DATASET = ROOT / "tests" / "fixtures" / "cooking"
SEQUENCE = (
    "join_quest:recipe_quest;"
    "buy:skillet;"
    "enter:bake_pie:skillet_1;"
    "submit:event_entry_bake_pie_1:success"
)


def run_preview(*arguments: str) -> dict:
    command = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(MANIFEST),
        "--bin",
        "dataset_preview",
        "--",
        "--dataset",
        str(DATASET),
        *arguments,
    ]
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    response = json.loads(completed.stdout)
    if not isinstance(response, dict):
        raise AssertionError("preview response must be a JSON object")
    return response


def main() -> None:
    validation = run_preview("--validate")
    assert validation["ok"] is True, validation
    assert validation["mode"] == "validate", validation
    assert validation["errors"] == [], validation

    first = run_preview("--seed", "42", "--execute-sequence", SEQUENCE)
    second = run_preview("--seed", "42", "--execute-sequence", SEQUENCE)
    assert first == second, "seeded preview JSON must be reproducible"
    assert first["ok"] is True, first
    assert first["mode"] == "execute", first
    assert any(
        diff["path"] == "event_history" for diff in first["state_diff"]
    ), first
    assert any(
        "step 4/4 applied" in message for message in first["diagnostics"]
    ), first

    print("preview JSON regression checks passed")


if __name__ == "__main__":
    main()
