import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  EventStartResult,
  EventEligibility,
  ObjectTransactionEligibility,
  ChampionshipCompetitor,
  EventResult,
  GameCatalog,
  GameState,
  ServiceType,
  TimeSpeed,
  EncounterState,
  EncounterResult,
  SponsorData,
} from '../types/game';
import type { ThemeColors } from '../types/theme';

const DATASET_PATHS_STORAGE_KEY = 'ttrpg-engine.dataset-paths';

export interface AppConfig {
  dataset_path: string;
  fullscreen: boolean;
  window_width: number;
  window_height: number;
  game_directory?: string;
  results_directory?: string;
  driver_names?: string[];
}

export interface RaceResultsPluginResponse {
  result: string;
  player_position: number;
  competitors: ChampionshipCompetitor[];
  damage_type: string;
  pole_position?: boolean;
  standings: Array<{ name: string; points: number }>;
  detected_file?: string;
  track_id?: string;
  racers?: Array<Record<string, string | number>>;
  driver_name?: string;
  aidb?: string;
  import_error?: string;
}

export function getRememberedDatasetPath(): string | null {
  return getRememberedDatasetPaths()[0] || null;
}

export function getRememberedDatasetPaths(): string[] {
  try {
    const paths = JSON.parse(localStorage.getItem(DATASET_PATHS_STORAGE_KEY) || '[]');
    return Array.isArray(paths)
      ? paths.filter((entry): entry is string => typeof entry === 'string' && entry.trim().length > 0)
      : [];
  } catch {
    return [];
  }
}

export function rememberDatasetPath(path: string): void {
  const normalized = path.trim();
  if (!normalized) return;
  try {
    const stored = JSON.parse(localStorage.getItem(DATASET_PATHS_STORAGE_KEY) || '[]');
    const paths = Array.isArray(stored) ? stored.filter((entry): entry is string => typeof entry === 'string') : [];
    localStorage.setItem(
      DATASET_PATHS_STORAGE_KEY,
      JSON.stringify([normalized, ...paths.filter((entry) => entry !== normalized)].slice(0, 20)),
    );
  } catch {
    localStorage.setItem(DATASET_PATHS_STORAGE_KEY, JSON.stringify([normalized]));
  }
}

export function forgetDatasetPath(path: string): void {
  try {
    const stored = JSON.parse(localStorage.getItem(DATASET_PATHS_STORAGE_KEY) || '[]');
    const paths = Array.isArray(stored) ? stored.filter((entry): entry is string => typeof entry === 'string') : [];
    localStorage.setItem(
      DATASET_PATHS_STORAGE_KEY,
      JSON.stringify(paths.filter((entry) => entry !== path)),
    );
  } catch {
    localStorage.removeItem(DATASET_PATHS_STORAGE_KEY);
  }
}

export const getGameState = () => invoke<GameState>('get_game_state');
export const getAppConfig = () => invoke<AppConfig>('get_app_config');
export const saveAppConfig = (config: AppConfig) => invoke<AppConfig>('save_app_config', { config });
export const getThemeColors = () => invoke<ThemeColors>('get_theme_colors');
export interface DatasetTable {
  file: string;
  headers: string[];
  rows: Array<Record<string, string>>;
}
export const listDatasetTables = (datasetPath: string) =>
  invoke<DatasetTable[]>('list_dataset_tables', { datasetPath });
export const saveDatasetTable = (
  datasetPath: string,
  file: string,
  headers: string[],
  rows: Array<Record<string, string>>,
) => invoke<void>('save_dataset_table', { datasetPath, file, headers, rows });
export const fetchCatalog = () => invoke<GameCatalog>('get_catalog');
export const listSponsors = () => invoke<SponsorData[]>('list_sponsors');
export const getDefaultDatasetDialogPath = () => invoke<string>('default_dataset_dialog_path');
export const isDatasetPath = (path: string) => invoke<boolean>('is_dataset_path', { path });
export const saveGame = () => invoke<string>('save_game');
export const loadGame = () => invoke<GameState>('load_game');
export interface SaveSlot {
  name: string;
  dataset_revision_number: number;
  current_dataset_revision_number: number;
  dataset_revision_matches: boolean;
  can_migrate: boolean;
  migration_steps: number;
}
export const listSaveSlots = (datasetPath: string) =>
  invoke<SaveSlot[]>('list_save_slots', { datasetPath });
export const getLatestSaveSlot = (datasetPath: string) =>
  invoke<SaveSlot | null>('latest_save_slot', { datasetPath });
export const startNewGame = (datasetPath: string, playerName: string) =>
  invoke<GameState>('start_new_game', { datasetPath, playerName });
export const saveGameAs = (slot: string) =>
  invoke<string>('save_game_as', { slot });
export const loadGameFrom = (datasetPath: string, slot: string) =>
  invoke<GameState>('load_game_from', { datasetPath, slot });
export const migrateSaveGameFrom = (datasetPath: string, slot: string) =>
  invoke<GameState>('migrate_save_game_from', { datasetPath, slot });
export const deleteSaveSlot = (datasetPath: string, slot: string) =>
  invoke<void>('delete_save_slot', { datasetPath, slot });
