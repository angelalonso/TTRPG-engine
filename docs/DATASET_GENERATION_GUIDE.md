# Dataset generation guide

Use this document as a specification when asking an LLM to generate a new
dataset for the TTRPG Engine. The engine is domain-neutral: the same CSV
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
| `quest_name` | Singular long-term objective label | Any text, for example `Quest` or `Championship` |
| `quest_plural` | Plural long-term objective/tab label | Any text, for example `Quests` or `Championships` |
| `activity_name` | Unified activity tab label | Any text; defaults to `Activities` |
| `currency_symbol` | Currency prefix | Any text, for example `$`, `EUR`, or `credits` |
| `days_per_year` | Calendar length | Positive integer, normally `365` |
| `age_name`, `budget_name`, `day_name`, `year_name` | UI labels | Any text |
| `market_category_<type>` | Label for one object type in the market | Any text |
| `market_default_sort` | Default market ordering | `price` or `name` |
| `inventory_service_bay_name` | Primary inventory tab label | Any text |
| `inventory_service_bay_types` | Object types shown in the primary inventory tab | Semicolon-separated object types |
| `event_log_name`, `character_sheet_name` | Dashboard section labels | Any text |
| `highest_license_name`, `equipment_readiness_name`, `owned_equipment_name` | Dashboard stat labels | Any text |
| `equipment_ready_message`, `equipment_not_ready_message` | Dashboard readiness messages | Any text |
| `event_count_name`, `competitor_name`, `competitor_plural`, `score_name` | Objective/challenge terminology | Any text |
| `dashboard_readiness_object_groups` | Optional dashboard equipment groups; alternatives use `|`, groups use `;` | For example `tool_a|tool_b;uniform` |
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
| `resource_role_currency` | Characteristic used for currency and monetary costs | An ID declared in `player.csv` |
| `resource_role_recovery` | Characteristic used for activities and recovery | An ID declared in `player.csv` |
| `resource_role_age` | Optional characteristic used as the starting age | An ID declared in `player.csv` |
| `terminal_currency_below_zero` | Whether a negative currency value ends a run | `true` or `false` |
| `terminal_recovery_at_or_below_zero` | Whether an empty recovery resource ends a run | `true` or `false` |

Resource roles are optional. In the version-1 compatibility adapter, a
dataset that declares no roles maps `budget`, `stamina`, and `age` when those
characteristics exist; these legacy names are not created automatically.
Otherwise all rule references must use IDs declared in `player.csv`. A missing
role means that resource is not a terminal condition, so a dataset can omit
currency, recovery, and age entirely.

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
`reputation`, `health`, or `skill`. Prefer dataset-specific IDs such as
`coins` and `energy`, then assign them with the optional resource-role keys
above. Characteristics are never created by a daily tick or by an effect
targeting an undeclared ID.

## `objects.csv`

Header:

```csv
id,type,name,price,policy_version,buyable,sellable,reward_only,unique,max_owned,use_policy,consume_policy,cost_1,cost_2,cost_3,cost_4,cost_5,cost_6,cost_7,cost_8,cost_9,cost_10,cost_11,cost_12,cost_13,cost_14,cost_15,service_1_interval_days,service_2_interval_days,service_3_interval_days,service_4_interval_days,service_5_interval_days,service_6_interval_days,service_7_interval_days,service_8_interval_days,service_9_interval_days,service_10_interval_days,service_11_interval_days,service_12_interval_days,service_13_interval_days,service_14_interval_days,service_15_interval_days,resale_initial_percent,resale_annual_percent,resale_min_percent,description_html,license_level,license_previous_id,requires_object_ids,license_fee,lifetime_days,availability_days,image_path
```

An object is something the player can buy, own, use, service, or take to an
event. The built-in UI groups objects by `type`.

