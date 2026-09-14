# Dataset generation guide

Use this document as a specification when asking an LLM to generate a new
dataset for the Economy Engine. The engine is domain-neutral: the same CSV
files can describe a racing career, a shop, a fantasy guild, a sports team, a
school, a business, or another simulation.

Create a new folder containing the CSV files below. HTML descriptions are
optional but strongly recommended for objects, actions, events, and quests.
Every HTML path is relative to the dataset folder, for example
`./html/object.html`.

## General CSV rules

- The first row is the header and must use the exact column names documented
  below.
- Use UTF-8 CSV. Quote a value when it contains a comma, quote, or line break.
- IDs should be stable, unique, lowercase snake_case where practical, and
  should not contain commas or semicolons.
- Numeric fields use plain decimal numbers. Use `0` for a numeric value that is
  intentionally zero.
- Empty fields are allowed where the column is optional. Do not use `null`.
- Boolean values are not used in the dataset CSVs; use numeric values, IDs, or
  empty fields instead.
- Lists inside one field use semicolons, such as
  `vehicle;equipment` or `car_a;car_b`.
- Paths use forward slashes and are relative to the dataset folder.
- Create all referenced IDs before using them in another CSV.

## `config.csv`

Header:

```csv
variable,value
```

Each row defines a configurable label or numeric setting. The `variable` is a
string key and `value` is text. Unknown keys are preserved and can be used by
the UI or future extensions.

Common keys:

| Key | Meaning | Possible values |
|---|---|---|
| `application_name` | Application title | Any text |
| `object_name` | Singular object label | Any text, for example `Book` |
| `object_plural` | Plural object label | Any text, for example `Books` |
| `inventory_name` | Inventory tab label | Any text |
| `dealer_name` | Market/store tab label | Any text |
| `event_name` | Singular event label | Any text |
| `event_plural` | Plural event label | Any text |
| `action_name` | Action tab label | Any text |
| `currency_symbol` | Currency prefix | Any text, for example `$`, `EUR`, or `credits` |
| `days_per_year` | Calendar length | Positive integer, normally `365` |
| `age_name`, `budget_name`, `day_name`, `year_name` | UI labels | Any text |
| `market_category_<type>` | Label for one object type in the market | Any text |
| `market_default_sort` | Default market ordering | `price` or `name` |
| `inventory_service_bay_name` | Built-in vehicle inventory tab | Any text |
| `inventory_tab_<id>_name` | Additional inventory tab label | Any text |
| `inventory_tab_<id>_types` | Types shown in that tab | Semicolon-separated object types |
| `speed_icon_*`, `settings_icon`, `save_icon`, `load_icon` | UI asset paths | Relative or packaged asset paths |
| `daily_stamina_recovery` | Stamina restored per normal day | Number |
| `weekend_stamina_recovery` | Stamina restored on weekend days | Number |
| `sickness_daily_probability` | Daily sickness chance | Number from `0` to `1` |
| `sickness_recovery_stamina` | Stamina after initial sickness recovery | Number |
| `sickness_final_recovery` | Stamina after final sickness recovery | Number |
| `sickness_event_name` | Sickness alert title | Any text |
| `sickness_event_message` | Sickness alert text | Any text |

## `player.csv`

Header:

```csv
id,name,value,min_value,max_value
```

Each row creates one numeric player characteristic.

| Column | Meaning | Possible values |
|---|---|---|
| `id` | Internal characteristic ID | Unique string, for example `budget` or `reputation` |
| `name` | Display name | Any text |
| `value` | Starting value | Number |
| `min_value` | Lower clamp | Number or empty for no lower bound |
| `max_value` | Upper clamp | Number or empty for no upper bound |

Typical characteristics include `age`, `budget`, `stamina`, `charisma`,
`reputation`, `health`, or `skill`. `budget` and `stamina` have special
behavior in the current engine; other characteristics are generic numeric
values.

## `objects.csv`

Header:

