# `gtr2career` automatic gameplay audit

This report records the supported automatic-play experiment against the
`gtr2career` dataset. It uses the Rust RL environment and the deterministic
heuristic player; it does not modify campaign saves.

## Reproduction

Build the authoritative RL bridge and run a bounded career:

```sh
./scripts/run_automatic_gameplay.sh
```

Equivalent direct command:

```sh
python3 playthrough/heuristic_play.py \
  --build \
  --seed 1 \
  --goal all-championships \
  --max-days 3650 \
  --max-steps 10000 \
  --fast-forward-days 20 \
  --output playthrough/automatic-gameplay-seed1.json
```

The route is deterministic for a fixed seed and records the dataset revision,
engine build, action-catalogue version, observations, legal sponsor actions,
resources, errors, and terminal status. `all-championships` means all 12
configured championship trophies, including the FIA GT Championship.

Latest bounded validation with the release bridge completed the Amateur
Championship, enrolled in the Fiesta ST Cup, and reached day 445 with £29,276
and no runtime errors before the 500-step limit. It did not yet verify the
complete all-championship goal; higher-tier progression remains an open
validation task.

For a quick smoke test that avoids the long all-championship route:

```sh
python3 playthrough/heuristic_play.py \
  --build --seed 1 \
  --goal championships:level=1,count=1 \
  --max-days 400 --max-steps 1000 \
  --fast-forward-days 20 \
  --output /tmp/gtr2career-level1.json
```

The completed route can be exported as a compact progress trace:

```sh
python3 scripts/export_automatic_gameplay_csv.py \
  /tmp/gtr2career-fia-gated.json \
  /tmp/gtr2career-progress.csv
```

For an interruptible run, enable periodic checkpoints:

```sh
CHECKPOINT=/tmp/gtr2career.checkpoint.json \
CHECKPOINT_EVERY=250 \
MAX_DAYS=6000 MAX_STEPS=6000 \
OUTPUT=/tmp/gtr2career.json \
scripts/run_automatic_gameplay.sh
```

Press `Ctrl-C` to save the latest checkpoint. Resume it with:

```sh
RESUME=/tmp/gtr2career.checkpoint.json \
CHECKPOINT=/tmp/gtr2career.checkpoint.json \
OUTPUT=/tmp/gtr2career-resumed.json \
scripts/run_automatic_gameplay.sh
```

The JSON report records elapsed wall-clock time, terminal status, blocker
counts, and whether the run was interrupted.

## Observed automatic-play results

- The Amateur Championship route is reproducibly completable. Existing seeds
  1, 2, and 3 finish on day 294 with the test day, prelude, and nine rounds
  resolved.
- The route can complete the Amateur, Fiesta ST, Honda Civic, Hot Hatch 200,
  and Formula Ford championships with sponsor-backed or purchased cars.
- The staged all-championship route reaches FIA GT with the deterministic
  simulator path. A successful continuation awarded FIA GT trophies after
  completing Radical, Porsche Cup, British GTO, and FIA N-GT.
- A prior route died shortly after buying the Fiesta ST because recurring
  living, loan, and vehicle costs consumed the remaining runway. This is a
  balance/progression blocker, not an RL legality failure.
- Race-result plugin calls are the dominant runtime cost. The headless route
  submits deterministic simulated results, so it proves engine progression and
  action legality, not real driving performance in an installed GTR2 copy.
- RL masks prevent illegal actions, preserve errors, and support checkpoint
  restore. Actions outside the catalogue or external plugin interactions remain
  explicitly non-replayable.

### Results and largest blockers

- The longest delays were calendar gaps between championship seasons, not
  illegal actions.
- Higher-tier purchases initially caused cash cliffs: buying Radical or the TVR
  too early left insufficient runway for living costs and repayments.
- FIA GT preparation initially bought its motorhome prerequisite while the
  lower-tier Porsche was already owned; acquisition-window and prerequisite
  gating removed that premature purchase.
- The route remains simulator-only: deterministic race-result plugin calls do
  not validate external GTR2 driving performance.

## Game improvements

### Blocking or misleading progression

- Add a visible career roadmap showing the next licence, championship season,
  required vehicle class, trailer, insurance, gear, manager tier, sponsor
  option, and estimated cash runway.
- **Implemented:** joining a championship after its usable calendar window is
  blocked with an explicit next-year explanation. The entry planner also reports
  whether a current-season event remains while continuing to show the next
  scheduled event and readiness costs.
- **Implemented:** the championship planner now projects budget to the next
  scheduled event, reports a budget shortfall, and distinguishes enrolment
  eligibility from first-event readiness.
- **Implemented:** higher-tier sponsor rows use explicit manager-level
  requirements; the sponsor UI exposes manager support, offer expiry, and
  cooldown-related state.

### Automatic gameplay and plugins

- **Implemented:** RL observations now include structured readiness records for
  every championship, including next event day, projected budget, shortfall,
  first-event readiness, and join eligibility.
- **Verified:** external GTR2 result import remains separate from deterministic
  simulator actions, and automatic-play reports remain explicitly simulator
  only.
