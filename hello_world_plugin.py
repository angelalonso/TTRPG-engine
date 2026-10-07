#!/usr/bin/env python3
"""Minimal sponsor-style Hello World plugin template.

Without arguments this opens a small themed browser window. ``--json`` exposes
the same start/return flow as a host-integrated modal plugin.
"""

from __future__ import annotations

import argparse
import json
import sys
import threading
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any


PLUGIN_ID = "hello_world"
PROTOCOL_VERSION = 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Hello World modal plugin template.")
    parser.add_argument(
        "--json",
        action="store_true",
        help="Use the line-oriented JSON protocol instead of opening a browser window.",
    )
    return parser.parse_args()


def state(status: str = "OPEN") -> dict[str, Any]:
    return {
        "protocol_version": PROTOCOL_VERSION,
        "plugin_id": PLUGIN_ID,
        "status": status,
        "title": "Hello World",
        "message": "Hello World",
    }


def run_json() -> int:
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            request = json.loads(line)
            if request.get("protocol_version") != PROTOCOL_VERSION:
                raise ValueError("unsupported protocol version")
            if request.get("plugin_id") != PLUGIN_ID:
                raise ValueError("request belongs to a different plugin")
            action = request.get("action", "open")
            response = state("CLOSED" if action in {"return", "close"} else "OPEN")
        except (json.JSONDecodeError, TypeError, ValueError) as error:
            response = {"status": "ERROR", "error": str(error)}
        print(json.dumps(response), flush=True)
        if response.get("status") == "CLOSED":
            return 0
    return 0


def page() -> str:
    return """<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>TTRPG Engine - Hello World</title>
  <style>
    :root {
      color-scheme: dark;
      background: #0f1114;
      color: #f3f4f5;
      font: 16px system-ui, sans-serif;
    }
    body { margin: 0; min-height: 100vh; display: grid; place-items: center; }
    main {
      width: min(640px, calc(100vw - 2rem));
      background: #111316;
      border: 1px solid #737b82;
      box-shadow: 0 24px 60px rgb(0 0 0 / 55%);
    }
    header { padding: 1.25rem; border-bottom: 1px solid #e5232b; }
    .eyebrow { color: #aeb4b9; font-size: .8rem; letter-spacing: .14em; }
    h1 { margin: .3rem 0 0; }
    section { margin: 1rem; padding: 1.25rem; border: 1px solid #3b4045; background: #1c2024; }
    footer { display: flex; justify-content: flex-end; padding: 0 1rem 1rem; }
    button {
      border: 1px solid #737b82; padding: .65rem 1rem; color: #f3f4f5;
      background: #30353a; cursor: pointer;
    }
    button:hover { background: #e5232b; }
  </style>
</head>
<body>
  <main>
    <header>
      <div class="eyebrow">TTRPG ENGINE / PLUGIN SERVICES</div>
      <h1>Hello World</h1>
    </header>
    <section><strong>Hello World</strong></section>
    <footer><button type="button" onclick="returnToGame()">Return</button></footer>
  </main>
  <script>
    async function returnToGame() {
      await fetch('/return', { method: 'POST' });
      document.querySelector('main').innerHTML = '<section>Returning to the game...</section>';
    }
  </script>
</body>
</html>"""


def run_browser() -> int:
    returned = threading.Event()

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:
            if self.path != "/":
                self.send_error(404)
                return
            content = page().encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(content)))
            self.end_headers()
            self.wfile.write(content)

        def do_POST(self) -> None:
            if self.path != "/return":
                self.send_error(404)
                return
            returned.set()
            content = b'{"status":"CLOSED"}'
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(content)))
            self.end_headers()
            self.wfile.write(content)

        def log_message(self, _format: str, *_args: Any) -> None:
            return

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    url = f"http://127.0.0.1:{server.server_port}/"
    if not webbrowser.open_new(url):
        server.server_close()
        raise RuntimeError("could not open the Hello World plugin window")
    try:
        while not returned.is_set():
            server.handle_request()
    finally:
        server.server_close()
    return 0


def main() -> int:
    return run_json() if parse_args().json else run_browser()


if __name__ == "__main__":
    raise SystemExit(main())
