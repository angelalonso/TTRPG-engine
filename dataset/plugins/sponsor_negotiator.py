#!/usr/bin/env python3
"""Data-driven sponsor matchmaking and negotiation plugin.

The plugin has two interfaces:

* ``--json``: line-delimited JSON for the game engine.
* default execution: a small themed window for standalone testing.

The JSON interface keeps the existing ``action_id`` actions for compatibility,
and adds proposal-oriented actions for sponsor offers and cold calls.
"""

from __future__ import annotations

import argparse
import csv
import json
import logging
import random
import sys
from dataclasses import dataclass
from datetime import date, timedelta
from pathlib import Path
from typing import Any


LOGGER = logging.getLogger("sponsor_negotiator")


TIERS = ("local", "regional", "national", "continental", "world")
NO_CAR_LABEL = "No car included"
TIER_LEVEL = {tier: index for index, tier in enumerate(TIERS)}
SUCCESS_STATUSES = {"SIGNED"}
LEGACY_ACTIONS = {"aggressive_pitch", "charm", "logic_rebuttal"}
TIER_MULTIPLIER = {
    "local": 1.0,
    "regional": 2.5,
    "national": 5.0,
    "continental": 10.0,
    "world": 25.0,
}


@dataclass(frozen=True)
class Sponsor:
    id: str
    name: str
    tier: str
    interested_tiers: tuple[str, ...]
    preferred_categories: tuple[str, ...]
    base_cash: int
    monthly_payment: int
    repair_value: int
    brand: str

    @classmethod
    def from_row(cls, row: dict[str, str]) -> "Sponsor":
        interests = tuple(
            value.strip().lower()
            for value in row.get("interested_race_tiers", "").split(";")
            if value.strip().lower() in TIER_LEVEL
        )
        if not interests:
            interests = (row.get("tier", "local").strip().lower(),)
        return cls(
            id=row["id"].strip(),
            name=row["name"].strip(),
            tier=row.get("tier", "local").strip().lower(),
            interested_tiers=interests,
            preferred_categories=tuple(
                value.strip().lower()
                for value in row.get("preferred_categories", "").split(";")
                if value.strip()
            ),
            base_cash=int(float(row.get("base_cash", "0") or 0)),
            monthly_payment=int(float(row.get("monthly_payment", "0") or 0)),
            repair_value=int(float(row.get("repair_value", "0") or 0)),
            brand=row.get("brand", "").strip(),
        )


DEFAULT_SPONSORS = (
    Sponsor(
        "local_repairmen",
        "Local Repairmen Group",
        "local",
        ("local", "regional", "national"),
        ("race", "championship", "quest", "year"),
        2500,
        250,
        3000,
        "vehicle maintenance",
    ),
    Sponsor(
        "local_restaurant",
        "The Trackside Restaurant",
        "local",
        ("local", "regional", "national"),
        ("race", "championship", "quest", "year"),
        1800,
        180,
        0,
        "hospitality",
    ),
    Sponsor(
        "local_pub",
        "The Racing Pub",
        "local",
        ("local", "regional", "national"),
        ("race", "championship", "quest", "year"),
        1400,
        140,
        0,
        "hospitality",
    ),
)


def load_sponsors(dataset_path: str | None) -> tuple[Sponsor, ...]:
    if dataset_path:
        path = Path(dataset_path) / "sponsors.csv"
        if path.is_file():
            with path.open(newline="", encoding="utf-8") as handle:
                rows = tuple(Sponsor.from_row(row) for row in csv.DictReader(handle))
            if rows:
                return rows
    return DEFAULT_SPONSORS


def positive_int(value: str) -> int:
    parsed = int(value)
    if parsed < 0:
        raise argparse.ArgumentTypeError("value must be non-negative")
    return parsed


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run sponsor matchmaking and negotiation.")
    parser.add_argument("--exp", type=positive_int, default=0)
    parser.add_argument("--results", type=positive_int, default=50)
    parser.add_argument("--charisma", type=positive_int, default=1)
    parser.add_argument("--has-agent", action=argparse.BooleanOptionalAction, default=False)
    parser.add_argument("--agent-level", type=positive_int, default=0)
    parser.add_argument("--podiums", type=positive_int, default=0)
    parser.add_argument("--wins", type=positive_int, default=0)
    parser.add_argument("--poles", "--pole-positions", dest="poles", type=positive_int, default=0)
    parser.add_argument("--championships-won", type=positive_int, default=0)
    parser.add_argument("--race-tier", choices=TIERS, default="local")
    parser.add_argument("--scope", choices=("race", "quest", "championship", "year"), default="race")
    parser.add_argument("--money-request", type=positive_int, default=None)
    parser.add_argument("--dataset-path", default=None)
    parser.add_argument("--seed", type=int, default=None)
    parser.add_argument("--sponsor", help="optional sponsor id or name")
    parser.add_argument("--target-id", default="")
    parser.add_argument("--target-name", default="")
    parser.add_argument("--approach", choices=("cold_call", "proposal"), default="cold_call")
    parser.add_argument("--result-file", default=None)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--interactive", action="store_true")
    parser.add_argument("--help-effects", action="store_true")
    return parser


def clamp(value: float, minimum: float, maximum: float) -> float:
    return max(minimum, min(maximum, value))


