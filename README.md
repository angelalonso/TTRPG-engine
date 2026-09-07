# Economy Engine

This is a configurable economy engine for games and simulations.

The runtime is domain-neutral. Objects, events, actions, inventory, and visible terminology are loaded from the selected dataset directory.

Each dataset can include:

- `objects.csv` for purchasable objects
- `costs.csv` for reusable cost definitions referenced by object `cost_1`, `cost_2`, and so on
- `cost_rules.csv` for rules that generate costs from days, events, actions, or object acquisition
- `cost_rule_conditions.csv` for optional rule conditions
- `events.csv` for scheduled events
- `actions.csv` for one-time or recurring actions
- `config.csv` for UI labels, with `variable,value` columns

`costs.csv` uses `id,name,amount` columns. Its IDs can be named `service_1_id`, `service_2_id`, and so on. Each object stores those IDs in `cost_1`, `cost_2`, and so on rather than embedding service prices, so multiple objects can share the same cost definition.

Cost rules support `day_elapsed`, `event_completed`, `action_completed`, and `object_acquired` triggers. Rules can match IDs or event tags, apply a probability and multiplier, charge immediately when funds are available, or remain pending until paid. Conditions currently support event/action/object/player facts with operators such as `equals`, `contains`, `greater_than`, and `less_than`.

## Current status

Under development, putting together all ideas that come to mind.

## How to test

cargo tauri dev

## How to compile

cargo tauri build

## Other requirements that may be needed
npx tsc --init # do once to prepare a tsconfig.json
