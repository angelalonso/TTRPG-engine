# Cooking fixture

This is a small non-racing dataset for validator and headless tests. It uses
the supported CSV contract only:

- `coins` and `culinary_skill` are dataset-defined characteristics; no legacy
  racing characteristic IDs are present.
- `recipe_quest` is a zero-fee quest.
- `bake_pie` is a deterministic quest activity, while `rent_skillet`
  exercises configured rental terms.
- `bake_with_ingredients` uses an `ALL` requirement group and consumes flour
  and an egg before granting a pie.
- `event_results.csv` applies supported characteristic, consumption, and
  object effects.

History-based recipe unlocks and non-sellable quest rewards are not represented
here; the fixture stays focused on the supported grouped-requirement,
consumption, and rental paths.
