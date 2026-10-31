# Logic requirements and current state

This document describes the generic game engine, its persistence contract, and
the content currently supplied by `gtr2career/`.

The program must remain dataset-agnostic. It owns state transitions,
requirements evaluation, time advancement, transactions, save/load, migration,
plugin integration, and generic calculations. It must not require racing
objects, championship names, sponsor names, or GTR2 filenames to be compiled
into the engine.

The dataset owns the actual game: names, labels, objects, prices, costs,
events, calendars, requirements, rewards, descriptions, images, icons,
championships, sponsors, and dependency manifests.

## 1. Core game model

The engine represents a campaign using:

- a player;
- player characteristics/resources;
- current day and time speed;
- owned object instances;
- object definitions from the catalog;
- active jobs, trades, sponsors, and obligations;
- event history;
- pending events and alerts;
- quests/championships and their progress;
- dataset identity and revision;
- catalog data loaded from the selected dataset.

The engine uses stable IDs for identity. Display names are never used as
identity keys.

Objects can be acquired, owned, serviced, rented, used, sold, expired,
loaned, supplied by a sponsor, or temporarily unavailable according to the
generic rules and dataset configuration.

## 2. Time and calendar

The game advances in days. Time can be:

- paused;
- one day every five seconds;
- one day every second;
- one week every second;
- real-time mode when a dataset/plugin supplies it.

The active speed can be changed through the GUI or keyboard shortcut.
Automatic ticks are disabled while paused and are coordinated with pending
alerts and modal game state.

Calendar labels and formatting are dataset-configurable. The engine supports
generic day/year calculations and the current GTR2 calendar configuration,
including the stable Monday-first calendar used by the racing dataset.

## 3. Characteristics and resources

The engine does not assume fixed names for all characteristics. Dataset
configuration identifies roles such as:

- currency;
- recovery/stamina;
- age;
- XP;
- reputation/paddock credibility.

Characteristics may define minimum and maximum values, display names, ranges,
progress bars, and modifiers. Generic logic handles resource changes,
validation, costs, recovery, terminal conditions, and forecast calculations.

`gtr2career/` supplies the racing-specific characteristics, labels, starting
values, living costs, XP behavior, paddock credibility, licences, and stamina
rules.

## 4. Requirements system

Requirements are evaluated by generic bindings and conditions. The engine can
check combinations of:

- owned objects;
- object types;
- stable object IDs;
- licences and prerequisite licences;
- services and service readiness;
- insurance;
- money;
- stamina/recovery;
- characteristics;
- active activities;
- calendar/date conditions;
- event history;
- quest/championship state;
- dataset-defined condition groups.

Requirement failures are returned as meaningful explanations so the GUI can
show missing licences, cars, equipment, services, insurance, funds, stamina,
or other blockers.

Requirement groups support reusable alternatives. For example, a dataset can
define a generic helmet requirement that accepts any object in the helmet
group, including branded helmets.

The program defines evaluation semantics. The dataset defines which
requirements exist and which objects satisfy them.

## 5. Objects and transactions

The generic object model supports:

- vehicles;
- equipment;
- trailers or transport;
- licences;
- insurance;
- companions/managers;
- trophies;
- arbitrary dataset-defined object types.

Object definitions may include:

- stable ID and display name;
- type/category;
- price;
- image and HTML description paths;
- requirements;
- purchase/availability windows;
- lifetime and expiration;
- resale rules;
- licence level and prerequisite;
- service/cost slots;
- power and weight metadata;
- dataset-defined attributes.

The engine handles purchase, sale, rental, insurance switching,
termination, service, expiration, and temporary unavailability. Prices,
costs, names, categories, and object-specific requirements come from the
dataset.

## 6. Service and maintenance logic

Service slots are generic and can represent:

- tyres;
- oil;
- engine;
- gearbox;
- insurance;
- brakes;
- cosmetic repair;
- bolt-on repair;
- factory rebuild;
- other dataset-defined maintenance.

Each configured cost can provide an amount, service interval, event tags,
repair behavior, availability effects, and messages. The engine determines
whether a service is required, whether it can be paid, and how servicing
changes object state.

