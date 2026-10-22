# AI change guide

Give an AI only the smallest file set in the tables below. Preserve stable
IDs. The HTML files are dataset content, not React components.

## Visuals

| Change | Pass these files |
|---|---|
| Shared catalogue card layout or colours | `gtr2career/html/gtr-card.css`, `gtr2career/html/gtr-theme.css` |
| One car or race-gear page | The matching `gtr2career/html/*.html`, row in `gtr2career/objects.csv`, and image in `gtr2career/img/` |
| One licence, insurance, companion, or trailer | Matching HTML, row in `objects.csv`, and image if used |
| One event | Matching HTML, row in `events.csv`, and image if used |
| One championship | Matching HTML, row in `quests.csv`, and image if used |
| Organizer/championship logo | The affected HTML and the image/SVG in `gtr2career/img/` |
| One job or venture | Matching `act_*.html` and row in `events.csv` |
| Market/dashboard labels | `gtr2career/config.csv` |
| React popup or catalogue image loading | `src/components/DetailModal.tsx`, `src/App.tsx`, `src/services/tauriApi.ts` |

Use `description_html` for the page and `image_path` for a dataset-relative
image. Keep the two-column catalogue structure and the existing colour class.

## Mechanics

| Change | Pass these files |
|---|---|
| Object price, requirement, service, or availability | `gtr2career/objects.csv`, `gtr2career/costs.csv`, `gtr2career/cost_rules.csv` |
| Event/job/venture rule | `gtr2career/events.csv`, related `event_*.csv`, and the relevant Rust engine module |
| Championship membership, rounds, standings, or rewards | `gtr2career/quests.csv`, `gtr2career/events.csv`, `src-tauri/src/engine/quest_runs.rs`, `src-tauri/src/engine/standings.rs` |
| Conditions or requirements | The matching condition CSV, `src-tauri/src/engine/conditions.rs`, or `src-tauri/src/engine/requirements.rs` |
| Effects or lifecycle changes | The matching effect CSV, `src-tauri/src/engine/effects.rs`, and `src-tauri/src/lib.rs` |
| Player state or save behavior | `src-tauri/src/lib.rs`, `src-tauri/src/engine/save.rs`, `docs/SAVE_FORMAT.md` |
| CSV field or loader contract | CSV file, `src-tauri/src/engine/loader.rs`, `src-tauri/src/engine/schema.rs` |
| Calendar/day processing | `src-tauri/src/lib.rs`, `src-tauri/src/engine/quest_runs.rs`, and the relevant dataset CSV |
| Plugin or sponsor behavior | `plugins/`, matching `gtr2career/plugins/`, `gtr2career/plugins.csv`, and `docs/SPONSOR_PLUGIN_TESTING.md` |

For a dataset change, increment `gtr2career/config.csv` and add one migration
step in `src-tauri/src/engine/dataset_migrations.rs`, a serialized fixture in
`src-tauri/src/engine/fixtures/`, and a regression test. Never rename/remove
a persisted field without a migration. Reject newer save revisions.

## Playtest and training

| Change | Pass these files |
|---|---|
| RL action catalogue, legal actions, or reward | `src-tauri/src/rl.rs`, `src-tauri/src/bin/rl_env.rs` |
| RL-to-Python observation adapter | `rl/gtr2career_env.py` |
| Training logs or PPO settings | `rl/train.py` |
| Automatic policy playtest or output | `rl/playtest.py`, optionally `rl/translate_playtest.py` |
| Evaluation metrics | `rl/evaluate.py` |
| RL commands/setup | `rl/README.md`, `rl/requirements.txt` |

Rust remains authoritative for transitions, legality, seeded randomness, and
goal status. The Python scripts select actions and report results.

## Validation

Use the smallest applicable checks:

```sh
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
python3 -m unittest tests/test_catalog_html.py
cargo run --manifest-path src-tauri/Cargo.toml --bin validate_dataset -- gtr2career
```

For RL changes, rebuild the bridge first:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --release --bin rl_env
```

The playtest/training scripts intentionally use CPU by default. A PyTorch
CUDA capability warning is non-fatal when the installed wheel does not support
the machine's GPU; CPU mode avoids that compatibility issue.
