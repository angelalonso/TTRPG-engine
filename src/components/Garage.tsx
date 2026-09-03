import React, { useState } from 'react';
import { maintainCar } from '../services/tauriApi';
import type { GameState, MaintenanceType, OwnedCar } from '../types/game';

interface GarageProps {
  gameState: GameState;
  onStateUpdate: (newState: GameState) => void;
}

export const Garage: React.FC<GarageProps> = ({ gameState, onStateUpdate }) => {
  const { player } = gameState;
  const [error, setError] = useState<string | null>(null);
  const [tireCounts, setTireCounts] = useState<Record<string, number>>({});

  const handleMaintenance = async (carId: string, maintType: MaintenanceType) => {
    try {
      setError(null);
      const updatedState = await maintainCar(carId, maintType);
      onStateUpdate(updatedState);
    } catch (err) {
      setError(String(err));
    }
  };

  const getTireCount = (carId: string) => tireCounts[carId] ?? 4; // Default to 4 sets (1 race worth)

  const handleTireCountChange = (carId: string, count: number) => {
    setTireCounts((prev) => ({ ...prev, [carId]: Math.max(1, count) }));
  };

  if (player.cars.length === 0) {
    return (
      <div style={styles.card}>
        <h2>🚗 Garage</h2>
        <p style={{ color: '#94a3b8' }}>
          You don't own any cars yet. Purchase one from the Dealership to start racing!
        </p>
      </div>
    );
  }

  return (
    <div style={styles.card}>
      <h2>🚗 Garage & Maintenance</h2>
      {error && <div style={styles.errorBox}>{error}</div>}

      <div style={styles.carGrid}>
        {player.cars.map((car: OwnedCar) => {
          const selectedTires = getTireCount(car.id);
          const totalTireCost = car.tire_set_cost * selectedTires;

          return (
            <div key={car.id} style={styles.carCard}>
              <div style={styles.carHeader}>
                <h3 style={{ margin: 0 }}>{car.name}</h3>
                <span
                  style={
                    car.tire_sets_available >= 4 &&
                    !car.needs_oil_change &&
                    !car.needs_engine_rebuild &&
                    !car.needs_gearbox_maint
                      ? styles.badgeReady
                      : styles.badgeNotReady
                  }
                >
                  {car.tire_sets_available >= 4 &&
                  !car.needs_oil_change &&
                  !car.needs_engine_rebuild &&
                  !car.needs_gearbox_maint
                    ? 'Race Ready'
                    : 'Service Required'}
                </span>
              </div>

              {/* Vehicle Condition Specs */}
              <div style={styles.statusGrid}>
                <div style={styles.statusItem}>
                  <span>Tire Sets Available:</span>
                  <strong>{car.tire_sets_available} / 4 minimum</strong>
                </div>
                <div style={styles.statusItem}>
                  <span>Engine Condition:</span>
                  <strong style={{ color: car.needs_engine_rebuild ? '#ef4444' : '#22c55e' }}>
                    {car.needs_engine_rebuild ? 'Needs Rebuild' : 'OK'}
                  </strong>
                </div>
                <div style={styles.statusItem}>
                  <span>Gearbox Service:</span>
                  <strong style={{ color: car.needs_gearbox_maint ? '#ef4444' : '#22c55e' }}>
                    {car.needs_gearbox_maint ? 'Service Required' : 'OK'}
                  </strong>
                </div>
                <div style={styles.statusItem}>
                  <span>Oil Status:</span>
                  <strong style={{ color: car.needs_oil_change ? '#ef4444' : '#22c55e' }}>
                    {car.needs_oil_change ? 'Oil Change Needed' : 'Fresh'}
                  </strong>
                </div>
              </div>

              {/* Maintenance Action Buttons */}
              <div style={styles.actionSection}>
                {/* Oil Change */}
                <button
                  style={styles.actionBtn}
                  disabled={!car.needs_oil_change || player.budget < car.oil_change_cost}
                  onClick={() => handleMaintenance(car.id, 'OilChange')}
                >
                  Oil Change (£{car.oil_change_cost})
                </button>

                {/* Engine Rebuild */}
                <button
                  style={styles.actionBtn}
                  disabled={!car.needs_engine_rebuild || player.budget < car.engine_rebuild_cost}
                  onClick={() => handleMaintenance(car.id, 'EngineRebuild')}
                >
                  Rebuild Engine (£{car.engine_rebuild_cost})
                </button>

                {/* Gearbox Service */}
                <button
                  style={styles.actionBtn}
                  disabled={!car.needs_gearbox_maint || player.budget < car.gearbox_maint_cost}
                  onClick={() => handleMaintenance(car.id, 'GearboxService')}
                >
                  Service Gearbox (£{car.gearbox_maint_cost})
                </button>

                {/* Buy Tire Sets */}
                <div style={styles.tirePurchaseRow}>
                  <div style={styles.tireInputGroup}>
                    <label style={{ fontSize: '0.8rem', color: '#94a3b8' }}>Buy Tires:</label>
                    <input
                      type="number"
                      min="1"
                      max="20"
                      value={selectedTires}
                      onChange={(e) => handleTireCountChange(car.id, parseInt(e.target.value) || 1)}
                      style={styles.numberInput}
                    />
                    <span style={{ fontSize: '0.85rem' }}>sets (£{totalTireCost})</span>
                  </div>

                  <button
                    style={styles.actionBtn}
                    disabled={player.budget < totalTireCost}
                    onClick={() =>
                      handleMaintenance(car.id, { BuyTires: selectedTires })
                    }
                  >
                    Purchase Tires
                  </button>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  card: {
    padding: '1rem',
    borderRadius: '8px',
    backgroundColor: '#1e293b',
    color: '#f8fafc',
    marginBottom: '1rem',
  },
  errorBox: {
    padding: '0.75rem',
    backgroundColor: '#7f1d1d',
    color: '#fca5a5',
    borderRadius: '4px',
    marginBottom: '1rem',
    fontSize: '0.9rem',
  },
  carGrid: {
    display: 'flex',
    flexDirection: 'column',
    gap: '1rem',
  },
  carCard: {
    padding: '1rem',
    borderRadius: '6px',
    backgroundColor: '#0f172a',
    border: '1px solid #334155',
  },
  carHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1rem',
  },
  badgeReady: {
    backgroundColor: '#15803d',
    color: '#ffffff',
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
    fontSize: '0.75rem',
    fontWeight: 'bold',
  },
  badgeNotReady: {
    backgroundColor: '#b91c1c',
    color: '#ffffff',
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
    fontSize: '0.75rem',
    fontWeight: 'bold',
  },
  statusGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '0.5rem',
    marginBottom: '1rem',
    fontSize: '0.875rem',
    backgroundColor: '#1e293b',
    padding: '0.75rem',
    borderRadius: '4px',
  },
  statusItem: {
    display: 'flex',
    justifyContent: 'space-between',
  },
  actionSection: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
  },
  tirePurchaseRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    flexWrap: 'wrap',
    gap: '0.5rem',
    marginTop: '0.5rem',
    paddingTop: '0.5rem',
    borderTop: '1px dashed #334155',
  },
  tireInputGroup: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
  },
  numberInput: {
    width: '50px',
    padding: '0.25rem',
    backgroundColor: '#1e293b',
    color: '#ffffff',
    border: '1px solid #475569',
    borderRadius: '4px',
    textAlign: 'center',
  },
  actionBtn: {
    padding: '0.5rem 0.75rem',
    borderRadius: '4px',
    border: '1px solid #3b82f6',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
};
