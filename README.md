# Economy Engine

This is a configurable economy engine for games and simulations.

The runtime is domain-neutral. Objects, events, actions, inventory, and visible terminology are loaded from the selected dataset directory.

Each dataset can include:

- `objects.csv` for purchasable objects
- `player.csv` for character/player characteristics such as budget, charisma, or future skills
- `config.csv` for UI labels, including `dealer_name` for the catalog/dealer tab
- `costs.csv` for reusable cost definitions referenced by object `cost_1`, `cost_2`, and so on
- `cost_rules.csv` for rules that generate costs from days, events, actions, or object acquisition
- `cost_rule_conditions.csv` for optional rule conditions
- `events.csv` for scheduled events
- `actions.csv` for one-time or recurring actions
- `config.csv` for UI labels, with `variable,value` columns

The dataset can define `days_per_year` in `config.csv`. The starting age is the
`age` row in `player.csv`. Object service schedules are configured with
`service_1_interval_days` through `service_4_interval_days` in `objects.csv`.

`player.csv` uses `id,name,value` columns. Each row becomes a numeric player
characteristic, and its value is initialized when a new game starts. The default dataset defines `age, Age, 18`, `budget, Budget, 20000`, and
`charisma, Charisma, 1`.

`objects.csv`, `actions.csv`, and `events.csv` may include a `description_html` column.
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
`market_default_sort` in `config.csv` selects `price` or `name`. Objects define
their own optional `availability_days` and `image_path`; matching images under
the dataset folder are shown as fitted market thumbnails.

Daily sickness is configured with `sickness_daily_probability`,
`sickness_recovery_stamina`, and `sickness_final_recovery` in `config.csv`.
Recurring actions can be stopped from the Actions tab. Work actions are jobs,
and only one job may be active at a time. Their
`success_rate` controls the probability of finding/starting the action.

Objects with `lifetime_days` expire automatically. The sample racing dataset
gives the helm and tracksuit a 2,000-day lifetime, and gloves and shoes a
1,000-day lifetime. Licences are not resellable.

Cost rules can be scoped to a specific owned object by adding an `object,id`
condition. The sample dataset uses this for race consumables, annual engine and
gearbox rebuilds, insurance, and one-percent random repair issues for each
vehicle.

`costs.csv` uses `id,name,amount` columns. Its IDs can be named `service_1_id`, `service_2_id`, and so on. Each object stores those IDs in `cost_1`, `cost_2`, and so on rather than embedding service prices, so multiple objects can share the same cost definition.

Cost rules support `day_elapsed`, `event_completed`, `action_completed`, and `object_acquired` triggers. Rules can match IDs or event tags, apply a probability and multiplier, charge immediately when funds are available, or remain pending until paid. Conditions currently support event/action/object/player facts with operators such as `equals`, `contains`, `greater_than`, and `less_than`.

Entering an event charges its entry fee and creates a pending participation. The
`Events` screen provides a text input for the user-entered result. Results matching
`success`, `successful`, `win`, `won`, `1`, `yes`, or `true` receive the event reward;
all other non-empty results are recorded as unsuccessful.

## Current status

Under development, putting together all ideas that come to mind.

## How to test

cargo tauri dev

## Dataset editor

Run `python3 dataset_editor.py` to open the guided dataset editor. It defaults
to creating `dataset_tutorial` and walks
through game settings, player characteristics, objects, events, actions, costs,
and cost rules in that order. The editor uses dropdowns for known units,
operators, trigger types, and payout modes while leaving the data model open
for other game genres. When a CSV does not exist or is empty, the editor
provides generic starter variables and one or more example rows to modify.

## How to compile

cargo tauri build

## Other requirements that may be needed
npx tsc --init # do once to prepare a tsconfig.json
