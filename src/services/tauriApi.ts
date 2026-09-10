import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  ActionResult,
  EventResult,
  GameCatalog,
  GameState,
  ServiceType,
  TimeSpeed,
} from '../types/game';

export const getGameState = () => invoke<GameState>('get_game_state');
export const fetchCatalog = () => invoke<GameCatalog>('get_catalog');
export const saveGame = () => invoke<string>('save_game');
export const loadGame = () => invoke<GameState>('load_game');
export const setTimeSpeed = (speed: TimeSpeed) => invoke<GameState>('set_time_speed', { speed });
export const tickGameDay = () => invoke<GameState>('tick_game_day');
export const buyObject = (objectId: string) => invoke<GameState>('buy_object', { objectId });
export const joinQuest = (questId: string) =>
  invoke<GameState>('join_quest', { questId });
export const sellObject = (objectId: string) => invoke<GameState>('sell_object', { objectId });
export const serviceObject = (objectId: string, serviceType: ServiceType) =>
  invoke<GameState>('service_object', { objectId, serviceType });
export const performAction = (actionId: string) =>
  invoke<ActionResult>('perform_action', { actionId });
export const enterEvent = (objectId: string, eventId: string) =>
  invoke<GameState>('enter_event', { objectId, eventId });
export const submitEventResult = (entryId: string, result: string, damageType: string) =>
  invoke<EventResult>('submit_event_result', { entryId, result, damageType });
export const loadDescription = (path: string) =>
  invoke<string>('load_description', { path });
export const dismissAlert = (alertId: string) =>
  invoke<GameState>('dismiss_alert', { alertId });
export const payCost = (costOccurrenceId: string) =>
  invoke<GameState>('pay_cost', { costOccurrenceId });
export const reloadDataset = (newPath: string) =>
  invoke<GameState>('reload_dataset', { newPath });

export async function selectDatasetFolder(defaultPath = './dataset'): Promise<string | null> {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath,
    });
    return Array.isArray(selected) ? selected[0] ?? null : selected;
  } catch (error) {
    console.error('Failed to open dataset picker:', error);
    return null;
  }
}
