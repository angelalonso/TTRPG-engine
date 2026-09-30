# Cooking fixture

This is a small non-racing dataset for validator and headless tests. It uses
the supported CSV contract only:

- `coins` and `culinary_skill` are dataset-defined characteristics; no legacy
  racing characteristic IDs are present.
- `recipe_quest` is a zero-fee quest.
- `bake_pie` is a deterministic quest activity.
- `event_results.csv` applies a supported characteristic effect.

The current CSV contract does not execute ingredient consumption, history-based
recipe unlocks, or non-sellable quest rewards. Those features are deliberately
not represented here.
