# Save format and compatibility

The save format is a compatibility contract. A change to persisted game state
must not make existing campaigns unusable.

## Version fields

Every current save contains:

- `save_version`: the serialized save format version;
- `schema_version`: the internal game-state schema version;
- `engine_version`: the engine build that last wrote the save;
- `dataset_revision`: a SHA-256 fingerprint of the dataset files used by the
  save.
- `dataset_revision_number`: the ordered dataset revision used by the save.

Current saves are dataset overlays, not embedded dataset snapshots. The save
stores player-specific state (for example the current date, changed
characteristics, owned object instances, obligations, pending events, and
history) while omitting the catalog and characteristics that still equal their
dataset defaults. On load, the selected dataset supplies the catalog and base
values, then the saved overlay is applied. Owned objects retain stable
`definition_id` and `instance_id` values; mutable instance state such as
service flags, expiry, loans, and acquisition day remains in the overlay.
If a referenced definition is no longer present, loading fails explicitly
until a save/dataset migration resolves the stable ID.

`engine_version` is diagnostic metadata. Migrations are selected by
`save_version` and `schema_version`, not by the engine version.

## Loading rules

1. Parse and validate the JSON payload.
2. Migrate it through each supported version until it reaches the current
   version.
3. Validate the resulting `GameState`.
4. Compare `dataset_revision` with the selected dataset. If it differs, offer
   the explicit dataset migration flow when a numbered migration is available.
   Also offer an explicit "load at your own risk" action for saves whose
   dataset fingerprint does not match, including saves with the same numbered
   revision. This bypasses only the fingerprint check; save-format validation
   and newer unsupported save-version checks still apply.
5. Dataset migrations must advance one numbered revision at a time. Every new
   dataset revision must add a corresponding migration handler and regression
   fixture/test before the revision number is incremented.
   When a migration is performed, the save's embedded catalog is replaced with
   the current dataset catalog. This ensures future costs, rewards, obligations,
   and other rules use the new dataset from the migration point onward; already
   recorded ledger entries and completed history are preserved.
6. If migration was needed, preserve the original file as a
   `.pre-migration.bak` file and write the current format.
7. Never modify a save that uses a newer unsupported version or has no complete
   migration chain. Loading at the user's own risk does not silently migrate
   such a save.

Older saves without version metadata are treated as legacy saves. A save with
an unknown or ambiguous value must fail explicitly rather than guessing.

## Rules for changing persisted state

- Do not rename or remove a persisted field without a migration.
- Every breaking savegame change must include a migration script before the
  change is considered complete.
- Add defaults for newly introduced fields.
- Preserve stable identifiers; display names are not identifiers.
- Increment the appropriate version when structure or meaning changes.
- Add a migration test and a serialized historical fixture.
- Add a dataset migration step in `src-tauri/src/engine/dataset_migrations.rs`
  for every new dataset revision. Do not skip revision numbers.
- Test that important gameplay state survives the migration.
- Update this document when compatibility behavior changes.

Dataset files are fingerprinted in sorted relative-path order, including file
names and contents, while excluding the `saves` directory. Changing a dataset
therefore produces an explicit compatibility error instead of silently
continuing with potentially different rules.

Dataset revision 2 expands competitive race and championship prize tables to
cover lower finishing positions. Its migration is state-preserving because the
change affects future event payouts, not the serialized shape of an existing
campaign.

Dataset revision 3 rebalances event income, recurring living and vehicle costs,
insurance, equipment lifetimes, job income, sponsorship terms, and loan
defaults. Its migration is state-preserving because these rules affect future
transactions and do not change the serialized shape of an existing campaign.

Dataset revision 4 raised the monthly living-cost baseline to £675 plus £75 per
owned vehicle, which produced £750/month for the starter garage, and moderately
adjusted job income to keep routine work viable. Its migration is
state-preserving because these rules affect future transactions and do not
change the serialized shape of an existing campaign.

Dataset revision 5 sets monthly living costs to £750 plus 1% of the combined
value of owned vehicles. Its migration is state-preserving because the rule
changes future transactions without changing the serialized shape of an
existing campaign.

Dataset revision 6 sets the stamina cost for every job application attempt to
25, regardless of whether the application succeeds. Its migration is
state-preserving because the rule changes future transactions without changing
the serialized shape of an existing campaign.

