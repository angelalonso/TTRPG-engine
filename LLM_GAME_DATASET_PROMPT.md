# LLM prompt workflow for creating games

This document is a reusable prompt and workflow for asking an LLM to design a
new game for the TTRPG Engine and generate its dataset. It is intended for
games and simulations in many genres: management, fantasy, business, school,
crafting, sports, politics, exploration, collecting, or any other game that
can be represented by player characteristics, objects, activities, events,
costs, and optional encounters.

Use the prompts in order. Upload the CSV files from an existing dataset when
the LLM needs to inspect an example or modify an existing game. The
`DATASET_GENERATION_GUIDE.md` file in this repository is the detailed schema
reference; this document is the LLM-facing design and generation workflow.

---

## How to use this workflow

1. Copy the **Base prompt** below into the LLM.
2. Replace every `{{...}}` placeholder in the **Game brief**.
3. Upload the current CSV files, if any. Upload HTML descriptions and images
   when they are relevant to the game.
4. Ask the LLM to complete one phase at a time. Do not ask it to invent the
   whole dataset in one response unless the dataset is very small.
5. After each generation phase, save the returned files locally and run the
   dataset validator before continuing.
6. In later iterations, upload the current files and ask for a targeted change.
   The LLM must preserve existing IDs and unrelated behavior unless a change
   explicitly requires otherwise.

The LLM should output plain files, not a prose approximation of CSV data.
When a response is too large, it should return a manifest and then provide
each CSV in separate responses.

---

## Game brief: information to provide explicitly

Complete this section before sending the prompts. If a value is undecided,
write `UNDECIDED`; the LLM should propose options instead of silently choosing
one.

```text
GAME TITLE:
{{title}}

GENRE / SETTING:
{{genre_and_setting}}

ONE-SENTENCE PLAYER FANTASY:
{{what_the_player_is_trying_to_do}}

PLAYER ROLE:
{{who_or_what_the_player_controls}}

PRIMARY GOAL:
{{how_a_player_wins_or_what_success_means}}

SECONDARY GOALS:
{{optional_goals_and_long_term_progression}}

FAILURE CONDITIONS:
{{how_the_player_can lose, stall, or receive a setback}}

EXPECTED GAME LENGTH:
{{for_example: 30 in-game days, 5 years, one campaign, endless sandbox}}

TIME MODEL:
{{calendar units, number of days per year, whether time advances automatically}}

PLAYER ACTIONS:
{{what the player can choose to do, including actions that consume time}}

SCHEDULED ACTIVITIES:
{{events that happen on particular dates or become available over time}}

OBJECTS / ASSETS:
{{things the player can buy, own, use, equip, consume, service, or sell}}

UPFRONT ECONOMY:
{{starting money, purchase prices, entry fees, wages, prizes, currencies}}

RECURRING ECONOMY:
{{daily, weekly, monthly, annual, per-use, maintenance, insurance, rent, or
other repeating costs and income}}

PLAYER CHARACTERISTICS:
{{numeric stats such as health, energy, reputation, skill, morale, knowledge,
influence, age, or budget}}

PROGRESSION:
{{licenses, levels, prerequisites, unlocks, object upgrades, quests, ranks}}

RANDOMNESS:
{{what should be random, probability expectations, and what must remain
deterministic or manually entered}}

ENCOUNTERS:
{{none, or describe turn-based conflicts/challenges and their win conditions}}

CONTENT VOLUME:
{{approximate number of player stats, objects, actions, events, quests, and
encounters}}

TONE AND LANGUAGE:
{{serious, humorous, realistic, dramatic, child-friendly, etc.}}

UI TERMINOLOGY:
{{what the tabs and concepts should be called in this game}}

ASSETS AVAILABLE:
{{CSV files, HTML descriptions, images, or an existing dataset to modify}}

NON-NEGOTIABLE RULES:
{{rules the LLM must never change or reinterpret}}
```

---

## Base prompt

```text
You are designing a playable dataset for the TTRPG Engine. The engine is
domain-neutral, but it is data-driven and only supports the concepts described
in the attached repository documentation and CSV schemas.

Use the game brief below as the source of truth for the intended design.
Treat uploaded CSV files as existing implementation, not as disposable
examples. First inspect their headers, IDs, references, and current behavior.

Your responsibilities:

1. Translate the game design into the engine's supported CSV model.
2. Keep the design coherent, playable, and internally consistent.
3. Preserve stable IDs when modifying an existing dataset.
4. Never invent a CSV column, engine feature, trigger, operator, resolution
   method, or relationship without clearly marking it as unsupported.
5. Prefer a simple supported implementation over an elaborate unsupported one.
6. Make all cross-file references resolve.
7. Use exact CSV headers and valid CSV escaping.
8. Explain important modeling decisions before generating large files.
9. Report assumptions, limitations, balance risks, and unresolved questions.
10. Never silently delete existing content. List proposed deletions separately
    and wait for explicit approval before applying them.

The requested output must be suitable for saving directly into a dataset
folder. Use UTF-8 CSV, stable lowercase snake_case IDs, decimal numbers for
numeric values, empty fields rather than null, semicolon-separated lists where
the schema specifies lists, and relative paths for HTML and image assets.
```

