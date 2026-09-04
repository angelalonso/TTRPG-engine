import type { GameState, TimeSpeed, MaintenanceType, RaceResult, ActionResult } from '../types/game';
/**
 * Fetches the current global game state from the Rust backend.
 */
export declare function getGameState(): Promise<GameState>;
/**
 * Updates the time simulation speed (e.g., Paused, OneDayPerSec, OneWeekPerSec).
 */
export declare function setTimeSpeed(speed: TimeSpeed): Promise<GameState>;
/**
 * Advances the game engine calendar by exactly 1 day.
 */
export declare function tickGameDay(): Promise<GameState>;
/**
 * Dismisses an alert pop-up from the pending alerts queue.
 */
export declare function dismissAlert(alertId: string): Promise<GameState>;
/**
 * Executes a maintenance operation (oil change, engine rebuild, gearbox service, or buying tires) on an owned car.
 */
export declare function maintainCar(carId: string, maintenanceType: MaintenanceType): Promise<GameState>;
/**
 * Purchases a new vehicle from the dealership catalog and adds it to the player's garage.
 */
export declare function buyCar(carId: string): Promise<GameState>;
/**
 * Performs a daily job or side activity to earn money or trigger events.
 */
export declare function performAction(actionId: string): Promise<ActionResult>;
/**
 * Enters an eligible car into a scheduled race event on the matching day of the year.
 */
export declare function enterRace(carId: string, raceId: string): Promise<RaceResult>;
//# sourceMappingURL=tauriApi.d.ts.map