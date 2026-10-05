"""Small logging helper shared by standalone Python plugins."""

from __future__ import annotations

import logging
import os
import sys
from typing import TextIO


def configure_logging(name: str, destination: str | None = None) -> logging.Logger:
    """Configure one plugin logger without corrupting JSON protocol output.

    ``stdout`` is the default for standalone GUI runs. JSON protocol callers
    should pass ``stderr`` or set ``TTRPG_LOG_DEST=stderr``.
    """
    target = (destination or os.environ.get("TTRPG_LOG_DEST", "stdout")).lower()
    stream: TextIO = sys.stderr if target == "stderr" else sys.stdout
    logger = logging.getLogger(name)
    logger.handlers.clear()
    logger.setLevel(logging.INFO)
    logger.propagate = False
    handler = logging.StreamHandler(stream)
    handler.setFormatter(logging.Formatter("%(asctime)s %(name)s %(levelname)s %(message)s"))
    logger.addHandler(handler)
    return logger