The current `gtr2career` dataset uses vehicle-specific cost IDs and
price-scaled values rather than shared generic vehicle maintenance IDs.

## 7. Events and outcomes

Events are generic scheduled opportunities. Dataset event definitions can
represent:

- track days;
- test days;
- meetings/social events;
- races;
- sponsor negotiations;
- work/trade activities;
- arbitrary future event types.

An event can define:

- day/date and recurrence;
- entry fee;
- reward pool;
- position rewards;
- requirements;
- eligible vehicles;
- rental support;
- stamina/resource costs;
- resolution method;
- result options;
- damage/service effects;
- XP and reputation effects;
- descriptions and images.

The engine handles eligibility, entering, pending results, result submission,
reward calculation, event history, alarms, and post-event state changes.

For classified results, configured position rewards are used. A classified
position without a configured reward does not incorrectly receive the winner's
flat reward.

Race results can be:

- entered manually;
- imported through the race-results plugin;
- overridden in-game when the plugin fails;
- extended with championship position, pole, and competitor data.

The current default is user-entered results. Automatic/random result
probabilities, reliability, DNF behavior, home-track effects, and preparation
modifiers remain dataset/plugin concerns until generalized.

## 8. Quests and championships

The engine treats quests as generic progress containers. The GTR2 dataset uses
them as championships.

A quest/championship can contain:

- joining requirements;
- join fee;
- a sequence or set of events;
- automatically enrolled races;
- standings;
- position scoring;
- round prizes;
- final prizes;
- sponsor/manager relationships;
- companions;
- progress and completion effects.

Joining a championship adds its relevant events to the calendar. The engine
tracks completed, pending-result, and not-started races, standings, rewards,
finishing positions, and final outcomes.

The requested GTR2 championship calendar, test days, preludes, race events,
entry fees, round prizes, final prizes, and championship export are stored in
`gtr2career/`, not in engine code.

## 9. Activities, obligations, and sponsors

The generic activity model supports:

- work;
- trade;
- sponsor negotiations;
- recurring payments;
- obligations;
- supplied or loaned objects;
- repair and entry-fee coverage;
- stop/quit actions;
- payment and failure consequences.

The engine tracks activity state, duration, payments, costs, obligations,
stamina effects, and history. Dataset rows define the actual jobs, trade
activities, sponsor offers, rates, probabilities, requirements, messages, and
consequences.

The current GTR2 sponsor and manager content is implemented through dataset
activities and sponsor definitions. More advanced generic modeling of sponsor
fit, manager tiers, negotiation cooldowns, credit history, and partial
obligation failure remains future work.

## 10. Forecasting and planning

The engine can calculate a non-mutating commitment preview and resource
forecast. Planning can include:

- event fees;
- required purchases;
- service costs;
- insurance;
- stamina/recovery;
- expected payments;
- configured obligations;
- future budget and resource values.

Forecast horizons are bounded by the program and default/configured values are
supplied by the dataset. Planning must not alter the live campaign state.

## 11. Plugins

The program provides generic plugin integration points for:

- race-results import/autodetection;
- intro content;
- sponsor/manager extensions;
- other dataset-defined external behavior.

The race-results integration supports selecting a result file, selecting the
player, reviewing imported racers and standings, pole position, resetting or
retrying imported data, overriding the result, and saving the accepted result.

The GTR2 installation and result directories are application configuration.
The actual expected files and download links are dataset content.

## 12. GTR2 dependency manifest

`gtr2career/requirements.txt` is a structured dependency manifest:

```text
type|name|expected relative file path|download URL
```

For example:

```text
Track|Pembrey|GameData/Locations/pembrey/Pembrey.AIW|https://example.invalid/pembrey
```

The program parses this format, displays it in the dependency plugin, and
checks configured paths relative to the user-selected GTR2 installation
folder. It rejects absolute paths and parent-directory traversal.

The dataset supplies:

- dependency type;
- human-readable name;
- expected relative file path;
- download URL.

## 13. Persistence and save compatibility

