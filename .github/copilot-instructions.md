# Persistence rules

Save compatibility is a supported feature of this project.

1. Never change the meaning of an existing persisted field without a migration.
2. Never remove or rename a persisted field directly.
3. Every breaking savegame change must include a migration script before the
   change is considered complete.
4. Increment the save/schema version for structural or semantic changes.
5. Add a migration from every supported previous version.
6. Add a serialized save fixture and a regression test for each migration.
7. Preserve stable IDs; do not infer identity from display names.
8. Reject newer save versions without modifying the original file.
9. Update `docs/SAVE_FORMAT.md` for every persistence change.
10. Run the Rust save compatibility tests before considering the change complete.
11. Every new dataset revision must increment `dataset_revision` and add a
    one-step migration handler, serialized fixture, and regression test before
    the dataset change is complete. Missing intermediate migrations are
    blocking errors.