Dataset revision 7 adds the gtr2career starting narrative, additional amateur
cars and track days, bad-luck outcomes, UK licence names, the doubled sickness
probability, and slower sponsor/progression gates. Its migration is
state-preserving because these changes affect future gameplay rules and content
without changing the serialized shape of an existing campaign.

Dataset revision 8 removes the amateur qualifier from the new vehicle names and
from the related vehicle service, championship, and event identifiers. Its
migration rewrites those persisted catalog and campaign references to their new
IDs and labels.

Dataset revision 9 adds the missing running-cost definitions for the VW Scirocco
1981 and Fiat 500 Abarth. Its migration is state-preserving because the catalog
already stores the stable service IDs; the revision makes the corresponding
cost definitions available to future transactions and UI displays.

Dataset revision 10 recalibrates the active amateur and hot-hatch race-car
running costs. Amateur cars use one shared semi-slick and racing-oil price,
with engine and gearbox rebuilds scaled to the car's value. Hot-hatch cars use
more expensive full slicks, the same racing-oil price, and Clio-Cup-like race
engine and gearbox rebuild costs. Its migration is state-preserving because the
service IDs and serialized catalog shape are unchanged.

Dataset revision 11 changes dashboard race-gear readiness so the three insurance
policies are one requirement group: owning any policy satisfies the insurance
requirement, and the missing-equipment message displays the generic label
“insurance” instead of listing policy types. Its migration is state-preserving.

Dataset revision 12 raises the base monthly living cost to £950 while preserving
the existing ownership scaling formula, and recalibrates selected job payouts,
frequency, and stamina costs. The 9-5 job pays £450/week, the 9-9 job pays
£650/week, local delivery pays £250/week with no attempt stamina cost, and small
repair service pays £200 once. Its migration is state-preserving.

Dataset revision 13 expands the annual calendar with monthly Pembrey, Castle
Combe, and Anglesey test or track-day events, and adds two amateur championships
plus a multi-class championship for the amateur and current hot-hatch vehicle
catalogues. Its migration is state-preserving because these additions do not
change the serialized campaign shape or existing stable IDs.

Dataset revision 14 removes the deprecated UK Hot Hatch Championship - National
All-Comers Series, its championship rounds, sponsor activity, and description
pages. Its migration is state-preserving; existing saved event history and
championship references remain untouched while the removed content is no longer
available in the current catalogue.

This catalog refresh is automatic for every explicit dataset migration,
including changes where a dataset fingerprint changes without a numbered
revision change. Dataset changes should still increment `dataset_revision` and
provide a one-step migration so the change is visible and auditable.

Dataset revision 15 adds XP progression, sponsor XP requirements, and
negative-budget recovery tracking. Revision 14 saves migrate to revision 15
without changing existing player values; the debt timer fields default to
`null`. The grace period is calculated once when the balance first becomes
negative: seven days per £500 owed, capped at 28 days.

Dataset revision 16 adds dedicated description pages for the Race Gear
catalogue entries. Revision 15 saves migrate to revision 16 without changing
serialized player state or stable object IDs; the change only gives each
gear object its own explanatory HTML description.

Dataset revision 35 removes obsolete work activity entries, including
`act_freelance_writing`, and their orphaned obligation definitions from the
authoritative `gtr2career` dataset. Revision 34 saves migrate to revision 35
by removing only those deprecated entries from the embedded catalog; player
inventory, history, and other stable campaign state are preserved.

If an application reports that the 34-to-35 migration is missing, it is
running an older executable. Rebuild/reinstall the application (and rebuild
`src-tauri/target/release/rl_env` for playtests); do not edit the save's
revision number. The current migration chain contains 34-to-35 and preserves
the original save as a `.pre-migration.bak` file.

Dataset revision 17 changes the Race Gear descriptions to two-column product
pages with item images and yellow race-themed accents. Revision 16 saves
migrate to revision 17 without changing serialized player state or stable
object IDs.

Dataset revision 18 brightens the right-side explanation text on vehicle and
Race Gear description pages from `#cccccc` to `#eeeeee`. Revision 17 saves
migrate to revision 18 without changing serialized player state or stable
object IDs.

Dataset revision 25 adds a helmet requirement to every track day and amateur
race. These events still do not require a licence. Revision 24 saves migrate
to revision 25 without changing the serialized player state or stable object
IDs.

Dataset revision 19 adds the `gtr2career/disclaimer.txt` introduction content.
Revision 18 saves migrate to revision 19 without changing serialized player
state or stable object IDs.

