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

This catalog refresh is automatic for every explicit dataset migration,
including changes where a dataset fingerprint changes without a numbered
revision change. Dataset changes should still increment `dataset_revision` and
provide a one-step migration so the change is visible and auditable.
