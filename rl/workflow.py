"""Shared CLI configuration and model-contract checks for local playtesting."""

from __future__ import annotations

import argparse
import hashlib
import json
import warnings
from dataclasses import asdict
from pathlib import Path
from typing import Any

from gtr2career_env import Gtr2CareerEnv, RustEnvConfig


def positive_int(value: str) -> int:
    number = int(value)
    if number <= 0:
        raise argparse.ArgumentTypeError("must be greater than zero")
    return number


def add_environment_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--dataset", default="gtr2career")
    parser.add_argument("--goal", default="charisma>=100")
    parser.add_argument("--max-days", type=positive_int, default=7300)
    parser.add_argument("--max-steps", type=positive_int, default=14600)
    parser.add_argument("--binary", default="src-tauri/target/release/rl_env")
    parser.add_argument("--reward-config", help="JSON reward/penalty overrides; omitted fields keep engine defaults")


def environment_config(args: argparse.Namespace) -> RustEnvConfig:
    return RustEnvConfig(**{
        key: getattr(args, key)
        for key in RustEnvConfig.__dataclass_fields__
    })


def model_path(path: str | Path) -> Path:
    path = Path(path)
    return path if path.suffix == ".zip" else Path(str(path) + ".zip")


def model_contract(env: Gtr2CareerEnv) -> dict[str, Any]:
    observation = env.debug_observation()
    digest = hashlib.sha256()
    dataset = Path(env.config.dataset)
    for path in sorted(dataset.rglob("*")):
        if path.is_file() and path.suffix in (".csv", ".py"):
            digest.update(path.relative_to(dataset).as_posix().encode())
            digest.update(path.read_bytes())
    return {
        "schema": 1,
        "goal": env.config.goal,
        "dataset_sha256": digest.hexdigest(),
        "observation_schema": observation["schema_version"],
        "characteristics": sorted(observation["characteristics"]),
        "action_keys": [action["key"] for action in env.action_definitions],
    }


def save_model(model: Any, output: str | Path, env: Gtr2CareerEnv) -> None:
    path = model_path(output)
    path.parent.mkdir(parents=True, exist_ok=True)
    model.save(path)
    metadata = {
        "contract": model_contract(env),
        "environment": asdict(env.config),
        "reward_overrides": (
            json.loads(Path(env.config.reward_config).read_text())
            if env.config.reward_config else {}
        ),
        "timesteps": model.num_timesteps,
        "training": {
            "gamma": model.gamma, "ent_coef": model.ent_coef,
            "n_steps": model.n_steps, "batch_size": model.batch_size,
            "seed": model.seed,
        },
    }
    Path(str(path) + ".json").write_text(json.dumps(metadata, indent=2) + "\n")


def check_model_contract(path: str | Path, env: Gtr2CareerEnv) -> None:
    metadata_path = Path(str(model_path(path)) + ".json")
    if not metadata_path.exists():
        warnings.warn(
            "Legacy model has no training metadata: goal, dataset and action order "
            "cannot be verified. Retrain if the dataset or goal has changed.",
            stacklevel=2,
        )
        return
    metadata = json.loads(metadata_path.read_text())
    actual = model_contract(env)
    differences = [
        key for key, value in actual.items()
        if metadata["contract"].get(key) != value
    ]
    if differences:
        raise ValueError(
            "Model does not match this run (" + ", ".join(differences)
            + "). Use the training dataset/goal or train a new model."
        )
