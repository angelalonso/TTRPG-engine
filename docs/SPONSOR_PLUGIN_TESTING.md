# Testing the sponsor negotiation plugin outside the game

The sponsor plugin can be tested without launching Tauri. In game mode it is a
UI-free state machine communicating through JSON on standard input and output.
The race-results plugin's interactive mode uses the same local browser-served
HTML, CSS, and JavaScript stack as the sponsor negotiation UI, using the shared
`public/plugin.css` theme and no additional GUI package.

## Logging

Plugins log operational events with timestamps. JSON protocol responses remain
clean on stdout; protocol logs go to stderr so they cannot corrupt a response.
Set `TTRPG_LOG_DEST=stdout` only when running a plugin interactively without a
machine consuming stdout as JSON. The Rust host mirrors plugin stderr with a
`[race-results-plugin]` or `[sponsor-negotiator-plugin]` prefix.

For a verbose standalone trace:

```sh
TTRPG_LOG_DEST=stdout python3 gtr2career/plugins/race_results.py
```

## Requirements

From the repository root, use Python 3:

```sh
python3 --version
```

The default commands use:

- plugin: `gtr2career/plugins/sponsor_negotiator.py`
- race-results plugin: `gtr2career/plugins/race_results.py`
- sponsor data: `gtr2career/sponsors.csv`
- dataset directory: `dataset`

## Single interactive JSON run

Start a cold call with no manager:

```sh
python3 gtr2career/plugins/sponsor_negotiator.py --json --dataset-path dataset --race-tier local --scope race --results 60 --podiums 3 --wins 1 --seed 42
```

The first JSON line is the matchmaking state. It includes the available
sponsors, whether this is a cold call, the ideal proposal, and the current
proposal. Send actions as additional JSON lines. For example:

```json
{"action_id":"accept_proposal"}
```

For a counter-proposal:

```json
{"action_id":"counter_proposal","proposal":{"initial_money":3000,"monthly_payment":250,"maintenance":true,"entry_fees":false,"gear":true,"car":false,"result_bonus":350,"dnf_penalty":100}}
```

The plugin returns `SIGNED`, `REJECTED`, or `BANNED` when the negotiation ends.
Use `--seed` to reproduce the same sponsor selection and random decisions.

The initial state exposes `median_benchmark`, sponsor and player walkaway
limits, current offer/request values, and `zopa_open`. The negotiation applies
concession decay, Gaussian luck, maximum rounds, walkaway limits, and a
settlement price inside the overlap zone. Critical d20 rolls can sign
immediately or terminate with a charisma penalty.

## Standalone themed window

For a visual smoke test, run:

```sh
python3 gtr2career/plugins/sponsor_negotiator.py \
  --dataset-path dataset --race-tier local --scope race \
  --results 60 --podiums 3 --wins 1 --poles 1
```

Use the action buttons to exercise aggressive, charm, logic, counteroffer, and
acceptance paths. The default window is only a test client; the game-facing
integration must use `--json`.

## Manager and tier comparisons

A manager creates sponsor-initiated proposals. The manager level controls how
far up the sponsor tiers the matchmaking process can reach:

```sh
python3 gtr2career/plugins/sponsor_negotiator.py \
  --json \
  --dataset-path dataset \
  --has-agent \
  --agent-level 5 \
  --race-tier national \
  --scope championship \
  --results 80 \
  --podiums 8 \
  --wins 4 \
  --championships-won 1 \
  --seed 42
```

Compare `--agent-level 0` with levels `2`, `5`, and `8`. Inspect:

- `cold_call`: `true` means the player initiated the negotiation.
- `proposal_expires`: manager proposals have an expiration date.
- `sponsors`: the sponsors available for the selected race tier.
- `approaching_sponsors`: up to three sponsor IDs brought by a manager.
- `sponsor_tier`: the selected sponsor's tier.
- `attraction_score`: the player's results and reputation score.
- `racer_value`: the tier-weighted achievement value used by the benchmark.

Race-tier filtering is intentional. The initial local sponsors accept local,
regional, and national races, but not continental or world races.

## Bulk runs

The repository includes a bulk harness:

```sh
python3 tools/sponsor_bulk_test.py \
  --runs 100 \
  --manager-level 0 \
  --race-tier local \
  --strategy accept
```

It prints a summary grouped by manager level and race tier. The harness starts
one fresh plugin process per run, varies the seed deterministically, records
the initial matchmaking state, sends the selected strategy, and records the
terminal status.

Run a complete manager/tier matrix:

```sh
python3 tools/sponsor_bulk_test.py \
  --matrix \
  --runs 100 \
  --results 75 \
  --podiums 6 \
  --wins 3 \
  --poles 2 \
  --format table
```

The matrix covers manager levels `0`, `2`, `5`, and `8` against all five race
tiers. This is useful for checking that higher manager levels reduce cold calls
and expose more important sponsors. With the current dataset, continental and
world rows correctly report `ERROR` because no sponsors for those tiers have
been configured yet.

Available strategies:

- `accept`: accept the first proposal.
- `logic`: send repeated logic rebuttals until the negotiation ends.

Export raw results for analysis:

```sh
python3 tools/sponsor_bulk_test.py \
  --matrix \
  --runs 1000 \
  --format csv > sponsor-results.csv
```

The JSON output contains one record per run:

```sh
python3 tools/sponsor_bulk_test.py \
  --runs 20 \
  --format json > sponsor-results.json
```

## Direct protocol scenarios

To test a selected sponsor, a counteroffer, and the ZOPA logic manually:

```sh
python3 gtr2career/plugins/sponsor_negotiator.py \
  --json --dataset-path dataset --sponsor local_repairmen \
  --race-tier local --scope championship --results 80 \
  --podiums 8 --wins 4 --championships-won 1 --poles 3 --seed 42 <<'EOF'
{"action_id":"counter_proposal","proposal":{"initial_money":12000,"monthly_payment":1200,"maintenance":true,"entry_fees":true,"gear":true,"car":false,"result_bonus":1500,"dnf_penalty":500}}
{"action_id":"accept_proposal"}
EOF
```

For reproducible comparisons, keep the same `--seed` and vary one input at a
time: manager level, race tier, results, achievement counts, or scope.

## Automated checks

Run the plugin unit tests:

```sh
python3 -m unittest tests/test_sponsor_plugin.py
```

Compile-check both the plugin and the bulk harness:

```sh
python3 -m py_compile \
  gtr2career/plugins/sponsor_negotiator.py \
  tools/sponsor_bulk_test.py
```

## What this harness does not test yet

The harness tests matchmaking, proposal construction, benchmark and walkaway
calculation, negotiation outcomes, race-tier filtering, concession decay, and
seeded repeatability. It does not yet test the unfinished in-game contract
lifecycle: applying accepted sponsorship terms to
saved game state, recurring sponsor payments, DNF counters, five-DNF automatic
cancellation, or sponsor benefits and penalties during race resolution.
