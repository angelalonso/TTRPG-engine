# TTRPG Engine

This is a configurable TTRPG engine for games and simulations.

The runtime is domain-neutral. Objects, activities, inventory, and visible
terminology are loaded from the selected dataset directory. Scheduled and player-started events are two presentations of the same event model:
each has a type and a resolution method (`manual`, `random`, or `encounter`).

Each dataset can include:

- `objects.csv` for purchasable objects
- `player.csv` for character/player characteristics such as budget, charisma, or future skills
- `config.csv` for UI labels, including `dealer_name` for the catalog/dealer tab
- `costs.csv` for reusable cost definitions referenced by object `cost_1`, `cost_2`, and so on
- `cost_rules.csv` for rules that generate costs from days, events, or object acquisition
- `cost_rule_conditions.csv` for optional rule conditions
- `events.csv` for scheduled, one-time, and recurring events
- `config.csv` for UI labels, with `variable,value` columns

The dataset can define `days_per_year` in `config.csv`. The starting age is the
`age` row in `player.csv`. Object service schedules are configured with
`service_1_interval_days` through `service_4_interval_days` in `objects.csv`.

`player.csv` uses `id,name,value` columns. Each row becomes a numeric player
characteristic, and its value is initialized when a new game starts. The default dataset defines `age, Age, 18`, `budget, Budget, 20000`, and
`charisma, Charisma, 1`.

`objects.csv` and `events.csv` may include a `description_html` column.
Its value is a path relative to the dataset folder, such as `./html/object_1.html`.
The file is loaded into the detail popup when the entry name is selected. The popup's
footer remains a separate UI area for actions such as acquiring an object, starting an
action, entering an event, or recording an event result.

Events also support `duration_value` and `duration_unit` columns. Day and week
durations advance the calendar by the corresponding number of whole days when
the event is entered. Hour and minute durations remain on the same calendar day.
Events can also define `charisma_reward`; successful events award both
`reward_pool` to the budget and `charisma_reward` to the player's charisma.
The sample dataset uses `race` events for cash and charisma rewards and
`track_day` events for small charisma gains without prize money.

Events tagged `social` are non-race sponsor, enthusiast, media, charity, or
owners' events. They can require one or more cars through
`required_object_ids` and award charisma without awarding money. Optional
event-specific outcomes belong in `event_outcomes.csv`, whose columns are
`event_id,outcome_id,probability,reward_pool_delta,charisma_reward_delta,message`.
A row with `outcome_id=failure` is tested after an apparent successful result,
so one dataset can define a 15% mishap for a social event while another omits
the behavior entirely.

Events may be grouped into a quest by setting the same
`quest_id` in `events.csv` and adding a row to `quests.csv` with
`id,name,success_points,failure_points`. The Events screen calculates the
player's points as results are recorded and shows the final quest result
after every round is complete. `required_object_ids` accepts semicolon-separated
base object IDs; a race with `formula_ford;renault_clio_cup` allows either car,
while a single ID requires that specific car.

The inventory screen's Service Bay tab only shows objects whose type is
`vehicle`; the Driver's room tab shows non-car equipment and licences.
Additional inventory tabs can be declared in `config.csv` with matching
`inventory_tab_<id>_name` and `inventory_tab_<id>_types` rows. The type value is
semicolon-separated, for example `inventory_tab_office_types,equipment;license`.
The Racing Market creates one purchase button per object type; optional
`market_category_<type>` labels control the button text.

### Object thumbnails

`market_default_sort` in `config.csv` selects `price` or `name`. Objects define
their own optional `availability_days` and `image_path`; matching images under
the dataset folder are shown as fitted market thumbnails. By default, a
thumbnail is loaded from `./img/<id>.jpeg`, where `<id>` is the object's `id`
from `objects.csv`. For example, the `honda_civic` object uses
`./img/honda_civic.jpeg`. Add a value in the `image_path` column to override
this convention when the filename or format differs.
`market_default_sort` in `config.csv` selects `price` or `name`. Objects define
their own optional `availability_days` and `image_path`; matching images under
the dataset folder are shown as fitted market thumbnails.

Daily sickness is configured with `sickness_daily_probability`,
`sickness_recovery_stamina`, and `sickness_final_recovery` in `config.csv`.
Recurring events can be stopped from the Activities tab. Work events are jobs,
and only one job may be active at a time. Their
`success_rate` controls the probability of finding/starting the event.

Objects with `lifetime_days` expire automatically. The sample racing dataset
gives the helmet and tracksuit a 2,000-day lifetime, and gloves and shoes a
1,000-day lifetime. Licences are not resellable.

Cost rules can be scoped to a specific owned object by adding an `object,id`
condition. The sample dataset uses this for race consumables, annual engine and
gearbox rebuilds, insurance, and one-percent random repair issues for each
vehicle.

`costs.csv` uses `id,name,amount` columns. Its IDs can be named `service_1_id`, `service_2_id`, and so on. Each object stores those IDs in `cost_1`, `cost_2`, and so on rather than embedding service prices, so multiple objects can share the same cost definition.

Cost rules support `day_elapsed`, `event_completed`, and `object_acquired` triggers. Rules can match IDs or event tags, apply a probability and multiplier, charge immediately when funds are available, or remain pending until paid. Conditions currently support event/object/player facts with operators such as `equals`, `contains`, `greater_than`, and `less_than`.

## Encounter system

The optional Encounter System is configured entirely through
`encounter_attributes.csv`, `encounter_actions.csv`, `encounter_objects.csv`,
`encounter_opponents.csv`, `encounter_outcomes.csv`, and
`encounter_config.csv`. It reuses the player's existing attributes and
inventory, supports turn-based encounter moves, opponent strategies, cooldowns,
loss conditions, retreat, and configurable consequences. A player-started event
can reference an encounter through its optional `encounter_id`; the Activities tab
then exposes that encounter using the linked event and configuration labels.