---

## Prompt 1: understand and formalize the game

Send this after the base prompt and game brief.

```text
PHASE 1 - GAME MODEL

Do not generate the full CSV files yet. Convert the game brief into a concise
engine-compatible design specification.

Return:

1. The core gameplay loop, including what advances time.
2. The player resources and characteristics, with starting values, minimums,
   maximums, and which ones are special currencies or limits.
3. The object categories and what each category means.
4. The player-started activities and scheduled activities.
5. The quests or long-term objectives.
6. The economy: upfront prices, recurring costs, event costs, rewards, and
   possible income.
7. The progression and prerequisite graph.
8. The random/manual/encounter resolution plan for every activity.
9. The files that will be needed, including optional files.
10. A table of unsupported or ambiguous requirements and the closest supported
    implementation for each.

Call out contradictions in the game brief. Do not resolve a contradiction
silently.
```

### What the engine can model

Use these concepts when translating the design:

- **Player characteristics**: numeric values in `player.csv`, such as money,
  health, energy, skill, reputation, influence, morale, or age. Values can
  have minimum and maximum clamps.
- **Objects**: purchasable or ownable things in `objects.csv`. An object can
  have a type, upfront `price`, service or recurring cost references, resale
  behavior, prerequisites, a license level, a lifetime, a delayed availability
  date, an HTML description, and an image.
- **Upfront object cost**: put the purchase amount in `objects.csv` as `price`.
- **Reusable costs**: define named amounts in `costs.csv`, then reference them
  from object cost slots or cost rules. This avoids copying the same amount
  into many rows.
- **Recurring or triggered cost**: use `cost_rules.csv`. A
  `day_elapsed` rule can represent daily, weekly, monthly, or annual behavior
  by choosing an appropriate `interval_days`; an `event_completed` rule can
  represent per-use or event-based costs; an `object_acquired` rule can
  represent acquisition consequences.
- **Cost restrictions**: use `cost_rule_conditions.csv` when a rule applies
  only to a particular object, object type, event, event tag, or player fact.
  Multiple conditions are combined as AND.
- **Player-started activities**: rows in `events.csv` using fields such as
  `type`, `base_cost`, `stamina_cost`, `success_rate`, `payout`, and payout
  frequency. These can represent work, study, trade, training, applications,
  crafting, travel, or other actions.
- **Scheduled activities**: rows in the same `events.csv` with a
  `day_of_year`. They can represent races, meetings, exams, shows,
  appointments, social activities, missions, or competitions.
- **Activity resolution**:
  - `manual`: the player enters the result, useful for rankings or externally
    decided outcomes.
  - `random`: the engine resolves success using `success_rate`.
  - `encounter`: the activity opens the optional turn-based encounter system.
- **Rewards**: use activity rewards, player characteristic changes, event
  outcomes, quests, sponsor payouts, or encounter consequences as appropriate.
- **Quests**: use `quests.csv` and link activities with `quest_id`.
  Championships and other tracked objectives can have points, join fees,
  licenses, final rewards, and optional competitor names.
- **Licenses and prerequisites**: represent unlock progression with license
  objects, `license_level`, `license_previous_id`, `license_fee`, and
  `requires_object_ids`.
- **Temporary or loaned objects**: sponsor activities can grant temporary
  loaned copies of objects. Do not model loan state as a new CSV column.
- **Descriptions and presentation**: use `description_html` paths and
  `config.csv` labels to adapt the interface to the game's vocabulary.
- **Encounters**: use the encounter CSV files only when a turn-based
  challenge adds meaningful gameplay. Attributes, actions, available objects,
  opponents, outcomes, and encounter configuration are separate data.

Do not claim that a generic custom rule will execute unless it maps to one of
these supported mechanisms.
```

---

## Prompt 2: design the data contract before filling rows

```text
PHASE 2 - DATA CONTRACT

Using the approved Phase 1 design, create a data contract before generating
content.

For every CSV file, provide:

- whether it is required or optional;
- its exact header;
- what each column means in this game;
- allowed values or units;
- which IDs it creates;
- which IDs it references;
- at least one representative example row in prose;
- validation rules and likely failure modes.

Then provide:

