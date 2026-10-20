"""Train a masked PPO policy for gtr2career.

This script is optional and requires rl/requirements.txt. The Rust simulator
remains the source of truth; this process only learns an action policy.
"""

from __future__ import annotations

import argparse
import time
from pathlib import Path

from gtr2career_env import Gtr2CareerEnv, RustEnvConfig


def _format_duration(seconds: float) -> str:
    total = max(0, int(seconds))
    hours, remainder = divmod(total, 3600)
    minutes, seconds = divmod(remainder, 60)
    if hours:
        return f"{hours}h {minutes:02d}m {seconds:02d}s"
    if minutes:
        return f"{minutes}m {seconds:02d}s"
    return f"{seconds}s"


def main() -> None:
    try:
        from sb3_contrib import MaskablePPO
        from stable_baselines3.common.callbacks import BaseCallback
    except ImportError as error:
        raise SystemExit(
            "Install rl/requirements.txt before training the PPO policy"
        ) from error

    parser = argparse.ArgumentParser()
    parser.add_argument("--dataset", default="gtr2career")
    parser.add_argument("--goal", default="charisma>=100")
    parser.add_argument("--max-days", type=int, default=7300)
    parser.add_argument("--max-steps", type=int, default=14600)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--timesteps", type=int, default=100_000)
    parser.add_argument("--n-steps", type=int, default=2048)
    parser.add_argument("--batch-size", type=int, default=64)
    parser.add_argument(
        "--progress",
        action="store_true",
        help="print training percentage, throughput, elapsed time, and ETA",
    )
    parser.add_argument("--progress-interval", type=int, default=1000)
    parser.add_argument("--output", default="rl/models/gtr2career-ppo")
    parser.add_argument("--binary", default="src-tauri/target/release/rl_env")
    args = parser.parse_args()

    env = Gtr2CareerEnv(
        RustEnvConfig(
            dataset=args.dataset,
            seed=args.seed,
            goal=args.goal,
            max_days=args.max_days,
            max_steps=args.max_steps,
            binary=args.binary,
        )
    )
    callback = None
    if args.progress:
        if args.progress_interval <= 0:
            raise SystemExit("--progress-interval must be greater than zero")

        class ProgressCallback(BaseCallback):
            def __init__(self, interval: int):
                super().__init__(verbose=0)
                self.interval = interval
                self.started_at = 0.0
                self.last_report = 0

            def _on_training_start(self) -> None:
                self.started_at = time.monotonic()
                self._report(force=True)

            def _on_step(self) -> bool:
                if self.num_timesteps - self.last_report >= self.interval:
                    self._report()
                return True

            def _on_training_end(self) -> None:
                self._report()

            def _report(self, force: bool = False) -> None:
                total = max(1, self.model._total_timesteps)
                completed = min(self.num_timesteps, total)
                if not force and completed == self.last_report:
                    return
                elapsed = time.monotonic() - self.started_at
                rate = completed / elapsed if elapsed > 0 else 0.0
                remaining = (total - completed) / rate if rate > 0 else 0.0
                percent = completed / total * 100
                eta = _format_duration(remaining) if rate > 0 else "calculating"
                print(
                    f"[progress] {completed:,}/{total:,} ({percent:.1f}%) "
                    f"elapsed={_format_duration(elapsed)} "
                    f"rate={rate:.1f} steps/s eta={eta}",
                    flush=True,
                )
                self.last_report = completed

        callback = ProgressCallback(args.progress_interval)
    model = MaskablePPO(
        "MultiInputPolicy",
        env,
        verbose=1,
        seed=args.seed,
        device="cpu",
        n_steps=args.n_steps,
        batch_size=args.batch_size,
    )
    model.learn(total_timesteps=args.timesteps, callback=callback)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    model.save(output)
    env.close()


if __name__ == "__main__":
    main()
