#!/usr/bin/env python3
"""Sponsor negotiation plugin with a mandatory Tk GUI and JSON result output.

Normal execution always opens the Tk GUI. The GUI prints one final JSON result
to stdout when the player completes the negotiation. An explicit --json mode
is retained for automated protocol tests and non-UI harnesses.
"""

from __future__ import annotations

import argparse
import json
import random
import sys
from dataclasses import dataclass
from typing import Any


SUCCESS_STATUSES = {"SIGNED"}


@dataclass(frozen=True)
class Sponsor:
    name: str
    skepticism: int
    base_pay: int
    preferred: str
    opening: str


SPONSORS = (
    Sponsor(
        "MegaCorp Racing",
        58,
        50_000,
        "logic",
        "Your crash rate is a bit high. Why should we risk our brand?",
    ),
    Sponsor(
        "Apex Dynamics",
        48,
        38_000,
        "charm",
        "We like your pace, but can you make fans care about our product?",
    ),
    Sponsor(
        "Northstar Tools",
        66,
        42_000,
        "aggressive",
        "We need a driver who will fight for every position, not make excuses.",
    ),
)


def positive_int(value: str) -> int:
    parsed = int(value)
    if parsed < 0:
        raise argparse.ArgumentTypeError("value must be non-negative")
    return parsed


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run the sponsor negotiation JSON plugin or its optional Tk GUI."
    )
    parser.add_argument("--exp", type=positive_int, default=0, help="overall experience points")
    parser.add_argument(
        "--results", type=positive_int, default=50, help="recent performance score (0-100)"
    )
    parser.add_argument("--charisma", type=positive_int, default=1, help="current charisma")
    parser.add_argument(
        "--has-agent",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="whether the player employs an agent",
    )
    parser.add_argument("--agent-level", type=positive_int, default=0, help="agent level (0-10)")
    parser.add_argument("--seed", type=int, default=None, help="optional deterministic RNG seed")
    parser.add_argument("--sponsor", help="optional sponsor name for deterministic selection")
    parser.add_argument(
        "--gui",
        action="store_true",
        help="compatibility flag; the GUI is always launched by default",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="run the line-delimited JSON protocol instead of the mandatory GUI",
    )
    return parser


def clamp(value: int, minimum: int, maximum: int) -> int:
    return max(minimum, min(maximum, value))


