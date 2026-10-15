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

Dataset revision 17 changes the Race Gear descriptions to two-column product
pages with item images and yellow race-themed accents. Revision 16 saves
migrate to revision 17 without changing serialized player state or stable
object IDs.

Dataset revision 18 brightens the right-side explanation text on vehicle and
Race Gear description pages from `#cccccc` to `#eeeeee`. Revision 17 saves
migrate to revision 18 without changing serialized player state or stable
object IDs.

Dataset revision 19 adds the `gtr2career/disclaimer.txt` introduction content.
Revision 18 saves migrate to revision 19 without changing serialized player
state or stable object IDs.
