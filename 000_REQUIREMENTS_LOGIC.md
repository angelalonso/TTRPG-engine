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
group, including the generic helmet and branded helmets such as the Bell RS7
Pro or Arai GP-6 Carbon. Requirements refer to the group, not one exact
catalogue version.

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

Licence definitions remain available to the UI even when their prerequisites
are unmet. The UI may explain the blockers, but must not remove configured
licences from the licence catalogue.

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

Insurance is a shared policy, not a vehicle service. It has no upfront
purchase fee. Its premium is the policy base plus a percentage of the total
purchase value of all owned vehicles, capped by policy tier, and one twelfth
of that premium is charged every month as part of living costs:

- third-party: £250 base + 1.5%, capped at £2,000/year;
- basic: £500 base + 3%, capped at £7,500/year;
- full: £750 base + 5%, capped at £30,000/year.

Vehicle definitions do not carry separate insurance service costs. A race
requires one active insurance policy, and only that policy's monthly share is
charged through living costs.

### Vehicle purchase-price evidence

Revision 49 updates all 42 configured vehicle purchase prices in
`gtr2career/objects.csv` and the read-only `gtr2career_vehicles.csv` export.
These are conservative **2026 GBP acquisition prices** for the game, not
collector valuations. The Scirocco, Fiesta ST150, Fiat 500 Abarth, and Honda
Civic are treated as road/track-converted cars. Every other vehicle is treated
as a race car. Where an exact listing was unavailable, the price uses a
comparable race-car class estimate rather than a rare-car premium.

Sources were checked on 1 November 2026. `Racecarsdirect` listings are asking
prices or comparable listings; `proxy` means a class-based estimate:

| Vehicle ID | Price | Evidence |
|---|---:|---|
| `scirocco` | £16,000 | [Racecarsdirect 167125](https://racecarsdirect.com/Advert/Details/167125), [164448](https://racecarsdirect.com/Advert/Details/164448) |
| `ford_fiesta_st150` | £16,500 | [Racecarsdirect 166391](https://racecarsdirect.com/Advert/Details/166391) |
| `honda_civic` | £21,000 | BRSCC Civic Cup listings on [Racecarsdirect](https://racecarsdirect.com/) (£19,995–£24,500) |
| `500_abarth` | £27,000 | [Racecarsdirect 164940](https://racecarsdirect.com/Advert/Details/164940), [165786](https://racecarsdirect.com/Advert/Details/165786) |
| `formula_ford` | £22,000 | [Racecarsdirect 165659](https://racecarsdirect.com/Advert/Details/165659), [96304](https://racecarsdirect.com/Advert/Details/96304), [167144](https://racecarsdirect.com/Advert/Details/167144) |
| `caterham_320` | £29,000 | [Racecarsdirect 166393](https://racecarsdirect.com/Advert/Details/166393), [165784](https://racecarsdirect.com/Advert/Details/165784); comparable Caterham race cars |
| `alfa_mito` | £27,000 | proxy: Hot Hatch 200 / Clio Cup class |
| `citroen_ds3` | £27,000 | proxy: Hot Hatch 200 / Clio Cup class |
| `ford_fiesta` | £27,000 | proxy and BRSCC Fiesta Cup listings on [Racecarsdirect](https://racecarsdirect.com/) |
| `seat_ibiza` | £27,000 | proxy: Hot Hatch 200 / Clio Cup class |
| `vauxhall_corsa` | £27,000 | proxy: Hot Hatch 200 / Corsa Cup class |
| `peugeot_207` | £27,000 | proxy: Hot Hatch 200 class; no comparable complete listing |
| `toyota_yaris` | £27,000 | proxy: Hot Hatch 200 class |
| `renault_clio` | £30,000 | Clio Cup listings on [Racecarsdirect](https://racecarsdirect.com/) |
| `giulietta` | £55,000 | proxy: TCR/Hot Hatch 300 race-car class |
| `punto` | £40,000 | [Racecarsdirect 159514](https://racecarsdirect.com/Advert/Details/159514), comparable Abarth race car |
| `focus` | £55,000 | [Racecarsdirect 161084](https://racecarsdirect.com/Advert/Details/161084), proxy: TCR class |
| `mazda3` | £45,000 | proxy: TCR/Hot Hatch 300 class |
| `megane` | £50,000 | proxy: TCR/Hot Hatch 300 class |
| `vwgolf` | £58,000 | [Racecarsdirect 151309](https://racecarsdirect.com/Advert/Details/151309), [162618](https://racecarsdirect.com/Advert/Details/162618) |
| `radical` | £32,000 | Radical SR3 listings on [Racecarsdirect](https://racecarsdirect.com/) |
| `porsche_996_cup` | £85,000 | Porsche 996 GT3 Cup listings on [Racecarsdirect](https://racecarsdirect.com/) |
| `tvr_t400r` | £45,000 | [Racecarsdirect 151268](https://racecarsdirect.com/Advert/Details/151268), comparable TVR race car |
| `mosler_mt900r` | £130,000 | [Racecarsdirect 164929](https://racecarsdirect.com/Advert/Details/164929) |
| `porsche_biturbo` | £115,000 | [Racecarsdirect 165562](https://racecarsdirect.com/Advert/Details/165562), comparable 993 GT2 race car |
| `morgan_aero` | £110,000 | [Racecarsdirect 152261](https://racecarsdirect.com/Advert/Details/152261) |
| `gillet_vertigo` | £90,000 | proxy: small-marque GT2 race cars; exceptional [Racecarsdirect 164317](https://racecarsdirect.com/Advert/Details/164317) excluded as an outlier |
| `bmw_z3mcoupe` | £80,000 | [Racecarsdirect 165402](https://racecarsdirect.com/Advert/Details/165402) |
| `nissan_350z` | £50,000 | 350Z race-car listings on [Racecarsdirect](https://racecarsdirect.com/) |
| `porsche_911_rs` | £190,000 | proxy: 964 RS/RSR and Carrera Cup listings on [Racecarsdirect](https://racecarsdirect.com/); model generation is ambiguous |
| `porsche_911_rsr` | £175,000 | 2009 GT3 RSR and comparable Porsche race-car listings on [Racecarsdirect](https://racecarsdirect.com/) |
| `ferrari_360` | £150,000 | Ferrari Challenge listings on [Racecarsdirect](https://racecarsdirect.com/); N-GT auction outliers excluded |
| `bmw_m3_gtr` | £75,000 | proxy: GTR tribute/replica and S54 race cars on [Racecarsdirect](https://racecarsdirect.com/); genuine homologation cars are excluded |
| `maserati_mc12` | £950,000 | proxy: MC12 road-car values and Maserati GT3 comparables; no direct GT1 listing |
| `corvette_c5r` | £700,000 | [Racecarsdirect 142935](https://racecarsdirect.com/Advert/Details/142935), factory C5-R comparable |
| `ferrari_f550` | £550,000 | proxy: Prodrive Ferrari 550 GT1; [Racecarsdirect 121963](https://racecarsdirect.com/Advert/Details/121963) is parts-only |
| `ferrari_f575` | £500,000 | proxy: Ferrari 550 GT1 and period GT1 race-car class |
| `murcielago_rgt` | £320,000 | proxy: Reiter Engineering GT1/GT2 race-car class |
| `lister_storm` | £180,000 | proxy: period GT1 race-car class; no complete listing found |
| `porsche_911_gt2` | £350,000 | [Racecarsdirect 165562](https://racecarsdirect.com/Advert/Details/165562), historic 993 GT2 race-car comparable |
| `saleen_s7r` | £280,000 | [Racecarsdirect 160150](https://racecarsdirect.com/Advert/Details/160150), [157457](https://racecarsdirect.com/Advert/Details/157457) |
| `viper_gtsr` | £220,000 | [Racecarsdirect 164560](https://racecarsdirect.com/Advert/Details/164560), [151500](https://racecarsdirect.com/Advert/Details/151500), [150634](https://racecarsdirect.com/Advert/Details/150634) |

The Caterham 320, several Hot Hatch variants, Porsche 911 RS, Ferrari 360,
BMW M3 GTR, and some GT1 cars represent ambiguous or fictionalized catalog
configurations. Their prices are intentionally class estimates, not claims
that an identical factory model trades at that amount.

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

Event participation has three distinct dataset semantics:

- **mandatory** events are required by the active quest/championship;
- **championship-enrolled** events are added to the player's scheduled event
  set when the championship is joined;
- **free-to-enter** events have no entry fee and remain optional unless a
  quest marks them as required.

The engine handles eligibility, entering, pending results, result submission,
reward calculation, event history, alarms, and post-event state changes.

Calculated monetary and characteristic rewards are rounded upward to the
nearest multiple of five before they are applied. The same rule is used for
percentage-derived service, insurance, and recurring payment amounts, so
calculated values do not introduce decimal currency.

For the active GTR2 dataset, positive XP awards are reduced to three quarters
of their configured amount and rounded up to the next whole XP point before
being applied.

For classified results, configured position rewards are used. A classified
position without a configured reward does not incorrectly receive the winner's
flat reward.

Race results can be:

- entered manually;
- imported through the race-results plugin;
- overridden in-game when the plugin fails;
- extended with championship position, pole, and competitor data.

The race-results plugin accepts DNF explicitly. Its editable result table must
contain the user's result, reject duplicate classified positions, and ask
before continuing when a rewarded position has no driver name.

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

Recorded championship results remain editable after submission. Editing
updates the stable event result and recorded finishing position without
changing the identity of the event. When the final race is completed, the
engine emits a completion alert with the final standings. Championship
membership and its calendar alarms are yearly: they expire at the year
boundary and must be joined again for the next year.

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

Bank and subprime loans cannot be stopped early through the normal activity
stop action. They may end early only when the player pays the full remaining
balance at once.

The engine tracks activity state, duration, payments, costs, obligations,
stamina effects, and history. Dataset rows define the actual jobs, trade
activities, sponsor offers, rates, probabilities, requirements, messages, and
consequences.

The current GTR2 sponsor and manager content is implemented through dataset
activities and sponsor definitions. More advanced generic modeling of sponsor
fit, manager tiers, negotiation cooldowns, credit history, and partial
obligation failure remains future work.

Sponsor negotiation must cap a signed DNF penalty at no more than half the
purchase value of the selected sponsored vehicle. The negotiation UI and
engine validation both enforce this limit.

Successfully obtaining a sponsor always awards charisma. The award is
monotonic with the sponsor's initial or configured monetary value and uses the
dataset `sponsor_charisma_per_1000` scale, with a minimum positive award.

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

The program parses this format, displays only the dependency name, download
link, and current presence state (`present` or `missing`) in the requirements
list, and checks configured paths relative to the user-selected GTR2
installation folder. The checked path is intentionally not shown to the user.
It rejects absolute paths and parent-directory traversal.

The dataset supplies:

- dependency type;
- human-readable name;
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

The current `gtr2career` dataset is revision **59**. Recent migrations cover:

- trailer ID renames;
- vehicle-specific maintenance-cost normalization;
- vehicle HTML regeneration and BMW M3 GTR page separation;
- removal of redundant vehicle HTML boilerplate;
- vehicle image centering;
- red vehicle catalog borders and CSS override behavior;
- relative dataset icon paths;
- the structured dependency manifest;
- conservative, source-documented vehicle purchase-price updates.
- sponsor charisma scaling and regenerated gear/insurance descriptions.
- licence asset-path normalization and complete licence catalogue visibility.
- vehicle catalog layout and concise, vehicle-name-free maintenance labels.
- monthly insurance billing with no upfront insurance purchase fee.

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
- `gtr2career_vehicles_*.csv`, `gtr2career_events.csv`, and
  `gtr2career_quests.csv` exports outside the dataset folder are
  read-only/reporting exports and do not define runtime behavior. The quest
  export additionally flattens championship round/calendar details into one
  row per quest. `gtr2career_events_quests.csv` combines both exports using a
  `record_type` column.

The dataset can be replaced by another dataset without changing the generic
engine, provided it supplies the required schema and valid references.

Catalog export tooling offers an interactive choice among cars, race gear,
races and championships, and money-making items. The selected export is
written as its corresponding CSV report; the export files are read-only
reporting artifacts and do not define runtime behavior.

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
