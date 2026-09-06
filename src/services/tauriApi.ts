import { invoke } from '@tauri-apps/api/core';
import type { GameState, TimeSpeed, ActionResult, RaceResult } from '../types/game';

export async function getGameState(): Promise<GameState> {
  return await invoke<GameState>('get_game_state');
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

export async function maintainCar(carId: string, maintenanceType: any): Promise<GameState> {
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
