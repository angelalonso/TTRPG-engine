import React, { useState } from 'react';
import { enterRace, getGameState } from '../services/tauriApi';
import type { GameState, RaceData, RaceResult, OwnedCar } from '../types/game';

interface RaceCalendarProps {
  gameState: GameState;
  onStateUpdate: (newState: GameState) => void;
}

export const RaceCalendar: React.FC<RaceCalendarProps> = ({
  gameState,
  onStateUpdate,
}) => {
  const { current_day, catalog, player } = gameState;
  const currentDayInYear = ((current_day - 1) % 365) + 1;

  const [selectedCarId, setSelectedCarId] = useState<string>(
    player.cars[0]?.id ?? ''
  );
  const [raceResult, setRaceResult] = useState<RaceResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const selectedCar = player.cars.find((c) => c.id === selectedCarId);

  const handleEnterRace = async (raceId: string) => {
    if (!selectedCarId) {
      setError('Please select a car from your garage first.');
      return;
    }

    try {
      setError(null);
      setIsSubmitting(true);

      const result = await enterRace(selectedCarId, raceId);
      setRaceResult(result);

      // Refresh global state after race execution
      const updatedState = await getGameState();
      onStateUpdate(updatedState);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  const formatPosition = (pos: RaceResult['position']): string => {
    if (typeof pos === 'string') {
      switch (pos) {
        case 'First':
          return '🥇 1st Place';
        case 'Second':
          return '🥈 2nd Place';
        case 'Third':
          return '🥉 3rd Place';
        default:
          return 'Unplaced';
      }
    }
    if (pos && typeof pos === 'object' && 'DNF' in pos) {
      return `❌ DNF (${pos.DNF})`;
    }
    return 'Unknown';
  };

  return (
    <div style={styles.card}>
      <h2>🏁 Race Calendar & Events</h2>
      <p style={{ color: '#94a3b8', fontSize: '0.9rem' }}>
        Today is <strong>Day {currentDayInYear}</strong> of the year. Races only take
        place on their exact scheduled day.
      </p>

      {error && <div style={styles.errorBox}>{error}</div>}

      {/* Race Outcome Modal / Notice */}
      {raceResult && (
        <div style={styles.resultBox}>
          <div style={styles.resultHeader}>
            <h3 style={{ margin: 0 }}>{raceResult.race_name} - Result</h3>
            <button
              onClick={() => setRaceResult(null)}
              style={styles.closeBtn}
            >
              ✕
            </button>
          </div>
          <div style={{ fontSize: '1.2rem', fontWeight: 'bold', margin: '0.5rem 0' }}>
            {formatPosition(raceResult.position)}
          </div>
          <p style={{ margin: '0.25rem 0' }}>{raceResult.message}</p>
          <div style={styles.resultFinancials}>
            <span>Entry Fee: -£{raceResult.entry_fee_paid}</span>
            <span style={{ color: raceResult.prize_awarded > 0 ? '#22c55e' : '#94a3b8' }}>
              Prize: +£{raceResult.prize_awarded}
            </span>
          </div>
        </div>
      )}

      {/* Vehicle Selector */}
      <div style={styles.carSelectSection}>
        <label style={{ fontWeight: 'bold', fontSize: '0.9rem' }}>
          Select Race Vehicle:
        </label>
        {player.cars.length === 0 ? (
          <span style={{ color: '#ef4444', fontSize: '0.875rem' }}>
            No cars in garage! Buy one from the Dealership.
          </span>
        ) : (
          <select
            value={selectedCarId}
            onChange={(e) => setSelectedCarId(e.target.value)}
            style={styles.selectInput}
          >
            {player.cars.map((car: OwnedCar) => (
              <option key={car.id} value={car.id}>
                {car.name} ({car.tire_sets_available} tires,{' '}
                {car.needs_oil_change || car.needs_engine_rebuild || car.needs_gearbox_maint
                  ? 'Needs Maint'
                  : 'Ready'}
                )
              </option>
            ))}
          </select>
        )}
      </div>

      {/* Race Schedule */}
      <div style={styles.raceList}>
        {catalog.races.map((race: RaceData) => {
          const daysUntil =
            ((race.day_of_year - currentDayInYear + 365) % 365);
          const isToday = daysUntil === 0;

          const isCarReady =
            selectedCar &&
            !selectedCar.needs_oil_change &&
            !selectedCar.needs_engine_rebuild &&
            !selectedCar.needs_gearbox_maint &&
            selectedCar.tire_sets_available >= 4;

          const hasFunds = player.budget >= race.entry_fee;
          const canEnter = isToday && isCarReady && hasFunds && !isSubmitting;

          return (
            <div
              key={race.id}
              style={{
                ...styles.raceCard,
                borderColor: isToday ? '#3b82f6' : '#334155',
                backgroundColor: isToday ? '#1e293b' : '#0f172a',
              }}
            >
              <div style={styles.raceInfo}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                  <h3 style={{ margin: 0 }}>{race.name}</h3>
                  {isToday && <span style={styles.todayBadge}>TODAY</span>}
                </div>
                <div style={styles.raceMeta}>
                  <span>Scheduled: Day {race.day_of_year}</span>
                  <span>Entry Fee: £{race.entry_fee}</span>
                  <span>Prize Pool: £{race.prize_pool}</span>
                </div>
              </div>

              <div style={styles.actionCol}>
                {!isToday && (
                  <span style={styles.countdownText}>
                    In {daysUntil} {daysUntil === 1 ? 'day' : 'days'}
                  </span>
                )}

                <button
                  onClick={() => handleEnterRace(race.id)}
                  disabled={!canEnter}
                  style={canEnter ? styles.enterBtn : styles.disabledBtn}
                >
                  {isSubmitting ? 'Simulating...' : 'Enter Race'}
                </button>

                {isToday && !canEnter && (
                  <span style={styles.reasonText}>
                    {!hasFunds
                      ? 'Low Budget'
                      : !isCarReady
                      ? 'Car Not Ready'
                      : 'Cannot Enter'}
                  </span>
                )}
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
  resultBox: {
    padding: '1rem',
    backgroundColor: '#0f172a',
    border: '2px solid #3b82f6',
    borderRadius: '6px',
    marginBottom: '1rem',
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
  carSelectSection: {
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
    marginBottom: '1rem',
    backgroundColor: '#0f172a',
    padding: '0.75rem',
    borderRadius: '6px',
  },
  selectInput: {
    padding: '0.5rem',
    borderRadius: '4px',
    backgroundColor: '#1e293b',
    color: '#ffffff',
    border: '1px solid #475569',
    flex: 1,
  },
  raceList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.75rem',
  },
  raceCard: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '1rem',
    borderRadius: '6px',
    border: '1px solid #334155',
  },
  raceInfo: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.25rem',
  },
  raceMeta: {
    display: 'flex',
    gap: '1rem',
    fontSize: '0.85rem',
    color: '#94a3b8',
  },
  todayBadge: {
    backgroundColor: '#2563eb',
    color: '#ffffff',
    fontSize: '0.7rem',
    fontWeight: 'bold',
    padding: '0.1rem 0.4rem',
    borderRadius: '3px',
  },
  actionCol: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-end',
    gap: '0.25rem',
  },
  countdownText: {
    fontSize: '0.8rem',
    color: '#64748b',
  },
  reasonText: {
    fontSize: '0.75rem',
    color: '#ef4444',
  },
  enterBtn: {
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    border: 'none',
    backgroundColor: '#16a34a',
    color: '#ffffff',
    fontWeight: 'bold',
    cursor: 'pointer',
  },
  disabledBtn: {
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    border: '1px solid #475569',
    backgroundColor: '#334155',
    color: '#64748b',
    cursor: 'not-allowed',
  },
};