1. An ID namespace and naming convention.
2. A dependency order for generating the files.
3. A reference map showing every planned cross-file link.
4. A balance budget for starting resources, income, expenses, and progression.
5. A list of content batches so the dataset can be generated incrementally.

Do not fill the entire dataset yet. Ask for approval only if a design choice
cannot be resolved from the game brief; otherwise choose a conservative,
engine-compatible default and state it.
```

### Canonical file roles

The usual dependency order is:

1. `config.csv` - labels, calendar settings, terminology, and global tuning.
2. `player.csv` - numeric player characteristics and starting values.
3. `costs.csv` - reusable amounts.
4. `objects.csv` - objects, licenses, prerequisites, and service references.
5. `quests.csv` - long-term objectives.
6. `events.csv` - player-started and scheduled activities.
7. `event_outcomes.csv` and `event_results.csv` - optional event-specific
   result behavior.
8. `cost_rules.csv` and `cost_rule_conditions.csv` - triggered costs,
   maintenance, damage, service, or other supported consequences.
9. Encounter CSV files - only when encounters are part of the approved design.
10. HTML and image assets - descriptions and presentation content.

The exact headers must come from the repository's current
`DATASET_GENERATION_GUIDE.md` and the uploaded files. If those sources
disagree, report the disagreement instead of guessing.

---

## Prompt 3: generate the dataset in controlled batches

````text
PHASE 3 - CSV GENERATION

Generate the dataset according to the approved data contract.

Start with the smallest complete playable vertical slice:

- configuration and labels;
- starting player characteristics;
- a small set of objects;
- at least one meaningful player action;
- at least one scheduled activity or objective;
- the minimum costs and prerequisites needed to play;
- descriptions only where they improve comprehension.

For each file, output:

FILE: relative/path/to/file.csv
```csv
<complete file contents, including the exact header>
```

After the files, output:

- IDs created;
- references checked;
- assumptions made;
- intentionally empty optional fields;
- known balance concerns;
- a short manual playthrough showing that the slice can start, progress,
  spend and earn resources, and reach a meaningful result.

Do not add placeholder rows such as "TODO object" to production CSVs. If more
content is needed, create the next content batch instead.
````

For a large game, use batches such as:

- Batch A: configuration, player stats, basic economy, and starter content.
- Batch B: the first progression tier and its objects/actions.
- Batch C: mid-game progression, quests, and recurring costs.
- Batch D: advanced content, optional branches, and encounters.
- Batch E: flavor HTML, images, tuning, and accessibility terminology.

---

## Prompt 4: validate the generated files

```text
PHASE 4 - DATASET VALIDATION

Inspect the generated or uploaded CSV files as if you were a strict dataset
validator. Do not rewrite them yet.

Check:

1. Every file has the exact expected header.
2. Every row has the correct number of CSV fields.
3. IDs are unique within their file and use the agreed convention.
4. Every referenced object, cost, player characteristic, event, quest,
   encounter, HTML file, and image exists.
5. Numeric values parse correctly and use valid ranges.
6. Probabilities are between 0 and 1.
7. Prices, costs, fees, and rewards are intentional and non-negative unless
   the schema explicitly permits a negative delta.
8. Calendar days fit inside days_per_year.
9. Semicolon-separated lists contain valid IDs and no accidental whitespace or
   commas.
10. Resolution methods have the required companion fields.
11. License and prerequisite chains do not contain cycles.
12. Recurring costs and income have a sensible frequency.
13. Activities can actually be entered with the available starting inventory.
14. Quests have enough linked activities to produce a meaningful result.
15. Encounter actions have valid attributes, targets, objects, and opponents.
16. HTML and image paths are relative and correspond to supplied assets.
17. The starting state is neither immediately bankrupt nor unable to act,
    unless that is explicitly part of the design.

Return a severity-ranked report:

- ERROR: the engine will reject the dataset or a reference is broken.
- WARNING: the dataset loads but behavior is probably unintended or unbalanced.
- NOTE: an optional improvement or design observation.

For every issue, include the file, row/ID, exact field, reason, and smallest
safe fix. Do not silently apply fixes.
```

When available, run the repository validator after saving the files:

```sh
make dataset-check DATASET_PATH=path/to/dataset
```

For balance and progression, use the headless playtest tool described in
`README.md` and ask the LLM to interpret the results only after the validator
passes.

---

## Prompt 5: iterate without breaking the game

```text
PHASE 5 - TARGETED ITERATION

The uploaded dataset is the current implementation. Apply only the following
requested change:

REQUEST:
{{describe_one_change_precisely}}

Constraints:

- preserve existing IDs and unrelated rows;
- preserve current behavior outside the requested change;
- update every dependent reference;
- do not introduce new columns or unsupported engine logic;
- identify any balance, progression, or migration consequence;
- show a before/after summary;
- return only changed files, unless an unchanged file must be regenerated
  because a reference or header depends on it;
- include a validation report and a short regression checklist.
```

Use one targeted iteration per request, for example:

- “Add a monthly rent cost to owned offices.”
- “Make this object available only after the player owns the permit.”
- “Add a manual final ranking to the championship.”
- “Replace the racing vocabulary with a wizard-academy vocabulary.”
- “Add a turn-based duel using the encounter system.”
- “Make this action recurring once per week and pay a salary.”

---

## Modeling examples for common requirements

### “An object costs money upfront and then has a monthly cost”

Model the upfront purchase in `objects.csv`:

- `price` is the amount paid when acquiring the object.
- Define the monthly amount in `costs.csv`.
- Add a `cost_rules.csv` rule with a `day_elapsed` trigger and an
  `interval_days` appropriate to the game's calendar.
- Add a condition in `cost_rule_conditions.csv` for the object ID or object
  type if the rule should apply only to some owned objects.

If the cost is a service requirement rather than a direct charge, use the
object's cost slot and service interval fields, and choose the supported
service resolution described in the schema.

### “An action can be training, work, trade, study, or sponsorship”

Use an `events.csv` row for each action. The `type` value is the activity
category shown to the game and may be custom text, while the behavior comes
from fields such as:

- `base_cost` for an upfront action cost;
- `stamina_cost` for a resource cost;
- `success_rate` and `resolution_method` for resolution;
- `payout`, `payout_freq_type`, `payout_freq`, and `payout_freq_unit` for
  one-time or recurring income;
- `description_html` for detailed explanation;
- sponsor-specific fields only when the action is actually a sponsor action.

Do not assume a custom `type` automatically creates new runtime behavior.

### “The column X can take these values, each with a different meaning”

Before generating rows, define the field as a documented enum:

```text
Field: resolution_method
Allowed values:
- manual: the player enters the outcome.
- random: the engine rolls using success_rate.
- encounter: the activity opens a configured encounter.
```

Then use only those values in the CSV. If the desired value is not supported,
map it to a supported field or report that code changes are required.

### “An object requires another object or a license”

- Use `requires_object_ids` for one or more object prerequisites.
- Use `license_previous_id` to create a license progression chain.
- Use `license_level` and `required_license_id` where the schema requires a
  level or a specific license.
- Create the prerequisite rows before the rows that reference them.

### “An event can have a chance of a special outcome”

Use the event outcome/result CSV supported by the current schema. Define the
event ID, outcome ID, probability, reward or characteristic deltas, and
message. Validate how the current loader distinguishes general event outcomes
from reported-result outcomes before choosing the file.

### “The game has combat or another turn-based challenge”

Use the encounter files:

- attributes define tracked values and loss conditions;
- actions define requirements, costs, success modifiers, effects, cooldowns,
  and flavor text;
- encounter objects connect inventory items to actions and bonuses;
- opponents define starting attributes, available actions, and strategy;
- outcomes map win/lose/draw to supported consequences;
- configuration defines turn order, turn limits, tiebreakers, retreat, RNG,
  opponent, and mode.

Link an activity to an encounter with its `encounter_id`. Do not simulate a
full combat system through arbitrary event text.

---

## Final handoff prompt

Use this when the dataset is ready for a final review.

```text
FINAL HANDOFF REVIEW

Review the complete uploaded dataset and produce a release handoff.

Return:

1. The game concept and player objective in five sentences or fewer.
2. A file manifest with row counts.
3. The important player characteristics, currencies, objects, actions, quests,
   recurring costs, and encounters.
4. A dependency and progression summary.
5. A new-player walkthrough covering the first meaningful game sequence.
6. A validator-style integrity report.
7. A balance report covering starting resources, likely income, recurring
   expenses, progression speed, and failure recovery.
8. A list of unsupported ideas that are only represented approximately.
9. A list of recommended playtests and the metrics to observe.
10. Exact changes recommended before release, separated into ERROR, WARNING,
    and OPTIONAL.

Do not claim that the dataset is valid without checking every cross-file
reference and every CSV row shape.
```

## Important constraints

- CSV headers and supported behavior are authoritative; prose descriptions do
  not create new runtime features.
- A column's free-form text values are not automatically executable rules.
- Keep IDs stable across iterations so saved games and references remain
  understandable.
- Do not put commas, semicolons, or line breaks in unquoted CSV fields.
- Do not use `null`; use an empty optional field.
- Generate content in manageable batches and validate after each batch.
- If a requested mechanic cannot be represented by the current engine, state
  that clearly and either propose the closest data-only approximation or
  identify the code change that would be required.