class Negotiation:
    def __init__(self, args: argparse.Namespace):
        self.rng = random.Random(args.seed)
        self.exp = args.exp
        self.results = clamp(args.results, 0, 100)
        self.charisma = args.charisma
        self.has_agent = args.has_agent
        self.agent_level = clamp(args.agent_level, 0, 10)
        self.attraction = self.results * 0.55 + min(self.exp, 500) * 0.08
        self.attraction += self.agent_level * 8 + (8 if self.has_agent else 0)
        if self.charisma > 0:
            self.attraction += min(self.charisma, 20) * 1.5
        self.cold_call = self.attraction < 45
        self.sponsor = self._select_sponsor(args.sponsor)
        self.leverage = 100
        self.skepticism = self.sponsor.skepticism + (10 if self.cold_call else 0)
        self.round = 0
        self.finished = False
        self.last_log = (
            "No sponsors approached. You make the first call."
            if self.cold_call
            else "Sponsors noticed your recent results and requested a meeting."
        )

    def _select_sponsor(self, requested: str | None) -> Sponsor:
        if requested:
            for sponsor in SPONSORS:
                if sponsor.name.casefold() == requested.casefold():
                    return sponsor
            raise ValueError(f"unknown sponsor: {requested}")
        weights = [max(1, int(self.attraction / 20) + index) for index, _ in enumerate(SPONSORS)]
        return self.rng.choices(SPONSORS, weights=weights, k=1)[0]

    def _options(self) -> list[dict[str, str]]:
        return [
            {
                "id": "aggressive_pitch",
                "text": "[Aggressive] Promise an unforgettable campaign and demand their confidence.",
            },
            {
                "id": "charm",
                "text": "[Charm] Connect the sponsor's brand to the fans and your personality.",
            },
            {
                "id": "logic_rebuttal",
                "text": "[Logic] Answer the objection with results, experience, and a concrete plan.",
            },
        ]

    def _ongoing(self, log: str | None = None) -> dict[str, Any]:
        if log is not None:
            self.last_log = log
        return {
            "status": "ONGOING",
            "sponsor_name": self.sponsor.name,
            "sponsor_dialogue": self._dialogue(),
            "player_leverage": self.leverage,
            "sponsor_skepticism": self.skepticism,
            "player_options": self._options(),
            "last_action_log": self.last_log,
            "round": self.round,
            "attraction_score": round(self.attraction, 2),
            "cold_call": self.cold_call,
        }

    def _dialogue(self) -> str:
        if self.round == 0:
            return self.sponsor.opening
        objections = (
            "Your plan sounds bold. What happens when the pressure rises?",
            "The numbers are promising, but our board wants a memorable story.",
            "One mistake can damage our reputation. Why should we trust you?",
            "We need proof that this partnership will outlast one good result.",
        )
        return objections[(self.round - 1) % len(objections)]

    def initial(self) -> dict[str, Any]:
        return self._ongoing()

    def _resolution(self, status: str, log: str, multiplier: float = 1.0) -> dict[str, Any]:
        self.finished = True
        response: dict[str, Any] = {
            "status": status,
            "sponsor_name": self.sponsor.name,
            "base_pay": self.sponsor.base_pay if status in SUCCESS_STATUSES else 0,
            "bonus_multiplier": round(multiplier, 2) if status in SUCCESS_STATUSES else 0.0,
            "stat_changes": {"charisma": self.charisma - 1}
            if status == "BANNED"
            else {},
            "final_log": log,
        }
        return response

    def apply(self, action_id: str) -> dict[str, Any]:
        if self.finished:
            return {"status": "ERROR", "error": "negotiation has already ended"}
        valid = {option["id"] for option in self._options()}
        if action_id not in valid:
            return {"status": "ERROR", "error": f"unknown action_id: {action_id}"}
        self.round += 1
        roll = self.rng.randint(1, 20)
        if roll == 1:
            self.leverage = 0
            return self._resolution(
                "BANNED",
                "Your argument collapsed under scrutiny. The sponsor ended the meeting.",
            )
        if roll == 20:
            return self._resolution(
                "SIGNED",
                "A perfect pitch closed the deal immediately.",
                1.5,
            )

        if action_id == "aggressive_pitch":
            damage = 15 + self.agent_level + (5 if self.results >= 70 else 0)
            risk = 7 if self.sponsor.preferred != "aggressive" else 3
            self.skepticism -= damage
            self.leverage -= risk
            log = f"You applied pressure. Skepticism fell by {damage}, but you risked {risk} leverage."
        elif action_id == "charm":
            damage = 8 + min(self.charisma, 12) // 2
            self.skepticism -= damage
            self.leverage -= 2 if self.has_agent else 4
            log = f"You made a personal connection. Skepticism fell by {damage}."
        else:
            damage = 17 if self.sponsor.preferred == "logic" else 10
            damage += min(self.results // 25, 4)
            self.skepticism -= damage
            self.leverage -= 1
            log = f"You answered the objection with evidence. Skepticism fell by {damage}."

        self.skepticism = max(0, self.skepticism)
        self.leverage = max(0, self.leverage)
        if self.skepticism <= 0:
            multiplier = 1.0 + min(self.attraction / 500, 0.5)
            return self._resolution("SIGNED", "The sponsor signed the contract.", multiplier)
        if self.leverage <= 0:
            return self._resolution("REJECTED", "You ran out of leverage before reaching an agreement.")
        if self.round >= 8:
            return self._resolution("REJECTED", "The meeting ran out of time without an agreement.")
        return self._ongoing(log)


def run_protocol(args: argparse.Namespace) -> int:
    try:
        negotiation = Negotiation(args)
    except ValueError as error:
        json.dump({"status": "ERROR", "error": str(error)}, sys.stdout)
        sys.stdout.write("\n")
        return 2

    print(json.dumps(negotiation.initial(), separators=(",", ":")), flush=True)
    while True:
        line = sys.stdin.readline()
        if not line:
            return 0
        try:
            request = json.loads(line)
            if not isinstance(request, dict):
                raise ValueError("request must be a JSON object")
            action_id = request.get("action_id")
            if not isinstance(action_id, str) or not action_id.strip():
                raise ValueError("request requires a non-empty action_id")
            response = negotiation.apply(action_id.strip())
        except (json.JSONDecodeError, TypeError, ValueError) as error:
            response = {"status": "ERROR", "error": str(error)}
        print(json.dumps(response, separators=(",", ":")), flush=True)
        if response.get("status") in {"SIGNED", "BANNED", "REJECTED"}:
            return 0


def run_gui(args: argparse.Namespace) -> int:
    try:
        import tkinter as tk
        from tkinter import messagebox
    except ImportError as error:
        print(f"Tkinter is unavailable: {error}", file=sys.stderr)
        return 2

    negotiation = Negotiation(args)
    window = tk.Tk()
    window.title("Sponsor Negotiation")
    window.configure(bg="#102a43")
    frame = tk.Frame(window, bg="#102a43", padx=20, pady=20)
    frame.pack(fill="both", expand=True)
    title = tk.Label(
        frame, text="SPONSOR NEGOTIATION", bg="#102a43", fg="#d9f0ff",
        font=("TkDefaultFont", 15, "bold"),
    )
    title.pack(anchor="w")
    sponsor_label = tk.Label(frame, bg="#102a43", fg="white", font=("TkDefaultFont", 11, "bold"))
    sponsor_label.pack(anchor="w", pady=(8, 2))
    dialogue = tk.Message(frame, width=560, bg="#e6f4ff", fg="#102a43", padx=10, pady=10)
    dialogue.pack(fill="x", pady=(0, 10))
    stats = tk.Label(frame, bg="#102a43", fg="#d9f0ff", justify="left")
    stats.pack(anchor="w")
    log = tk.Label(frame, bg="#102a43", fg="#b9d6ea", justify="left", wraplength=560)
    log.pack(anchor="w", pady=(8, 12))
    buttons = tk.Frame(frame, bg="#102a43")
    buttons.pack(fill="x")

    def render(state: dict[str, Any]) -> None:
        sponsor_label.configure(text=state.get("sponsor_name", negotiation.sponsor.name))
        if state.get("status") == "ONGOING":
            dialogue.configure(text=state["sponsor_dialogue"])
            stats.configure(
                text=f"Leverage: {state['player_leverage']}    "
                f"Skepticism: {state['sponsor_skepticism']}    Round: {state['round']}"
            )
            log.configure(text=state["last_action_log"])
            for child in buttons.winfo_children():
                child.destroy()
            for option in state["player_options"]:
                tk.Button(
                    buttons, text=option["text"], anchor="w", justify="left",
                    command=lambda action=option["id"]: render(negotiation.apply(action)),
                ).pack(fill="x", pady=2)
        else:
            print(json.dumps(state, separators=(",", ":")), flush=True)
            dialogue.configure(text=state.get("final_log", state.get("error", "Negotiation ended")))
            stats.configure(text=f"Status: {state.get('status', 'ERROR')}")
            log.configure(text=f"Base pay: {state.get('base_pay', 0)}")
            for child in buttons.winfo_children():
                child.destroy()
            tk.Button(buttons, text="Close", command=window.destroy).pack(anchor="e")
            if state.get("status") == "ERROR":
                messagebox.showerror("Sponsor negotiation", state.get("error", "Unknown error"))

    render(negotiation.initial())
    window.mainloop()
    return 0


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.json:
        return run_protocol(args)
    else:
        return run_gui(args)


if __name__ == "__main__":
    raise SystemExit(main())
