#!/usr/bin/env python3
"""Reusable Hello World plugin template for TTRPG Engine.

Normal execution opens a small themed GUI. ``-inputlist`` and ``--json`` keep
the plugin useful as a protocol template for future plugins.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


PLUGIN_ID = "plugin_template"
PROTOCOL_VERSION = 1
RESULT_SCHEMA_VERSION = 1
INPUT_LIST = {
    "inputs": [
        {
            "key": "player_name",
            "type": "string",
            "label": "Player Name",
            "required": True,
        }
    ]
}


def parser() -> argparse.ArgumentParser:
    argument_parser = argparse.ArgumentParser(
        description="Hello World plugin template for TTRPG Engine."
    )
    argument_parser.add_argument(
        "-inputlist",
        "--inputlist",
        action="store_true",
        help="Print the structured list of inputs required by this plugin.",
    )
    argument_parser.add_argument(
        "--json",
        action="store_true",
        help="Use the line-oriented JSON plugin protocol instead of opening the GUI.",
    )
    return argument_parser


def response_for(request: dict[str, Any]) -> dict[str, Any]:
    if request.get("protocol_version") != PROTOCOL_VERSION:
        raise ValueError("unsupported protocol version")
    if request.get("plugin_id") != PLUGIN_ID:
        raise ValueError(f"request belongs to '{request.get('plugin_id')}', expected '{PLUGIN_ID}'")

    payload = request.get("payload")
    if not isinstance(payload, dict):
        raise ValueError("payload must be an object")
    player_name = str(payload.get("player_name", "")).strip()
    if not player_name:
        raise ValueError("player_name is required")

    return {
        "protocol_version": PROTOCOL_VERSION,
        "plugin_id": PLUGIN_ID,
        "result_schema_version": RESULT_SCHEMA_VERSION,
        "results": [
            {
                "id": "greeting",
                "value": f"Hello, {player_name}!",
                "label": "Greeting",
            }
        ],
    }


def run_protocol() -> int:
    try:
        request = json.load(sys.stdin)
        response = response_for(request)
    except (json.JSONDecodeError, TypeError, ValueError) as error:
        json.dump({"error": str(error)}, sys.stdout)
        sys.stdout.write("\n")
        return 1

    json.dump(response, sys.stdout)
    sys.stdout.write("\n")
    return 0


def css_colors() -> dict[str, str]:
    """Read the shared plugin theme when the script is run from the repository."""
    stylesheet = Path(__file__).resolve().parents[1] / "public" / "plugin.css"
    try:
        source = stylesheet.read_text(encoding="utf-8")
    except OSError:
        return {}
    return dict(re.findall(r"--([\w-]+):\s*(#[0-9a-fA-F]{6});", source))


def run_tk_gui() -> int:
    import tkinter as tk
    from tkinter import ttk

    colors = css_colors()
    background = colors.get("app-background", "#0f172a")
    surface = colors.get("surface-background", "#1e293b")
    foreground = colors.get("primary-text", "#f8fafc")
    secondary = colors.get("secondary-text", "#e2e8f0")
    border = colors.get("control-border", "#7b8794")
    control = colors.get("control-background", "#334155")
    accent = colors.get("primary-accent", "#2563eb")

    window = tk.Tk()
    window.title("TTRPG Engine - Hello World")
    window.geometry("480x260")
    window.minsize(360, 200)
    window.configure(background=background)

    style = ttk.Style(window)
    style.theme_use("clam")
    style.configure("Plugin.TFrame", background=surface)
    style.configure("Plugin.TLabel", background=surface, foreground=foreground, font=("TkDefaultFont", 11))
    style.configure("Plugin.Title.TLabel", background=surface, foreground=foreground, font=("TkDefaultFont", 18, "bold"))
    style.configure(
        "Plugin.TButton",
        background=control,
        foreground=secondary,
        bordercolor=border,
        padding=(16, 9),
    )
    style.map("Plugin.TButton", background=[("active", accent)])

    outer = ttk.Frame(window, style="Plugin.TFrame", padding=2)
    outer.pack(fill="both", expand=True, padx=18, pady=18)
    content = ttk.Frame(outer, style="Plugin.TFrame", padding=24)
    content.pack(fill="both", expand=True)
    ttk.Label(content, text="Hello World", style="Plugin.Title.TLabel").pack(pady=(16, 10))
    ttk.Label(
        content,
        text="This is the standard TTRPG Engine plugin template.",
        style="Plugin.TLabel",
        justify="center",
    ).pack(pady=(0, 24))
    ttk.Button(content, text="Close", style="Plugin.TButton", command=window.destroy).pack()
    window.mainloop()
    return 0


def run_webview_gui() -> int:
    """Use the shared HTML/CSS stack when optional pywebview is installed."""
    import webview  # type: ignore[import-not-found]

    stylesheet = (Path(__file__).resolve().parents[1] / "public" / "plugin.css").as_uri()
    html = f"""<!doctype html>
<html><head><meta charset="utf-8"><title>Hello World</title>
<link rel="stylesheet" href="{stylesheet}">
<style>
html, body {{ min-height: 100%; }}
body {{ display: grid; place-items: center; }}
</style></head>
<body><main class="plugin-window">
<header class="plugin-window__header"><h1>Hello World</h1></header>
<section class="plugin-window__content">
<p>This is the standard TTRPG Engine plugin template.</p>
</section>
<footer class="plugin-window__footer">
<button onclick="window.pywebview.api.close()">Close</button>
</footer>
</main></body></html>"""

    class Api:
        def close(self) -> None:
            webview.windows[0].destroy()

    webview.create_window("TTRPG Engine - Hello World", html=html, js_api=Api(), width=480, height=260)
    webview.start()
    return 0


def run_gui() -> int:
    try:
        import webview  # type: ignore[import-not-found]  # noqa: F401
    except ImportError:
        return run_tk_gui()
    return run_webview_gui()


def main() -> int:
    arguments = parser().parse_args()
    if arguments.inputlist:
        json.dump(INPUT_LIST, sys.stdout)
        sys.stdout.write("\n")
        return 0
    return run_protocol() if arguments.json else run_gui()


if __name__ == "__main__":
    raise SystemExit(main())