- Add a manager offer lifecycle: negotiated baseline offer, expiry, optional
  player renegotiation, cooldown, and accepted-term snapshot. Persist it with
  a dataset migration if it becomes campaign state.
- Rework the sponsor dialog into three persistent sections: sponsor baseline,
  manager contribution/leverage, and player counteroffer. Show the walkaway
  boundary and the effect of each requested term before submission.
- Add sponsor-fit, reputation, results, licence, and manager-tier explanations
  to rejection messages. A rejection should identify the limiting term instead
  of only reporting failure.
- **Covered:** sponsor lifecycle tests cover accepted terms, recurring
  payments, DNF penalties, cancellation, supplied vehicles, and equipment.

### Feel and quality-of-life ideas

- Amateur drivers should have a useful no-manager career path; managers should
  become valuable when entering national/international championships.
- Professional drivers should not need a permanent daily job to fund ordinary
  race preparation. Replace it with sponsorship, appearance fees, retainers,
  coaching, or occasional optional work.
- Add preparation choices (testing, coaching, setup, rest) that trade money,
  stamina, and reliability without becoming mandatory grind.
- Add season-end summaries, licence milestones, sponsor history, championship
  promotion, and a clear FIA GT qualification checklist.
- Add a “why this is blocked” dependency graph for cars, licences, trailers,
  insurance, manager level, and sponsor terms.

## Dataset improvements

### Current blockers

- Higher-tier vehicle prices, trailer requirements, entry fees, insurance, and
  maintenance create a large cash cliff after level-1 championships. The
  dataset needs a tested financing/sponsorship runway for each promotion step.
- Only a subset of early championships has configured sponsor activities. The
  dataset needs realistic national, continental, and world sponsor offers,
  including FIA GT-compatible manager/sponsor content.
- Some championship calendars open early in the year. Joining near the end of
  a season can consume the membership without a usable event sequence.
- The current dataset has 12 championships, 42 vehicles, licences, gear,
  trailers, insurance, managers, sponsor rows, and track/event assets, but
  several higher-tier descriptions and visuals remain generic or placeholder
  content.

### Balance risks to review

- Re-test the 9-to-9 job, loans, living costs, insurance, vehicle maintenance,
  and sponsorship payments as a complete annual budget, not as isolated rows.
- Ensure prize money and recurring sponsor income cover the next required
  purchase only after the player has earned the appropriate licence and
  manager tier.
- **Verified:** market flipping is not a positive-return unconditional shortcut,
  crypto trade has negative expected return, and loan ventures carry explicit
  repayment obligations.
- Publish expected total preparation cost per championship: enrolment, all
  entries, service, insurance, trailer, manager, and a conservative failure
  buffer.
- Verify that each race's accepted vehicle group matches the championship
  description and the intended class; keep the Honda Civic enthusiasts event
  Honda-only.

### Missing or inconsistent content

- Add unique, non-placeholder championship and high-tier vehicle artwork and
  descriptions.
- Regenerate catalog HTML whenever `events.csv`, `quests.csv`, or organizer
  assets change. The previous Python tests had hard-coded organizer
  expectations that no longer matched the dataset's configured organizer
  families.
- Keep regression assertions for organizer identity derived from each row's
  configured `organizer_image_path` and HTML content.
- The live dataset tests validate current event/obligation references and
  generated work-asset consistency. The authoritative `validate_dataset`
  command now also checks organizer/logo/picture paths and local images
  referenced by generated HTML. Its loader regression test requires the
  current dataset to have no missing generated assets.

The historical `act_*` activity IDs are mixed: some remain valid current IDs
for loans, trade, formation, and sponsorship, while the old work IDs were
renamed. Live dataset tests must use the current event IDs and validate
obligation `event_id` values against those rows. Migration fixtures and the
revision-78 rename migration must retain the historical IDs they are meant to
translate; they must not be globally rewritten.

## Required regression coverage

The audit requires tests for:

- all event/object/sponsor/quest references and image paths;
- live work-event and obligation references use current stable IDs, while
  historical `act_*` IDs remain covered only by migration fixtures;
- staged licence prerequisites and championship-level eligibility;
- no manager requirement for the Amateur path and an explicit manager/sponsor
  requirement for professional sponsor paths once configured;
- no daily-job dependency in a professional route once the dataset defines the
  replacement sponsorship/retainer path;
- annual budget runway for each championship transition;
- no reward/trade shortcut that bypasses licence, vehicle, or manager gates;
- manager-offer expiry, optional renegotiation, and accepted-term persistence;
- RL action-mask legality, deterministic replay, checkpoint restore, and clear
  simulator-only reporting.

## Prioritized improvement actions

The following is an implementation-oriented order of work. The prices and
payments below are proposed first-pass targets, not final balance claims; each
change must be validated by the automatic-play route and the annual budget
tests before being adopted.

### 1. Fix progression economy and cash runway

- Add a budget test for every championship transition. Require the player to
  retain at least 90 days of living costs, recurring payments, and one
  conservative repair event after buying the required vehicle and equipment.