```csv
id,type,name,price,cost_1,cost_2,cost_3,cost_4,cost_5,cost_6,cost_7,cost_8,cost_9,cost_10,cost_11,cost_12,cost_13,cost_14,cost_15,service_1_interval_days,service_2_interval_days,service_3_interval_days,service_4_interval_days,service_5_interval_days,service_6_interval_days,service_7_interval_days,service_8_interval_days,service_9_interval_days,service_10_interval_days,service_11_interval_days,service_12_interval_days,service_13_interval_days,service_14_interval_days,service_15_interval_days,resale_initial_percent,resale_annual_percent,resale_min_percent,description_html,license_level,license_previous_id,requires_object_ids,license_fee,lifetime_days,availability_days,image_path
```

An object is something the player can buy, own, use, service, or take to an
event. The built-in UI groups objects by `type`.

| Column group | Meaning | Possible values |
|---|---|---|
| `id` | Internal object ID | Unique string |
| `type` | Object category | Any string; built-in UI treats `vehicle`, `equipment`, `license`, and `item` specially |
| `name` | Display name | Any text |
| `price` | Purchase price | Number zero or greater |
| `cost_1` ... `cost_15` | IDs of recurring/service costs | Existing IDs from `costs.csv`, or empty |
| `service_1_interval_days` ... `service_15_interval_days` | Service interval for the matching cost | Whole number of days; `0` disables that interval |
| `resale_initial_percent` | Fraction of purchase price immediately resellable | Number from `0` to `1` |
| `resale_annual_percent` | Multiplier applied per elapsed year | Usually `0` to `1` |
| `resale_min_percent` | Lowest resale fraction | Number from `0` to `1` |
| `description_html` | Detail page path | Relative HTML path, or empty |
| `license_level` | Required/provided license level | Whole number; `0` means none |
| `license_previous_id` | Previous license required | Existing object ID, or empty |
| `requires_object_ids` | Purchase prerequisites | Semicolon-separated object IDs, or empty |
| `license_fee` | Fee used when the object is a license | Number; usually `0` for non-licenses |
| `lifetime_days` | Normal ownership lifetime | Whole number; `0` means no normal expiry |
| `availability_days` | Days after purchase before usable | Whole number; `0` means immediately available |
| `image_path` | Market thumbnail override | Relative image path; empty uses `./img/<id>.jpeg` |

The runtime also supports loaned objects. `loaned` is a runtime property, not
an `objects.csv` column: sponsor actions create a temporary copy of an object,
mark it loaned, and remove it when its expiration day is reached.

For a thumbnail, put an image at `./img/<id>.jpeg`. If the filename or format
does not match the ID convention, set `image_path`, for example
`./img/special-cover.png`.

## `costs.csv`

Header:

```csv
id,name,amount
```

Defines reusable money costs referenced by object service slots or cost rules.

| Column | Meaning | Possible values |
|---|---|---|
| `id` | Internal cost ID | Unique string |
| `name` | Display name | Any text |
| `amount` | Money charged | Number zero or greater |

## `cost_rules.csv`

Header:

```csv
id,cost_id,trigger_type,trigger_ref,amount_multiplier,probability,interval_days,charge_mode,resolution_mode,pending_message,message,damage_type,unavailable_days,event_interval,no_event_days
```

Cost rules create charges or object-service requirements when something happens.

| Column | Meaning | Possible values |
|---|---|---|
| `id` | Rule ID | Unique string |
| `cost_id` | Cost to apply | Existing `costs.csv` ID |
| `trigger_type` | What activates the rule | `day_elapsed`, `event_completed`, `action_completed`, or `object_acquired` |
| `trigger_ref` | Optional matching ID/tag/reference | Existing ID, tag, or empty |
| `amount_multiplier` | Multiplier applied to the cost | Number |
| `probability` | Chance of applying the rule | Number from `0` to `1` |
| `interval_days` | Repeat interval for day rules | Whole number; `0` means no interval restriction |
| `charge_mode` | How the charge is handled | `immediate` |
| `resolution_mode` | Result of the rule | `charge`, `object_service`, or another supported engine mode |
| `pending_message` | Message when a service charge is pending | Any text or empty |
| `message` | Message after a charge/service is applied | Any text or empty |
| `damage_type` | Service/damage category | Any string or empty |
| `unavailable_days` | Days an object becomes unavailable | Whole number |
| `event_interval` | Apply every N matching events | Whole number; `0` means every matching event |
| `no_event_days` | Require this many days since the last race/event | Whole number; usually `0` |