class Negotiation:
    def __init__(self, args: argparse.Namespace):
        self.rng = random.Random(args.seed)
        self.args = args
        self.sponsors = load_sponsors(args.dataset_path)
        self.results = clamp(args.results, 0, 100)
        self.charisma = args.charisma
        self.manager_level = clamp(args.agent_level if args.has_agent else 0, 0, 10)
        self.has_manager = bool(args.has_agent and self.manager_level > 0)
        self.race_tier = args.race_tier
        self.scope = args.scope
        self.attraction = self._attraction()
        self.available = self._available_sponsors()
        self.cold_call = args.approach == "cold_call"
        self.sponsor = self._select_sponsor(args.sponsor)
        self.ideal = self._ideal_terms(self.sponsor)
        self.benchmark_value = self._benchmark_value(self.sponsor)
        self.sponsor_reservation = self.benchmark_value * (1 + min(0.25, 0.08 + self.manager_level * 0.01))
        self.player_reservation = self.benchmark_value * (1 - min(0.25, 0.08 + self.attraction / 1000))
        self.sponsor_offer_value = self.benchmark_value * (0.78 if self.cold_call else 0.98)
        self.player_request_value = self.benchmark_value * (1.05 if self.cold_call else 1.15)
        self.current_terms = self._terms_at_value(
            self.player_request_value if self.cold_call else self.sponsor_offer_value
        )
        self.round = 0
        self.finished = False
        self.objection = "Your recent results must justify this package."
        self.last_log = (
            "You are calling the sponsor without an existing proposal."
            if self.cold_call
            else "The sponsor has approached you with an opening proposal."
        )

        # Compatibility state for the original action buttons.
        self.leverage = 100
        self.skepticism = 50 + (10 if self.cold_call else 0)
        self.max_skepticism = self.skepticism

    def _attraction(self) -> float:
        achievement = (
            self.args.poles * 1.0
            + self.args.podiums * 1.8
            + self.args.wins * 3.5
            + self.args.championships_won * 10.0
        )
        return (
            self.results * 0.55
            + min(self.args.exp, 500) * 0.08
            + achievement * TIER_MULTIPLIER[self.race_tier]
            + self.manager_level * 8
            + (8 if self.has_manager else 0)
            + min(self.charisma, 20) * 1.5
        )

    def _available_sponsors(self) -> list[Sponsor]:
        if not self.has_manager:
            max_tier = 0
        else:
            max_tier = min(4, max(0, int(self.manager_level / 2)))
            if self.attraction >= 120:
                max_tier = min(4, max_tier + 1)
        candidates = [
            sponsor
            for sponsor in self.sponsors
            if TIER_LEVEL.get(sponsor.tier, 0) <= max_tier
            and self.race_tier in sponsor.interested_tiers
        ]
        return candidates or [
            sponsor for sponsor in self.sponsors if self.race_tier in sponsor.interested_tiers
        ]

    def _benchmark_value(self, sponsor: Sponsor) -> float:
        scope_factor = {"race": 1, "quest": 4, "championship": 8, "year": 12}[self.scope]
        achievement = (
            self.args.poles * 1.0
            + self.args.podiums * 1.8
            + self.args.wins * 3.5
            + self.args.championships_won * 10.0
        )
        prestige = max(1.0, 1 + achievement * TIER_MULTIPLIER[self.race_tier] / 100)
        return max(1.0, sponsor.base_cash * scope_factor * prestige)

    def _racer_value(self) -> float:
        achievement = (
            self.args.poles * 1.0
            + self.args.podiums * 1.8
            + self.args.wins * 3.5
            + self.args.championships_won * 10.0
        )
        return achievement * TIER_MULTIPLIER[self.race_tier]

    def _select_sponsor(self, requested: str | None) -> Sponsor:
        if requested:
            for sponsor in self.sponsors:
                if requested.casefold() in {sponsor.id.casefold(), sponsor.name.casefold()}:
                    return sponsor
            raise ValueError(f"unknown sponsor: {requested}")
        if not self.available:
            raise ValueError(f"no sponsors are interested in {self.race_tier} races")
        return self.rng.choice(self.available)

    def _ideal_terms(self, sponsor: Sponsor) -> dict[str, Any]:
        scope_factor = {"race": 1, "quest": 4, "championship": 8, "year": 12}[self.scope]
        tier_factor = TIER_LEVEL[sponsor.tier] + 1
        performance_factor = 1 + self.attraction / 100
        cash = round(sponsor.base_cash * scope_factor * performance_factor / tier_factor)
        monthly = round(sponsor.monthly_payment * performance_factor) if self.scope == "year" else 0
        request = self.args.money_request if self.args.money_request is not None else cash
        return {
            "scope": self.scope,
            "race_tier": self.race_tier,
            "initial_money": max(0, request),
            "monthly_payment": monthly,
            "maintenance": sponsor.repair_value > 0,
            "entry_fees": self.scope in {"championship", "year"},
            "gear": sponsor.tier != "local" or self.scope != "race",
            "car": False,
            "result_bonus": round(max(100, cash * 0.1)),
            "dnf_penalty": round(max(0, cash * 0.04)),
        }

    def _terms_at_value(self, value: float) -> dict[str, Any]:
        terms = dict(self.ideal)
        target = max(0.0, value)
        fixed_terms = {
            "initial_money": 0,
            "monthly_payment": 0,
            "result_bonus": 0,
            "dnf_penalty": 0,
        }
        fixed_terms.update({key: terms[key] for key in ("scope", "race_tier", "car", "entry_fees", "gear", "maintenance")})
        optional_value = (
            ("car", 12000),
            ("maintenance", 5000),
            ("gear", 3000),
            ("entry_fees", 2500),
        )
        for key, contribution in optional_value:
            if fixed_terms[key] and self._proposal_value(fixed_terms) > target:
                fixed_terms[key] = False
        fixed_value = self._proposal_value(fixed_terms)
        numeric_value = self._proposal_value(self.ideal) - self._proposal_value({
            **self.ideal,
            "initial_money": 0,
            "monthly_payment": 0,
            "result_bonus": 0,
            "dnf_penalty": 0,
        })
        ratio = max(0.0, (target - fixed_value) / max(1.0, numeric_value))
        terms.update(fixed_terms)
        for key in ("initial_money", "monthly_payment", "result_bonus", "dnf_penalty"):
            terms[key] = round(self.ideal[key] * ratio)
        return terms

    @staticmethod
    def _adjust_terms(terms: dict[str, Any], percentage: float) -> dict[str, Any]:
        adjusted = dict(terms)
        for key in ("initial_money", "monthly_payment", "result_bonus", "dnf_penalty"):
            adjusted[key] = round(adjusted[key] * (1 + percentage))
        return adjusted

    def _proposal_value(self, terms: dict[str, Any]) -> float:
        value = terms["initial_money"] + terms["monthly_payment"] * 12
        value += terms["result_bonus"] * 3
        value += 3000 if terms["gear"] else 0
        value += 12000 if terms["car"] else 0
        value += 5000 if terms["maintenance"] else 0
        value += 2500 if terms["entry_fees"] else 0
        value -= terms["dnf_penalty"] * 2
        return float(value)

    def _acceptance_ratio(self) -> float:
        return self._proposal_value(self.current_terms) / max(1.0, self.benchmark_value)

    def _expires_on(self) -> str | None:
        if self.cold_call:
            return None
        return (date.today() + timedelta(days=14)).isoformat()

    def _sponsor_rows(self) -> list[dict[str, Any]]:
        return [
            {
                "id": sponsor.id,
                "name": sponsor.name,
                "tier": sponsor.tier,
                "interested_race_tiers": list(sponsor.interested_tiers),
                "proposal_available": sponsor in self.available,
                "can_cold_call": self.race_tier in sponsor.interested_tiers,
                "brand": sponsor.brand,
                "opening_offer": round(self._benchmark_value(sponsor) * 0.9),
            }
            for sponsor in self.sponsors
        ]

    def _options(self) -> list[dict[str, str]]:
        return [
            {"id": "aggressive_pitch", "text": "[Aggressive] Push for a larger package."},
            {"id": "charm", "text": "[Charm] Sell the partnership and its audience value."},
            {"id": "logic_rebuttal", "text": "[Logic] Match the package to results and race tier."},
            {"id": "counter_proposal", "text": "[Counter] Submit the current package for review."},
        ]

    def initial(self) -> dict[str, Any]:
        return {
            "status": "ONGOING",
            "phase": "matchmaking",
            "sponsor_name": self.sponsor.name,
            "sponsor_id": self.sponsor.id,
            "sponsor_tier": self.sponsor.tier,
            "sponsor_brand": self.sponsor.brand,
            "race_tier": self.race_tier,
            "scope": self.scope,
            "attraction_score": round(self.attraction, 2),
            "racer_value": round(self._racer_value(), 2),
            "median_benchmark": round(self.benchmark_value, 2),
            "sponsor_walkaway": round(self.sponsor_reservation, 2),
            "player_walkaway": round(self.player_reservation, 2),
            "sponsor_offer_value": round(self.sponsor_offer_value, 2),
            "player_request_value": round(self.player_request_value, 2),
            "zopa_open": self.sponsor_offer_value >= self.player_request_value,
            "approaching_sponsors": [
                sponsor.id for sponsor in self.available[:3]
            ] if self.has_manager else [],
            "manager_level": int(self.manager_level),
            "cold_call": self.cold_call,
            "proposal_expires": self._expires_on(),
            "sponsors": self._sponsor_rows(),
            "proposal": self.current_terms,
            "ideal_proposal": self.ideal,
            "sponsor_dialogue": self.last_log,
            "player_options": self._options(),
            "player_leverage": self.leverage,
            "sponsor_skepticism": self.skepticism,
            "player_leverage_max": 100,
            "sponsor_skepticism_max": self.max_skepticism,
            "last_action_log": self.last_log,
            "objection": self.objection,
            "round": self.round,
        }

    def _resolution(
        self,
        status: str,
        log: str,
        multiplier: float = 0,
        stat_changes: dict[str, int] | None = None,
    ) -> dict[str, Any]:
        self.finished = True
        response = {
            "status": status,
            "sponsor_id": self.sponsor.id,
            "sponsor_name": self.sponsor.name,
            "sponsor_tier": self.sponsor.tier,
            "scope": self.scope,
            "repair_value": self.sponsor.repair_value,
            "proposal": self.current_terms if status == "SIGNED" else {},
            "base_pay": self.current_terms["initial_money"] if status == "SIGNED" else 0,
            "bonus_multiplier": round(multiplier, 2),
            "stat_changes": stat_changes or ({"charisma": -1} if status == "BANNED" else {}),
            "final_log": log,
        }
        if status == "SIGNED":
            response["agreement"] = {
                "sponsor_id": self.sponsor.id,
                "sponsor_name": self.sponsor.name,
                "sponsor_tier": self.sponsor.tier,
                "scope": self.scope,
                "race_tier": self.race_tier,
                "proposal": self.current_terms,
            }
        return response

    def _ongoing(self, log: str) -> dict[str, Any]:
        self.last_log = log
        state = self.initial()
        state["phase"] = "negotiation"
        return state

    def _legacy_apply(self, action_id: str) -> dict[str, Any]:
        self.round += 1
        roll = self.rng.randint(1, 20)
        if roll == 1:
            return self._resolution(
                "BANNED",
                "A critical misunderstanding ended the negotiation.",
                0,
                {"charisma": -1},
            )
        if roll == 20:
            self.current_terms = self._terms_at_value(self.sponsor_reservation)
            return self._resolution("SIGNED", "A perfect pitch closed the deal at the sponsor's maximum package.", 1.5)
        step = max(1.0, self.benchmark_value * 0.10 * (0.75 ** self.round))
        if action_id == "aggressive_pitch":
            self.player_request_value += step * 0.35
            self.leverage = max(0, self.leverage - (7 if self.sponsor.tier != "local" else 3))
            log = "You pushed hard for a larger package, but spent some leverage."
        elif action_id == "charm":
            self.player_request_value -= step * (0.45 + min(self.charisma, 10) / 40)
            log = "Your presentation improved the sponsor's confidence."
        else:
            self.player_request_value -= step * (0.65 if self.results >= 60 else 0.35)
            log = "You tied the package to measurable results and race level."
        luck = self.rng.gauss(0, self.benchmark_value * 0.015)
        sponsor_step = max(1.0, step * (0.9 + self.manager_level * 0.03)) + luck
        self.sponsor_offer_value = min(
            self.sponsor_reservation,
            self.sponsor_offer_value + max(0.0, sponsor_step),
        )
        self.current_terms = self._terms_at_value(
            min(self.sponsor_offer_value, self.player_request_value)
        )
        if self.sponsor_offer_value >= self.player_request_value:
            final_value = self.player_request_value + 0.5 * (self.sponsor_offer_value - self.player_request_value)
            self.current_terms = self._terms_at_value(final_value)
            return self._resolution("SIGNED", "The negotiation reached a fair agreement.", 1.0)
        if self.leverage == 0 or self.round >= 8:
            return self._resolution("REJECTED", "The meeting ended without an agreement.")
        self.objection = (
            "The sponsor still questions whether the results justify the requested package."
        )
        return self._ongoing(log)

    def apply(self, request: dict[str, Any]) -> dict[str, Any]:
        if self.finished:
            return {"status": "ERROR", "error": "negotiation has already ended"}
        action_id = request.get("action_id")
        if action_id == "new_session":
            scope = request.get("scope")
            if scope not in {"race", "championship", "year"}:
                return {"status": "ERROR", "error": "invalid sponsorship period"}
            values = vars(self.args).copy()
            values["scope"] = scope
            self.__init__(argparse.Namespace(**values))
            return self.initial()
        if action_id in {"select_target", "select_car"}:
            return self.initial()
        if action_id in LEGACY_ACTIONS:
            return self._legacy_apply(action_id)
        if action_id == "counter_proposal":
            proposal = request.get("proposal")
            if not isinstance(proposal, dict):
                return {"status": "ERROR", "error": "counter_proposal requires proposal"}
            self.current_terms.update({key: value for key, value in proposal.items() if key in self.ideal})
            if self.scope != "year":
                self.current_terms["monthly_payment"] = 0
            self.round += 1
            self.player_request_value = self._proposal_value(self.current_terms)
            if self.player_request_value > self.sponsor_reservation * 1.02:
                return self._resolution("REJECTED", "The sponsor rejected a package beyond its walkaway limit.")
            if self.sponsor_offer_value >= self.player_request_value:
                return self._resolution("SIGNED", "Both sides accepted the negotiated package.", 1.0)
            concession = max(1.0, self.benchmark_value * 0.10 * (0.75 ** self.round))
            self.sponsor_offer_value = min(
                self.sponsor_reservation,
                self.sponsor_offer_value + concession + self.rng.gauss(0, self.benchmark_value * 0.015),
            )
            self.current_terms = self._terms_at_value(self.sponsor_offer_value)
            if self.sponsor_offer_value >= self.player_request_value:
                return self._resolution("SIGNED", "The sponsor's counteroffer entered the agreement zone.", 1.0)
            if self.round >= 6:
                return self._resolution("REJECTED", "The negotiation reached its limit without agreement.")
            return self._ongoing("The sponsor countered with a revised package.")
        if action_id == "accept_proposal":
            self.current_terms = self._terms_at_value(self.sponsor_offer_value)
            self.player_request_value = self.sponsor_offer_value
            return self._resolution("SIGNED", "You accepted the sponsor's proposal.", 1.0)
        if action_id == "select_sponsor":
            sponsor_id = request.get("sponsor_id")
            self.sponsor = self._select_sponsor(sponsor_id)
            self.ideal = self._ideal_terms(self.sponsor)
            self.benchmark_value = self._benchmark_value(self.sponsor)
            self.sponsor_reservation = self.benchmark_value * (1 + min(0.25, 0.08 + self.manager_level * 0.01))
            self.player_reservation = self.benchmark_value * (1 - min(0.25, 0.08 + self.attraction / 1000))
            self.sponsor_offer_value = self.benchmark_value * (0.78 if self.cold_call else 0.98)
            self.player_request_value = self.benchmark_value * (1.05 if self.cold_call else 1.15)
            self.current_terms = self._terms_at_value(
                self.player_request_value if self.cold_call else self.sponsor_offer_value
            )
            return self._ongoing(f"You opened negotiations with {self.sponsor.name}.")
        return {"status": "ERROR", "error": f"unknown action_id: {action_id}"}


