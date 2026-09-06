use crate::engine::loader::{ActionData, RaceData};
use crate::engine::{Car, Player};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceType {
    EngineRebuild,
    GearboxService,
    OilChange,
    BuyTires(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RacePosition {
    First,
    Second,
    Third,
    Unplaced,
    DNF(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResult {
    pub race_name: String,
    pub position: RacePosition,
    pub entry_fee_paid: f64,
    pub prize_awarded: f64,
    pub message: String,
}

pub fn validate_race_entry(
    player: &Player,
    car: &Car,
    race: &RaceData,
    current_day: u32,
) -> Result<(), String> {
    let day_in_year = ((current_day.saturating_sub(1)) % 365) + 1;
    if day_in_year != race.day_of_year {
        return Err(format!(
            "Race takes place on day {}, today is day {}.",
            race.day_of_year, day_in_year
        ));
    }

    if player.budget < race.entry_fee {
        return Err(format!(
            "Insufficient funds. Entry fee is £{:.2}, you have £{:.2}.",
            race.entry_fee, player.budget
        ));
    }

    if car.needs_engine_rebuild {
        return Err("Car requires an engine rebuild before racing.".into());
    }
    if car.needs_gearbox_maint {
        return Err("Car gearbox needs servicing before racing.".into());
    }
    if car.needs_oil_change {
        return Err("Car needs an oil change before racing.".into());
    }
    if car.tire_sets_available < 4 {
        return Err(format!(
            "Requires 4 fresh tire sets for the race weekend (Current: {}).",
            car.tire_sets_available
        ));
    }

    Ok(())
}

pub fn perform_maintenance(
    player: &mut Player,
    car: &mut Car,
    maint: MaintenanceType,
) -> Result<(), String> {
    let cost = match maint {
        MaintenanceType::EngineRebuild => car.engine_rebuild_cost,
        MaintenanceType::GearboxService => car.gearbox_maint_cost,
        MaintenanceType::OilChange => car.oil_change_cost,
        MaintenanceType::BuyTires(count) => car.tire_set_cost * count as f64,
    };

    if player.budget < cost {
        return Err(format!("Cannot afford maintenance. Cost: £{:.2}", cost));
    }

    player.budget -= cost;

    match maint {
        MaintenanceType::EngineRebuild => car.needs_engine_rebuild = false,
        MaintenanceType::GearboxService => car.needs_gearbox_maint = false,
        MaintenanceType::OilChange => car.needs_oil_change = false,
        MaintenanceType::BuyTires(count) => car.tire_sets_available += count,
    }

    Ok(())
}

pub fn enter_race(
    player: &mut Player,
    car: &mut Car,
    race: &RaceData,
    actions: &[ActionData],
    roll: f64,
) -> Result<RaceResult, String> {
    player.budget -= race.entry_fee;

    car.tire_sets_available -= 4;
    car.needs_oil_change = true;

    let (position, prize_percentage, message) = match roll {
        r if r > 0.95 => (
            RacePosition::DNF("Engine overheated on lap 12!".into()),
            0.0,
            "Did Not Finish due to mechanical failure.".into(),
        ),
        r if r > 0.80 => (
            RacePosition::First,
            1.0,
            "Sensational victory! You dominated the race!".into(),
        ),
        r if r > 0.60 => (
            RacePosition::Second,
            0.5,
            "Great performance! Finished 2nd on the podium.".into(),
        ),
        r if r > 0.40 => (
            RacePosition::Third,
            0.25,
            "P3 finish! Earned a spot on the bottom step of the podium.".into(),
        ),
        _ => (
            RacePosition::Unplaced,
            0.0,
            "Finished outside the prize positions.".into(),
        ),
    };

    let prize_awarded = race.prize_pool * prize_percentage;
    player.budget += prize_awarded;

    for active in &player.active_actions {
        if let Some(action) = actions.iter().find(|a| a.id == active.action_id) {
            if action.payout_freq_unit.trim() == "race" {
                player.budget += action.payout;
            }
        }
    }

    Ok(RaceResult {
        race_name: race.name.clone(),
        position,
        entry_fee_paid: race.entry_fee,
        prize_awarded,
        message,
    })
}
