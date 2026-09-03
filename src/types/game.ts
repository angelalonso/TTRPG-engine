/**
 * Time progression speeds supported by the tick engine backend.
 */
export type TimeSpeed =
  | 'Paused'
  | 'OneDayEveryFiveSec'
  | 'OneDayPerSec'
  | 'OneWeekPerSec'
  | 'RealTime';

/**
 * Maintenance operations that can be performed on an owned vehicle in the garage.
 */
export type MaintenanceType =
  | 'EngineRebuild'
  | 'GearboxService'
  | 'OilChange'
  | { BuyTires: number };

/**
 * Possible race finishing positions returned by the backend simulator,
 * including Serde untagged/adjacently tagged DNF variants.
 */
export type RacePosition =
  | 'First'
  | 'Second'
  | 'Third'
  | 'Unplaced'
  | { DNF: string }
  | string;

/**
 * Dealership specifications for purchasable vehicles in the global catalog.
 */
export interface CarData {
  id: string;
  name: string;
  price: number;
  engine_rebuild_cost: number;
  gearbox_maint_cost: number;
  oil_change_cost: number;
  tire_set_cost: number;
}

/**
 * Daily jobs, side activities, or financial actions available to the player.
 */
export interface ActionData {
  id: string;
  name: string;
  type: string;
  base_cost: number;
  risk_factor: number;
  success_rate: number;
  payout: number;
}

/**
 * Scheduled race event metadata defined in the game catalog.
 */
export interface RaceData {
  id: string;
  name: string;
  day_of_year: number;
  entry_fee: number;
  prize_pool: number;
}

/**
 * Complete game catalog containing static reference data loaded from Rust backend.
 */
export interface GameCatalog {
  cars: CarData[];
  actions: ActionData[];
  races: RaceData[];
}

/**
 * Vehicle instance owned by the player, tracking condition and available tire inventory.
 */
export interface OwnedCar {
  id: string;
  name: string;
  price: number;
  engine_rebuild_cost: number;
  gearbox_maint_cost: number;
  oil_change_cost: number;
  tire_set_cost: number;
  needs_oil_change: boolean;
  needs_engine_rebuild: boolean;
  needs_gearbox_maint: boolean;
  tire_sets_available: number;
}

/**
 * Player stats, age metrics, budget, and owned garage inventory.
 */
export interface Player {
  age_days: number;
  budget: number;
  cars: OwnedCar[];
}

/**
 * Global game state payload synchronized between Rust backend and React frontend.
 */
export interface GameState {
  player: Player;
  catalog: GameCatalog;
  time_speed: TimeSpeed;
  current_day: number;
}

/**
 * Outcome payload returned after executing the `enter_race` IPC command.
 */
export interface RaceResult {
  race_name: string;
  position: RacePosition;
  entry_fee_paid: number;
  prize_awarded: number;
  message: string;
}

/**
 * Outcome payload returned after executing the `perform_action` IPC command.
 */
export interface ActionResult {
  action_name: string;
  success: boolean;
  payout_received: number;
  cost_paid: number;
  message: string;
}
