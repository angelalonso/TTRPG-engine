# Sponsor negotiation plugin testing

The plugin has two useful test modes:

## One-command smoke suite

From the repository root, run:

```bash
python3 tools/sponsor_test.py
```

This checks the JSON protocol, signed agreements, year/championship/race payment rules, multi-round negotiations, invalid-action recovery, and exact target-vehicle data. It exits non-zero if any check fails, so it can be used in CI or before tuning negotiation logic.

## Repeatable bulk simulations

Run many seeded negotiations and inspect the outcome distribution:

```bash
python3 tools/sponsor_bulk_test.py --runs 100 --matrix
```

Useful variations:

```bash
# Compare conservative acceptance across all sponsor tiers
python3 tools/sponsor_bulk_test.py --runs 250 --matrix --strategy accept

# Exercise the negotiation actions repeatedly
python3 tools/sponsor_bulk_test.py --runs 250 --matrix --strategy logic

# Save detailed results for analysis
python3 tools/sponsor_bulk_test.py --runs 1000 --matrix --format csv > sponsor-results.csv
python3 tools/sponsor_bulk_test.py --runs 1000 --matrix --format json > sponsor-results.json
```

The `--seed` option makes a run reproducible. Change it when exploring another random sample:

```bash
python3 tools/sponsor_bulk_test.py --runs 100 --seed 42 --race-tier national --scope year
```

## Standalone JSON protocol

The JSON mode is useful when changing the state machine itself:

```bash
printf '%s\n' \
  '{"action_id":"accept_proposal"}' \
  | python3 dataset/plugins/sponsor_negotiator.py \
      --json --dataset-path dataset --scope year --seed 7
```

Diagnostics and logging go to stderr; JSON responses remain on stdout. The final response should have a terminal status of `SIGNED`, `REJECTED`, or `BANNED` and a signed response should contain an `agreement` object.

Actions currently useful for protocol tests include:

- `accept_proposal`
- `counter_proposal` with a `proposal` object
- `logic_rebuttal`
- `charm`
- `aggressive_pitch`
- `not_an_action` to verify error recovery

## GUI testing

Run the themed GUI directly:

```bash
python3 dataset/plugins/sponsor_negotiator.py \
  --dataset-path dataset \
  --scope championship \
  --target-id ford_fiesta_st150_national \
  --target-name "Ford Fiesta ST150 Championship - National"
```

Check the following manually:

1. The selected scope shows the correct target selector.
2. Race and championship scopes show zero monthly payment and do not allow editing it.
3. The car selector includes `No car included`.
4. A target with exact vehicle requirements only offers those vehicle definitions.
5. The live `Now` and `Others (approx)` totals change as package fields change.
6. Sending a proposal shows sponsor counters; accepting it writes the agreement when `--result-file` is supplied.
7. Closing the window without signing exits cleanly and does not create a false agreement.

The main program charges a randomized 8–16 stamina for every negotiation attempt
before launching the plugin. This cost is paid even when the sponsor rejects the
proposal or the player closes the window, and a signed agreement does not charge
stamina a second time.

## Adding realism tests

When tuning the model, add deterministic assertions to `tests/test_sponsor_plugin.py` or `tools/sponsor_test.py`. Prefer tests that state an invariant rather than one exact random result, for example:

- race/championship monthly payment is always zero;
- yearly monthly payment is non-negative;
- signed agreements contain the selected scope and target;
- counter proposals stay within the sponsor walkaway limit;
- invalid actions return `ERROR` without ending the negotiation;
- exact required vehicle IDs are preserved;
- seeded runs produce the same response sequence.

Use `--seed` in every regression test involving randomness.
