import React, { useEffect, useState } from 'react';
import { fetchCatalog, performAction, buyCar } from '../services/tauriApi';
import type { GameCatalog, GameState, ActionData } from '../types/game';

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

  const formatFreq = (act: ActionData) => {
    if (act.payout_freq_type === 'once') return 'one-time';
    const s = act.payout_freq > 1 ? 's' : '';
    return `every ${act.payout_freq} ${act.payout_freq_unit}${s}`;
  };

  const handleAction = async (actionId: string) => {
    try {
      setError(null);
      await performAction(actionId);
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

  if (!catalog) return <div style={{ padding: '1rem', color: '#94a3b8' }}>Loading Catalog...</div>;

  return (
    <div style={{ padding: '1rem', backgroundColor: '#1e293b', borderRadius: '8px', color: '#f8fafc' }}>
      <h2 style={{ marginTop: 0, borderBottom: '1px solid #334155', paddingBottom: '0.5rem' }}>
        ⚡ Available Actions
      </h2>
      {error && <div style={{ color: '#ef4444', marginBottom: '1rem' }}>{error}</div>}

      <ul style={{ listStyle: 'none', padding: 0 }}>
        {catalog.actions.map((act) => {
          const isActive = gameState.player.active_actions?.some((a) => a.action_id === act.id);

          return (
            <li
              key={act.id}
              style={{
                marginBottom: '0.75rem',
                padding: '0.75rem',
                backgroundColor: '#0f172a',
                borderRadius: '6px',
                border: '1px solid #334155',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              <div>
                <strong>{act.name}</strong> — Cost: £{act.base_cost} | Success Rate:{' '}
                {(act.success_rate * 100).toFixed(0)}% | Payout: £{act.payout.toLocaleString()} ({formatFreq(act)})
              </div>
              <button
                onClick={() => handleAction(act.id)}
                disabled={gameState.player.budget < act.base_cost || isActive}
                style={{
                  padding: '0.4rem 0.8rem',
                  backgroundColor: isActive
                    ? '#064e3b'
                    : gameState.player.budget >= act.base_cost
                    ? '#2563eb'
                    : '#475569',
                  color: isActive ? '#86efac' : '#ffffff',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: gameState.player.budget >= act.base_cost && !isActive ? 'pointer' : 'not-allowed',
                  fontWeight: 'bold',
                }}
              >
                {isActive ? 'Active' : 'Execute'}
              </button>
            </li>
          );
        })}
      </ul>

      <h2 style={{ marginTop: '2rem', borderBottom: '1px solid #334155', paddingBottom: '0.5rem' }}>
        🚘 Dealership
      </h2>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1rem' }}>
        {catalog.cars.map((car) => (
          <div
            key={car.id}
            style={{
              padding: '1rem',
              backgroundColor: '#0f172a',
              borderRadius: '6px',
              border: '1px solid #334155',
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-between',
            }}
          >
            <div>
              <h3 style={{ margin: '0 0 0.5rem 0', color: '#f59e0b' }}>{car.name}</h3>
              <div style={{ fontSize: '1.25rem', fontWeight: 'bold', color: '#22c55e', marginBottom: '1rem' }}>
                £{car.price.toLocaleString()}
              </div>
              <div style={{ fontSize: '0.85rem', color: '#94a3b8', lineHeight: '1.6', marginBottom: '1rem' }}>
                <div>Engine Rebuild: £{car.engine_rebuild_cost.toLocaleString()}</div>
                <div>Gearbox Maintenance: £{car.gearbox_maint_cost.toLocaleString()}</div>
                <div>Oil Change: £{car.oil_change_cost.toLocaleString()}</div>
                <div>Tire Set: £{car.tire_set_cost.toLocaleString()}</div>
              </div>
            </div>
            <button
              onClick={() => handleBuyCar(car.id)}
              disabled={gameState.player.budget < car.price}
              style={{
                width: '100%',
                padding: '0.5rem',
                backgroundColor: gameState.player.budget >= car.price ? '#16a34a' : '#475569',
                color: '#ffffff',
                border: 'none',
                borderRadius: '4px',
                cursor: gameState.player.budget >= car.price ? 'pointer' : 'not-allowed',
                fontWeight: 'bold',
              }}
            >
              {gameState.player.budget >= car.price ? 'Buy Car' : 'Insufficient Funds'}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
