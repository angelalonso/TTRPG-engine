# Creating a Wizard Game with the Dataset Editor

The economy engine is driven by CSV files, so a wizard game can reuse the same
calendar, inventory, events, actions, costs, and save system without changing
the Rust or React code.

## 1. Create a dataset

Run:

```bash
python3 dataset_editor.py
```

Choose a new folder, such as `wizard_dataset`, and work through the sections.
The editor writes the same files used by the racing example:

| File | Wizard-game use |
|---|---|
| `config.csv` | Game title, labels, currency, calendar, and inventory tabs |
| `player.csv` | Mana, health, reputation, gold, or spell power |
| `objects.csv` | Wands, robes, familiars, spell books, and ingredients |
| `events.csv` | Exams, duels, quests, tournaments, and festivals |
| `quests.csv` | A wizard league, school tournament, or other multi-event quest |
| `actions.csv` | Studying, brewing, trading, or practising magic |
| `costs.csv` | Potion ingredients, repairs, tuition, and recurring expenses |
| `cost_rules.csv` | Costs caused by time, events, actions, or acquisitions |

Start the game with:

```bash
DATASET_PATH=./wizard_dataset cargo tauri dev
```

## 2. Define the player

In `player.csv`, create characteristics with numeric values:

```csv
id,name,value,min_value,max_value
budget,Gold,500,0,
mana,Mana,100,0,100
health,Health,80,0,100
reputation,Reputation,0,0,100
```

Finite `max_value` values automatically produce coloured bars on the
dashboard. Mana and health therefore show a green, yellow, or red remaining
amount.

## 3. Add magical objects

Objects use a generic type field, so the same catalogue can contain several
categories:

```csv
id,type,name,price,description_html,lifetime_days
oak_wand,wand,Oak Wand,120,./html/oak_wand.html,0
apprentice_robe,equipment,Apprentice Robe,80,./html/apprentice_robe.html,0
phoenix_feather,ingredient,Phoenix Feather,250,./html/phoenix_feather.html,30
```

An object can require other objects with `requires_object_ids`, for example a
master wand may require an apprentice wand. Use `lifetime_days` for temporary
charms, festival passes, or ingredients.

To create inventory tabs, add matching rows to `config.csv`:

```csv
inventory_tab_study_name,Study
inventory_tab_study_types,wand;spellbook
inventory_tab_potions_name,Potions
inventory_tab_potions_types,ingredient;potion
```

The default inventory tab shows `vehicle` objects as the Garage. For a wizard
game, configure the visible labels so that the same tab represents the
appropriate collection, or use additional tabs for separate collections.

## 4. Add events and requirements

Events have a scheduled `day_of_year`, an entry fee, rewards, and a result
entered by the player. A wizard duel can require a specific wand or one of
several wands:

```csv
id,name,day_of_year,entry_fee,reward_pool,charisma_reward,duration_value,duration_unit,tags,description_html,required_license_id,required_object_ids,quest_id
school_duel,School Duel,40,25,150,5,1,day,duel,./html/event.html,,oak_wand;elder_wand,
```

`required_object_ids` is semicolon-separated and means “one of these objects”.
`required_license_id` can represent an academic licence or rank.

## 5. Joinable wizard quests

Quests are multi-event goals. A quest with `type=championship` can charge a
joining fee and require a licence:

```csv
id,type,name,success_points,failure_points,join_fee,required_license_id,description_html
dueling_league,championship,Inter-School Dueling League,10,0,100,novice_license,./html/quest.html
```

Give several events the same `quest_id`. Those events are highlighted and
cannot be entered until the player joins from the quest details. The Events
screen calculates points and shows the final result after all rounds. Other
quest types can reuse the same structure for story arcs, apprenticeships, or
multi-step investigations.

## 6. Actions and economy

Actions can model wizard activities:

```csv
id,name,type,base_cost,stamina_cost,risk_factor,success_rate,payout,payout_freq_type,payout_freq,payout_freq_unit,description_html
study_spells,Study Spellcraft,study,0,5,0,0.85,0,once,0,day,./html/action.html
brew_potions,Brew Potions,trade,20,8,0.2,0.75,90,once,0,day,./html/action.html
```

Use cost rules for monthly tuition, ingredient consumption, damage to a wand,
or penalties after failed duels. An `object_service` rule can turn a wand or
familiar into a service task shown in the inventory screen.

## 8. Detailed CSV reference

### `config.csv`

Configuration is a string key/value table. It controls labels and general
rules without code changes:

```csv
application_name,Arcane Academy
currency_symbol,g
days_per_year,360
inventory_tab_study_name,Study Desk
inventory_tab_study_types,wand;spellbook
```

Use `days_per_year` for a fictional calendar, `currency_symbol` for gold or
credits, and `inventory_tab_<id>_name` plus
`inventory_tab_<id>_types` for additional inventory screens.

### `player.csv`

Each row defines a numeric player variable:

```csv
id,name,value,min_value,max_value
budget,Gold,500,0,
mana,Mana,100,0,100
health,Health,80,0,100
reputation,Reputation,0,0,100
```