export const setTimeSpeed = (speed: TimeSpeed) => invoke<GameState>('set_time_speed', { speed });
export const tickGameDay = () => invoke<GameState>('tick_game_day');
export const buyObject = (objectId: string) => invoke<GameState>('buy_object', { objectId });
export const switchInsurance = (objectId: string) =>
  invoke<GameState>('switch_insurance', { objectId });
export const terminateInsurance = (objectId: string) =>
  invoke<GameState>('terminate_insurance', { objectId });
export const joinQuest = (questId: string) =>
  invoke<GameState>('join_quest', { questId });
export const sellObject = (objectId: string) => invoke<GameState>('sell_object', { objectId });
export const serviceObject = (objectId: string, serviceType: ServiceType) =>
  invoke<GameState>('service_object', { objectId, serviceType });
export const performEvent = (eventId: string) =>
  invoke<EventStartResult>('perform_event', { eventId });
export const quitEvent = (eventId: string) =>
  invoke<GameState>('quit_event', { eventId });
export const enterEvent = (objectId: string, eventId: string) =>
  invoke<GameState>('enter_event', { objectId, eventId });
export const getEventEligibility = (eventId: string) =>
  invoke<EventEligibility[]>('get_event_eligibility', { eventId });
export const getObjectTransactionEligibility = () =>
  invoke<ObjectTransactionEligibility[]>('get_object_transaction_eligibility');
export const rentEvent = (objectId: string, eventId: string) =>
  invoke<GameState>('rent_event', { objectId, eventId });
export const submitEventResult = (
  entryId: string,
  result: string,
  damageType: string,
  playerPosition?: number,
  competitors: ChampionshipCompetitor[] = [],
  pluginResponse?: RaceResultsPluginResponse,
) => invoke<EventResult>('submit_event_result', {
  entryId,
  result,
  damageType,
  playerPosition,
  competitors,
  pluginResponse,
});
export const openRaceResultsPlugin = (entryId: string) =>
  invoke<RaceResultsPluginResponse>('open_race_results_plugin', { entryId });
export const autodetectRaceResultsPlugin = (entryId: string, resultsFile?: string) =>
  invoke<RaceResultsPluginResponse>('autodetect_race_results_plugin', {
    entryId,
    resultsFile: resultsFile || null,
  });
export const openSponsorNegotiation = (
  eventId = '',
  sponsorId?: string,
  approach: 'cold_call' | 'proposal' = 'cold_call',
) => invoke<SponsorNegotiationStart>('open_sponsor_negotiation', { eventId, sponsorId, approach });
export interface SponsorNegotiationStart {
  state: SponsorNegotiationState;
  game_state: GameState;
}
export interface SponsorNegotiationState {
  status: string;
  phase?: string;
  sponsor_name: string;
  sponsor_id: string;
  sponsor_tier: string;
  sponsor_brand: string;
  race_tier: string;
  scope: string;
  attraction_score: number;
  manager_level: number;
  cold_call: boolean;
  proposal: Record<string, number | boolean>;
  ideal_proposal: Record<string, number | boolean>;
  sponsor_dialogue: string;
  last_action_log: string;
  objection: string;
  round: number;
  agreement?: Record<string, unknown>;
  final_log?: string;
  target_options?: Array<{ id: string; name: string }>;
  vehicle_options?: Array<{ id: string; name: string; price: number }>;
  selected_target?: string;
  selected_car?: string;
  error?: string;
  [key: string]: unknown;
}
export const sponsorNegotiationAction = (action: Record<string, unknown>) =>
  invoke<{ state: SponsorNegotiationState; game_state: GameState }>('sponsor_negotiation_action', { action });
export const closeSponsorNegotiation = () => invoke<void>('close_sponsor_negotiation');
export const loadDescription = (path: string) =>
  invoke<string>('load_description', { path });
export const loadDatasetAsset = (path: string) =>
  invoke<string>('load_dataset_asset', { path });
export const dismissAlert = (alertId: string) =>
  invoke<GameState>('dismiss_alert', { alertId });
export const toggleAlarm = (eventId: string) =>
  invoke<GameState>('toggle_alarm', { eventId });
export const setPopupCategories = (categories: string[]) =>
  invoke<GameState>('set_popup_categories', { categories });
export const payCost = (costOccurrenceId: string) =>
  invoke<GameState>('pay_cost', { costOccurrenceId });
export const reloadDataset = (newPath: string) =>
  invoke<GameState>('reload_dataset', { newPath });
export const startEncounter = (encounterId: string, opponentId: string) =>
  invoke<EncounterState>('start_encounter', { encounterId, opponentId });
export const resolveEncounterTurn = (actionId?: string) =>
  invoke<EncounterState | EncounterResult>('resolve_encounter_turn', { actionId });
export const retreatEncounter = () => invoke<EncounterResult>('retreat_encounter');

export async function selectDatasetFolder(defaultPath = './gtr2career'): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath,
  });
  return Array.isArray(selected) ? selected[0] ?? null : selected;
}

export async function selectRaceResultsFile(defaultPath = ''): Promise<string | null> {
  const selected = await open({
    directory: false,
    multiple: false,
    defaultPath: defaultPath || undefined,
    filters: [
      { name: 'GTR2 result files', extensions: ['txt'] },
      { name: 'All files', extensions: ['*'] },
    ],
  });
  return Array.isArray(selected) ? selected[0] ?? null : selected;
}