Saves contain campaign state and compatibility metadata, not a private copy of
the entire dataset. Loading a save reads the current selected dataset and
reconstructs the campaign using the persisted state.

The persistence contract requires:

- stable persisted IDs;
- explicit dataset revision numbers;
- rejection of newer unsupported revisions without modifying the save;
- one-step migrations between supported revisions;
- serialized fixture coverage for every migration;
- regression tests for every migration;
- documentation in `docs/SAVE_FORMAT.md`.

The current `gtr2career` dataset is revision **47**. Recent migrations cover:

- trailer ID renames;
- vehicle-specific maintenance-cost normalization;
- vehicle HTML regeneration and BMW M3 GTR page separation;
- removal of redundant vehicle HTML boilerplate;
- vehicle image centering;
- red vehicle catalog borders and CSS override behavior;
- relative dataset icon paths;
- the structured dependency manifest.

Dataset revisions describe changes to dataset semantics and serialized
compatibility. They do not move racing rules into the engine.

## 14. Dataset loading and validation

The loader reads CSV and optional content files from the selected dataset
folder. It reports warnings for unsupported optional columns and errors for
invalid or unresolved required data.

Validation checks include:

- required dataset files;
- CSV parsing;
- stable IDs;
- references between tables;
- object, cost, event, quest, and requirement references;
- asset and description references where applicable;
- dataset configuration and revision metadata.

The engine remains tolerant of dataset-specific optional columns while keeping
generic required schema fields typed and validated.

## 15. What belongs in the program

The program/reusable engine contains:

- React screens and layouts;
- generic tabs, tables, dialogs, forms, and modal behavior;
- time advancement and speed control;
- generic resource, characteristic, and calendar calculations;
- object ownership and transaction state transitions;
- requirement evaluation;
- event and quest/championship progression;
- standings and reward calculation;
- save/load and migration machinery;
- dataset loading and validation;
- safe asset/HTML loading;
- race-results plugin integration;
- dependency-manifest parsing and path checking;
- error handling, loading states, accessibility labels, and keyboard behavior.

It must not contain GTR2-specific vehicle names, championship names, prices,
tracks, sponsor names, filenames, or hardcoded racing progression rules.

## 16. What belongs in `gtr2career/`

The active dataset contains:

- `config.csv`: labels, title, calendar settings, icon paths, theme/config
  values, resource roles, forecast defaults, and dataset revision;
- `objects.csv`: vehicles, equipment, trailers, licences, insurance,
  companions, managers, trophies, prices, metadata, services, and paths;
- `costs.csv` and `cost_rules.csv`: maintenance, repair, insurance, damage,
  intervals, probabilities, messages, and event tags;
- `events.csv` and related outcome/effect tables: track days, tests,
  meetings, races, sponsor events, fees, rewards, and results;
- quest/championship tables: championship membership, race calendars,
  standings, round prizes, final prizes, and requirements;
- sponsor and activity tables: jobs, trade, sponsors, negotiations, payments,
  obligations, and supplied objects;
- requirement and condition tables: licences, vehicles, equipment, services,
  money, stamina, and alternatives;
- `requirements.txt`: GTR2 dependency names, expected relative files, and
  download URLs;
- `img/`: dataset images and icons;
- `html/`: dataset descriptions and catalog pages;
- `gtr2career_vehicles_*.csv` exports outside the dataset folder are
  read-only/reporting exports and do not define runtime behavior.

The dataset can be replaced by another dataset without changing the generic
engine, provided it supplies the required schema and valid references.

## 17. Known future extensions

These are not current requirements of the generic implementation but are
natural extension points:

- automatic/random race result modeling with class, reliability, preparation,
  and home-track modifiers;
- generalized credit scoring and loan default consequences;
- richer sponsor-fit and manager-tier probability formulas;
- generic work obligation grace periods and partial pay;
- richer damage probability based on vehicle age, mileage, and intensity;
- dataset-defined calendar visibility filters;
- multiple starting stories and intro content;
- automatic vehicle dashboard-image composition from owned equipment;
- cross-dataset validation by running the same engine against a non-racing
  dataset.