`value` is the starting value. `min_value` and `max_value` constrain the
variable; finite maximum values also produce dashboard bars. Use variables for
resources, skills, reputation, fatigue, or story counters.

### `objects.csv`

Objects are catalogue entries that can be purchased or granted:

```csv
id,type,name,price,description_html,lifetime_days
oak_wand,wand,Oak Wand,120,./html/oak_wand.html,0
temporary_charm,charm,Temporary Charm,40,./html/charm.html,30
```

`type` controls grouping and inventory tabs. `lifetime_days=0` means permanent;
otherwise the inventory shows the remaining days and the object expires.
`requires_object_ids` is a semicolon-separated prerequisite list.

### `actions.csv`

Actions are player-initiated activities. `type` is a free-form category used
for organization; `base_cost`, `stamina_cost`, `risk_factor`, and
`success_rate` control the economy:

```csv
id,name,type,base_cost,stamina_cost,risk_factor,success_rate,payout,payout_freq_type,payout_freq,payout_freq_unit,description_html
study,Study Ancient Runes,study,0,5,0,0.9,0,once,0,day,./html/action.html
brew,Brew Healing Potion,craft,20,8,0.2,0.75,90,once,0,day,./html/action.html
```

Use actions for studying, crafting, travelling, trading, or training.

### `events.csv`

Events are scheduled opportunities. `day_of_year` determines when they occur;
`entry_fee`, `reward_pool`, and `charisma_reward` determine their economy.
`required_object_ids` accepts one of several objects. `required_license_id`
can represent a rank, spell certification, or school permission:

```csv
id,name,day_of_year,entry_fee,reward_pool,charisma_reward,duration_value,duration_unit,tags,description_html,required_license_id,required_object_ids,quest_id
moon_exam,Moon Exam,40,25,150,5,1,day,exam,./html/event.html,,oak_wand,
```

Put `race` in `tags` when an event should consume racing services and ask for
damage; wizard datasets can use any other tags.

### `costs.csv`

Costs are reusable named amounts referenced by objects and rules:

```csv
id,name,amount
spell_ink,Spell ink,8
wand_repair,Wand repair,45
```

### `cost_rules.csv`

Rules say when a cost is created. Common triggers are `day_elapsed`,
`event_completed`, and action-related triggers. `charge_mode` controls whether
the amount is charged immediately or becomes a pending ledger item.

```csv
id,cost_id,trigger_type,trigger_ref,amount_multiplier,probability,interval_days,charge_mode,resolution_mode,pending_message,message,damage_type,unavailable_days,event_interval,no_event_days
wand_repair_after_duel,wand_repair,event_completed,duel,1.0,1.0,0,immediate,object_service,Wand needs repair,,wand_damage,3,0,0
```

`damage_type` connects a rule to the damage selected in the result popup;
`unavailable_days` temporarily blocks the affected object. A damage rule should
use `resolution_mode=object_service`: it creates a service requirement on the
affected object instead of charging immediately. The player then pays it from
the object's inventory card, just like tire or oil service. For example, a
three-level wizard familiar damage model could use `minor_damage`,
`serious_damage`, and `critical_damage` as the three `damage_type` values:

```csv
minor_repair,minor_repair,event_completed,duel,1.0,1.0,0,immediate,object_service,Repair the familiar before the next duel.,,minor_damage,0,0,0
serious_repair,serious_repair,event_completed,duel,1.0,1.0,0,immediate,object_service,The familiar needs a serious repair.,,serious_damage,2,0,0
critical_repair,critical_repair,event_completed,duel,1.0,1.0,0,immediate,object_service,The familiar needs a full recovery.,,critical_damage,7,0,0
```

`probability` can make an event or daily cost occasional, while
`interval_days` creates annual, monthly, or weekly costs. A cost rule may be
restricted to an object with `cost_rule_conditions.csv`; the engine will attach
an object service to a matching inventory item and expose the payment there.

### `cost_rule_conditions.csv`

Conditions narrow a rule to a particular object, event, or player variable:

```csv
rule_id,subject_type,subject_ref,operator,value
wand_repair_after_duel,object,type,equals,wand
```

Supported subjects include `object`, `event`, `action`, and `player`.
Operators such as `equals`, `contains`, and numeric comparisons let one rule
serve many catalogue entries.

For example, this condition applies a wand repair rule only to wand objects:

```csv
rule_id,subject_type,subject_ref,operator,value
wand_repair_after_duel,object,type,equals,wand
```

Multiple rows for the same rule are combined, so a rule can require both
`object,type,equals,familiar` and `event,tag,contains,duel`.

## 7. Descriptions and images

HTML descriptions live inside the dataset folder. Reference them from
`description_html`, for example `./html/oak_wand.html`. Images should be
stored under the dataset and referenced with relative paths. The engine embeds
local description images safely when opening a detail screen.

After editing CSVs, use Settings in the application to reload the dataset.
Save games are stored in the dataset's `saves` folder using the integrated
SQLite save database.
