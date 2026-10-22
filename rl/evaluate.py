"""Evaluate with the same masks, limits, diagnostics and metrics as playtesting."""

import sys

from playtest import main


if __name__ == "__main__":
    # Evaluation retains its historical held-out seed range.
    if "--seed" not in sys.argv and not any(arg.startswith("--seed=") for arg in sys.argv):
        sys.argv.extend(["--seed", "10000"])
    main()
