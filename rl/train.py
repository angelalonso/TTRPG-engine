"""Train a masked PPO policy for gtr2career.

This script is optional and requires rl/requirements.txt. The Rust simulator
remains the source of truth; this process only learns an action policy.
"""

from __future__ import annotations

import argparse
import os
import signal
import statistics
import sys
import time

from gtr2career_env import Gtr2CareerEnv
from workflow import (
    add_environment_arguments, check_model_contract, environment_config,
    positive_int, save_model,
)


def _format_duration(seconds: float) -> str:
    total = max(0, int(seconds))
    hours, remainder = divmod(total, 3600)
    minutes, seconds = divmod(remainder, 60)
    if hours:
        return f"{hours}h {minutes:02d}m {seconds:02d}s"
    if minutes:
        return f"{minutes}m {seconds:02d}s"
    return f"{seconds}s"


def _handle_training_signal(signum: int, _frame: object) -> None:
    signal_name = signal.Signals(signum).name
    print(
        f"\nTraining interrupted by {signal_name}; saving interrupted checkpoint.",
        file=sys.stderr,
        flush=True,
    )
    raise SystemExit(128 + signum)


def main() -> None:
    parser = argparse.ArgumentParser()
    add_environment_arguments(parser)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--timesteps", type=positive_int, default=100_000)
    parser.add_argument("--n-steps", type=positive_int, default=2048)
    parser.add_argument("--batch-size", type=positive_int, default=64)
    parser.add_argument("--gamma", type=float, default=0.99, help="Future reward discount per action; consider 0.999 for distant goals")
    parser.add_argument("--ent-coef", type=float, default=0.0, help="Exploration bonus; try 0.01 if the policy repeats one strategy")
    parser.add_argument("--checkpoint-interval", type=positive_int, default=10000)
    parser.add_argument("--resume", help="Continue a saved model; timesteps counts additional training")
    parser.add_argument("--verbose-steps", action="store_true", help="Explain every training action and its reward components (large logs; may slow training)")
    parser.add_argument(
        "--progress",
        action="store_true",
        help="print training percentage, throughput, elapsed time, and ETA",
    )
    parser.add_argument("--progress-interval", type=positive_int, default=1000)
    parser.add_argument(
        "--episode-log-interval",
        type=positive_int,
        default=10,
        help="print success rate and mean game days after this many episodes",
    )
    parser.add_argument("--output", default="rl/models/gtr2career-ppo")
    args = parser.parse_args()
    if args.n_steps < 2 or args.batch_size < 2 or args.batch_size > args.n_steps or args.n_steps % args.batch_size:
        parser.error("--n-steps must be >= 2 and divisible by --batch-size (which must be >= 2)")
    if not 0 < args.gamma < 1 or not 0 <= args.ent_coef <= 1:
        parser.error("--gamma must be between 0 and 1; --ent-coef must be between 0 and 1 inclusive")
    try:
        os.environ.setdefault("CUDA_VISIBLE_DEVICES", "")
        from sb3_contrib import MaskablePPO
        from stable_baselines3.common.callbacks import BaseCallback
    except ImportError as error:
        raise SystemExit(
            "Install rl/requirements.txt before training the PPO policy"
        ) from error
    signal.signal(signal.SIGTERM, _handle_training_signal)
    signal.signal(signal.SIGINT, _handle_training_signal)

    env = Gtr2CareerEnv(environment_config(args))

    class TrainingLogCallback(BaseCallback):
        def __init__(self, progress: bool):
            super().__init__(verbose=0)
            self.show_progress = progress
            self.started_at = 0.0
            self.last_progress = 0
            self.episodes = 0
            self.goals = 0
            self.days: list[int] = []
            self.successful_days: list[int] = []
            self.outcomes: dict[str, int] = {}
            self.last_checkpoint = 0
            self.start_timesteps = 0

        def _on_training_start(self) -> None:
            self.started_at = time.monotonic()
            self.start_timesteps = self.num_timesteps
            self.last_checkpoint = self.num_timesteps
            self.last_progress = self.num_timesteps
            print(
                f"[training] started timesteps={args.timesteps:,} "
                f"seed={args.seed} goal={args.goal}",
                flush=True,
            )

        def _on_step(self) -> bool:
            infos = self.locals.get("infos", [])
            dones = self.locals.get("dones", [])
            for index, (info, done) in enumerate(zip(infos, dones)):
                if args.verbose_steps:
                    action_id = int(self.locals["actions"][index])
                    print(
                        f"[practice] action={env.action_definitions[action_id]['key']} "
                        f"step={self.num_timesteps} day={info['current_day']} "
                        f"days_elapsed={info['days_elapsed']} status={info.get('status')} "
                        f"progress={info.get('goal_progress')} "
                        f"reward={float(self.locals['rewards'][index]):+.3f} "
                        f"components={info.get('reward_components', {})}",
                        flush=True,
                    )
                if not done:
                    continue
                self.episodes += 1
                if info.get("status") == "goal_reached":
                    self.goals += 1
                    self.successful_days.append(max(0, int(info["current_day"]) - 1))
                status = info.get("status", "unknown")
                self.outcomes[status] = self.outcomes.get(status, 0) + 1
                self.days.append(max(0, int(info.get("current_day", 1)) - 1))
                if self.episodes % args.episode_log_interval == 0:
                    self._report_episodes()
            if self.show_progress and self.num_timesteps - self.last_progress >= args.progress_interval:
                self._report_progress()
            if self.num_timesteps - self.last_checkpoint >= args.checkpoint_interval:
                save_model(self.model, f"{args.output}-step-{self.num_timesteps}", env)
                self.last_checkpoint = self.num_timesteps
                print(f"[checkpoint] saved step {self.num_timesteps}", flush=True)
            return True

        def _on_training_end(self) -> None:
            if self.episodes:
                self._report_episodes()
            if self.show_progress:
                self._report_progress(force=True)
            print(
                f"[training] finished episodes={self.episodes} "
                f"success_rate={self.goals / max(1, self.episodes):.1%} "
                f"mean_days={statistics.mean(self.days) if self.days else 0:.1f}",
                flush=True,
            )

        def _report_episodes(self) -> None:
            print(
                f"[training] episodes={self.episodes} "
                f"success_rate={self.goals / max(1, self.episodes):.1%} "
                f"successful_mean_days={statistics.mean(self.successful_days) if self.successful_days else 'n/a'} "
                f"outcomes={self.outcomes}",
                flush=True,
            )
            if self.goals == 0:
                print("[hint] No wins yet: try the goal manually, inspect a random baseline and consider an easier milestone. More timesteps alone may not help.", flush=True)

        def _report_progress(self, force: bool = False) -> None:
            total = max(1, self.model._total_timesteps - self.start_timesteps)
            completed = min(self.num_timesteps - self.start_timesteps, total)
            if not force and self.num_timesteps == self.last_progress:
                return
            elapsed = time.monotonic() - self.started_at
            rate = completed / elapsed if elapsed > 0 else 0.0
            remaining = (total - completed) / rate if rate > 0 else 0.0
            print(
                f"[progress] {completed:,}/{total:,} ({completed / total * 100:.1f}%) "
                f"elapsed={_format_duration(elapsed)} rate={rate:.1f} steps/s "
                f"eta={_format_duration(remaining) if rate > 0 else 'calculating'}",
                flush=True,
            )
            self.last_progress = self.num_timesteps

    model = None
    try:
        if args.resume:
            check_model_contract(args.resume, env)
            model = MaskablePPO.load(
                args.resume, env=env, device="cpu", seed=args.seed,
                n_steps=args.n_steps, batch_size=args.batch_size,
                gamma=args.gamma, ent_coef=args.ent_coef,
            )
        else:
            model = MaskablePPO(
                "MultiInputPolicy", env, verbose=1, seed=args.seed, device="cpu",
                n_steps=args.n_steps, batch_size=args.batch_size,
                gamma=args.gamma, ent_coef=args.ent_coef,
            )
        model.learn(
            total_timesteps=args.timesteps,
            callback=TrainingLogCallback(args.progress),
            reset_num_timesteps=not bool(args.resume),
        )
        save_model(model, args.output, env)
    except (KeyboardInterrupt, SystemExit):
        if model is not None:
            save_model(model, f"{args.output}-interrupted", env)
            print(f"[checkpoint] saved {args.output}-interrupted.zip", file=sys.stderr)
        raise
    finally:
        env.close()


if __name__ == "__main__":
    main()