Entering an event charges its entry fee and creates a pending participation. The
`Events` screen provides a text input for the user-entered result. Results matching
`success`, `successful`, `win`, `won`, `1`, `yes`, or `true` receive the event reward;
all other non-empty results are recorded as unsuccessful.

## Current status

Under development, putting together all ideas that come to mind.

## How to test

Use the Makefile for the standard checks:

```sh
make help
make check
make run
```

`make fmt-check` checks Rust formatting. `make check` runs formatting,
strict Rust Clippy, all Rust tests, the frontend
type-check/build, and dataset validation. Validate another dataset with:

```sh
make dataset-check DATASET_PATH=dataset_wizards
```

The same quality gates run in GitHub Actions for pushes and pull requests.

## Automated playtesting and external API

The repository includes a seeded, headless batch simulator:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --bin playtest -- \
  --dataset dataset --seed 42 --runs 1 --max-days 365 \
  --strategy greedy --goal charisma>=100 --verbosity trace
```

Useful batch and tuning examples:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --bin playtest -- \
  --dataset dataset_racing --seed 100 --runs 100 \
  --strategy random --max-days 365 --verbosity summary
cargo run --manifest-path src-tauri/Cargo.toml --bin playtest -- \
  --dataset dataset_wizards --runs 20 --strategy greedy \
  --override event.train.success_rate=0.9 \
  --outcome type:championship=fixed:1 --output runs.json
```

The simulator uses the same engine rules as the application, supports
`random`, `greedy`, and `required-only` strategies, and resolves encounter
actions headlessly. `--seed` makes a run reproducible; batch runs increment
the seed for each run. `--outcome event:<id>=fixed:<rank>` or
`--outcome type:<type>=fixed:<rank>` supplies deterministic manual event
results. Use `--verbosity summary|run|trace`, `--speed paced`, `--pace-ms`,
`--max-turns` (0 uses the default day-based safety cap),
`--override`, `--output`, and `--log` to control reporting and experiments.
Difficulty buckets can be tuned with `--too-easy-below-days`,
`--hard-above-days`, and `--near-impossible-above-days`.
Currently supported numeric overrides are `event.<id>.success_rate` (or
`success_probability`), `action.<id>.success_rate`, `object.<id>.price`, and
`cost_rule.<id>.probability`.

For repeatable experiments, put these settings in a JSON file and pass
`--config playtest.example.json` **after Cargo's `--` separator**. Without
that separator, Cargo interprets `--config` as its own TOML configuration
option. Command-line values take precedence over values in the file. Goals support the original `characteristic>=value` form,
one championship (`championship:<quest_id>`), or a number of trophies at a
level (`championships:level=1,count=2`). The `--unique-paths` option retries
the next seed when a batch produces an identical action/outcome path; leave it
off when measuring the unfiltered random distribution. Trace output includes
the complete eligible possibilities considered on every turn. The JSON
`output` contains each run's ordered `path` list, so a result can be scored or
analysed without parsing console text.

Example configuration:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --bin playtest -- \
  --config playtest.example.json
```

For scripting a live, non-UI game process, run:

```sh
DATASET_PATH=dataset cargo run --manifest-path src-tauri/Cargo.toml --bin game_api
curl http://127.0.0.1:8787/state
curl http://127.0.0.1:8787/events
curl -X POST -d '{"id":"event_id"}' http://127.0.0.1:8787/activity
curl -X POST http://127.0.0.1:8787/advance
```

The API also exposes `GET /health`. Requests are limited to a 16 KiB header
and 1 MiB JSON body because this server is intended for local development,
not public deployment.

Events can be entered with
`POST /event` and `{"event_id":"...","object_id":"..."}` before submitting
their result.

The API also accepts `POST /event-result` with
`{"entry_id":"...","result":"success"}`. It is intentionally a small
localhost-only HTTP server using Rust's standard library.

## Dataset editor

Run `python3 dataset_editor.py` to open the guided dataset editor. It defaults
to creating `dataset_tutorial` and walks
through game settings, player characteristics, objects, events, costs,
and cost rules in that order. The editor uses dropdowns for known units,
operators, trigger types, and payout modes while leaving the data model open
for other game genres. When a CSV does not exist or is empty, the editor
provides generic starter variables and one or more example rows to modify.

For a visual, mockup-driven workflow, run `python3 dataset_gui.py`. This opens a
separate wizard that starts at the Dashboard, lets you add and name inventory
tabs, configure the dealer, create items and reusable costs, then add events
and quests. The live application mockup updates as you work. Object types,
inventory-tab types, service costs, intervals, licensing fields, lifetime,
availability, images, and prerequisites are free-form so the tool is not tied
to the racing dataset. The Events and cost rules step also exposes one-time
and recurring events, sponsor fields, cost triggers, pending/immediate
charging, service resolution, event damage rules, and rule conditions. Use
Export dataset to write the resulting CSV files to the selected folder.

The same tool also has a Colors step. It edits `dataset/colors.csv` (or the
selected export folder's `colors.csv`) using `element_id`, `label`,
`hex_color`, `category`, and `default_hex` columns. `hex_color` and
`default_hex` must be six-digit `#RRGGBB` values. The tool creates the file
with defaults when it is missing, groups entries by category, previews changes
live, supports the native color picker and per-entry/global reset, and writes
atomically. The Tauri app reads this file from the active dataset at startup
through `get_theme_colors`; missing or malformed rows fall back to the built-in
defaults so the app can still launch.

## How to compile

cargo tauri build
