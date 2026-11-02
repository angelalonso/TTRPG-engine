#!/usr/bin/env python3
"""Generate 240x180 JPEG thumbnails beside dataset images.

Usage:
    python3 scripts/generate_thumbnails.py gtr2career/img dataset_w_more/img
"""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image, ImageOps


IMAGE_EXTENSIONS = {".jpg", ".jpeg", ".png", ".webp"}
THUMBNAIL_SIZE = (240, 180)


def generate_thumbnail(source: Path, force: bool) -> bool:
    if source.stem.lower().startswith("thumb_"):
        return False

    destination = source.with_name(f"thumb_{source.stem}.jpeg")
    if destination.exists() and not force:
        return False

    with Image.open(source) as image:
        thumbnail = ImageOps.fit(
            image.convert("RGB"),
            THUMBNAIL_SIZE,
            method=Image.Resampling.LANCZOS,
        )
        thumbnail.save(destination, format="JPEG", quality=85, optimize=True)
    return True


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate 240x180 JPEG thumbnails for dataset images."
    )
    parser.add_argument(
        "directories",
        nargs="+",
        type=Path,
        help="Directories to scan recursively for source images.",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Regenerate thumbnails that already exist.",
    )
    args = parser.parse_args()

    generated = 0
    for directory in args.directories:
        if not directory.is_dir():
            parser.error(f"not a directory: {directory}")
        for source in sorted(directory.rglob("*")):
            if source.is_file() and source.suffix.lower() in IMAGE_EXTENSIONS:
                if generate_thumbnail(source, args.force):
                    generated += 1

    print(f"Generated {generated} thumbnail(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
