import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { GameState, TimeSpeed, ActionResult, RaceResult, GameCatalog, MaintenanceType } from '../types/game';

export async function getGameState(): Promise<GameState> {
  return await invoke<GameState>('get_game_state');
}

export async function fetchCatalog(): Promise<GameCatalog> {
  return await invoke<GameCatalog>('get_catalog');
}

export async function setTimeSpeed(speed: TimeSpeed): Promise<GameState> {
  return await invoke<GameState>('set_time_speed', { speed });
}

export async function tickGameDay(): Promise<GameState> {
  return await invoke<GameState>('tick_game_day');
}

export async function buyCar(carId: string): Promise<GameState> {
  return await invoke<GameState>('buy_car', { carId });
}

export async function maintainCar(carId: string, maintenanceType: MaintenanceType): Promise<GameState> {
  return await invoke<GameState>('maintain_car', { carId, maintenanceType });
}

export async function performAction(actionId: string): Promise<ActionResult> {
  return await invoke<ActionResult>('perform_action', { actionId });
}

export async function enterRace(carId: string, raceId: string): Promise<RaceResult> {
  return await invoke<RaceResult>('enter_race', { carId, raceId });
}

export async function dismissAlert(alertId: string): Promise<GameState> {
  return await invoke<GameState>('dismiss_alert', { alertId });
}

export async function reloadDataset(newPath: string): Promise<GameState> {
  return await invoke<GameState>('reload_dataset', { newPath });
}

export async function selectDatasetFolder(defaultPath?: string): Promise<string | null> {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: defaultPath || './dataset',
    });
    if (Array.isArray(selected)) {
      return selected[0] ?? null;
    }
    return selected;
  } catch (err) {
    console.error('Failed to open file picker:', err);
    return null;
  }
}
