export type TimeSpeed =
  | 'Paused'
  | 'OneDayEveryFiveSec'
  | 'OneDayPerSec'
  | 'OneWeekPerSec'
  | 'RealTime';

export type MaintenanceType =
  | 'OilChange'
  | 'EngineRebuild'
  | 'GearboxService'
  | { BuyTires: number };

export type RacePosition =
  | { Placement: string }
  | { DNF: { dnf: string } };

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

export interface ActiveAction {
  action_id: string;
  start_day: number;
}

export interface GameAlert {
  id: string;
  title: string;
  message: string;
}

export interface Player {
  age_days: number;
  budget: number;
  cars: OwnedCar[];
  active_actions: ActiveAction[];
}

export interface CarSpec {
  id: string;
  name: string;
  price: number;
  engine_rebuild_cost: number;
  gearbox_maint_cost: number;
  oil_change_cost: number;
  tire_set_cost: number;
}

export interface ActionSpec {
  id: string;
  name: string;
  description: string;
  base_cost: number;
  payout: number;
  payout_freq: number;
  payout_freq_unit: string;
  payout_freq_type: string;
  success_rate: number;
}

export interface RaceSpec {
  id: string;
  name: string;
  day_of_year: number;
  entry_fee: number;
  prize_pool: number;
}

export interface GameCatalog {
  cars: CarSpec[];
  actions: ActionSpec[];
  races: RaceSpec[];
}

export interface GameState {
  player: Player;
  catalog: GameCatalog;
  time_speed: TimeSpeed;
  current_day: number;
  pending_alerts: GameAlert[];
}

export interface RaceResult {
  race_name: string;
  position: RacePosition;
  entry_fee_paid: number;
  prize_awarded: number;
  message: string;
}

export interface ActionResult {
  action_name: string;
  success: boolean;
  payout_received: number;
  cost_paid: number;
  message: string;
}