Use `cost_rule_conditions.csv` to limit a rule to particular objects, object
types, actions, events, or player facts.

## `cost_rule_conditions.csv`

Header:

```csv
rule_id,subject_type,subject_ref,operator,value
```

| Column | Meaning | Possible values |
|---|---|---|
| `rule_id` | Rule being constrained | Existing `cost_rules.csv` ID |
| `subject_type` | Kind of fact to inspect | `object`, `event`, `action`, or `player` |
| `subject_ref` | Field within that fact | Examples: `id`, `type`, `tags`, or a characteristic ID |
| `operator` | Comparison operation | `equals`, `contains`, `greater_than`, or `less_than` |
| `value` | Value to compare against | Text or number represented as text |

All conditions attached to a rule must match for the rule to apply. Multiple
conditions therefore behave like logical AND.

## `actions.csv`

Header:

```csv
id,name,type,base_cost,stamina_cost,risk_factor,success_rate,payout,payout_freq_type,payout_freq,payout_freq_unit,description_html,sponsor_quest_id,sponsor_object_id,sponsor_payouts,sponsor_equipment_ids
```

Actions are player-started activities such as jobs, trades, study, projects,
or sponsorship applications.

| Column | Meaning | Possible values |
|---|---|---|
| `id` | Internal action ID | Unique string |
| `name` | Display name | Any text |
| `type` | Action category | `work`, `trade`, `sponsor`, or any custom text |
| `base_cost` | Money paid to start | Number zero or greater |
| `stamina_cost` | Stamina spent to start | Number zero or greater |
| `risk_factor` | Available risk metadata | Number; custom actions may use it |
| `success_rate` | Chance the action succeeds | Number from `0` to `1` |
| `payout` | One-time or recurring payout | Number |
| `payout_freq_type` | Payout lifecycle | `once` or `recurring` |
| `payout_freq` | Frequency number | Whole number |
| `payout_freq_unit` | Frequency unit | `day`, `week`, `month`, or `year` |
| `description_html` | Detail page path | Relative HTML path, or empty |
| `sponsor_quest_id` | Championship targeted by a sponsor action | Existing quest ID; required for `sponsor` |
| `sponsor_object_id` | Object loaned by a sponsor | Existing object ID; required for `sponsor` |
| `sponsor_payouts` | Result-based sponsor payments | Semicolon-separated entries such as `1:5000;2:3000;default:500` |
| `sponsor_equipment_ids` | Additional equipment loaned by a sponsor | Semicolon-separated existing object IDs |

Sponsor actions are championship-specific. A successful sponsor action
automatically joins the referenced quest (while still enforcing its required
licence), creates loaned copies of `sponsor_object_id` and every object in
`sponsor_equipment_ids`, keeps them until the end of the current year, and pays
the configured amount after each matching championship event. Loaned objects
cannot be sold.

## `events.csv`

Header:

```csv
id,name,day_of_year,entry_fee,reward_pool,charisma_reward,duration_value,duration_unit,tags,description_html,required_license_id,required_object_ids,quest_id
```

Events are scheduled calendar activities. They can be races, social events,
meetings, shows, jobs, appointments, or any other activity that uses an owned
object.

| Column | Meaning | Possible values |
|---|---|---|
| `id` | Internal event ID | Unique string |
| `name` | Display name | Any text |
| `day_of_year` | Scheduled calendar day | Whole number from `1` to `days_per_year` |
| `entry_fee` | Money paid when entering | Number zero or greater |
| `reward_pool` | Money awarded after success | Number zero or greater |
| `charisma_reward` | Charisma awarded after success | Number; normally zero or positive |
| `duration_value` | Duration amount | Whole or decimal number |
| `duration_unit` | Duration unit | `minutes`, `hours`, `days`, or `weeks` |
| `tags` | Semicolon-separated categories | Any tags; `race`, `track_day`, `championship`, and `social` have built-in behavior |
| `description_html` | Detail page path | Relative HTML path, or empty |
| `required_license_id` | License required to enter | Existing object ID, or empty |
| `required_object_ids` | Eligible owned objects | Semicolon-separated IDs; any listed ID qualifies |
| `quest_id` | Championship/quest containing this event | Existing quest ID, or empty |

