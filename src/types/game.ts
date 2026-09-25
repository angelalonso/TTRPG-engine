export type TimeSpeed = 'Paused' | 'OneDayEveryFiveSec' | 'OneDayPerSec' | 'OneWeekPerSec' | 'RealTime';

export type ServiceType =
  | 'Service1' | 'Service2' | 'Service3' | 'Service4' | 'Service5'
  | 'Service6' | 'Service7' | 'Service8' | 'Service9' | 'Service10'
  | 'Service11' | 'Service12' | 'Service13' | 'Service14' | 'Service15';

export interface ObjectData {
  id: string;
  type: string;
  object_type: string;
  name: string;
  price: number;
  cost_1: string;
  cost_2: string;
  cost_3: string;
  cost_4: string;
  [key: `cost_${number}`]: string | number;
  service_1_interval_days: number;
  service_2_interval_days: number;
  service_3_interval_days: number;
  service_4_interval_days: number;
  resale_initial_percent: number;
  resale_annual_percent: number;
  resale_min_percent: number;
  description_html: string;
  license_level: number;
  license_previous_id: string;
  requires_object_ids: string;
  license_fee: number;
  lifetime_days: number;
  availability_days: number;
  image_path?: string;
  trophy_championship?: string;
  trophy_position?: number;
  trophy_level?: number;
  expires_day: number;
  loaned: boolean;
  unavailable_until_day: number;
}

export interface CostData {
  id: string;
  name: string;
  amount: number;
}

export interface EventData {
  id: string;
  name: string;
  day_of_year: number;
  entry_fee: number;
  reward_pool: number;
  charisma_reward: number;
  duration_value: number;
  duration_unit: string;
  tags: string;
  description_html: string;
  required_license_id: string;
  required_object_ids: string;
  quest_id: string;
  position_rewards?: string;
  type: string;
  resolution_method: ResolutionMethod;
  success_rate: number;
  base_cost: number;
  stamina_cost: number;
  risk_factor: number;
  payout: number;
  payout_freq_type: string;
  payout_freq: number;
  payout_freq_unit: string;
  sponsor_quest_id?: string;
  sponsor_object_id?: string;
  sponsor_payouts?: string;
  sponsor_equipment_ids?: string;
  encounter_id?: string;
}

export type ResolutionMethod = 'manual' | 'random' | 'encounter';

export interface ActivityData {
  id: string;
  name: string;
  activity_type: string;
  resolution_method: ResolutionMethod;
  description_html: string;
  base_cost: number;
  stamina_cost: number;
  success_rate: number;
  payout: number;
  payout_freq_type: string;
  payout_freq: number;
  payout_freq_unit: string;
  scheduled: boolean;
}

export interface ObligationData {
  id: string;
  event_id: string;
  resource: string;
  amount: number;
  interval: number;
  interval_unit: string;
  due_days: string;
  max_payments: number;
  fault_limit: number;
  fault_consequence: string;
  completion_consequence: string;
  skip_when_sick: boolean;
  fault_blocks_payout: boolean;
  fault_title: string;
  fault_message: string;
  fault_log: string;
  limit_title: string;
  limit_message: string;
  limit_log: string;
}

export interface EventOutcomeData {
  event_id: string;
  outcome_id: string;
  probability: number;
  reward_pool_delta: number;
  charisma_reward_delta: number;
  message: string;
}

export interface EventResultData {
  result_id: string;
  event_id: string;
  event_tags: string;
  reported_result: string;
  probability: number;
  reward_pool_delta: number;
  effects: string;
  message: string;
}

export interface QuestData {
  id: string;
  type: string;
  name: string;
  success_points: number;
  failure_points: number;
  join_fee: number;
  required_license_id: string;
  description_html: string;
  championship_rewards?: string;
  driver_names?: string;
  level?: number;
}

export interface CostRule {
  id: string;
  cost_id: string;
  trigger_type: string;
  trigger_ref: string;
  amount_multiplier: number;
  probability: number;
  interval_days: number;
  charge_mode: string;
  resolution_mode: string;
  pending_message: string;
  message: string;
  damage_type: string;
  unavailable_days: number;
}

export interface CostCondition {
  rule_id: string;
  subject_type: string;
  subject_ref: string;
  operator: string;
  value: string;
}

export interface GameLabels {
  values: Record<string, string>;
}

