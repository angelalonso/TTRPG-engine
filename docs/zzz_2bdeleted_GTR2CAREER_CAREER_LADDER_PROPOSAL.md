# gtr2career career ladder - proposal only

This is a balancing/design proposal for manual editing, not implemented rules.
No prices, rewards, levels, prerequisites or setback consequences were changed
by the planning/UI work.

## Problem to solve

Club and National versions of the Civic and Fiesta championships currently
share level 1, basic licence, GBP 250 enrolment and effectively the same prize
table. Several other differently named series also occupy level 1. A new
player cannot easily tell which is a first step, an alternative branch or a
promotion. Higher-tier cash rewards alone can also encourage skipping the
intended learning path.

Make progression legible through preparation, affordability and season
commitment, not just increasingly expensive cars.

## Proposed ladder

| Stage | Existing content to build on | Main lesson | Suggested gate and differentiation |
|---|---|---|---|
| Foundation | Work, Formation, track days and social events | Maintain a sustainable budget and stamina reserve | Show starter milestones in dataset HTML: stable income, basic kit, first safe practice event. Do not require a loan just to learn. |
| Local amateur | `amateur_south_wales_series` | Complete a small local calendar with inexpensive transport | First competitive goal; no mandatory professional licence. Short season, clear low running-cost route and modest prizes. |
| Wider amateur | `amateur_uk_circuit_series` | Plan a larger calendar without exhausting savings | Optional bridge or alternative to local amateur, not an artificial duplicate grind. More dates/expenses should justify the distinction. |
| Club | `honda_civic_fm_club`, `ford_fiesta_st150_club`, `uk_hot_hatch_club` | Choose a sustainable specialist branch | Consider level 2 after an amateur trophy; basic licence plus branch-appropriate equipment. Different calendars or cost profiles, comparable chances of progression. |
| National | `honda_civic_fm_national`, `ford_fiesta_st150_national` | Manage a demanding season and higher uncertainty | Consider level 3 after a club trophy; an upgraded licence if its prerequisites exist. Stronger competition and larger expenses before larger expected returns. |
| Specialist branches | `uk_hh200_championship`, `uk_formula_ford_championship`, `uk_club_multiclass_series` | Commit to different vehicles and financial strategies | Position explicitly as club-level alternatives or post-club challenges. Give each a stated entry route rather than treating every expensive car as an automatic promotion. |

The levels above are suggestions. Existing generic previous-level trophy gates
can implement a simple chain. Branch-specific recognition, alternative routes,
or more nuanced qualification should be data-driven when expressible; otherwise
use dataset plugins, not racing-specific engine conditionals.

## Balance from complete seasons, not headline prizes

Before choosing exact amounts, total each route's car, prerequisites, equipment,
insurance, entry fees, expected wear/servicing, living costs and wages lost to
stamina/calendar conflicts. Separate upfront assets from irreversible season
expenses. Use conservative results such as lower midfield finishes as well
as wins; an external racing result is not guaranteed by the RL simulator.

Aim for a player to understand these three numbers on each championship page:
minimum credible preparation budget, estimated season running costs, and a
recommended reserve. The new live planner supplies current purchase/entry
estimates; manually authored HTML must clearly label longer-term estimates
and assumptions rather than inventing engine guarantees.

Prefer several viable branches over a single hidden cheapest car. Price
differences can be balanced by maintenance, reliability, calendar demands or
equipment reuse. Avoid mandatory disposable purchases immediately before a
licence/vehicle becomes obsolete.

Keep setbacks understandable. The current one-fault loan terminal consequence
is severe: the confirmation and forecast now expose it, but changing it to
repossession, debt restructuring or a multi-fault grace period is a separate
balance decision requiring explicit dataset edits and compatibility work.

## How to judge an edited ladder

Use the same goal, limits, reward settings and held-out seeds for every variant.
Record manual and random references, trained success rate, successful median
and p90 days, terminal causes, time waiting for money/calendar dates, and the
chosen branch. These are acceptance criteria to choose before tuning, not
measurements already obtained:

- A novice can identify the next appropriate championship from its page.
- At least one starter route works without knowing an undocumented exploit.
- Moving from amateur to club to national changes the preparation challenge.
- Faster routes retain acceptable success; immediate failure never counts as
  improvement.
- A model does not succeed solely by submitting unrealistically favorable
  external results or farming a reward loop.
- Multiple branches can succeed without one dominating every cost/risk axis.

For promising variants, evaluate at least 100 episodes and inspect several
failed and successful traces. Simulator outcome submission measures planning
under assumed results, not driving ability. Real-player playtesting remains
necessary.

## Editing and save-compatibility checklist

Edit `quests.csv`, `events.csv`, `objects.csv`, requirements/cost/effect tables
and the corresponding HTML together. Preserve all existing stable IDs and
completed history. Do not implement this ladder by matching display names in
Rust or TypeScript.

Changing a quest's level changes the meaning of persisted trophy/progression
state. Do not simply renumber CSV levels: decide how existing memberships,
earned trophies and previous-level eligibility are migrated. Document whether
existing players are grandfathered; preserve recognized achievements rather
than silently invalidating them.

The planning content is dataset revision 37. A later ladder change must use
the next revision after whatever is current then, with every intermediate
one-step migration, serialized old-revision fixture and regression. Increment
save/schema versions as well if persisted structure or meaning changes, add
all supported migration paths, update `SAVE_FORMAT.md`, and run Rust save
compatibility checks before considering the balance update complete.
