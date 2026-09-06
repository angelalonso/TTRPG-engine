use crate::engine::loader::RaceData;
use crate::engine::race::{
    enter_race, perform_maintenance, validate_race_entry, MaintenanceType,
};
use crate::engine::{Car, GameState, Player};

fn setup_test_car() -> Car {
    Car {
        id: "c1".into(),
        name: "Track Special".into(),
        price: 10000.0,
        engine_rebuild_cost: 1000.0,
        gearbox_maint_cost: 500.0,
        oil_change_cost: 100.0,
        tire_set_cost: 200.0,
        needs_oil_change: false,
        needs_engine_rebuild: false,
        needs_gearbox_maint: false,
        tire_sets_available: 4,
    }
}

fn setup_test_race() -> RaceData {
    RaceData {
        id: "r1".into(),
        name: "Silverstone Trophy".into(),
        day_of_year: 100,
        entry_fee: 500.0,
        prize_pool: 3000.0,
    }
}

// #[test]
// fn test_race_entry_validation() {
//     let mut car = setup_test_car();
//     let race = setup_test_race();
// 
//     let mut player = Player::new(18, 2000.0);
//     player.cars.push(car.clone());
// 
//     assert!(validate_race_entry(&player, &car, &race, 100).is_ok());
// 
//     let mut broke_player = Player::new(18, 200.0);
//     broke_player.cars.push(car.clone());
//     assert!(validate_race_entry(&broke_player, &car, &race, 100).is_err());
// 
//     assert!(validate_race_entry(&player, &car, &race, 101).is_err());
// 
//     car.tire_sets_available = 3;
//     player.cars[0] = car.clone();
//     assert!(validate_race_entry(&player, &car, &race, 100).is_err());
// 
//     car.tire_sets_available = 4;
//     car.needs_engine_rebuild = true;
//     player.cars[0] = car.clone();
//     assert!(validate_race_entry(&player, &car, &race, 100).is_err());
// }
#[test]
fn test_race_entry_validation() {
    let mut car = setup_test_car();
    let race = setup_test_race();

    let mut player = Player::new(18, 2000.0);
    player.cars.push(car.clone());

    // Replacing .is_ok() with .expect() reveals the exact Err message when it panics
    validate_race_entry(&player, &car, &race, 100).expect("Baseline race entry validation failed");

    let mut broke_player = Player::new(18, 200.0);
    broke_player.cars.push(car.clone());
    assert!(validate_race_entry(&broke_player, &car, &race, 100).is_err());

    assert!(validate_race_entry(&player, &car, &race, 101).is_err());

    car.tire_sets_available = 3;
    player.cars[0] = car.clone();
    assert!(validate_race_entry(&player, &car, &race, 100).is_err());

    car.tire_sets_available = 4;
    car.needs_engine_rebuild = true;
    player.cars[0] = car.clone();
    assert!(validate_race_entry(&player, &car, &race, 100).is_err());
}

#[test]
fn test_maintenance_repairs_and_expenses() {
    let mut player = Player::new(18, 2000.0);
    let mut car = setup_test_car();
    car.needs_oil_change = true;

    perform_maintenance(&mut player, &mut car, MaintenanceType::OilChange).unwrap();

    assert!(!car.needs_oil_change);
    assert_eq!(player.budget, 1900.0);

    perform_maintenance(&mut player, &mut car, MaintenanceType::BuyTires(4)).unwrap();
    assert_eq!(car.tire_sets_available, 8);
    assert_eq!(player.budget, 1100.0);
}

#[test]
fn test_race_execution_consumes_resources_and_flags_oil() {
    let mut game = GameState::new();
    game.player.budget = 2000.0;
    let mut car = setup_test_car();
    let race = setup_test_race();

    let result = enter_race(&mut game.player, &mut car, &race, &game.catalog.actions, 0.85).unwrap();

    assert_eq!(car.tire_sets_available, 0);
    assert!(car.needs_oil_change);
    assert_eq!(result.entry_fee_paid, 500.0);
}
