# Read-only planning and commitment guidance

The engine owns generic transition and eligibility queries. Dataset folders
own terminology, preparation-group choices, HTML layout and game-specific
logic. No new saved fields are introduced by planning.

## Dashboard forecast

`get_resource_forecast` clones the current game and advances the existing
daily transition for 30 days by default. The API accepts 1 through 365 days
and rejects invalid/overflowing horizons. Resources come from configured
`resource_role_currency` and `resource_role_recovery`, not racing terminology
or hardcoded characteristic names.

The projection includes current recurring payouts, independent obligation
counters and weekdays, daily costs, recovery, existing illness progression,
expiry, missed-event handling and terminal consequences. It stops at death
and exposes daily balances, changes, notes and the projected terminal date.
It does not spend real money, consume real RNG, change saved state, invoke
dataset Python plugins or make new player choices.

Assumptions matter:

- Rules/effect bindings with probability below one and new random sickness
  are excluded. This is neither an expected-value calculation nor a worst case.
- Existing illness, missed work, skipped payouts and terminal loan faults
  can affect the result. "No new incidents" does not mean guaranteed wages.
- Future purchases, results/prizes, voluntary actions, negotiations and
  external plugin outcomes are not invented.
- "Lowest daily balance" includes today's starting value and each closing
  daily value, not the minimum between transactions within a day. A weekly
  wage cannot retrospectively pay an obligation the engine processed earlier.

Override `forecast_title`, `forecast_assumptions`, `forecast_current`,
`forecast_end`, `forecast_minimum`, `forecast_daily`, `forecast_terminal`,
`forecast_no_resources`, `forecast_error`, and generic `planning_*` display
labels in `config.csv` as needed.

## Starting commitments

The UI requests `get_commitment_preview` before starting a normal activity.
Recurring obligations, a positive base fee or an upfront recovery cost that
uses the remaining recovery require confirmation. Time is paused, the query
is refreshed, and accepting invokes the real action, which validates again.
Cancelling spends nothing; time remains paused until the player resumes it.
Dataset-specific negotiated interactions keep their existing negotiation flow.

The popup shows base/upfront resource costs, conditional payout, obligation
amounts/frequency, restricted weekdays, first due date, finite payment totals,
fault limits and configured consequences. A limit of one means the first
missed obligation can trigger its consequence, not one free missed payment.
The conditional projection stages a successful acceptance, known start effects,
completion costs and the active commitment. Random outcome effects and
external plugin results are not guaranteed.

Customize `commitment_title`, `commitment_intro`, `commitment_confirm`,
`commitment_upfront`, `commitment_payout`, `commitment_projection`,
`commitment_due_days`, `commitment_first_due`, `commitment_payments`,
`commitment_total`, `commitment_ongoing`, `commitment_no_due_date` and
`commitment_fault_limit`. Keep assumptions explicit.
This is a warning layer, not a new rule that makes all bad decisions illegal.

## Entry guidance inside dataset HTML

Place this slot wherever the dataset wants the live guidance:

```html
<section class="entry-planner"
         data-engine-slot="quest-plan"
         data-quest-id="stable_quest_id">
  <h2>How do I enter?</h2>
  <p>Open this page in the game for live requirements and cost estimates.</p>
</section>
```

For gtr2career it lives in the right-hand aside on all championship pages.
Styles belong to the dataset's stylesheet. The existing sandbox remains;
the engine inserts text/DOM nodes, not dataset scripts or executable HTML.
The stable `data-quest-id` must match the quest, never its display name.

`get_quest_join_eligibility` uses the real enrolment action on a clone.
The list's Join button follows this decision. Enrolling and being ready for
a particular scheduled event are deliberately separate: you need not already
own every possible event object simply to enrol.

`get_quest_entry_plan` estimates possible preparations for each next scheduled
event. It uses the real acquisition/prerequisite and entry validators. A clone
with synthetic funds lets it distinguish an otherwise valid setup from a
funding shortfall; actual funds are never modified.

Optional `config.csv` keys:

```csv
variable,value
quest_planning_object_groups,protective_equipment;permit;tool_a|tool_b
quest_planning_object_groups:stable_quest_id,permit;special_tool
quest_planning_title,How do I enter?
```

Semicolons separate required preparation groups; pipes represent alternatives.
Each token matches a stable object ID or its `requirement_group`. Per-quest
configuration replaces the global list. The planner recursively buys missing
prerequisites and picks the cheapest successfully acquired alternative per
group. It then tests selected event-object candidates and reports up to three
options ordered by readiness and estimated cost.

These groups describe suggestions, not new enforced prerequisites. Keep real
requirements in existing object/event bindings and dataset rules. Specialized
conditions, negotiations or game-specific progression belong in dataset data
or plugins, not new named branches in the engine.

Each option includes purchases, enrolment fee, one event entry fee, total
estimate, additional funds required and unresolved blockers. Owned items
expired by the event date are not counted as ready; acquire/renew close enough
to the event. A setup can still be blocked by current recovery or previous
achievement requirements. Future recovery, income and automatic achievements
are not assumed.

This is **not** a globally optimal shopping solver or full-season budget:
shared alternative dependencies, changing prices, rentals, future running
costs, future servicing and negotiated support are outside the estimate.
Current active sponsorship coverage is used where the real fee rule allows it.
Expired owned items may need a separately timed replacement. Recheck entry
on the actual date and use the resource forecast for existing commitments.

Override `quest_planning_assumptions`, `quest_planning_no_selection`,
`quest_planning_join_fee`, `quest_planning_can_join`, `quest_planning_joined`,
`quest_planning_entry_fee`, `quest_planning_estimate`,
`quest_planning_shortfall`, `quest_planning_ready`, `quest_planning_blocker`,
`quest_planning_no_purchases` and `quest_planning_no_events` for dataset wording.

Dataset content changes still need an incremented numbered revision, a one-step
migration, historical fixture/regression and `SAVE_FORMAT.md` update. Planning
queries do not bypass save compatibility.