Dataset revision 24 expands the gtr2career licence progression from National B
and National A through International C, International B, International A, and
Super Licence. The Super Licence is reward-only and cannot be purchased.
Revision 23 saves migrate to revision 24 without changing serialized player
state or stable object IDs.

Dataset revision 20 gives every gtr2career championship its own description
page and keeps every race description addressable as a separate HTML asset.
Revision 19 saves migrate to revision 20 without changing serialized player
state or stable object IDs.

Dataset revision 21 gives each gtr2career licence and insurance object its own
description page instead of sharing the generic object page. Revision 20 saves
migrate to revision 21 without changing serialized player state or stable
object IDs.

Dataset revision 22 makes licence requirements apply at the championship level
only. Individual races, including amateur races, and trackdays no longer
require a licence; joining a championship continues to require its configured
licence. Revision 21 saves migrate to revision 22 without changing serialized
player state or stable object IDs.

Revision 23 removes the licence requirement from the amateur circuit series.
Amateur races and trackdays do not require a licence, while non-amateur
championships retain their configured licence requirement. Revision 22 saves
migrate to revision 23 without changing serialized player state or stable
object IDs.

Dataset revision 26 adds manager levels with level-scaled monthly fees, UK HH200
and Formula Ford championships, trailer equipment tiers, corrected finance
activities, and the Bank Loan activity. Revision 25 saves migrate to revision
26 without changing the serialized player state or stable object IDs.

Dataset revision 27 corrects the vehicle catalog so the existing Hot Hatch 200
cars use their proper names and replaces the mistaken standalone `hh200`
vehicle with the distinct Caterham 320 definition. Revision 26 saves migrate
legacy `hh200` inventory and requirement references to `caterham_320`.

Dataset revision 28 corrects the Formula Ford specification and purchase price
to £25,000, 510 kg, and 145 BHP. Revision 27 saves migrate without changing
serialized player state or stable object IDs.

Dataset revision 29 adds the Goodwood Festival of Speed social event on 14 July,
awarding +4 paddock cred. Revision 28 saves migrate without changing
serialized player state or stable object IDs.

Dataset revision 30 gives the three trailer tiers their proper catalog pages
and blank image assets: Open Trailer, Closed Trailer, and Motorhome. Revision
29 saves migrate without changing serialized player state or stable object IDs.

Dataset revision 31 classifies the legacy Race Car Trailer as a trailer instead
of race gear, so it appears in the trailer subtab. Revision 30 saves migrate
without changing serialized player state or stable object IDs.

Dataset revision 32 updates the three trailer catalog entries to use the
dedicated `trailer_open.jpeg`, `trailer_closed.jpeg`, and
`trailer_motorhome.jpeg` assets. Revision 31 saves migrate without changing
serialized player state or stable object IDs.

Dataset revision 33 removes the obsolete Race Car Trailer object and its
vehicle prerequisite. Revision 32 saves migrate by removing obsolete
`race_trailer` inventory and catalog entries; other inventory and stable object
IDs are preserved.

Dataset revision 34 adds a typed event-completion effect that awards 19 XP when
the Formation venture succeeds. Revision 33 saves migrate without changing
serialized player state or stable IDs.

Dataset revision 36 adds shared catalogue-card pages for cars, race gear,
licences, insurance, companions, trailers, jobs, ventures, events, and
championships. Events and championships now support optional `image_path`
assets. Revision 35 saves migrate without changing serialized player state or
stable IDs.
The optional image fields default to empty for older catalog snapshots; current
overlay saves obtain them from the selected dataset. No player-state field is
renamed or reinterpreted, so save version 2 and schema version 1 are unchanged.

Dataset revision 37 adds dataset-owned planning labels, shared preparation
groups and live entry-planner slots in championship HTML. It does not rebalance
prices, payouts, prerequisites, obligations or progression levels. The 36-to-37
migration preserves the entire campaign payload except its numbered dataset
revision; loading refreshes the catalogue/content from the selected dataset.
Stable object/quest IDs, individual repayment/fault counters, memberships and
history are preserved. The serialized revision-36 fixture and a full 0-to-37
chain regression cover this step.

Resource forecasts, commitment previews and entry estimates are transient query
results, not new persisted fields. Save version 2 and schema version 1 remain
unchanged. Python playtest GUI experiment snapshots are separate diagnostic
files under ignored `rl/data/gui/`, not campaign saves.
