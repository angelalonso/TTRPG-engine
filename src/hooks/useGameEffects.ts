import { useEffect, type Dispatch, type SetStateAction } from 'react';
import { getLabel } from '../types/game';
import type { GameState } from '../types/game';
import {
  getThemeColors,
  loadDatasetAsset,
  tickGameDay,
} from '../services/tauriApi';

interface UseGameEffectsOptions {
  gameState: GameState | null;
  setGameState: Dispatch<SetStateAction<GameState | null>>;
  setMessage: (message: string) => void;
  setMarketImages: (images: Record<string, string>) => void;
  setMarketSort: (sort: 'name' | 'price') => void;
  setPlayerImage: (image: string) => void;
}

export const useGameEffects = ({
  gameState,
  setGameState,
  setMessage,
  setMarketImages,
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
    const imageObjects = gameState.catalog.objects
      .map((object) => ({ object, path: (object.image_path || '').trim() || `./img/${object.id}.jpeg` }));
    Promise.all(imageObjects.map(async ({ object, path }) => {
      try {
        return [object.id, await loadDatasetAsset(path)] as const;
      } catch {
        return null;
      }
    })).then((entries) => {
      setMarketImages(Object.fromEntries(entries.filter((entry): entry is readonly [string, string] => entry !== null)));
    });
    const configuredSort = getLabel(gameState.catalog, 'market_default_sort', 'price');
    if (configuredSort === 'name' || configuredSort === 'price') setMarketSort(configuredSort);
  }, [gameState?.catalog, setMarketImages, setMarketSort]);

  useEffect(() => {
    if (!gameState) return;
    const configured = gameState.catalog.player_characteristics.find((entry) => entry.id === 'picture_file')?.name.trim();
    if (!configured) {
      setPlayerImage('/img/player.jpeg');
      return;
    }
    loadDatasetAsset(configured)
      .then(setPlayerImage)
      .catch(() => setPlayerImage('/img/player.jpeg'));
  }, [gameState?.catalog, gameState?.dataset_path, setPlayerImage]);
};