Events tagged `social` are displayed as Social events. They do not update race
wear/damage tracking. A successful social event normally awards
`charisma_reward`. Optional event-specific outcomes are defined in
`event_outcomes.csv`, with columns
`event_id,outcome_id,probability,reward_pool_delta,charisma_reward_delta,message`.
For example, a `failure` row with probability `0.15` gives one event its own
mishap chance and message; events without such a row have no mishap behavior.

## `quests.csv`

Header:

```csv
id,type,name,success_points,failure_points,join_fee,required_license_id,description_html,championship_rewards,driver_names
```

Quests group events into a tracked objective. A championship is a quest whose
`type` is `championship`.

| Column | Meaning | Possible values |
|---|---|---|
| `id` | Internal quest ID | Unique string |
| `type` | Quest category | `championship`, `quest`, `generic`, or custom text |
| `name` | Display name | Any text |
| `success_points` | Default success points metadata | Number |
| `failure_points` | Failure points metadata | Number |
| `join_fee` | Money paid to join | Number zero or greater |
| `required_license_id` | License required to join | Existing object ID, or empty |
| `description_html` | Detail page path | Relative HTML path, or empty |
| `championship_rewards` | Final-position prizes | Semicolon-separated `position:amount` entries |
| `driver_names` | Optional configured competitors | Semicolon-separated names |

Every event with a given `quest_id` belongs to that quest. Championship events
also accept finishing positions and competitor positions so the UI can compute
standings.

## Cross-file generation checklist

When generating a new dataset:

1. Decide the player characteristics and add them to `player.csv`.
2. Define reusable costs in `costs.csv`.
3. Define objects in `objects.csv`, using only existing cost and prerequisite
   IDs.
4. Define actions in `actions.csv`, including sponsor fields only for sponsor
   actions.
5. Define quests in `quests.csv`.
6. Define scheduled events in `events.csv`, linking only to existing object,
   license, and quest IDs.
7. Add `cost_rules.csv` and `cost_rule_conditions.csv` for recurring charges,
   failure consequences, object maintenance, or other triggered effects.
8. Add the referenced HTML files and optional images.
9. Check that all IDs are unique, every reference resolves, every
   `day_of_year` is within the configured year, and every CSV row has the same
   number of fields as its header.

## Minimal example for a different game

For a fantasy academy dataset:

```csv
# objects.csv
spellbook,item,Apprentice Spellbook,100,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,./html/spellbook.html,0,,,,0,0,./img/spellbook.jpeg
```

Use a normal CSV row without the comment:

```csv
spellbook,item,Apprentice Spellbook,100,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,./html/spellbook.html,0,,,,0,0,./img/spellbook.jpeg
```

Then create events such as `exam_day` requiring `spellbook`, actions such as
`study`, and a quest such as `first_year_exams`. Rename labels in
`config.csv` so the UI says `Academy`, `Lessons`, and `Exams` instead of
`Garage`, `Actions`, and `Races`.

## Encounter CSV files

The optional Encounter System is split across six CSV files:

- `encounter_attributes.csv` defines existing player attributes used by an
  encounter, their bounds, visibility, and loss conditions.
- `encounter_actions.csv` defines available moves, requirements, resource
  costs, success modifiers, effects, cooldowns, and flavor text.
- `encounter_objects.csv` connects existing inventory objects to encounter
  actions and success-rate bonuses.
- `encounter_opponents.csv` defines opponent profiles, starting attributes,
  available action IDs, and strategy.
- `encounter_outcomes.csv` maps `win`, `lose`, or `draw` to consequences such
  as object grants, attribute changes, or a configured custom action.
- `encounter_config.csv` defines the display label, turn order, turn limit,
  tiebreaker, retreat policy, RNG mode, and opponent ID.

Add `encounter_id` to an `actions.csv` row to make that action the entry point
for the corresponding encounter. The sample dataset links one Honda sponsor
action to a showdown; winning it activates that specific sponsor through a
`custom_event` consequence.
