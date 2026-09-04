import { invoke } from '@tauri-apps/api/core';
/**
 * Fetches the current global game state from the Rust backend.
 */
export async function getGameState() {
    return await invoke('get_game_state');
}
/**
 * Updates the time simulation speed (e.g., Paused, OneDayPerSec, OneWeekPerSec).
 */
export async function setTimeSpeed(speed) {
    return await invoke('set_time_speed', { speed });
}
/**
 * Advances the game engine calendar by exactly 1 day.
 */
export async function tickGameDay() {
    return await invoke('tick_game_day');
}
/**
 * Dismisses an alert pop-up from the pending alerts queue.
 */
export async function dismissAlert(alertId) {
    return await invoke('dismiss_alert', { alertId });
}
/**
 * Executes a maintenance operation (oil change, engine rebuild, gearbox service, or buying tires) on an owned car.
 */
export async function maintainCar(carId, maintenanceType) {
    return await invoke('maintain_car', {
        carId,
        maintenanceType,
    });
}
/**
 * Purchases a new vehicle from the dealership catalog and adds it to the player's garage.
 */
export async function buyCar(carId) {
    return await invoke('buy_car', { carId });
}
/**
 * Performs a daily job or side activity to earn money or trigger events.
 */
export async function performAction(actionId) {
    return await invoke('perform_action', { actionId });
}
/**
 * Enters an eligible car into a scheduled race event on the matching day of the year.
 */
export async function enterRace(carId, raceId) {
    return await invoke('enter_race', {
        carId,
        raceId,
    });
}
//# sourceMappingURL=tauriApi.js.map