| Column group | Meaning | Possible values |
|---|---|---|
| `id` | Internal object ID | Unique string |
| `type` | Object category | Any string; built-in UI treats `vehicle`, `equipment`, `license`, and `item` specially |
| `name` | Display name | Any text |
| `price` | Purchase price | Number zero or greater |
| `policy_version` | Object policy contract version | `1` for legacy-compatible defaults, `2` for explicit policy semantics |
| `buyable` | Direct acquisition permission | `true` or `false`; defaults to `true` |
| `sellable` | Resale permission | `true` or `false`; defaults to `true` |
| `reward_only` | Acquisition source restriction | `true` means rules may grant the object but direct purchase is rejected |
| `unique` | Single-instance ownership policy | `true` or `false`; implies `max_owned=1` |
| `max_owned` | Maximum owned instances | Whole number; `0` means unlimited |
| `use_policy` | Whether an owned object may be used | `unrestricted`, `usable`, or `not_usable` |
| `consume_policy` | Consumption timing | `never`, `on_use`, or `on_acquire` |
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

Policy columns are optional for version-1 datasets. Missing values preserve the
legacy racing defaults: licences remain unique and non-sellable, trophy and
achievement-style reward IDs cannot be bought or sold, and ordinary objects
remain repeatedly buyable and sellable. Version-2 rows should set the policy
columns explicitly; invalid combinations such as `reward_only=true` with
`buyable=true`, or `unique=true` with `max_owned>1`, fail dataset validation.

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
| `trigger_type` | What activates the rule | `day_elapsed`, `event_completed`, `event_completed`, or `object_acquired` |
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
| `subject_type` | Kind of fact to inspect | `object`, `event`, `event`, or `player` |
| `subject_ref` | Field within that fact | Examples: `id`, `type`, `tags`, or a characteristic ID |
| `operator` | Comparison operation | `equals`, `contains`, `greater_than`, or `less_than` |
| `value` | Value to compare against | Text or number represented as text |

All conditions attached to a rule must match for the rule to apply. Multiple
conditions therefore behave like logical AND.

## Activities and resolution methods

Player-started and scheduled entries are both events: the player starts or enters
one, and a resolution method determines how its outcome is produced. The
loader exposes a unified `catalog.activities` view for the UI while retaining
the specialized CSV files for backwards-compatible datasets.

Supported `resolution_method` values are:

- `manual`: the player enters the result, typically for a scheduled event.
- `random`: the engine rolls against `success_rate`.
- `encounter`: the activity starts the configured turn-based encounter.

## `events.csv`

Header:

```csv
id,name,type,base_cost,stamina_cost,risk_factor,success_rate,payout,payout_freq_type,payout_freq,payout_freq_unit,description_html,sponsor_quest_id,sponsor_object_id,sponsor_payouts,sponsor_equipment_ids,encounter_id,resolution_method
```

Player-started events are activities such as jobs, trades, study, projects,
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
| `encounter_id` | Encounter started by this activity | Existing `encounter_config.csv` ID; required for `encounter` |
| `resolution_method` | How the outcome is calculated | `manual`, `random`, or `encounter`; defaults to `random` |

Sponsor actions are championship-specific. A successful sponsor action
automatically joins the referenced quest (while still enforcing its required
licence), creates loaned copies of `sponsor_object_id` and every object in
`sponsor_equipment_ids`, keeps them until the end of the current year, and pays
the configured amount after each matching championship event. Loaned objects
cannot be sold.

## `events.csv`

Header:

