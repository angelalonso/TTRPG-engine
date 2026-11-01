# GUI requirements and current state

This document describes the current user interface and the boundary between
the reusable program and the active `gtr2career/` dataset.

The program supplies the screens, controls, interaction patterns, validation,
loading states, error handling, modal behavior, and generic rendering logic.
The dataset supplies visible names, labels, descriptions, images, icons,
categories, colors, calendar labels, requirements, and the content displayed
by those screens.

The interface must remain usable with another dataset. Racing-specific names
and assumptions belong in `gtr2career/`, not in the generic React components.

## 1. Startup and dataset selection

The startup screen provides four choices:

1. **Load Game**
   - Opens the save-slot chooser for the selected valid dataset.
   - Does not silently load the newest save.
   - Validates the save's dataset identity and revision before loading.
   - Offers migration when a supported older dataset revision is found.
   - Provides an explicit “load at own risk” path for a mismatched dataset
     fingerprint.
2. **New Game**
   - Requires a player name.
   - Allows selection of a remembered valid dataset.
   - Allows adding another dataset by selecting its folder.
   - Allows removing remembered dataset paths.
   - Creates a game and enters the dataset's intro flow.
3. **Dataset Editor**
   - Opens the editor for a selected dataset without changing the engine.
4. **Quit Game**
   - Requests confirmation and closes the application.

Remembered dataset paths are stored by the application. Dataset content is
read from the selected folder and is not copied into the program.

## 2. New game and intro flow

The New Game screen collects the minimum information required to create a
campaign:

- player name;
- dataset folder;
- dataset validity state.

Creating a game loads the catalog and starts the intro plugin/content flow.
Loading an existing save or opening an already joined championship does not
repeat the intro flow.

The intro flow is a dataset/plugin concern. The program supplies the scene
container, navigation, error handling, and close behavior; the dataset or
plugin supplies the story text, images, choices, and starting-state content.

## 3. Main layout

The main screen contains:

- a persistent header;
- a persistent navigation row;
- a scrollable main content area;
- modal and popup layers above the content.

The header displays dataset-configured application title text and the current
player status:

- date;
- currency and budget;
- XP;
- recovery/stamina resource;
- a stamina progress bar when the dataset provides the relevant range.

The header's right side contains:

- pause;
- one day every five seconds;
- one day every second;
- one week every second;
- save;
- load;
- configuration;
- exit.

The Space key toggles between paused time and the previous non-paused speed
when focus is not inside a text input or other editable field.

### 3.1 Top-right icon loading

The program contains bundled fallback SVG icons under the application
`./img/` directory:

```text
./img/pause.svg
./img/normal.svg
./img/fast.svg
./img/fastest.svg
./img/save.svg
./img/load.svg
./img/settings.svg
```

Icon configuration is read from dataset `config.csv` using these keys:

```text
speed_icon_paused
speed_icon_normal
speed_icon_fast
speed_icon_fastest
save_icon
load_icon
settings_icon
```

When a dataset value exists, the program treats it as a path relative to the
dataset folder and loads it through the dataset asset API. PNG, JPEG, GIF,
WebP, and SVG assets are supported by that API. For example:

```csv
settings_icon,./img/settings.png
```

If a key is absent, the bundled relative SVG fallback is used. If a
configured asset cannot be loaded, the error is logged and the corresponding
bundled fallback remains visible.

The browser must not be expected to resolve a dataset file by using
`/img/...` directly. Dataset assets are filesystem assets and must be loaded
through the application API.

## 4. Navigation tabs

The standard navigation tabs are:

- Dashboard;
- Inventory;
- Market;
- Events;
- Championships;
- Activities.

The tab row remains visible while the main content scrolls. Dashboard is the
default tab. Names and labels can be replaced by dataset values.

## 5. Dashboard

The dashboard renders a generic player overview:

- player image;
- age;
- XP;
- configured characteristics;
- currency/budget;
- income-source count;
- living costs;
- race/paddock reputation when configured;
- highest licence when applicable;
- race-gear readiness;
- active status/progress bars for characteristics with configured ranges.

The Event Log button opens recorded events in reverse chronological order.

The resource forecast shows current and projected values for configured
resources, including budget and recovery/stamina. The forecast horizon is
validated to 1–365 days and can be supplied by the dataset through
`forecast_days`.

The program renders the same dashboard for other datasets. The dataset
defines which characteristics, resource roles, labels, images, and forecast
values exist.

## 6. Inventory

Inventory is divided into dataset-defined sub-tabs. If no sub-tabs are
configured, the program falls back to object types.

Each owned object may display:

- name;
- image;
- description;
- type/category;
- expiration;
- loaned state;
- temporary unavailability;
- service state;
- sale availability and the reason a sale is blocked.

For serviceable objects, the UI shows every configured service, its cost,
readiness/requirement state, service interval where available, and the service
action.

The inventory can also show dataset-specific concepts such as:

- active insurance;
- sponsor agreements;
- supplied or loaned vehicles/equipment;
- companions or managers;
- trophy results and achievement statistics.

The program supplies the rendering and action flow. Object names, object
types, service slots, descriptions, images, costs, and dataset-specific
inventory tabs are supplied by the dataset.

## 7. Market

The Market displays purchasable or otherwise obtainable objects. A dataset
can define market category names and ordering, including categories such as
cars, race gear, trailers, licences, and insurance.

The market supports:

- category and sub-tab filters;
- name filtering;
- sorting by name, price, owned count, or availability;
- item descriptions;
- purchase actions;
- insurance purchase and policy switching;
- unavailable-state explanations;
- requirement and budget feedback.