- As an initial balancing pass, reduce the Radical SR3 price from `47000` to
  `40000`, the Porsche 996 Cup from `70000` to `60000`, and the TVR T400R from
  `80000` to `70000`.
- Reduce the FIA GT prerequisite motorhome price from `125000` to `90000`, or
  provide a documented sponsor/finance route that covers the `35000` difference.
- Reduce the Level 1 manager monthly fee from `750` to `500` until the first
  professional sponsor is secured; then make the full fee payable.
- Add a one-time promotion grant after each first championship trophy:
  `5000` after level 1, `12000` after national, and `25000` after international
  promotion. Grants must be tied to the trophy/licence milestone and must not
  be repeatable.
- Re-run seeds 1, 2, and 3 and require the Fiesta ST, Radical, Porsche Cup,
  British GTO, and FIA GT transitions to finish without a negative cash
  balance or forced daily-job dependency.

### 2. Make progression requirements visible before commitment

- Add a roadmap panel for the next three eligible championships showing:
  licence, vehicle class, required trailer, insurance, gear, manager level,
  sponsor requirement, next event day, total preparation cost, and projected
  cash remaining.
- Add a pre-purchase warning when a purchase leaves less than the 90-day
  runway; include the exact projected shortfall.
- Keep championship enrolment reversible until the player confirms the
  projected first-event budget.
- Add a single `readiness_reason` list to RL observations and the UI, with
  stable codes such as `missing_license`, `missing_trailer`,
  `season_closed`, `budget_shortfall`, and `manager_required`.

### 3. Correct calendar and season-window behavior

- Treat a championship membership as valid for the next usable event sequence,
  not merely the current calendar year.
- If the current season has no remaining usable event, do not consume the
  membership; show the next event day and the estimated holding cost.
- Add a calendar regression for enrolment on days 1, 80, 100, 150, 280, and
  350 for every championship.
- Add a configurable minimum gap of 14 days between the entry deadline and the
  first event so preparation cannot be silently impossible.

### 4. Complete sponsorship and manager progression

- Add at least one sponsor row for every configured tier:
  national, continental, and world sponsors must cover the corresponding
  championship tiers and licence requirements.
- Add a sponsor-offer state machine:
  `offered -> negotiated -> accepted -> expired/cancelled`, with an explicit
  30-day offer expiry and 30-day renegotiation cooldown.
- Persist the accepted payment, supplied vehicle/equipment, DNF penalty, expiry,
  and cancellation terms in campaign state.
- Make the initial professional offer cover at least the manager fee plus
  `950` monthly living costs; display the remaining monthly surplus before
  acceptance.
- Change rejection messages to name the limiting term, for example:
  “requires National A licence”, “manager level 2 required”, or
  “requested payment exceeds sponsor limit”.

### 5. Replace the professional daily-job dependency

- Add a sponsorship/retainer income path that becomes available after the first
  professional trophy and does not require a daily job.
- Set the initial retainer target to at least `1500` per month, increasing by
  sponsor tier; ensure it covers ordinary living costs but not every race cost.
- Keep daily work as an optional catch-up activity rather than a required
  source of baseline professional income.
- Add a route test that completes a professional championship with no daily-job
  actions after the replacement income path is unlocked.

### 6. Add preparation choices without mandatory grind

- Add `testing`, `coaching`, `setup`, and `rest` actions with explicit costs:
  testing costs money and improves reliability, coaching costs money and
  improves performance, setup trades stamina for performance, and rest restores
  stamina without advancing the championship.
- Show the expected effect and cash/stamina cost before confirmation.
- Cap preparation benefits per event so repeated preparation cannot become an
  optimal waiting exploit.

### 7. Explain every blocked action

- Replace generic blocked-action text with a dependency list containing the
  missing object, current value, required value, acquisition cost, and next
  available date.
- Add a “Why blocked?” view for cars, licences, trailers, insurance, manager
  levels, sponsor terms, and championship entry.
- Test that every blocked action has at least one actionable explanation and
  that stable IDs, not display names, are used to identify dependencies.

### 8. Preserve automatic-play and save/data compatibility

- Keep simulator-only race results explicitly labelled in every report and
  reject any implication that deterministic plugin results validate real GTR2
  driving performance.
- Add replay tests for action masks, illegal-action rejection, deterministic
  seeds, checkpoint restore, interrupted runs, and resumed output.
- Any price, sponsor, calendar, or progression change must increment
  `dataset_revision`, add a one-step migration from the current revision, add a
  serialized fixture and regression test, and update `docs/SAVE_FORMAT.md`.
- Reject newer save versions without modifying the original save file.

### 9. Improve content and generated assets

- Replace placeholder descriptions and artwork for every championship and every
  vehicle used from Radical onward.
- Regenerate catalog HTML whenever `events.csv`, `quests.csv`, organizer data,
  or asset paths change.
- Make the authoritative dataset validator fail on missing organizer logos,
  vehicle images, championship images, or generated HTML references.

