import React, { useEffect, useState } from 'react';
import { fetchCatalog, executeAction, buyCar } from '../services/tauriApi';
import type { GameCatalog, GameState } from '../types/game';

export const CatalogView: React.FC<{
  gameState: GameState;
  onStateUpdate: (newState: GameState) => void;
}> = ({ gameState, onStateUpdate }) => {
  const [catalog, setCatalog] = useState<GameCatalog | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchCatalog()
      .then(setCatalog)
      .catch((err) => setError(String(err)));
  }, []);

  const handleAction = async (actionId: string) => {
    try {
      setError(null);
      const updatedState = await executeAction(actionId);
      onStateUpdate(updatedState);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleBuyCar = async (carId: string) => {
    try {
      setError(null);
      const updatedState = await buyCar(carId);
      onStateUpdate(updatedState);
    } catch (err) {
      setError(String(err));
    }
  };

  if (!catalog) return <div>Loading Catalog...</div>;

  return (
    <div style={{ padding: '1rem' }}>
      <h2>Available Actions</h2>
      {error && <div style={{ color: 'red', marginBottom: '1rem' }}>{error}</div>}

      <ul>
        {catalog.actions.map((act) => (
          <li key={act.id} style={{ marginBottom: '0.5rem' }}>
            <strong>{act.name}</strong> - Cost: £{act.base_cost} | Success Rate:{' '}
            {(act.success_rate * 100).toFixed(0)}%
            <button
              onClick={() => handleAction(act.id)}
              disabled={gameState.player.budget < act.base_cost}
              style={{ marginLeft: '1rem' }}
            >
              Execute
            </button>
          </li>
        ))}
      </ul>

      <h2>Dealership</h2>
      <ul>
        {catalog.cars.map((car) => (
          <li key={car.id} style={{ marginBottom: '0.5rem' }}>
            <strong>{car.name}</strong> - Price: £{car.price}
            <button
              onClick={() => handleBuyCar(car.id)}
              disabled={gameState.player.budget < car.price}
              style={{ marginLeft: '1rem' }}
            >
              Buy Car
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
};
