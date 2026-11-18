# Autonomous playthrough journal

This folder contains reproducible observations from an autonomous playthrough
of the `gtr2career` dataset. It is intentionally separate from save data and
training artifacts.

The playthrough will cover:

- the Rust headless/RL observation and legal-action interface;
- heuristic and random trajectories;
- progression, economy, championship, and realism findings;
- a recommended route for future imitation learning or masked-RL training.

Generated traces and reports should record the dataset revision, engine build,
seed, goal, limits, and action catalogue version.

## Local reruns

From the repository root, build and run the deterministic sponsor-backed
heuristic locally:

```sh
python3 playthrough/heuristic_play.py \
  --build \
  --seed 1 \
  --goal all-championships \
  --max-days 3650 \
  --max-steps 10000 \
  --output playthrough/heuristic-route-all-championships-seed1.json
```

`all-championships` is expanded from the championship events currently
configured in `gtr2career/events.csv`. To run a different experiment, change
the seed, limits, goal, and output path. The script records the resolved goal,
dataset revision, current git build, action catalogue version, and full
step trace. The same `src-tauri/target/release/rl_env` binary can be driven by
an RL trainer over its line-delimited JSON interface; `actions`, `observation`,
and `step` requests are documented by `src-tauri/src/bin/rl_env.rs`.

## Completed milestone

The service-enabled RL bridge and heuristic route can now complete the Amateur
Championship. Seeds 1, 2, and 3 all reached the level-1 championship goal on
day 294, with ending budgets of £4,595, £3,295, and £3,345 respectively.
The route entered and resolved the test day, prelude, and nine championship
rounds, servicing the Ford Fiesta between events when the legal-action mask
required it.

The viable starter strategy is:

1. Start the 9-to-9 job (`act_work_b`).
2. Take the personal and bank loans.
3. Buy a helmet, third-party insurance, and Ford Fiesta ST150.
4. Join the Amateur Championship.
5. Resolve pending results successfully and service every legal service action
   before the next race.

The earlier 9-to-5 strategy failed at day 121 because its £450 weekly income
could not cover £950 monthly living costs plus both loan obligations. The
£650 weekly 9-to-9 job survives the same route. This is recorded as an
economic balance finding, not hidden by changing the death rules.

## Unexpected behavior and strategy findings

- Enrolling in the next championship immediately after the final race of the
  current championship can miss the next championship calendar. The
  membership is accepted, but no race becomes available before the annual
  cycle ends. The heuristic now enrolls near the next season boundary.
- The first multi-championship attempt also kept postponing the next car
  purchase because it reserved an arbitrary £5,000 buffer. That was a
  playthrough policy error, not an engine failure; the buffer is now £1,000
  and will be reassessed against living costs and loan obligations.
- With the reduced buffer, the route bought the Fiesta on day 890, joined the
  Fiesta ST Cup, won a race, and still died on day 900 when recurring costs
  drove the budget below zero. This is an unexpected progression/economy
  result: a player can have a job and a successful race result yet have no
  viable runway after the required vehicle purchase.
- In the early-enrollment experiment, the RL legal-action mask allowed the
  Fiesta ST Cup test day to be entered with the Scirocco, although the event
  CSV requires `ford_fiesta_st150`. The same championship membership was
  recreated in later annual cycles without producing championship progress.
  These are engine/data-integrity findings, not strategy assumptions; the
  configured-vehicle requirement is covered by a dedicated regression test.
- After correcting that eligibility bug, the route correctly waits for the
  required Fiesta instead of using the Scirocco. It then cannot acquire the
  £19,000 car before the next usable championship round and dies from recurring
  costs after buying it too late. This is now isolated as an acquisition,
  financing, or calendar-balance blocker.

## Evaluation goals

The playthrough continues to evaluate:

1. whether the game is doable;
2. whether the game behaves unexpectedly;
3. whether the test/RL interface is good enough;
4. whether all championships currently configured can be won.

The next experiment uses championship sponsor activities where available.
Those deterministic headless sponsor interactions grant a loaned championship
vehicle and sponsored equipment instead of requiring a purchase.

The first sponsor attempt found that all five sponsor activities referenced
obsolete championship IDs, so they could never become legal after enrollment.
Dataset revision 71 corrected those targets and the Hot Hatch sponsor car;
revision 72 bound them to the configured `honda_showdown` encounter, and
revision 73 replaced the obsolete `sponsored_shoes` equipment reference.
The heuristic now acquires the required licence before starting the sponsor
activity. The latest `heuristic-route-sponsor-encounter-fixed-seed1.json`
trace starts the Fiesta sponsor, resolves the deterministic encounter, and
ends with the loaned `ford_fiesta_st150` plus sponsored equipment in inventory.
A direct Rust regression test covers that activation path. This fixes sponsor
activation, although the full all-championships goal is still not demonstrated.

The new `all-championships` goal covers all 12 currently configured
championships, including levels 1 through 3. It is implemented in the Rust RL
goal specification rather than being reduced to a hard-coded level-1 count.