```csv
id,name,day_of_year,entry_fee,reward_pool,charisma_reward,duration_value,duration_unit,tags,description_html,required_license_id,required_object_ids,quest_id,type,resolution_method,success_rate,encounter_id
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
| `type` | Activity type shown by the UI | `race`, `social`, `track_day`, or any custom text |
| `resolution_method` | How the outcome is calculated | `manual`, `random`, or `encounter`; defaults to `manual` |
| `success_rate` | Chance used by `random` resolution | Number from `0` to `1`; defaults to `1` |
| `encounter_id` | Encounter started by this activity | Existing `encounter_config.csv` ID; required for `encounter` |

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
4. Define actions in `events.csv`, including sponsor fields only for sponsor
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
`Garage`, `Activities`, and `Races`.

## Encounter CSV files

The optional Encounter System is split across six CSV files:

- `encounter_attributes.csv` defines existing player attributes used by an
  encounter, their bounds, visibility, and loss conditions.
- `encounter_events.csv` defines available moves, requirements, resource
  costs, success modifiers, effects, cooldowns, and flavor text.
- `encounter_objects.csv` connects existing inventory objects to encounter
  actions and success-rate bonuses.
- `encounter_opponents.csv` defines opponent profiles, starting attributes,
  available action IDs, and strategy.
- `encounter_outcomes.csv` maps `win`, `lose`, or `draw` to consequences such
  as object grants, attribute changes, or a configured custom action.
- `encounter_config.csv` defines the display label, turn order, turn limit,
  tiebreaker, retreat policy, RNG mode, and opponent ID.

Add `encounter_id` to an `events.csv` row to make that action the entry point
for the corresponding encounter. The sample dataset links one Honda sponsor
action to a showdown; winning it activates that specific sponsor through a
`custom_event` consequence.

## Versioned generic authoring contract

The generic rule contract is versioned independently from the legacy racing
CSV layout. New datasets should declare:

```csv
variable,value
authoring_contract_version,1
```

Version 1 uses the following concepts. A **requirement** is a gate: it must
evaluate true before an operation is available. A **modifier** changes a
numeric value such as success probability, cost, or reward. An **effect**
changes state after the operation resolves. These concepts must not be
combined; for example, an object count used as a success bonus is a modifier,
not an acquisition requirement.

### Requirement groups and selectors

Requirement rows form a tree. Every group has one of these operators:

- `all`: every child must pass;
- `any`: at least one child must pass;
- `not`: exactly one child is negated.

The supported version-1 selector subjects are:
`player_characteristic`, `object_definition`, `object_type`,
`object_count`, `event_history`, `quest_status`, `active_event`,
`calendar`, and `fact`. Comparisons use `equals`, `not_equals`,
`greater_than`, `greater_or_equal`, `less_than`, `less_or_equal`,
`contains`, or `starts_with`. Numeric comparisons must parse successfully;
invalid numbers are dataset errors, never zero.

An object-count condition is evaluated by object **definition ID** or object
type and counts owned instances only after its filters are applied. The
contract must state whether loaned, rented, expired, or unavailable instances
count. A typical requirement is:

```text
all(
  player_characteristic(skill) >= 5,
  object_count(definition=ingredient, usable=true) >= 3
)
```

### Triggers, effects, and ordering

Version-1 triggers are `event_started`, `event_entered`, `event_completed`,
`object_acquired`, `object_sold`, `encounter_completed`, `quest_joined`,
`quest_completed`, and `day_elapsed`.

Effects are applied in deterministic order: validate requirements, resolve the
base result, apply modifiers, apply characteristic changes, apply object
grants/removals/consumption, update availability or service state, then record
history and repeat receipts. A failed or invalid effect aborts the operation
before its pending attempt is removed.

Supported effect operations are:
`add_characteristic`, `set_characteristic`, `multiply_characteristic`,
`grant_object`, `remove_object`, `consume_object`,
`set_object_availability`, `set_object_service`, `start_event`,
`join_quest`, and `leave_quest`.

### Ownership, transfer, and repetition

Object definitions may declare `buyable`, `reward_only`, `unique`,
`max_owned`, `sellable`, or `non_sellable` policy. Transfer terms are
`none`, `loan`, `rental`, and `returnable`. A reward-only object can be
granted by an event but cannot be purchased; a non-sellable reward cannot be
sold. These policies apply identically to listings, execution, previews, and
plugins.

Quest participation is identified by a quest-run ID, not merely by the quest
definition ID. Event attempts, object grants, recurring income, and effects
must each declare their repetition key. A repeated trigger with the same key
is ignored or rejected according to the dataset policy; it must never pay
twice accidentally.

Resource roles are declarations, not mandatory names. A dataset may designate
one characteristic as currency, another as action energy, and another as age
or time. New rules reference the declared characteristic ID. Version-1
datasets retain compatibility mappings for `budget`, `stamina`, `charisma`,
and `age`; new datasets must not rely on those names.

### Dataset-selected plugins

Optional extensions are declared in `plugins.csv`; the engine does not
implicitly launch Python or any other racing-specific plugin. The manifest
columns are:

```csv
id,entrypoint,protocol_version,capability,result_schema,result_schema_version,required,dispatch
cookbook,plugins/cookbook.py,1,provide_result;evaluate_custom_fact,generic_result,1,true,cooking
```

`entrypoint` is relative to the selected dataset and must not be absolute or
escape it with `..`. Supported protocol and result-schema versions are
currently both `1`. `capability` values are `normalize_result`,
`evaluate_custom_fact`, and `provide_result`. A required plugin must have a
valid entrypoint; missing optional entrypoints are reported as warnings.
Events may select a manifest with their optional `plugin_id` column. Unknown
plugin references, capabilities, protocol versions, schema versions, and
malformed paths are validation errors.

The typed protocol boundary consists of a request containing a protocol
version, plugin ID, operation, and payload, and a response containing only
facts, results, and proposed typed effects. Responses must repeat the
manifest's plugin ID, protocol version, and result-schema version. Unknown
JSON fields are rejected. Plugins may propose effects for engine validation,
but cannot return arbitrary game state. Transport, subprocess execution,
timeouts, and cancellation are intentionally host responsibilities and are
not part of this contract yet.

### Minimal complete example

The following rows describe a cooking event that consumes three ingredients,
grants a non-sellable dish, and pays recurring income after a successful quest:

```csv
# requirements.csv
id,group_id,group_operator,subject,selector,operator,value
meal_ingredients,meal_gate,all,object_count,definition=ingredient,greater_or_equal,3

