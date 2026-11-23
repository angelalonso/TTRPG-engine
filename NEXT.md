# Next steps

The current implementation work is complete for the requested player profile,
sponsor testing, and automatic-play diagnostics.

Remaining engineering priorities:

- finish validating automatic progression beyond the Fiesta ST Cup through all
  configured championships;
- implement or explicitly defer preparation choices (`testing`, `coaching`,
  `setup`, and `rest`);
- validate a fully professional, non-daily-job funding route;
- reconstruct dataset migrations 79–101 only if their historical semantics can
  be recovered safely. The active dataset remains revision 104 for now.

Compatibility rules remain mandatory: preserve stable IDs, add migrations and
fixtures for persisted changes, reject newer save versions without modifying
the source file, and run save compatibility tests.
