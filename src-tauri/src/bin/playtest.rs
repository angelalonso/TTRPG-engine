use serde::Serialize;
use std::collections::HashSet;
use std::env;
use ttrpg_engine_lib::{
    advance_day, apply_action, buy_object_for_sim, legal_action_ids, new_game, GameState,
};

#[derive(Clone)]
struct Candidate {
    game: GameState,
    actions: Vec<String>,
    purchases: Vec<String>,
}

#[derive(Serialize)]
struct Plan {
    rank: usize,
    outcome: String,
    days: u32,
    goal_value: f64,
    budget: f64,
    stamina: f64,
    actions: String,
    purchases: String,
}

fn arg(args: &[String], name: &str, default: &str) -> String {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| default.into())
}

fn metric(game: &GameState, id: &str) -> f64 {
    game.player.characteristics.get(id).copied().unwrap_or(0.0)
}

fn signature(candidate: &Candidate, goal: &str) -> String {
    let mut inventory: Vec<_> = candidate.game.player.inventory.iter().map(|item| item.id.clone()).collect();
    inventory.sort();
    format!(
        "{}|{:.2}|{:.2}|{:.2}|{}|{}",
        candidate.game.current_day,
        metric(&candidate.game, goal),
        metric(&candidate.game, "budget"),
        metric(&candidate.game, "stamina"),
        inventory.join(";"),
        candidate.game.player.active_actions.iter().map(|action| action.action_id.as_str()).collect::<Vec<_>>().join(";"),
    )
}

fn score(candidate: &Candidate, goal: &str, target: f64) -> f64 {
    let progress = metric(&candidate.game, goal) / target.max(1.0);
    let budget = metric(&candidate.game, "budget") / 10_000.0;
    let stamina = metric(&candidate.game, "stamina") / 100.0;
    let inventory = candidate.game.player.inventory.len() as f64 * 0.01;
    progress * 100.0 + budget + stamina + inventory
}

fn can_buy(game: &GameState, object_id: &str) -> bool {
    let object = match game.catalog.objects.iter().find(|object| object.id == object_id) {
        Some(object) => object,
        None => return false,
    };
    !object.id.starts_with("trophy_")
        && object.id != "trophies"
        && object.id != "business_proposal"
        && object.id != "lower_cost"
        && !game.player.inventory.iter().any(|owned| {
            owned.id == object.id || owned.id.starts_with(&format!("{}_", object.id))
        })
}