def run_protocol(args: argparse.Namespace) -> int:
    LOGGER.info("starting sponsor negotiation JSON session")
    try:
        negotiation = Negotiation(args)
    except ValueError as error:
        print(json.dumps({"status": "ERROR", "error": str(error)}))
        return 2
    print(json.dumps(negotiation.initial(), separators=(",", ":")), flush=True)
    for line in sys.stdin:
        try:
            request = json.loads(line)
            if not isinstance(request, dict):
                raise ValueError("request must be a JSON object")
            response = negotiation.apply(request)
        except (json.JSONDecodeError, TypeError, ValueError) as error:
            response = {"status": "ERROR", "error": str(error)}
        print(json.dumps(response, separators=(",", ":")), flush=True)
        if response.get("status") in {"SIGNED", "BANNED", "REJECTED"}:
            return 0
    return 0


def run_engine_request(args: argparse.Namespace, request: dict[str, Any]) -> int:
    """Answer the engine's one-shot plugin protocol without contaminating stdout."""
    payload = request.get("payload")
    if not isinstance(payload, dict):
        payload = {}
    values = {
        "exp": payload.get("exp", payload.get("experience", args.exp)),
        "results": payload.get("results", args.results),
        "charisma": payload.get("charisma", args.charisma),
        "has_agent": payload.get("has_manager", payload.get("has_agent", args.has_agent)),
        "agent_level": payload.get("manager_level", payload.get("agent_level", args.agent_level)),
        "podiums": payload.get("podiums", args.podiums),
        "wins": payload.get("wins", args.wins),
        "poles": payload.get("poles", payload.get("pole_positions", args.poles)),
        "championships_won": payload.get("championships_won", args.championships_won),
        "race_tier": payload.get("race_tier", args.race_tier),
        "scope": payload.get("scope", args.scope),
        "approach": payload.get("approach", args.approach),
        "money_request": payload.get("money_request", args.money_request),
        "dataset_path": payload.get("dataset_path", args.dataset_path),
        "sponsor": None,
        "seed": args.seed,
    }
    try:
        negotiation = Negotiation(argparse.Namespace(**values))
        value = negotiation.initial()
    except (TypeError, ValueError) as error:
        value = {"status": "ERROR", "error": str(error)}
    response = {
        "protocol_version": request.get("protocol_version", 1),
        "plugin_id": request.get("plugin_id", "sponsor_negotiator"),
        "result_schema_version": 1,
        "results": [{
            "id": "sponsor_negotiation",
            "value": value,
            "label": "Sponsor negotiation",
        }],
    }
    print(json.dumps(response, separators=(",", ":")))
    return 0


