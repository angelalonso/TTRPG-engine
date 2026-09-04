import { invoke } from '@tauri-apps/api/core';
import type {
  GameState,
  TimeSpeed,
  MaintenanceType,
  RaceResult,
  ActionResult,
} from '../types/game';

/**
 * Fetches the current global game state from the Rust backend.
 */
export async function getGameState(): Promise<GameState> {
  return await invoke<GameState>('get_game_state');
}

/**
 * Updates the time simulation speed (e.g., Paused, OneDayPerSec, OneWeekPerSec).
 */
export async function setTimeSpeed(speed: TimeSpeed): Promise<GameState> {
  return await invoke<GameState>('set_time_speed', { speed });
}

/**
 * Advances the game engine calendar by exactly 1 day.
 */
export async function tickGameDay(): Promise<GameState> {
  return await invoke<GameState>('tick_game_day');
}

/**
 * Dismisses an alert pop-up from the pending alerts queue.
 */
export async function dismissAlert(alertId: string): Promise<GameState> {
  return await invoke<GameState>('dismiss_alert', { alertId });
}

/**
 * Executes a maintenance operation (oil change, engine rebuild, gearbox service, or buying tires) on an owned car.
 */
export async function maintainCar(
  carId: string,
  maintenanceType: MaintenanceType
): Promise<GameState> {
  return await invoke<GameState>('maintain_car', {
    carId,
    maintenanceType,
  });
}

/**
 * Purchases a new vehicle from the dealership catalog and adds it to the player's garage.
 */
export async function buyCar(carId: string): Promise<GameState> {
  return await invoke<GameState>('buy_car', { carId });
}

/**
 * Performs a daily job or side activity to earn money or trigger events.
 */
export async function performAction(actionId: string): Promise<ActionResult> {
  return await invoke<ActionResult>('perform_action', { actionId });
}

/**
 * Enters an eligible car into a scheduled race event on the matching day of the year.
 */
export async function enterRace(
  carId: string,
  raceId: string
): Promise<RaceResult> {
  return await invoke<RaceResult>('enter_race', {
    carId,
    raceId,
  });
}
