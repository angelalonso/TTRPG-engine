import { invoke } from '@tauri-apps/api/core';
import type {
  GameCatalog,
  GameState,
  TimeSpeed,
  MaintenanceType,
  ActionResult,
  RaceResult,
} from '../types/game';

/**
 * Fetches the current global game state from the Rust backend.
 */
export async function getGameState(): Promise<GameState> {
  return await invoke('get_game_state');
}

/**
 * Fetches the game catalog containing cars, actions, and races.
 */
export async function fetchCatalog(): Promise<GameCatalog> {
  return await invoke('fetch_catalog');
}

/**
 * Updates the time simulation speed and returns the updated game state.
 */
export async function setTimeSpeed(speed: TimeSpeed): Promise<GameState> {
  return await invoke('set_time_speed', { speed });
}

/**
 * Advances the game engine calendar by 1 day and returns the updated game state.
 */
export async function tickGameDay(): Promise<GameState> {
  return await invoke('tick_game_day');
}

/**
 * Dismisses an alert pop-up from the pending alerts queue.
 */
export async function dismissAlert(alertId: string): Promise<GameState> {
  return await invoke('dismiss_alert', { alertId });
}

/**
 * Executes a maintenance operation on an owned car and returns the updated game state.
 */
export async function maintainCar(
  carId: string,
  maintenanceType: MaintenanceType
): Promise<GameState> {
  return await invoke('maintain_car', {
    carId,
    maintenanceType,
  });
}

/**
 * Purchases a new vehicle and returns the updated game state.
 */
export async function buyCar(carId: string): Promise<GameState> {
  return await invoke('buy_car', { carId });
}

/**
 * Performs a daily job or side activity and returns an ActionResult.
 */
export async function performAction(actionId: string): Promise<ActionResult> {
  return await invoke('perform_action', { actionId });
}

/**
 * Enters an eligible car into a scheduled race event and returns a RaceResult.
 */
export async function enterRace(
  carId: string,
  raceId: string
): Promise<RaceResult> {
  return await invoke('enter_race', {
    carId,
    raceId,
  });
}
