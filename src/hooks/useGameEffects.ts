import { useEffect, type Dispatch, type SetStateAction } from 'react';
import { getLabel } from '../types/game';
import type { GameState } from '../types/game';
import { getThemeColors, tickGameDay } from '../services/tauriApi';
import { loadCachedDatasetAsset } from '../services/datasetAssetCache';
import { datasetThumbnailPath } from '../utils/assets';

interface UseGameEffectsOptions {
  gameState: GameState | null;
  setGameState: Dispatch<SetStateAction<GameState | null>>;
  setMessage: (message: string) => void;
  setMarketSort: (sort: 'name' | 'price' | 'owned' | 'availability') => void;
  setPlayerImage: (image: string) => void;
}

export const useGameEffects = ({
  gameState,
  setGameState,
  setMessage,
  setMarketSort,
  setPlayerImage,
}: UseGameEffectsOptions) => {
  useEffect(() => {
    getThemeColors()
      .then((colors) => {
        for (const [elementId, color] of Object.entries(colors)) {
          document.documentElement.style.setProperty(`--${elementId.replaceAll('_', '-')}`, color);
        }
      })
      .catch((error) => setMessage(String(error)));
  }, [setMessage]);

  useEffect(() => {
    if (!gameState || gameState.time_speed === 'Paused' || gameState.pending_alerts.length > 0) return;
    const interval = gameState.time_speed === 'OneDayEveryFiveSec' ? 5000
      : gameState.time_speed === 'OneWeekPerSec' ? 1000 / 7 : 1000;
    const timer = setInterval(() => tickGameDay().then(setGameState).catch((error) => setMessage(String(error))), interval);
    return () => clearInterval(timer);
  }, [gameState?.time_speed, gameState?.pending_alerts.length, setGameState, setMessage]);

  useEffect(() => {
    if (!gameState) return;
    const configuredSort = getLabel(gameState.catalog, 'market_default_sort', 'price');
    if (configuredSort === 'name' || configuredSort === 'price' || configuredSort === 'owned' || configuredSort === 'availability') {
      setMarketSort(configuredSort);
    }
  }, [gameState?.catalog, setMarketSort]);

  useEffect(() => {
    if (!gameState) return;
    const configured = gameState.catalog.player_characteristics.find((entry) => entry.id === 'picture_file')?.name.trim();
    if (!configured) {
      setPlayerImage('/img/player_driver.jpeg');
      return;
    }
    loadCachedDatasetAsset(
      datasetThumbnailPath(configured),
      `${gameState.dataset_path}:${datasetThumbnailPath(configured)}`,
    )
      .then(setPlayerImage)
      .catch(() => setPlayerImage(''));
  }, [gameState?.catalog, gameState?.dataset_path, setPlayerImage]);
};