The licence category shows every configured licence, including licences whose
current prerequisites are not yet met. Unavailable licences remain visible
with their requirement explanation, image, and HTML description.

The program does not contain racing prices or object names. Those values come
from dataset tables such as `objects.csv`, `costs.csv`, `requirements.csv`,
and `config.csv`.

## 8. Events

Events can be filtered by:

- name;
- alarmed/all state;
- eligible vehicle;
- date or days remaining;
- type;
- entry fee;
- reward.

Event rows show the configured event name, date, type, entry fee, reward, and
available actions. Details show the event description, resolution method,
requirements, preparation guidance, and relevant costs.

Event details distinguish between:

- mandatory events required by an active championship;
- events automatically enrolled or scheduled when their championship is
  joined;
- optional events that are free to enter or otherwise available for manual
  entry.

The UI reports unmet requirements such as:

- licence;
- vehicle;
- equipment;
- service;
- insurance;
- budget;
- stamina/recovery;
- other dataset-defined conditions.

Events may support:

- owned vehicle selection;
- rental vehicle selection;
- alarm creation;
- manual result entry;
- race-results plugin import;
- championship finishing position;
- pole position;
- competitor/standings data;
- pending-result review.

Completed event entries show the selected vehicle, result, outcome, rewards,
and relevant recorded history.

## 9. Championships

The Championships tab renders dataset-defined quests or championship
content. It supports filtering and sorting by:

- name;
- vehicle;
- number of races;
- status;
- progress.

Championship details can show:

- description;
- joining requirements;
- join fee;
- event prizes by position;
- final prizes by position;
- sponsor or manager requirements;
- current standings;
- race status;
- finishing positions;
- received rewards.

The Championships tab provides a checkbox to include all configured
championships. When it is unchecked, the normal eligibility visibility rules
may hide championships above the player's current licence progression.

Joining is blocked when requirements are missing. Pending sponsor or other
requirements use a neutral warning presentation rather than the joined-state
highlight.

The program supplies the progress and standings views. Championship names,
events, calendar dates, entry fees, prizes, descriptions, requirements,
competitors, and sponsor content are dataset data.

## 10. Activities

Activities are divided into:

- Work;
- Trade;
- Sponsor.

The Work table can show job type, pay, frequency, requirements, stamina cost,
and start/stop actions.

The Trade table can show activity type, cost, success chance, return,
requirements, and start/stop actions.

The Sponsor tab can show sponsor funds, initial payment, recurring payment,
repair coverage, supplied objects, requirements, and negotiation actions.

Active activities show their type, status, payment information, obligations,
and stop action. Selecting an activity opens its details before starting it.

## 11. Configuration popup

The configuration popup can:

- change and reload the active dataset folder;
- configure the GTR2 installation folder;
- configure the GTR2 race-results folder;
- enable or disable fullscreen;
- set window width and height;
- configure popup categories for income, costs, incoming events, and alarms;
- enable or disable tutorial hints.

Changes can be saved or cancelled. Invalid paths and save/load errors are
shown in the popup rather than silently ignored.

The program owns application configuration such as window dimensions and
remembered paths. Dataset labels, icon paths, theme values, and game rules
remain in the selected dataset.

## 12. Dataset editor

The Dataset Editor is a generic CSV editor. It can:

- list dataset tables;
- select a table and row;
- edit cell values;
- show reference values;
- hide columns;
- add rows;
- delete rows;
- save one table;
- save all tables;
- exit without saving.

The editor does not define the meaning of racing objects in code. It exposes
the tables and metadata supplied by the selected dataset.

## 13. Other dialogs and popups

The program supplies reusable dialogs for:

- save/load slots;
- event log;
- object/event/activity details;
- rental cars;
- race result entry;
- sponsor negotiation;
- encounters;
- requirements/dependency checks;
- feedback;
- confirmations;
- alerts;
- tutorial hints.

Dialogs support loading states, validation, explicit errors, and appropriate
close behavior. Escape closes the highest-priority open popup. Exit from the
main screen requires confirmation.

HTML descriptions are loaded from the dataset through the backend, displayed
in a sandboxed frame, and have their local image/stylesheets embedded or
resolved safely inside the dataset boundary.
Escape closes the highest-priority open popup, including an HTML description
popup, without exiting the game while another popup is active.

## 14. GTR2 dependency screen/plugin

The GTR2 dependency screen reads the selected dataset's `requirements.txt`.
The current structured format is:

```text
type|name|expected relative file path|download URL
```

Example:

```text
Track|Pembrey|GameData/Locations/pembrey/Pembrey.AIW|https://example.invalid/pembrey
```

The UI displays the dependency type, name, expected path, download URL, and
installed/not-installed state. Paths are checked relative to the configured
GTR2 installation folder. Empty paths are displayed as not configured and
are not checked.

The program supplies parsing, display, path validation, and external-link
opening. `gtr2career/requirements.txt` supplies the actual GTR2 dependency
list.

## 15. GUI quality and error rules

The GUI must:

- disable controls while an operation is loading;
- show errors in a banner, popup, or form error area;
- avoid silently replacing failed operations with success-shaped output;
- preserve keyboard and Escape behavior;
- keep the header and navigation usable while content scrolls;
- use accessible button labels and titles for icon-only controls;
- use dataset labels instead of hardcoded racing terminology where practical;
- keep generic layout and interaction logic in the program;
- keep racing-specific content in the dataset.