def _shared_theme() -> dict[str, str]:
    stylesheet = Path(__file__).resolve().parents[2] / "public" / "plugin.css"
    try:
        source = stylesheet.read_text(encoding="utf-8")
    except OSError:
        return {}
    import re

    return dict(re.findall(r"--([\w-]+):\s*(#[0-9a-fA-F]{6});", source))


def _load_target_options(
    dataset_path: str | None,
) -> tuple[
    list[tuple[str, str]],
    list[tuple[str, str]],
    list[tuple[str, str]],
    dict[str, list[str]],
    dict[str, str],
    dict[str, float],
]:
    if not dataset_path:
        return [], [], [], {}, {}, {}
    root = Path(dataset_path)

    def read_rows(filename: str) -> list[dict[str, str]]:
        path = root / filename
        if not path.is_file():
            return []
        try:
            with path.open(newline="", encoding="utf-8") as handle:
                return list(csv.DictReader(handle))
        except (OSError, csv.Error):
            return []

    def options(rows: list[dict[str, str]]) -> list[tuple[str, str]]:
        return [
            (row.get("id", "").strip(), row.get("name", "").strip())
            for row in rows
            if row.get("id", "").strip() and row.get("name", "").strip()
        ]

    quests_rows = read_rows("quests.csv")
    event_rows = read_rows("events.csv")
    vehicle_rows = [
        row for row in read_rows("objects.csv")
        if row.get("type", "").strip().lower() == "vehicle"
    ]
    vehicle_prices = {
        row.get("id", "").strip(): float(row.get("price", "0") or 0)
        for row in vehicle_rows
        if row.get("id", "").strip()
    }
    required_by_event = {
        row.get("id", "").strip(): [
            required.strip()
            for required in row.get("required_object_ids", "").split(";")
            if required.strip()
        ]
        for row in event_rows
        if row.get("id", "").strip()
    }
    return (
        options(quests_rows),
        options(event_rows),
        options(vehicle_rows),
        {
            row.get("id", "").strip(): required_by_event.get(row.get("id", "").strip(), [])
            for row in event_rows
            if row.get("id", "").strip()
        },
        {
            row.get("id", "").strip(): row.get("quest_id", "").strip()
            for row in event_rows
            if row.get("id", "").strip()
        },
        vehicle_prices,
    )


