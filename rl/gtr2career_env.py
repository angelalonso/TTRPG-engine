"""Gymnasium adapter for the local Rust RL environment.

The module imports Gymnasium lazily so repository tooling can still inspect
the adapter when optional ML dependencies are not installed.
"""

from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass
from typing import Any

try:
    import numpy as np
except ImportError:  # pragma: no cover - optional training dependency
    np = None

try:
    import gymnasium as gym
except ImportError:  # pragma: no cover - optional training dependency
    gym = None


_GymEnv = gym.Env if gym is not None else object


@dataclass
class RustEnvConfig:
    dataset: str = "gtr2career"
    seed: int = 1
    goal: str = "charisma>=100"
    max_days: int = 7300
    max_steps: int = 14600
    binary: str = "src-tauri/target/release/rl_env"


class Gtr2CareerEnv(_GymEnv):
    """A small Gymnasium-compatible wrapper around the persistent Rust process."""

    metadata = {"render_modes": []}

    def __init__(self, config: RustEnvConfig | None = None):
        if gym is None:
            raise RuntimeError(
                "Gymnasium is optional; install rl/requirements.txt first"
            )
        if np is None:
            raise RuntimeError("NumPy is required; install rl/requirements.txt first")
        super().__init__()
        self.config = config or RustEnvConfig()
        command = [
            os.path.abspath(self.config.binary),
            "--dataset",
            self.config.dataset,
            "--seed",
            str(self.config.seed),
            "--goal",
            self.config.goal,
            "--max-days",
            str(self.config.max_days),
            "--max-steps",
            str(self.config.max_steps),
        ]
        self._process = subprocess.Popen(
            command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        self._actions = self._request({"op": "actions"})["actions"]
        self.action_definitions = tuple(self._actions)
        self._characteristic_keys: list[str] = []
        initial = self._request({"op": "observation"})["observation"]
        self._characteristic_keys = sorted(initial["characteristics"])
        self.action_space = gym.spaces.Discrete(len(self._actions))
        self.observation_space = gym.spaces.Dict(
            {
                "numeric": gym.spaces.Box(
                    low=-np.inf,
                    high=np.inf,
                    shape=(len(self._characteristic_keys) + 4,),
                    dtype=np.float32,
                ),
                "action_mask": gym.spaces.MultiBinary(len(self._actions)),
            }
        )

    def _request(self, request: dict[str, Any]) -> dict[str, Any]:
        if self._process.stdin is None or self._process.stdout is None:
            raise RuntimeError("RL environment process is not available")
        self._process.stdin.write(json.dumps(request) + "\n")
        self._process.stdin.flush()
        line = self._process.stdout.readline()
        if not line:
            raise RuntimeError("RL environment process exited unexpectedly")
        response = json.loads(line)
        if "error" in response:
            raise RuntimeError(response["error"])
        return response

    def _observation(self, raw: dict[str, Any]) -> dict[str, Any]:
        values = raw["characteristics"]
        numeric = [
            float(values.get(key, 0.0)) for key in self._characteristic_keys
        ]
        numeric.extend(
            [
                float(raw["current_day"]),
                float(raw["days_per_year"]),
                float(len(raw["inventory_ids"])),
                float(len(raw["quest_ids"])),
            ]
        )
        return {
            "numeric": np.asarray(numeric, dtype=np.float32),
            "action_mask": np.asarray(raw["legal_action_mask"], dtype=np.int8),
        }

    def debug_observation(self) -> dict[str, Any]:
        """Return the structured engine observation for playtest diagnostics."""
        return self._request({"op": "observation"})["observation"]

    def reset(self, *, seed: int | None = None, options: dict | None = None):
        del options
        if seed is not None:
            super().reset(seed=seed)
        actual_seed = self.config.seed if seed is None else seed
        raw = self._request({"op": "reset", "seed": int(actual_seed)})[
            "observation"
        ]
        return self._observation(raw), {"seed": actual_seed}

    def step(self, action: int):
        result = self._request({"op": "step", "action_id": int(action)})["result"]
        observation = self._observation(result["observation"])
        self._last_action_mask = observation["action_mask"]
        return (
            observation,
            float(result["reward"]),
            bool(result["terminated"]),
            bool(result["truncated"]),
            result["info"],
        )

    def action_masks(self):
        """Return the current mask for sb3-contrib's MaskablePPO."""
        if not hasattr(self, "_last_action_mask"):
            raw = self._request({"op": "observation"})["observation"]
            self._last_action_mask = self._observation(raw)["action_mask"]
        return self._last_action_mask.astype(bool)

    def close(self):
        if self._process.poll() is None:
            self._process.terminate()
            self._process.wait(timeout=5)