# effects.csv
id,trigger,operation,target,value,repeat_key
consume_ingredients,event_completed,consume_object,ingredient,3,meal_attempt
grant_dish,event_completed,grant_object,dish,1,meal_attempt
start_income,event_completed,start_event,weekly_catering,1,meal_attempt
```

The corresponding `dish` object uses `reward_only;non_sellable`, while
`weekly_catering` declares a recurring payout and its own active-event
repetition key. The exact tables above are the version-2 extension surface;
version-1 datasets continue using the existing specialized CSV files and are
adapted without changing their meaning.

Invalid examples include a `not` group with two children, a comparison against
an unknown characteristic or object definition, a negative or fractional
object quantity, an unknown effect target, a cyclic follow-up event, or a
probability outside `0..1`. Each must produce a validation diagnostic naming
the source row and field.

## Capability discovery and previews

The engine is the authoritative source for supported tables, fields, enum
values, requirement subjects, effects, plugin capabilities, and expression
functions. Editors and other tools can retrieve that contract with:

```sh
cargo run --quiet --manifest-path src-tauri/Cargo.toml \
  --bin validate_dataset -- --capabilities
```

For staged, non-mutating checks use the preview CLI:

```sh
cargo run --quiet --manifest-path src-tauri/Cargo.toml \
  --bin dataset_preview -- --dataset ./my-dataset --validate
cargo run --quiet --manifest-path src-tauri/Cargo.toml \
  --bin dataset_preview -- --dataset ./my-dataset \
  --seed 42 --explain 'object_count:ingredient==3'
```

Preview commands never write saves or configuration. A missing or unsupported
plugin is reported as a validation error; it is not silently replaced with a
successful result. The Python editor uses the same capability JSON for table
discovery and field explanations, with legacy fields retained only as an
explicit compatibility fallback.

The repository includes small proof fixtures at
`tests/fixtures/pony_stable` and `tests/fixtures/cooking`. They deliberately
use `coins`, `energy`, and domain-neutral event/object names rather than the
legacy `budget`, `stamina`, or racing vocabulary. Validate them with the same
`validate_dataset` command before using them in a playtest.