def _legacy_tk_gui(args: argparse.Namespace) -> int:
    """Legacy fallback retained for reference; the interactive UI is web-based."""
    import tkinter as tk
    from tkinter import ttk

    LOGGER.info("starting sponsor negotiation GUI")
    quests, races, vehicles, required_by_event, quest_by_event, vehicle_prices = _load_target_options(args.dataset_path)
    colors = _shared_theme()
    background = colors.get("app-background", "#0f172a")
    surface = colors.get("surface-background", "#1e293b")
    foreground = colors.get("primary-text", "#f8fafc")
    secondary = colors.get("secondary-text", "#e2e8f0")
    subtle = colors.get("subtle-text", "#94a3b8")
    danger = colors.get("danger-text", "#f87171")
    border = colors.get("control-border", "#7b8794")
    control = colors.get("control-background", "#334155")
    accent = colors.get("primary-accent", "#2563eb")

    window = tk.Tk()
    window.title("TTRPG Engine - Sponsor Negotiation")
    window.geometry("760x800")
    window.minsize(560, 700)
    window.configure(background=background)
    style = ttk.Style(window)
    style.theme_use("clam")
    style.configure("Plugin.TFrame", background=surface)
    style.configure("Plugin.TLabel", background=surface, foreground=foreground)
    style.configure("Plugin.Subtle.TLabel", background=surface, foreground=subtle)
    style.configure("Plugin.Title.TLabel", background=surface, foreground=foreground, font=("TkDefaultFont", 24, "bold"))
    style.configure("Plugin.Tier.TLabel", background=surface, foreground=secondary, font=("TkDefaultFont", 11))
    style.configure("Plugin.Counter.TLabel", background=surface, foreground=danger)
    style.configure("Plugin.TButton", background=control, foreground=secondary, bordercolor=border, padding=(12, 7))
    style.map("Plugin.TButton", background=[("active", accent)])
    style.configure("Plugin.TEntry", fieldbackground=control, foreground=foreground)
    style.configure(
        "Plugin.TCombobox",
        fieldbackground=control,
        background=control,
        foreground=foreground,
        arrowcolor=foreground,
    )
    style.map(
        "Plugin.TCombobox",
        fieldbackground=[("readonly", control), ("disabled", control)],
        background=[("readonly", control), ("active", control)],
        foreground=[("readonly", foreground), ("disabled", subtle)],
    )
    window.option_add("*TCombobox*Listbox.background", control)
    window.option_add("*TCombobox*Listbox.foreground", foreground)
    window.option_add("*TCombobox*Listbox.selectBackground", accent)
    window.option_add("*TCombobox*Listbox.selectForeground", foreground)

    outer = ttk.Frame(window, style="Plugin.TFrame", padding=2)
    outer.pack(fill="both", expand=True, padx=16, pady=16)
    content = ttk.Frame(outer, style="Plugin.TFrame", padding=20)
    content.pack(fill="both", expand=True)
    scope_var = tk.StringVar(value=args.scope)
    target_var = tk.StringVar()
    car_selection_var = tk.StringVar()
    manager_var = tk.StringVar()
    status_var = tk.StringVar()
    agreement_var = tk.StringVar()
    agreement_value_var = tk.StringVar()
    counter_labels: dict[str, ttk.Label] = {}
    fields: dict[str, tk.Variable] = {}
    negotiation: Negotiation | None = None
    last_submitted: dict[str, Any] = {}
    car_menu: ttk.Combobox | None = None
    proposal_sent = False

    header = ttk.Frame(content, style="Plugin.TFrame")
    header.pack(fill="x")
    sponsor_name = ttk.Label(header, style="Plugin.Title.TLabel")
    sponsor_name.pack(anchor="w")
    sponsor_tier = ttk.Label(header, style="Plugin.Tier.TLabel")
    sponsor_tier.pack(anchor="w", pady=(2, 10))

    scope_frame = ttk.Frame(content, style="Plugin.TFrame")
    scope_frame.pack(fill="x", pady=(0, 8))
    ttk.Label(scope_frame, text="Sponsorship period:", style="Plugin.TLabel").pack(side="left")
    scope_menu = ttk.Combobox(
        scope_frame,
        textvariable=scope_var,
        values=("year", "championship", "race"),
        state="readonly",
        width=18,
        style="Plugin.TCombobox",
    )
    scope_menu.pack(side="left", padx=(8, 0))
    target_menu = ttk.Combobox(scope_frame, textvariable=target_var, state="readonly", width=34, style="Plugin.TCombobox")

    def save_agreement(state: dict[str, Any]) -> None:
        if not args.result_file or state.get("status") != "SIGNED":
            return
        agreement = dict(state.get("agreement", {}))
        options = quests if scope_var.get() == "championship" else races
        selected_target_id = next(
            (item_id for item_id, name in options if name == target_var.get()),
            "",
        )
        agreement["target_id"] = selected_target_id or (
            args.target_id if scope_var.get() != "championship" else ""
        )
        agreement["target_name"] = target_var.get() or args.target_name
        agreement["car_object_id"] = (
            next(
                (item_id for item_id, name in vehicles if name == car_selection_var.get()),
                "",
            )
            if car_selection_var.get() != NO_CAR_LABEL
            else ""
        )
        Path(args.result_file).write_text(
            json.dumps({"status": "SIGNED", "agreement": agreement}, separators=(",", ":")),
            encoding="utf-8",
        )
        LOGGER.info("saved signed sponsor agreement to %s", args.result_file)

    package = ttk.Frame(content, style="Plugin.TFrame")
    package.pack(fill="x", pady=(4, 10))
    ttk.Label(package, text="Sponsorship package", style="Plugin.TLabel", font=("TkDefaultFont", 12, "bold")).grid(row=0, column=0, columnspan=3, sticky="w", pady=(0, 5))
    status = ttk.Label(content, textvariable=status_var, style="Plugin.Subtle.TLabel", wraplength=760, justify="left")
    status.pack(fill="x", pady=(0, 8))
    agreement_summary = ttk.Label(content, textvariable=agreement_var, style="Plugin.Tier.TLabel", wraplength=760, justify="left")
    agreement_summary.pack(fill="x", pady=(0, 8))
    agreement_value = ttk.Label(content, textvariable=agreement_value_var, style="Plugin.Tier.TLabel", wraplength=760, justify="left")
    agreement_value.pack(fill="x", pady=(0, 8))

    def new_negotiation() -> None:
        nonlocal negotiation, last_submitted, proposal_sent
        values = vars(args).copy()
        values["scope"] = scope_var.get()
        negotiation = Negotiation(argparse.Namespace(**values))
        LOGGER.info(
            "new GUI negotiation sponsor=%s scope=%s race_tier=%s manager_level=%s",
            negotiation.sponsor.id,
            negotiation.scope,
            negotiation.race_tier,
            negotiation.manager_level,
        )
        last_submitted = {}
        proposal_sent = not negotiation.cold_call
        state = negotiation.initial()
        sponsor_name.configure(text=state.get("sponsor_name", ""))
        sponsor_tier.configure(
            text=(
                f"{state.get('sponsor_tier', '').title()} sponsor"
                f" · {state.get('sponsor_brand', '')}"
            ).rstrip(" ·")
        )
        manager_var.set(f"Manager: Lv. {state.get('manager_level', 0)}" if state.get("manager_level", 0) else "Manager: None")
        status_var.set(state.get("sponsor_dialogue", ""))
        target_menu.pack_forget()
        if scope_var.get() == "championship":
            target_menu.configure(values=[name for _, name in quests])
            target_menu.pack(side="left", padx=(8, 0))
        elif scope_var.get() == "race":
            target_menu.configure(values=[name for _, name in races])
            target_menu.pack(side="left", padx=(8, 0))
        if args.target_name:
            target_var.set(args.target_name)
        elif args.target_id:
            options = quests if scope_var.get() == "championship" else races
            target_var.set(next((name for item_id, name in options if item_id == args.target_id), ""))
        car_selection_var.set(NO_CAR_LABEL)
        render_fields(state)
        send_button.configure(
            text="Send counter-proposal" if proposal_sent else "Send proposal"
        )

    def update_agreement_total(*_args: Any) -> None:
        def numeric(key: str) -> float:
            variable = fields.get(key)
            if variable is None:
                return 0.0
            try:
                return float(variable.get())
            except (TypeError, ValueError):
                return 0.0

        selected_car_id = next(
            (item_id for item_id, name in vehicles if name == car_selection_var.get()),
            "",
        )
        car_value = vehicle_prices.get(selected_car_id, 0.0)
        gear_variable = fields.get("gear")
        gear_value = 3000.0 if gear_variable and bool(gear_variable.get()) else 0.0
        initial_money = numeric("initial_money")
        monthly_payment = numeric("monthly_payment")
        result_bonus = numeric("result_bonus")
        dnf_penalty = numeric("dnf_penalty")
        maintenance_value = negotiation.sponsor.repair_value if (
            negotiation and fields.get("maintenance") and bool(fields["maintenance"].get())
        ) else 0.0
        now_total = car_value + gear_value + initial_money
        other_total = monthly_payment + result_bonus - dnf_penalty + maintenance_value
        agreement_value_var.set(
            f"Now: {now_total:,.0f} "
            f"(car {car_value:,.0f} + gear {gear_value:,.0f} + initial payment {initial_money:,.0f})\n"
            f"Others (approx: {other_total:,.0f} "
            f"(monthly payment {monthly_payment:,.0f} + "
            f"(bonus {result_bonus:,.0f} - penalties {dnf_penalty:,.0f}) + "
            f"maintenance median {maintenance_value:,.0f}))"
        )

    def available_vehicles() -> list[tuple[str, str]]:
        if scope_var.get() == "year":
            return vehicles
        target_options = quests if scope_var.get() == "championship" else races
        target_id = next(
            (item_id for item_id, name in target_options if name == target_var.get()),
            args.target_id,
        )
        if scope_var.get() == "championship":
            required_ids = {
                required_id
                for event_id, required in required_by_event.items()
                if quest_by_event.get(event_id) == target_id
                for required_id in required
            }
        else:
            required_ids = set(required_by_event.get(target_id, []))
        if not required_ids:
            return vehicles
        return [
            vehicle for vehicle in vehicles
            if vehicle[0] in required_ids
        ]

    def refresh_car_options() -> None:
        if car_menu is None:
            return
        options = available_vehicles()
        names = [NO_CAR_LABEL, *[name for _, name in options]]
        car_menu.configure(values=names)
        if car_selection_var.get() not in names:
            car_selection_var.set(NO_CAR_LABEL)

    def render_fields(state: dict[str, Any]) -> None:
        nonlocal car_menu
        for child in package.grid_slaves():
            if int(child.grid_info().get("row", 0)) > 0:
                child.destroy()
        fields.clear()
        counter_labels.clear()
        proposal = state.get("proposal", {}) or {}
        for row, key in enumerate(("car", "entry_fees", "gear", "maintenance", "initial_money", "monthly_payment", "result_bonus", "dnf_penalty"), start=1):
            label = key.replace("_", " ").title()
            ttk.Label(package, text=label, style="Plugin.TLabel").grid(row=row, column=0, sticky="w", pady=2)
            value = proposal.get(key, False if key in {"car", "entry_fees", "gear", "maintenance"} else 0)
            variable: tk.Variable = tk.BooleanVar(value=bool(value)) if isinstance(value, bool) else tk.StringVar(value=str(value))
            fields[key] = variable
            if key == "car":
                editor = ttk.Frame(package, style="Plugin.TFrame")
                editor.grid(row=row, column=1, sticky="w")
                car_menu = ttk.Combobox(
                    editor,
                    textvariable=car_selection_var,
                    values=[NO_CAR_LABEL],
                    state="readonly",
                    width=34,
                    style="Plugin.TCombobox",
                )
                car_menu.pack(side="left")
                car_menu.bind(
                    "<<ComboboxSelected>>",
                    lambda _event, target=variable: target.set(car_selection_var.get() != NO_CAR_LABEL),
                )
                refresh_car_options()
            elif isinstance(variable, tk.BooleanVar):
                ttk.Checkbutton(package, variable=variable).grid(row=row, column=1, sticky="w")
            else:
                editor = ttk.Frame(package, style="Plugin.TFrame")
                editor.grid(row=row, column=1, sticky="w")
                disabled = key == "monthly_payment" and scope_var.get() != "year"
                ttk.Entry(
                    editor,
                    textvariable=variable,
                    width=12,
                    style="Plugin.TEntry",
                    state="disabled" if disabled else "normal",
                ).pack(side="left")

                def adjust(amount: int, target: tk.StringVar = variable) -> None:
                    try:
                        current = int(target.get())
                    except ValueError:
                        current = 0
                    target.set(str(max(0, current + amount)))

                ttk.Button(
                    editor,
                    text="-",
                    width=2,
                    style="Plugin.TButton",
                    command=lambda target=variable: adjust(-1, target),
                ).pack(side="left", padx=(4, 0))
                ttk.Button(
                    editor,
                    text="+",
                    width=2,
                    style="Plugin.TButton",
                    command=lambda target=variable: adjust(1, target),
                ).pack(side="left", padx=(2, 0))
                if disabled:
                    for button in editor.winfo_children()[1:]:
                        button.configure(state="disabled")
            counter = ttk.Label(package, text="", style="Plugin.Counter.TLabel")
            counter.grid(row=row, column=2, sticky="w", padx=(12, 0))
            counter_labels[key] = counter
            variable.trace_add("write", update_agreement_total)
        package.columnconfigure(1, weight=1)
        car_selection_var.trace_add("write", update_agreement_total)
        update_agreement_total()

    def render_state(state: dict[str, Any]) -> None:
        proposal = state.get("proposal", {}) or {}
        failed = state.get("status") in {"REJECTED", "BANNED"}
        for key, label in counter_labels.items():
            if failed:
                label.configure(text="Negotiation abandoned")
            elif key in last_submitted and proposal.get(key) != last_submitted.get(key):
                label.configure(text=f"Sponsor counter: {proposal.get(key)}")
            else:
                label.configure(text="")
        status_var.set(state.get("final_log") or state.get("last_action_log") or state.get("sponsor_dialogue", ""))
        if state.get("status") == "SIGNED":
            agreement_var.set(
                "Agreement signed: "
                f"{state.get('scope', '').title()} · "
                f"initial {state.get('proposal', {}).get('initial_money', 0)} · "
                f"monthly {state.get('proposal', {}).get('monthly_payment', 0)} · "
                f"bonus {state.get('proposal', {}).get('result_bonus', 0)} · "
                f"penalty {state.get('proposal', {}).get('dnf_penalty', 0)}"
            )
        else:
            agreement_var.set("")
        button_state = "disabled" if state.get("status") != "ONGOING" else "normal"
        send_button.configure(state=button_state)
        accept_button.configure(state="normal" if proposal_sent and state.get("status") == "ONGOING" else "disabled")
        accept_button.pack_forget()
        if proposal_sent:
            accept_button.pack(side="left", padx=(8, 0))
        if state.get("proposal") and state.get("status") == "SIGNED":
            for key, variable in fields.items():
                if key not in state["proposal"]:
                    continue
                value = state["proposal"][key]
                if isinstance(variable, tk.BooleanVar):
                    variable.set(bool(value))
                else:
                    variable.set(str(value))
        if state.get("status") == "SIGNED":
            selected_id = state.get("agreement", {}).get("car_object_id", "")
            if selected_id:
                car_selection_var.set(next(
                    (name for item_id, name in vehicles if item_id == selected_id),
                    NO_CAR_LABEL,
                ))
            elif not state.get("proposal", {}).get("car", False):
                car_selection_var.set(NO_CAR_LABEL)

    def send_proposal() -> None:
        nonlocal proposal_sent
        if negotiation is None:
            return
        proposal: dict[str, Any] = {}
        try:
            for key, variable in fields.items():
                if key == "car":
                    proposal[key] = car_selection_var.get() != NO_CAR_LABEL
                elif isinstance(variable, tk.BooleanVar):
                    proposal[key] = variable.get()
                else:
                    proposal[key] = int(variable.get())
        except ValueError:
            LOGGER.warning("proposal rejected locally because a monetary field was not an integer")
            status_var.set("Enter whole numbers in the monetary package fields.")
            return
        if scope_var.get() != "year":
            proposal["monthly_payment"] = 0
        proposal_sent = True
        last_submitted.clear()
        last_submitted.update(proposal)
        LOGGER.info("sending GUI proposal sponsor=%s proposal=%s", negotiation.sponsor.id, proposal)
        state = negotiation.apply({"action_id": "counter_proposal", "proposal": proposal})
        LOGGER.info(
            "received GUI proposal response sponsor=%s status=%s message=%s",
            negotiation.sponsor.id,
            state.get("status"),
            state.get("final_log") or state.get("last_action_log") or state.get("error", ""),
        )
        if state.get("status") == "ERROR":
            status_var.set(f"Proposal error: {state.get('error', 'unknown error')}")
            return
        render_state(state)
        save_agreement(state)

    def accept_sponsor_proposal() -> None:
        if negotiation is None or not proposal_sent:
            return
        LOGGER.info("accepting sponsor proposal sponsor=%s", negotiation.sponsor.id)
        state = negotiation.apply({"action_id": "accept_proposal"})
        LOGGER.info(
            "accepted sponsor proposal sponsor=%s status=%s message=%s",
            negotiation.sponsor.id,
            state.get("status"),
            state.get("final_log") or state.get("error", ""),
        )
        if state.get("status") == "ERROR":
            status_var.set(f"Proposal error: {state.get('error', 'unknown error')}")
            return
        render_state(state)
        save_agreement(state)

    ttk.Label(content, textvariable=manager_var, style="Plugin.Subtle.TLabel").pack(anchor="w")
    actions = ttk.Frame(content, style="Plugin.TFrame")
    actions.pack(anchor="w", pady=(8, 0))
    send_button = ttk.Button(actions, text="Send proposal", style="Plugin.TButton", command=send_proposal)
    send_button.pack(side="left")
    accept_button = ttk.Button(
        actions,
        text="Accept sponsor proposal",
        style="Plugin.TButton",
        command=accept_sponsor_proposal,
    )
    ttk.Button(content, text="Close", style="Plugin.TButton", command=window.destroy).pack(anchor="e", pady=(8, 0))
    scope_menu.bind("<<ComboboxSelected>>", lambda _event: new_negotiation())
    target_menu.bind("<<ComboboxSelected>>", lambda _event: refresh_car_options())
    new_negotiation()
    window.mainloop()
    return 0


