#[cfg(test)]
mod tests {
    use crate::engine::{Car, GameState, Player, TimeSpeed};

    #[test]
    fn test_initial_player_state() {
        let state = GameState::new();
        assert_eq!(state.player.age_days, 18 * 365);
        assert_eq!(state.player.budget, 2000.0);
        assert_eq!(state.time_speed, TimeSpeed::Paused);
    }

    #[test]
    fn test_car_maintenance_and_race_eligibility() {
        let mut car = Car {
            id: "c1".into(),
            name: "Hatchback".into(),
            price: 3500.0,
            engine_rebuild_cost: 800.0,
            gearbox_maint_cost: 400.0,
            oil_change_cost: 80.0,
            tire_set_cost: 300.0,
            needs_oil_change: false,
            needs_engine_rebuild: false,
            needs_gearbox_maint: false,
            tire_sets_available: 4,
        };

        assert!(car.is_race_ready());

        car.tire_sets_available = 3;
        assert!(!car.is_race_ready());

        car.tire_sets_available = 4;
        car.needs_oil_change = true;
        assert!(!car.is_race_ready());
    }

    #[test]
    fn test_day_tick_and_auto_pause_on_event() {
        let mut game = GameState::new();
        game.time_speed = TimeSpeed::OneDayPerSec;

        let event = game.tick_day();

        assert_eq!(game.player.age_days, (18 * 365) + 1);

        if event.is_some() {
            assert_eq!(game.time_speed, TimeSpeed::RealTime);
        }
    }

    #[test]
    fn test_high_risk_trading_probability() {
        let mut player = Player::new(18, 2000.0);

        let _success = player.execute_trading(500.0, 0.3, 2500.0, 0.85);
        assert!(player.budget > 2000.0 || player.budget < 2000.0);
    }
}