export interface GameCatalog {
  player_characteristics: PlayerCharacteristicData[];
  objects: ObjectData[];
  costs: CostData[];
  cost_rules: CostRule[];
  cost_conditions: CostCondition[];
  events: EventData[];
  activities: ActivityData[];
  obligations: ObligationData[];
  event_outcomes: EventOutcomeData[];
  event_results: EventResultData[];
  quests: QuestData[];
  labels: GameLabels;
  encounter_attributes: EncounterAttributeData[];
  encounter_actions: EncounterActionData[];
  encounter_objects: EncounterObjectData[];
  encounter_opponents: EncounterOpponentData[];
  encounter_outcomes: EncounterOutcomeData[];
  encounter_configs: EncounterConfigData[];
  dataset_warnings: string[];
}
export interface EncounterAttributeData { attribute_id: string; display_name: string; min_value: number; max_value: number; is_loss_condition: boolean; visible_to_player: boolean; }
export interface EncounterActionData {
  action_id: string;
  display_name: string;
  usable_by: string;
  requires_attribute_id: string;
  requires_attribute_min?: number;
  requires_object_id: string;
  target_attribute_id: string;
  base_success_rate: number;
  effect_on_success: number;
  effect_on_failure: number;
  result_max?: number;
  defense_reduction?: number;
}
export interface EncounterObjectData { object_id: string; enables_action_id: string; success_rate_bonus: number; consumable_in_encounter: boolean; }
export interface EncounterOpponentData { opponent_id: string; display_name: string; starting_attributes?: string; available_action_ids?: string; strategy: string; action_weights?: string; scripted_actions?: string; }
export interface EncounterOutcomeData { outcome_id: string; trigger: string; consequence_type: string; consequence_target: string; consequence_value: string; probability: number; }
export interface EncounterConfigData { encounter_id: string; display_label: string; turn_order: string; max_turns: number; tiebreaker: string; allow_retreat: boolean; rng_mode: string; opponent_id: string; mode?: string; player_starting_attributes?: string; }
export interface EncounterState { encounter_id: string; opponent_id: string; turn: number; current_actor: string; attributes: Record<string, Record<string, number>>; cooldowns: Record<string, number>; log: EncounterLogEntry[]; finished: boolean; outcome?: string; }
export interface EncounterLogEntry { turn_number: number; actor: string; action_id: string; success: boolean; effects_applied: Record<string, number>; text: string; }
export interface EncounterResult { encounter_id: string; opponent_id: string; outcome: string; final_attribute_values: Record<string, Record<string, number>>; consequences_applied: string[]; full_log: EncounterLogEntry[]; }

export interface PlayerCharacteristicData {
  id: string;
  name: string;
  value: number;
  min_value: number;
  max_value: number;
}

export interface OwnedObject extends ObjectData {
  id: string;
  service_1_needed: boolean;
  service_2_needed: boolean;
  service_3_needed: boolean;
  service_4_needed: boolean;
  purchase_day: number;
}

export interface ActiveEvent {
  event_id: string;
  start_day: number;
  obligation_payments?: number;
  obligation_faults?: number;
}

export interface ChampionshipCompetitor {
  name: string;
  position: number;
}

export interface ChampionshipResult {
  event_id: string;
  race_day: number;
  player_position: number;
  competitors: ChampionshipCompetitor[];
}

export interface GameAlert {
  id: string;
  title: string;
  message: string;
}

export interface Player {
  name: string;
  age_days: number;
  characteristics: Record<string, number>;
  inventory: OwnedObject[];
  active_events: ActiveEvent[];
  last_event_day: number | null;
  dead?: boolean;
}

export interface GameState {
  player: Player;
  catalog: GameCatalog;
  dataset_path: string;
  time_speed: TimeSpeed;
  current_day: number;
  days_per_year: number;
  pending_alerts: GameAlert[];
  cost_ledger: CostOccurrence[];
  pending_events: PendingEvent[];
  event_history: EventHistory[];
  quest_memberships: QuestMembership[];
  championship_results: ChampionshipResult[];
  event_log: EventLogEntry[];
  active_encounter?: EncounterState | null;
  last_encounter_result?: EncounterResult | null;
  pending_sponsor_event_id?: string | null;
  alarm_event_ids: string[];
  popup_categories: string[];
}

export interface EventLogEntry {
  id: string;
  day: number;
  event: string;
}

export interface QuestMembership {
  quest_id: string;
  joined_day: number;
}

export interface PendingEvent {
  id: string;
  event_id: string;
  object_id: string;
  entered_day: number;
  rented?: boolean;
}

export interface EventHistory {
  id: string;
  event_id: string;
  object_id: string;
  entered_day: number;
  result: string;
  outcome: string;
  reward_awarded: number;
  charisma_reward_awarded: number;
  damage_type: string;
}

export interface CostOccurrence {
  id: string;
  cost_id: string;
  rule_id: string;
  amount: number;
  created_day: number;
  due_day: number;
  status: string;
  source_type: string;
  source_id: string;
}

export interface EventStartResult {
  event_name: string;
  success: boolean;
  payout_received: number;
  cost_paid: number;
  message: string;
}

export interface EventResult {
  event_name: string;
  outcome: string;
  entry_fee_paid: number;
  reward_awarded: number;
  charisma_reward_awarded: number;
  sponsor_payment: number;
  message: string;
  damage_type: string;
}

export function getLabel(catalog: GameCatalog, key: string, fallback: string): string {
  return catalog.labels?.values?.[key] || fallback;
}

export function getCharacteristic(player: Player, id: string): number {
  return player.characteristics?.[id] ?? 0;
}