def run_gui(args: argparse.Namespace) -> int:
    """Run the sponsor negotiation as a local HTML/CSS/JavaScript application."""
    import threading
    import webbrowser
    from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

    quests, races, vehicles, required_by_event, quest_by_event, vehicle_prices = _load_target_options(
        args.dataset_path
    )
    root = Path(__file__).resolve().parents[2]
    stylesheet = (root / "public" / "plugin.css").read_text(encoding="utf-8")
    completed = threading.Event()
    state_lock = threading.Lock()
    session: dict[str, Any] = {
        "negotiation": Negotiation(args),
        "selected_target": args.target_name,
        "selected_car": NO_CAR_LABEL,
        "error": "",
    }

    def target_options(scope: str) -> list[tuple[str, str]]:
        return quests if scope == "championship" else races

    def available_vehicles(scope: str, target_name: str) -> list[tuple[str, str]]:
        if scope == "year":
            return vehicles
        options = target_options(scope)
        target_id = next((item_id for item_id, name in options if name == target_name), "")
        if not target_id:
            return []
        if scope == "championship":
            required_ids = {
                required_id
                for event_id, required in required_by_event.items()
                if quest_by_event.get(event_id) == target_id
                for required_id in required
            }
        else:
            required_ids = set(required_by_event.get(target_id, []))
        if not required_ids:
            return vehicles
        return [vehicle for vehicle in vehicles if vehicle[0] in required_ids]

    def save_agreement(state: dict[str, Any]) -> None:
        if not args.result_file or state.get("status") != "SIGNED":
            return
        negotiation: Negotiation = session["negotiation"]
        agreement = dict(state.get("agreement", {}))
        selected_target = session["selected_target"]
        options = target_options(negotiation.scope)
        selected_target_id = next(
            (item_id for item_id, name in options if name == selected_target),
            "",
        )
        agreement["target_id"] = selected_target_id or (
            args.target_id if negotiation.scope != "championship" else ""
        )
        agreement["target_name"] = selected_target or args.target_name
        selected_car = session["selected_car"]
        agreement["car_object_id"] = (
            next((item_id for item_id, name in vehicles if name == selected_car), "")
            if selected_car != NO_CAR_LABEL
            else ""
        )
        Path(args.result_file).write_text(
            json.dumps({"status": "SIGNED", "agreement": agreement}, separators=(",", ":")),
            encoding="utf-8",
        )
        LOGGER.info("saved signed sponsor agreement to %s", args.result_file)

    def current_view() -> dict[str, Any]:
        negotiation: Negotiation = session["negotiation"]
        state = negotiation.initial()
        state["selected_target"] = session["selected_target"]
        state["selected_car"] = session["selected_car"]
        state["target_options"] = [
            {"id": item_id, "name": name}
            for item_id, name in target_options(negotiation.scope)
        ]
        state["vehicle_options"] = [
            {"id": item_id, "name": name, "price": vehicle_prices.get(item_id, 0)}
            for item_id, name in available_vehicles(negotiation.scope, session["selected_target"])
        ]
        state["error"] = session["error"]
        return state

    def apply_action(request: dict[str, Any]) -> dict[str, Any]:
        with state_lock:
            session["error"] = ""
            action = request.get("action")
            if action == "close":
                completed.set()
                return {"status": "CLOSED"}
            if action == "new_session":
                scope = request.get("scope")
                if scope not in {"race", "championship", "year"}:
                    return {"status": "ERROR", "error": "invalid sponsorship period"}
                values = vars(args).copy()
                values["scope"] = scope
                session["negotiation"] = Negotiation(argparse.Namespace(**values))
                session["selected_target"] = args.target_name
                session["selected_car"] = NO_CAR_LABEL
                return current_view()
            negotiation: Negotiation = session["negotiation"]
            if action == "select_target":
                session["selected_target"] = str(request.get("target_name", ""))
                session["selected_car"] = NO_CAR_LABEL
                return current_view()
            if action == "select_car":
                session["selected_car"] = str(request.get("car_name", NO_CAR_LABEL))
                return current_view()
            if action == "counter_proposal":
                proposal = request.get("proposal")
                if not isinstance(proposal, dict):
                    return {"status": "ERROR", "error": "proposal must be an object"}
                if negotiation.scope != "year":
                    proposal["monthly_payment"] = 0
                state = negotiation.apply({"action_id": "counter_proposal", "proposal": proposal})
            elif action == "accept_proposal":
                state = negotiation.apply({"action_id": "accept_proposal"})
            else:
                return {"status": "ERROR", "error": f"unknown UI action: {action}"}
            if state.get("status") == "SIGNED":
                save_agreement(state)
            state["selected_target"] = session["selected_target"]
            state["selected_car"] = session["selected_car"]
            state["target_options"] = [
                {"id": item_id, "name": name}
                for item_id, name in target_options(negotiation.scope)
            ]
            state["vehicle_options"] = [
                {"id": item_id, "name": name, "price": vehicle_prices.get(item_id, 0)}
                for item_id, name in available_vehicles(negotiation.scope, session["selected_target"])
            ]
            return state

    def page() -> str:
        initial = json.dumps(current_view(), separators=(",", ":")).replace("</", "<\\/")
        safe_css = stylesheet.replace("</style", "<\\/style")
        return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Sponsor Negotiation</title>
  <style>{safe_css}</style>
  <style>
    html, body {{ min-height: 100%; }}
    body {{ overflow: auto; padding: 3rem 1rem 2.5rem !important; }}
    .negotiation-window {{ width: min(980px, 100%); margin: 0 auto; }}
    .topline {{ display: flex; justify-content: space-between; gap: 1rem; align-items: end; border-bottom: 1px solid #e5232b; padding-bottom: .8rem; }}
    .eyebrow {{ color: #aeb4b9; font-size: .9rem; letter-spacing: .14em; }}
    .topline h1 {{ margin: .15rem 0 0; }}
    .sponsor-tier {{ color: #fff59d; font-size: 1.05rem; }}
    .toolbar, .actions {{ display: flex; flex-wrap: wrap; gap: .7rem; align-items: center; }}
    .toolbar {{ margin: 1rem 0; padding: .8rem; border: 1px solid #3b4045; background: rgb(15 17 20 / 90%); }}
    .toolbar label {{ display: flex; align-items: center; gap: .45rem; }}
    .dashboard {{ display: grid; grid-template-columns: minmax(0, 1.3fr) minmax(240px, .7fr); gap: 1rem; }}
    .panel {{ background: rgb(15 17 20 / 94%); border: 1px solid #3b4045; padding: 1rem; }}
    .panel h2 {{ margin-top: 0; }}
    .dialogue {{ min-height: 4rem; color: #d4d7da; }}
    .metrics {{ display: grid; grid-template-columns: repeat(2, 1fr); gap: .65rem; }}
    .metric {{ border-left: 3px solid #e5232b; background: #1c2024; padding: .65rem .75rem; }}
    .metric small {{ display: block; color: #aeb4b9; text-transform: uppercase; }}
    .metric strong {{ display: block; margin-top: .15rem; font-size: 1.2rem; }}
    .package {{ display: grid; grid-template-columns: minmax(150px, .7fr) minmax(170px, 1fr) minmax(130px, .7fr); gap: .55rem .8rem; align-items: center; }}
    .package label {{ color: #d4d7da; }}
    .package input, .package select {{ width: 100%; }}
    .package input[type="checkbox"] {{ width: auto; accent-color: #e5232b; }}
    .counter {{ color: #fff59d; font-size: .9rem; }}
    .log {{ margin-top: 1rem; border-top: 1px solid #3b4045; padding-top: .8rem; color: #aeb4b9; white-space: pre-wrap; }}
    .error {{ border: 1px solid #ff5960; background: #3a1518; color: #fff; padding: .7rem; }}
    .signed {{ border-color: #8ed081; box-shadow: 0 0 22px rgb(71 170 85 / 18%); }}
    .signed h2 {{ color: #b9f0ab !important; }}
    @media (max-width: 760px) {{ .dashboard {{ grid-template-columns: 1fr; }} .topline {{ align-items: start; flex-direction: column; }} .package {{ grid-template-columns: 1fr; }} }}
  </style>
</head>
<body>
  <main class="negotiation-window">
    <header class="topline">
      <div><div class="eyebrow">TTRPG ENGINE / DRIVER SERVICES</div><h1>SPONSOR NEGOTIATION</h1></div>
      <div id="sponsor-tier" class="sponsor-tier"></div>
    </header>
    <section class="toolbar">
      <label>Sponsorship period <select id="scope"></select></label>
      <label id="target-wrap">Target <select id="target"></select></label>
      <label>Vehicle <select id="car"></select></label>
      <span id="manager"></span>
    </section>
    <div id="error"></div>
    <div class="dashboard">
      <section class="panel">
        <h2 id="sponsor-name"></h2>
        <p id="dialogue" class="dialogue"></p>
        <div class="metrics">
          <div class="metric"><small>Attraction score</small><strong id="attraction"></strong></div>
          <div class="metric"><small>Race tier</small><strong id="race-tier"></strong></div>
          <div class="metric"><small>Round</small><strong id="round"></strong></div>
          <div class="metric"><small>Negotiation status</small><strong id="status"></strong></div>
        </div>
        <p id="log" class="log"></p>
      </section>
      <aside class="panel">
        <h2>Negotiation brief</h2>
        <p>Build a package that reflects your results, audience value, and the sponsor's walkaway limit.</p>
        <p id="objection"></p>
        <p id="agreement"></p>
      </aside>
    </div>
    <section id="package-panel" class="panel" style="margin-top:1rem">
      <h2>Proposed package</h2>
      <div id="package" class="package"></div>
      <div class="actions" style="margin-top:1rem">
        <button id="send">Send proposal</button>
        <button id="accept">Accept sponsor proposal</button>
        <button id="close">Close</button>
      </div>
    </section>
  </main>
  <script>
    let state = {initial};
    let proposalSent = !state.cold_call;
    const numberKeys = ['initial_money', 'monthly_payment', 'result_bonus', 'dnf_penalty'];
    const booleanKeys = ['entry_fees', 'gear', 'maintenance'];
    const labels = {{ initial_money: 'Initial payment', monthly_payment: 'Monthly payment', result_bonus: 'Result bonus', dnf_penalty: 'DNF penalty', entry_fees: 'Entry fees covered', gear: 'Race gear included', maintenance: 'Maintenance covered' }};
    const $ = (id) => document.getElementById(id);
    const money = (value) => Number(value || 0).toLocaleString();
    async function act(action, extra = {{}}) {{
      const response = await fetch('/api/action', {{
        method: 'POST',
        headers: {{ 'Content-Type': 'application/json' }},
        body: JSON.stringify({{ action, ...extra }})
      }});
      state = await response.json();
      if (action === 'new_session') proposalSent = !state.cold_call;
      render();
    }}
    function renderSelect(select, options, value, onChange) {{
      select.replaceChildren(...options.map((option) => {{
        const item = document.createElement('option');
        item.value = option.name ?? option;
        item.textContent = option.name ?? option;
        item.selected = item.value === value;
        return item;
      }}));
      select.onchange = onChange;
    }}
    function renderPackage() {{
      const package = $('package');
      package.replaceChildren();
      const proposal = state.proposal || {{}};
      const keys = [...booleanKeys, ...numberKeys];
      for (const key of keys) {{
        const label = document.createElement('label');
        label.textContent = labels[key];
        const editor = document.createElement(key === 'entry_fees' || key === 'gear' || key === 'maintenance' ? 'input' : 'input');
        editor.type = booleanKeys.includes(key) ? 'checkbox' : 'number';
        if (editor.type === 'checkbox') editor.checked = Boolean(proposal[key]);
        else {{ editor.value = proposal[key] ?? 0; editor.min = '0'; }}
        editor.dataset.key = key;
        const counter = document.createElement('span');
        counter.className = 'counter';
        counter.id = 'counter-' + key;
        package.append(label, editor, counter);
      }}
    }}
    function proposalFromForm() {{
      const proposal = {{}};
      document.querySelectorAll('#package input[data-key]').forEach((input) => {{
        proposal[input.dataset.key] = input.type === 'checkbox' ? input.checked : Number(input.value || 0);
      }});
      proposal.car = $('car').value !== 'No car included';
      return proposal;
    }}
    function render() {{
      const negotiationStatus = state.status || 'ONGOING';
      $('sponsor-name').textContent = state.sponsor_name || 'Sponsor';
      $('sponsor-tier').textContent = `${{(state.sponsor_tier || '').toUpperCase()}} / ${{state.sponsor_brand || 'PARTNERSHIP'}}`;
      $('dialogue').textContent = state.sponsor_dialogue || '';
      $('attraction').textContent = Number(state.attraction_score || 0).toFixed(1);
      $('race-tier').textContent = state.race_tier || '';
      $('round').textContent = state.round || 0;
      $('status').textContent = negotiationStatus;
      $('log').textContent = state.final_log || state.last_action_log || '';
      $('objection').textContent = state.objection || '';
      $('manager').textContent = state.manager_level ? `MANAGER LV. ${{state.manager_level}}` : 'NO MANAGER';
      $('error').replaceChildren();
      if (state.error) {{ const error = document.createElement('p'); error.className = 'error'; error.textContent = state.error; $('error').append(error); }}
      renderSelect($('scope'), ['race', 'championship', 'year'].map((name) => ({{ name }})), state.scope, (event) => act('new_session', {{ scope: event.target.value }}));
      const targets = state.target_options || [];
      $('target-wrap').style.display = state.scope === 'year' ? 'none' : 'flex';
      renderSelect($('target'), targets, state.selected_target || '', (event) => act('select_target', {{ target_name: event.target.value }}));
      const cars = [{{ name: 'No car included' }}, ...(state.vehicle_options || [])];
      renderSelect($('car'), cars, state.selected_car || 'No car included', (event) => act('select_car', {{ car_name: event.target.value }}));
      renderPackage();
      const terminal = ['SIGNED', 'REJECTED', 'BANNED'].includes(negotiationStatus);
      $('package-panel').classList.toggle('signed', negotiationStatus === 'SIGNED');
      $('send').disabled = terminal;
      $('accept').disabled = terminal || !proposalSent;
      $('send').textContent = state.cold_call ? 'Send proposal' : 'Send counter-proposal';
      $('agreement').textContent = state.agreement ? 'Agreement signed. Close this window to return to the game.' : '';
      for (const key of numberKeys) {{
        const current = state.proposal?.[key];
        const ideal = state.ideal_proposal?.[key];
        if (current !== undefined && ideal !== undefined && current !== ideal) $('counter-' + key).textContent = `Ideal: ${{money(ideal)}}`;
      }}
    }}
    $('send').onclick = () => {{ proposalSent = true; act('counter_proposal', {{ proposal: proposalFromForm() }}); }};
    $('accept').onclick = () => act('accept_proposal');
    $('close').onclick = () => {{ navigator.sendBeacon('/api/action', JSON.stringify({{ action: 'close' }})); window.close(); }};
    window.addEventListener('beforeunload', () => navigator.sendBeacon('/api/action', JSON.stringify({{ action: 'close' }})));
    render();
  </script>
</body>
</html>"""

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, format: str, *args: Any) -> None:
            LOGGER.debug("sponsor web UI: " + format, *args)

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
            if self.path != "/api/action":
                self.send_error(404)
                return
            try:
                length = int(self.headers.get("Content-Length", "0"))
                request = json.loads(self.rfile.read(length))
                response = apply_action(request if isinstance(request, dict) else {})
            except (ValueError, TypeError, json.JSONDecodeError) as error:
                response = {"status": "ERROR", "error": str(error)}
            content = json.dumps(response, separators=(",", ":")).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(content)))
            self.end_headers()
            self.wfile.write(content)

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    url = f"http://127.0.0.1:{server.server_port}/"
    LOGGER.info("starting sponsor negotiation web UI at %s", url)
    if not webbrowser.open_new(url):
        server.server_close()
        raise RuntimeError("could not open the sponsor negotiation web UI in a browser")
    try:
        while not completed.wait(0.25):
            server.handle_request()
    finally:
        server.server_close()
    return 0


def main(argv: list[str] | None = None) -> int:
    logging.basicConfig(
        level=logging.INFO,
        stream=sys.stderr,
        format="%(asctime)s sponsor_negotiator %(levelname)s %(message)s",
    )
    args = build_parser().parse_args(argv)
    if args.help_effects:
        print(
            "Having an agent adds 8 attraction. "
            "Sponsor matching uses achievements weighted by race tier. "
            "Manager levels unlock higher sponsor tiers; cold calls start at "
            "a conservative package while incoming proposals start higher. Negotiations "
            "counter within fifteen percent of the ideal package. "
            "agent_level * 8 contributes to attraction."
        )
        return 0
    if args.interactive:
        return run_protocol(args)
    if args.json:
        first_line = sys.stdin.readline()
        if first_line:
            try:
                request = json.loads(first_line)
            except json.JSONDecodeError:
                request = None
            if isinstance(request, dict) and "protocol_version" in request and "operation" in request:
                return run_engine_request(args, request)
            # Preserve the line for the interactive protocol's first action.
            class PrefixedStdin:
                def __init__(self, prefix: str, remainder: Any):
                    self.prefix = prefix
                    self.remainder = remainder
                    self.used = False

                def __iter__(self):
                    if not self.used:
                        self.used = True
                        yield self.prefix
                    yield from self.remainder

            original_stdin = sys.stdin
            sys.stdin = PrefixedStdin(first_line, original_stdin)  # type: ignore[assignment]
            try:
                return run_protocol(args)
            finally:
                sys.stdin = original_stdin
        return run_protocol(args)
    return run_gui(args)


if __name__ == "__main__":
    raise SystemExit(main())