fn expand(candidate: &Candidate, goal: &str, target: f64, object_limit: usize) -> Vec<Candidate> {
    let mut purchases = vec![None];
    let mut object_ids: Vec<String> = candidate.game.catalog.objects.iter()
        .filter(|object| can_buy(&candidate.game, &object.id))
        .filter(|object| object.price <= metric(&candidate.game, "budget"))
        .map(|object| object.id.clone())
        .collect();
    object_ids.sort_by(|a, b| {
        let left = candidate.game.catalog.objects.iter().find(|object| object.id == *a).map(|object| object.price).unwrap_or(0.0);
        let right = candidate.game.catalog.objects.iter().find(|object| object.id == *b).map(|object| object.price).unwrap_or(0.0);
        left.partial_cmp(&right).unwrap_or(std::cmp::Ordering::Equal)
    });
    object_ids.truncate(object_limit);
    purchases.extend(object_ids.into_iter().map(Some));

    let mut expanded = Vec::new();
    for purchase in purchases {
        let mut game = candidate.game.clone();
        let mut purchase_history = candidate.purchases.clone();
        if let Some(object_id) = purchase {
            if buy_object_for_sim(&mut game, &object_id).is_err() {
                continue;
            }
            purchase_history.push(object_id);
        }

        let mut actions = legal_action_ids(&game);
        actions.push(String::new());
        for action_id in actions {
            let mut next = game.clone();
            let mut action_history = candidate.actions.clone();
            if !action_id.is_empty() {
                let action = match next.catalog.actions.iter().find(|action| action.id == action_id) {
                    Some(action) => action,
                    None => continue,
                };
                if !action.encounter_id.trim().is_empty() {
                    continue;
                }
                if apply_action(&mut next, &action_id).is_err() {
                    continue;
                }
                action_history.push(action_id);
            }
            if advance_day(&mut next).is_err() {
                continue;
            }
            let branch = Candidate { game: next, actions: action_history, purchases: purchase_history.clone() };
            if metric(&branch.game, goal) >= target
                || (metric(&branch.game, "budget") >= 0.0 && metric(&branch.game, "stamina") > 0.0)
            {
                expanded.push(branch);
            }
        }
    }
    expanded
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dataset = arg(&args, "--dataset", "dataset");
    let max_days: u32 = arg(&args, "--max-days", "365").parse().unwrap_or(365);
    let beam_width: usize = arg(&args, "--beam-width", "64").parse().unwrap_or(64).max(1);
    let object_limit: usize = arg(&args, "--object-limit", "12").parse().unwrap_or(12);
    let plan_count: usize = arg(&args, "--plans", "5").parse().unwrap_or(5).max(1);
    let goal = arg(&args, "--goal", "budget");
    let target: f64 = arg(&args, "--target", "100000").parse().unwrap_or(100000.0);
    let output = arg(&args, "--output", "");

    let initial = Candidate { game: new_game(dataset.clone()), actions: vec![], purchases: vec![] };
    let catalog_actions = initial.game.catalog.actions.iter()
        .filter(|action| action.encounter_id.trim().is_empty())
        .count();
    let catalog_objects = initial.game.catalog.objects.len();
    println!(
        "dataset={} actions_considered={} objects_read={} beam_width={} max_days={}",
        dataset, catalog_actions, catalog_objects, beam_width, max_days
    );

    let mut frontier = vec![initial];
    let mut completed = Vec::new();
    for _ in 0..max_days {
        let mut next = Vec::new();
        for candidate in &frontier {
            if metric(&candidate.game, &goal) >= target {
                completed.push(candidate.clone());
                continue;
            }
            if metric(&candidate.game, "budget") < 0.0 || metric(&candidate.game, "stamina") <= 0.0 {
                continue;
            }
            next.extend(expand(candidate, &goal, target, object_limit));
        }
        if next.is_empty() {
            break;
        }
        next.sort_by(|a, b| score(b, &goal, target).partial_cmp(&score(a, &goal, target)).unwrap_or(std::cmp::Ordering::Equal));
        let mut seen = HashSet::new();
        frontier = next.into_iter()
            .filter(|candidate| seen.insert(signature(candidate, &goal)))
            .take(beam_width)
            .collect();
    }
    completed.extend(frontier.into_iter().filter(|candidate| metric(&candidate.game, &goal) >= target));
    completed.sort_by(|a, b| score(b, &goal, target).partial_cmp(&score(a, &goal, target)).unwrap_or(std::cmp::Ordering::Equal));
    let mut unique_paths = HashSet::new();
    let plans: Vec<Plan> = completed.into_iter()
        .filter(|candidate| unique_paths.insert(format!("{}|{}", candidate.actions.join("|"), candidate.purchases.join("|"))))
        .take(plan_count)
        .enumerate().map(|(index, candidate)| Plan {
        rank: index + 1,
        outcome: "success".into(),
        days: candidate.game.current_day.saturating_sub(1),
        goal_value: metric(&candidate.game, &goal),
        budget: metric(&candidate.game, "budget"),
        stamina: metric(&candidate.game, "stamina"),
        actions: candidate.actions.join("|"),
        purchases: candidate.purchases.join("|"),
    }).collect();

    if plans.is_empty() {
        println!("No successful path found.");
        return;
    }
    for plan in &plans {
        println!(
            "#{:02} days={} {}={:.1} budget={:.1} stamina={:.1} purchases=[{}] actions=[{}]",
            plan.rank, plan.days, goal, plan.goal_value, plan.budget, plan.stamina, plan.purchases, plan.actions
        );
    }
    if !output.is_empty() {
        if output.ends_with(".json") {
            std::fs::write(&output, serde_json::to_string_pretty(&plans).expect("serialize plans")).expect("write output");
        } else {
            let mut csv = String::from("rank,outcome,days,goal_value,budget,stamina,actions,purchases\n");
            for plan in &plans {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{}\n",
                    plan.rank, plan.outcome, plan.days, plan.goal_value, plan.budget,
                    plan.stamina, plan.actions, plan.purchases
                ));
            }
            std::fs::write(&output, csv).expect("write output");
        }
    }
}
