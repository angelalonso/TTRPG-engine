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
