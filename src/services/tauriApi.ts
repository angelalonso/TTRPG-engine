import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  EventStartResult,
  ChampionshipCompetitor,
  EventResult,
  GameCatalog,
  GameState,
  ServiceType,
  TimeSpeed,
  EncounterState,
  EncounterResult,
} from '../types/game';
import type { ThemeColors } from '../types/theme';

export const getGameState = () => invoke<GameState>('get_game_state');
export const getThemeColors = () => invoke<ThemeColors>('get_theme_colors');
export const fetchCatalog = () => invoke<GameCatalog>('get_catalog');
export const getDefaultDatasetDialogPath = () => invoke<string>('default_dataset_dialog_path');
export const saveGame = () => invoke<string>('save_game');
export const loadGame = () => invoke<GameState>('load_game');
export interface SaveSlot { name: string }
export const listSaveSlots = (datasetPath: string) =>
  invoke<SaveSlot[]>('list_save_slots', { datasetPath });
export const startNewGame = (datasetPath: string, playerName: string) =>
  invoke<GameState>('start_new_game', { datasetPath, playerName });
export const saveGameAs = (slot: string) =>
  invoke<string>('save_game_as', { slot });
export const loadGameFrom = (datasetPath: string, slot: string) =>
  invoke<GameState>('load_game_from', { datasetPath, slot });
export const setTimeSpeed = (speed: TimeSpeed) => invoke<GameState>('set_time_speed', { speed });
export const tickGameDay = () => invoke<GameState>('tick_game_day');
export const buyObject = (objectId: string) => invoke<GameState>('buy_object', { objectId });
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
export const submitEventResult = (
  entryId: string,
  result: string,
  damageType: string,
  playerPosition?: number,
  competitors: ChampionshipCompetitor[] = [],
) => invoke<EventResult>('submit_event_result', {
  entryId,
  result,
  damageType,
  playerPosition,
  competitors,
});
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

export async function selectDatasetFolder(defaultPath = './dataset'): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath,
  });
  return Array.isArray(selected) ? selected[0] ?? null : selected;
}
