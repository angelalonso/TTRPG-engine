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

`engine_version` is diagnostic metadata. Migrations are selected by
`save_version` and `schema_version`, not by the engine version.

## Loading rules

1. Parse and validate the JSON payload.
2. Migrate it through each supported version until it reaches the current
   version.
3. Validate the resulting `GameState`.
4. Compare `dataset_revision` with the selected dataset.
5. If migration was needed, preserve the original file as a
   `.pre-migration.bak` file and write the current format.
6. Never modify a save that uses a newer unsupported version.

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
- Test that important gameplay state survives the migration.
- Update this document when compatibility behavior changes.

Dataset files are fingerprinted in sorted relative-path order, including file
names and contents, while excluding the `saves` directory. Changing a dataset
therefore produces an explicit compatibility error instead of silently
continuing with potentially different rules.
