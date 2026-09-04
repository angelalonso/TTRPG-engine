import React, { useState } from 'react';
import { performAction, buyCar, getGameState } from '../services/tauriApi';
import type {
  GameState,
  ActionData,
  CarData,
  ActionResult,
} from '../types/game';

interface ActionsViewProps {
  gameState: GameState;
  onStateUpdate: (newState: GameState) => void;
}

export const ActionsView: React.FC<ActionsViewProps> = ({
  gameState,
  onStateUpdate,
}) => {
  const { player, catalog } = gameState;
  const [actionResult, setActionResult] = useState<ActionResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loadingActionId, setLoadingActionId] = useState<string | null>(null);
  const [loadingCarId, setLoadingCarId] = useState<string | null>(null);

  const formatFreq = (action: ActionData) => {
    if (action.payout_freq_type === 'once') return 'One-time';
    const s = action.payout_freq > 1 ? 's' : '';
    return `Every ${action.payout_freq} ${action.payout_freq_unit}${s}`;
  };

  const handlePerformAction = async (actionId: string) => {
    try {
      setError(null);
      setLoadingActionId(actionId);
      const result = await performAction(actionId);
      setActionResult(result);

      const updatedState = await getGameState();
      onStateUpdate(updatedState);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoadingActionId(null);
    }
  };

  const handleBuyCar = async (carId: string) => {
    try {
      setError(null);
      setLoadingCarId(carId);
      const updatedState = await buyCar(carId);
      onStateUpdate(updatedState);
      setActionResult(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoadingCarId(null);
    }
  };

  return (
    <div style={styles.container}>
      {error && <div style={styles.errorBox}>{error}</div>}

      {/* Action Outcome Feedback */}
      {actionResult && (
        <div
          style={{
            ...styles.resultBox,
            borderColor: actionResult.success ? '#22c55e' : '#ef4444',
          }}
        >
          <div style={styles.resultHeader}>
            <h3 style={{ margin: 0 }}>
              {actionResult.success ? '✅ Success:' : '❌ Failed:'}{' '}
              {actionResult.action_name}
            </h3>
            <button
              onClick={() => setActionResult(null)}
              style={styles.closeBtn}
            >
              ✕
            </button>
          </div>
          <p style={{ margin: '0.5rem 0' }}>{actionResult.message}</p>
          <div style={styles.resultFinancials}>
            {actionResult.cost_paid > 0 && (
              <span style={{ color: '#ef4444' }}>
                Cost: -£{actionResult.cost_paid.toLocaleString()}
              </span>
            )}
            {actionResult.payout_received > 0 && (
              <span style={{ color: '#22c55e' }}>
                Payout: +£{actionResult.payout_received.toLocaleString()}
              </span>
            )}
          </div>
        </div>
      )}

      {/* Daily Actions & Jobs */}
      <section style={styles.section}>
        <h2 style={{ margin: 0 }}>⚡ Daily Jobs & Side Activities</h2>
        <p style={{ color: '#94a3b8', fontSize: '0.85rem' }}>
          Perform work or side jobs to earn regular income.
        </p>

        <div style={styles.grid}>
          {catalog.actions.map((action: ActionData) => {
            const canAfford = player.budget >= action.base_cost;
            const isLoading = loadingActionId === action.id;
            const isActive = player.active_actions?.some(
              (a) => a.action_id === action.id
            );

            return (
              <div key={action.id} style={styles.card}>
                <div style={styles.cardHeader}>
                  <h3 style={{ margin: 0 }}>{action.name}</h3>
                  <span style={styles.typeBadge}>{action.type}</span>
                </div>

                <div style={styles.statsRow}>
                  <div>
                    <span style={styles.label}>Base Cost:</span>
                    <div>£{action.base_cost.toLocaleString()}</div>
                  </div>
                  <div>
                    <span style={styles.label}>Payout ({formatFreq(action)}):</span>
                    <div style={{ color: '#22c55e', fontWeight: 'bold' }}>
                      +£{action.payout.toLocaleString()}
                    </div>
                  </div>
                  <div>
                    <span style={styles.label}>Success Rate:</span>
                    <div>{(action.success_rate * 100).toFixed(0)}%</div>
                  </div>
                  <div>
                    <span style={styles.label}>Risk Factor:</span>
                    <div>{((action.risk_factor ?? 0) * 100).toFixed(0)}%</div>
                  </div>
                </div>

                <button
                  onClick={() => handlePerformAction(action.id)}
                  disabled={!canAfford || isLoading || isActive}
                  style={
                    isActive
                      ? styles.activeBtn
                      : canAfford && !isLoading
                      ? styles.primaryBtn
                      : styles.disabledBtn
                  }
                >
                  {isLoading
                    ? 'Executing...'
                    : isActive
                    ? 'Active (Started)'
                    : `Start ${action.name}`}
                </button>
              </div>
            );
          })}
        </div>
      </section>

      {/* Dealership Catalog */}
      <section style={styles.section}>
        <h2 style={{ margin: 0 }}>🏎️ Dealership Catalog</h2>
        <p style={{ color: '#94a3b8', fontSize: '0.85rem' }}>
          Purchase cars to build your garage racing fleet.
        </p>

        <div style={styles.grid}>
          {catalog.cars.map((car: CarData) => {
            const canAfford = player.budget >= car.price;
            const ownedCount = player.cars.filter(
              (c) => c.name === car.name
            ).length;
            const isLoading = loadingCarId === car.id;

            return (
              <div key={car.id} style={styles.card}>
                <div style={styles.cardHeader}>
                  <h3 style={{ margin: 0 }}>{car.name}</h3>
                  {ownedCount > 0 && (
                    <span style={styles.ownedBadge}>Owned: {ownedCount}</span>
                  )}
                </div>

                <div style={styles.priceTag}>
                  £{car.price.toLocaleString()}
                </div>

                <div style={styles.maintPreview}>
                  <span style={styles.label}>Maintenance Costs:</span>
                  <div style={styles.maintGrid}>
                    <span>Oil Change: £{car.oil_change_cost}</span>
                    <span>Tire Set: £{car.tire_set_cost}</span>
                    <span>Engine Rebuild: £{car.engine_rebuild_cost}</span>
                    <span>Gearbox Service: £{car.gearbox_maint_cost}</span>
                  </div>
                </div>

                <button
                  onClick={() => handleBuyCar(car.id)}
                  disabled={!canAfford || isLoading}
                  style={
                    canAfford && !isLoading
                      ? styles.buyBtn
                      : styles.disabledBtn
                  }
                >
                  {isLoading
                    ? 'Purchasing...'
                    : `Buy Vehicle (£${car.price.toLocaleString()})`}
                </button>
              </div>
            );
          })}
        </div>
      </section>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '1.5rem',
  },
  errorBox: {
    padding: '0.75rem',
    backgroundColor: '#7f1d1d',
    color: '#fca5a5',
    borderRadius: '4px',
    fontSize: '0.9rem',
  },
  resultBox: {
    padding: '1rem',
    backgroundColor: '#0f172a',
    border: '2px solid',
    borderRadius: '6px',
  },
  resultHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  closeBtn: {
    background: 'none',
    border: 'none',
    color: '#94a3b8',
    fontSize: '1.2rem',
    cursor: 'pointer',
  },
  resultFinancials: {
    display: 'flex',
    gap: '1rem',
    fontSize: '0.9rem',
    marginTop: '0.5rem',
    fontWeight: 'bold',
  },
  section: {
    backgroundColor: '#1e293b',
    padding: '1rem',
    borderRadius: '8px',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
    gap: '1rem',
    marginTop: '1rem',
  },
  card: {
    backgroundColor: '#0f172a',
    border: '1px solid #334155',
    borderRadius: '6px',
    padding: '1rem',
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'space-between',
    gap: '0.75rem',
  },
  cardHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  typeBadge: {
    fontSize: '0.75rem',
    backgroundColor: '#334155',
    color: '#38bdf8',
    padding: '0.2rem 0.5rem',
    borderRadius: '4px',
    fontWeight: 'bold',
  },
  ownedBadge: {
    fontSize: '0.75rem',
    backgroundColor: '#15803d',
    color: '#ffffff',
    padding: '0.2rem 0.5rem',
    borderRadius: '4px',
    fontWeight: 'bold',
  },
  priceTag: {
    fontSize: '1.4rem',
    fontWeight: 'bold',
    color: '#38bdf8',
  },
  statsRow: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '0.5rem',
    fontSize: '0.85rem',
    backgroundColor: '#1e293b',
    padding: '0.5rem',
    borderRadius: '4px',
  },
  label: {
    fontSize: '0.75rem',
    color: '#94a3b8',
    display: 'block',
  },
  maintPreview: {
    fontSize: '0.8rem',
    backgroundColor: '#1e293b',
    padding: '0.5rem',
    borderRadius: '4px',
  },
  maintGrid: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '0.25rem',
    color: '#cbd5e1',
    marginTop: '0.25rem',
  },
  primaryBtn: {
    padding: '0.5rem 0.75rem',
    borderRadius: '4px',
    border: 'none',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    fontWeight: 'bold',
    cursor: 'pointer',
  },
  activeBtn: {
    padding: '0.5rem 0.75rem',
    borderRadius: '4px',
    border: '1px solid #16a34a',
    backgroundColor: '#064e3b',
    color: '#86efac',
    fontWeight: 'bold',
    cursor: 'default',
  },
  buyBtn: {
    padding: '0.5rem 0.75rem',
    borderRadius: '4px',
    border: 'none',
    backgroundColor: '#16a34a',
    color: '#ffffff',
    fontWeight: 'bold',
    cursor: 'pointer',
  },
  disabledBtn: {
    padding: '0.5rem 0.75rem',
    borderRadius: '4px',
    border: '1px solid #475569',
    backgroundColor: '#334155',
    color: '#64748b',
    cursor: 'not-allowed',
    fontWeight: 'bold',
  },
};
