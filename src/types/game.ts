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
  expires_day: number;
  unavailable_until_day: number;
}

export interface CostData {
  id: string;
  name: string;
  amount: number;
}

export interface ActionData {
  id: string;
  name: string;
  type: string;
  base_cost: number;
  stamina_cost: number;
  risk_factor: number;
  success_rate: number;
  payout: number;
  payout_freq_type: string;
  payout_freq: number;
  payout_freq_unit: string;
  description_html: string;
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
  actions: ActionData[];
  events: EventData[];
  quests: QuestData[];
  labels: GameLabels;
}

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
  characteristics: Record<string, number>;
  inventory: OwnedObject[];
  active_actions: ActiveAction[];
  last_action_day: number | null;
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

export interface ActionResult {
  action_name: string;
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
  message: string;
  damage_type: string;
}

export function getLabel(catalog: GameCatalog, key: string, fallback: string): string {
  return catalog.labels?.values?.[key] || fallback;
}

export function getCharacteristic(player: Player, id: string): number {
  return player.characteristics?.[id] ?? 0;
}
