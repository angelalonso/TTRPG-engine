# Playtest decision control and path diversity

## Purpose

The playtest runner needs two separate kinds of control:

1. **Control within a run**: decide which eligible action should be selected next.
2. **Diversity across runs**: avoid spending a batch on many runs that follow the same action chain.

These should not be implemented as one mechanism. A run must remain reproducible for a given seed, while the batch runner should be able to explore different valid strategies and action paths.

## Current behavior and limitations

The runner has useful foundations: seeded runs, eligibility filtering, goal-aware logic, configurable minimum stamina, trace logging, and the `unique_paths` option. The legacy policy still permits the repetition seen in `playtest.out`; the new `top-k` and `diverse` policies add configurable controls without removing the legacy baseline.

### Repetition within a run

- `GoalAwareStrategy` uses a mostly fixed ordering: events, purchases, quests, then actions.
- The legacy policy preserves that behavior, including forcing the first eligible championship quest for championship goals.
- Actions are selected by a deterministic score. If an action such as `act_event_staffing` or `act_trade` remains the highest-scoring candidate, it can be selected repeatedly.
- The `top-k` and `diverse` policies add recent-action memory, action cooldowns, maximum streak penalties, and stagnation penalties.
- Repeated `Wait` decisions are also expected when no eligible action exists, but they should be distinguished from strategic repetition.

### Diversity across runs

`unique_paths` compares complete paths only after a run finishes. If two runs differ at one late decision, they are considered different even if they share almost the entire prefix. If a path is duplicated, the runner retries with another seed, but this does not change the policy that generated the repeated chain.

This means `unique_paths` is useful as a basic duplicate safeguard, but it is not a reliable exploration strategy.

## Implemented control model

### 1. Configurable priorities within a run

The runner now supports a policy layer that scores all currently eligible candidates. A useful conceptual score is:

```text
score =
    category priority
  + estimated progress toward the goal
  + expected reward
  - resource and failure risk
  - recent-use penalty
  - batch novelty penalty
```

The policy supports strict category priorities plus weighted action scoring. Strict priorities are appropriate where legality or timing matters, such as a scheduled goal race. Weighted scoring is appropriate for interchangeable income, training, or preparation actions.

Example configuration:

```json
{
  "priorities": {
    "goal_event": 1000,
    "scheduled_event": 900,
    "required_purchase": 800,
    "income": 300,
    "training": 200,
    "optional_purchase": 50,
    "wait": 0
  }
}
```

The `top-k` and `diverse` policies select reproducibly from the top-scoring candidates using the game RNG. A top-K choice is preferable to an unseeded random choice because it preserves reproducibility.

### 2. Per-run repetition controls

The `top-k` and `diverse` policies maintain decision history for the current run and apply configurable penalties or temporary exclusions:

```json
{
  "repetition": {
    "max_same_action_streak": 2,
    "cooldown_days": {
      "act_event_staffing": 2,
      "act_trade": 3
    },
    "novelty_window": 12,
    "stagnation_turns": 20
  }
}
```

Recommended rules:

- Penalize an action after it has been used recently.
- Temporarily exclude an action after its maximum consecutive-use streak.
- Apply cooldowns by action ID or action category.
- Penalize repeated short sequences, not just individual actions.
- Permit repetition when the action is mandatory, the only legal safe action, or required to meet a scheduled event.
- Detect stagnation using a state signature containing goal progress, budget, stamina, inventory, active jobs, quest memberships, and a coarse day/resource bucket.
- If the state has not materially improved after the configured number of turns, switch to a fallback policy or end the run with an explicit `stagnation` outcome.

Mandatory actions must bypass repetition rules. Examples include scheduled races, required payments, purchases necessary for a target event, and actions needed to avoid immediate death.

### 3. Cross-run path diversity

Instead of requiring complete paths to be unique, track common prefixes and short action n-grams across the batch. At each decision, apply a novelty penalty to candidates that would continue a frequently observed prefix:

```text
novelty penalty =
    number of previous runs sharing the relevant prefix
    × configured penalty
```

A prefix trie or hashes of the last 5–10 decisions are sufficient. This gives the batch runner a practical diversity mode without invalidating reproducibility.

The runner should expose separate modes:

- `deterministic`: choose the highest-scoring candidate.
- `seeded-top-k`: choose reproducibly among the best K candidates.
- `diverse`: apply cross-run prefix and n-gram penalties.
- `random-eligible`: choose uniformly among eligible candidates as an exploration baseline.

The seed used for every run should still be recorded. Diversity should come from the policy, not from silently replacing a completed run with an unrelated seed.

## Goal configuration

The original `playtest.json` used:

```json
"goal": "trophy:level=1,count=1"
```

The parser now accepts `trophy:level=...,count=...` as an alias. The checked-in configurations use the clearer supported form:

```json
"goal": "championships:level=1,count=1"
```

Unknown goal formats are still interpreted as characteristic goals for backwards compatibility. Configuration validation should eventually reject them instead of silently converting them into a different goal.

## Available profiles

The repository includes ready-to-run profiles:

| Profile | Command | Behavior |
|---|---|---|
| Deterministic | `make playtest-deterministic` | Existing legacy strategy behavior; reproducible baseline |
| Top-K | `make playtest-top-k` | Seeded choice among the three best candidates, with streak and cooldown penalties |
| Diverse | `make playtest-diverse` | Seeded top-four choice plus penalties for actions already common in the batch |
| Required | `make playtest-required` | Required-actions baseline using the existing `required` strategy |

The generic target remains available:

```sh
make playtest PLAYTEST_CONFIG=playtest.top-k.json
```

The new policy options are also available as CLI flags:

```text
--policy legacy|top-k|diverse
--top-k N
--max-same-action-streak N
--cooldown-days N
--novelty-penalty N
--novelty-window N
--stagnation-turns N
```

## Further improvements

1. Add a regression test for every supported goal alias and reject unknown formats when practical.
2. Replace action-frequency novelty with prefix and n-gram novelty tracking.
3. Add explicit per-action cooldown maps and mandatory-action metadata.
4. Expand trace output to include candidate scores, repetition penalties, rejected candidates, selected action, and whether the action was mandatory.

This separation keeps normal game behavior unchanged while making playtest runs controllable, debuggable, reproducible, and useful for batch experimentation.
