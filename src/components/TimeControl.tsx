import React, { useEffect } from 'react';
import { setTimeSpeed, tickGameDay } from '../services/tauriApi';
import { getCharacteristic } from '../types/game';
import type { GameState, TimeSpeed } from '../types/game';

interface TimeControlProps {
  gameState: GameState;
  onStateUpdate: (newState: GameState) => void;
}

const SPEED_INTERVALS: Record<TimeSpeed, number | null> = {
  Paused: null,
  OneDayEveryFiveSec: 5000,
  OneDayPerSec: 1000,
  OneWeekPerSec: 142, // ~7 ticks per second (1 week / sec)
  RealTime: 1000,
};

export const TimeControl: React.FC<TimeControlProps> = ({
  gameState,
  onStateUpdate,
}) => {
  const { current_day, time_speed, player } = gameState;
  const budget = getCharacteristic(player, 'budget');

  const daysPerYear = gameState.days_per_year;
  const year = Math.floor((current_day - 1) / daysPerYear) + 1;
  const dayOfYear = ((current_day - 1) % daysPerYear) + 1;
  const ageYears = Math.floor(player.age_days / daysPerYear);
  const ageDaysRemaining = player.age_days % daysPerYear;
  const labels = gameState.catalog.labels.values;

  useEffect(() => {
    const intervalMs = SPEED_INTERVALS[time_speed];
    if (intervalMs === null) return;

    const timer = setInterval(async () => {
      try {
        const updatedState = await tickGameDay();
        onStateUpdate(updatedState);
      } catch (err) {
        console.error('Tick execution failed:', err);
      }
    }, intervalMs);

    return () => clearInterval(timer);
  }, [time_speed, onStateUpdate]);

  const handleSpeedChange = async (speed: TimeSpeed) => {
    try {
      await setTimeSpeed(speed);
      onStateUpdate({ ...gameState, time_speed: speed });
    } catch (err) {
      console.error('Failed to change speed:', err);
    }
  };

  return (
    <div style={styles.card}>
      {/* Calendar & player status */}
      <div style={styles.statsGrid}>
        <div style={styles.statBox}>
          <span style={styles.label}>{labels.year_name || 'Year'}</span>
          <div style={styles.value}>{year}</div>
        </div>
        <div style={styles.statBox}>
          <span style={styles.label}>{labels.day_name || 'Day'}</span>
          <div style={styles.value}>
            {dayOfYear} <span style={styles.subtext}>/ {daysPerYear}</span>
          </div>
        </div>
        <div style={styles.statBox}>
          <span style={styles.label}>{labels.age_name || 'Age'}</span>
          <div style={styles.value}>
            {ageYears}y {ageDaysRemaining}d
          </div>
        </div>
        <div style={styles.statBox}>
          <span style={styles.label}>{labels.budget_name || 'Budget'}</span>
          <div style={styles.value}>
            {labels.currency_symbol || '$'}{budget.toLocaleString()}
          </div>
        </div>
      </div>

      {/* Speed Controls */}
      <div style={styles.controlsRow}>
        <button
          style={time_speed === 'Paused' ? styles.activeBtn : styles.btn}
          onClick={() => handleSpeedChange('Paused')}
        >
          ⏸ Pause
        </button>
        <button
          style={
            time_speed === 'OneDayEveryFiveSec' ? styles.activeBtn : styles.btn
          }
          onClick={() => handleSpeedChange('OneDayEveryFiveSec')}
        >
          ▶ 1d / 5s
        </button>
        <button
          style={time_speed === 'OneDayPerSec' ? styles.activeBtn : styles.btn}
          onClick={() => handleSpeedChange('OneDayPerSec')}
        >
          ▶▶ 1d / 1s
        </button>
        <button
          style={
            time_speed === 'OneWeekPerSec' ? styles.activeBtn : styles.btn
          }
          onClick={() => handleSpeedChange('OneWeekPerSec')}
        >
          ⏩ 1w / 1s
        </button>

        {time_speed === 'RealTime' && (
          <span style={styles.eventBadge}>
            ⚠️ {labels.real_time_mode_name || 'Real-time mode'}
          </span>
        )}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  card: {
    padding: '1rem',
    borderRadius: '8px',
    backgroundColor: 'var(--surface-background)',
    color: 'var(--primary-text)',
    marginBottom: '1rem',
  },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
    gap: '1rem',
    marginBottom: '1rem',
  },
  statBox: {
    display: 'flex',
    flexDirection: 'column',
  },
  label: {
    fontSize: '0.75rem',
    color: 'var(--muted-text)',
    fontWeight: 'bold',
  },
  value: {
    fontSize: '1.25rem',
    fontWeight: 'bold',
  },
  subtext: {
    fontSize: '0.85rem',
    color: 'var(--control-border)',
  },
  controlsRow: {
    display: 'flex',
    gap: '0.5rem',
    alignItems: 'center',
    flexWrap: 'wrap',
  },
  btn: {
    padding: '0.5rem 0.75rem',
    borderRadius: '4px',
    border: '1px solid var(--control-border)',
    backgroundColor: 'var(--control-background)',
    color: 'var(--white-text)',
    cursor: 'pointer',
  },
  activeBtn: {
    padding: '0.5rem 0.75rem',
    borderRadius: '4px',
    border: '1px solid var(--info-border)',
    backgroundColor: 'var(--primary-accent)',
    color: 'var(--white-text)',
    fontWeight: 'bold',
    cursor: 'pointer',
  },
  eventBadge: {
    padding: '0.5rem 0.75rem',
    borderRadius: '4px',
    backgroundColor: 'var(--danger-action)',
    color: 'var(--white-text)',
    fontSize: '0.85rem',
    fontWeight: 'bold',
  },
};
