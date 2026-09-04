import React from 'react';
import type { GameState, ActiveAction, ActionData } from '../types/game';

interface CharacterOverviewProps {
  gameState: GameState;
  characterName: string;
}

export const CharacterOverview: React.FC<CharacterOverviewProps> = ({
  gameState,
  characterName,
}) => {
  const { player, catalog, current_day } = gameState;

  const ageYears = Math.floor(player.age_days / 365);
  const remainingDays = player.age_days % 365;

  const getIntervalDays = (freq: number, unit: string): number => {
    const u = unit.trim().toLowerCase();
    if (u.startsWith('day')) return freq * 1;
    if (u.startsWith('month')) return freq * 30;
    if (u.startsWith('year')) return freq * 365;
    return freq;
  };

  const getNextPayoutInfo = (active: ActiveAction, action: ActionData) => {
    const interval = getIntervalDays(action.payout_freq, action.payout_freq_unit);
    if (interval <= 0) return { nextDay: current_day, daysRemaining: 0 };

    const daysElapsed = current_day - active.start_day;
    const remainder = daysElapsed % interval;
    const daysRemaining = remainder === 0 && daysElapsed > 0 ? 0 : interval - remainder;
    const nextDay = current_day + daysRemaining;

    return { nextDay, daysRemaining };
  };

  return (
    <div style={styles.container}>
      {/* Player Bio Header */}
      <div style={styles.card}>
        <div style={styles.profileHeader}>
          <div>
            <h2 style={styles.characterTitle}>🏎️ {characterName}</h2>
            <span style={styles.subtitle}>Driver & Team Manager Profile</span>
          </div>
        </div>

        <div style={styles.statsGrid}>
          <div>
            <span style={styles.label}>Age:</span>
            <div style={styles.statValue}>
              {ageYears} yrs, {remainingDays} days
            </div>
          </div>
          <div>
            <span style={styles.label}>Current Budget:</span>
            <div style={{ ...styles.statValue, color: '#22c55e' }}>
              £{player.budget.toLocaleString()}
            </div>
          </div>
          <div>
            <span style={styles.label}>Current Day:</span>
            <div style={styles.statValue}>Day {current_day}</div>
          </div>
          <div>
            <span style={styles.label}>Garage Cars:</span>
            <div style={styles.statValue}>{player.cars.length} Vehicles</div>
          </div>
        </div>
      </div>

      {/* Ongoing Actions & Situations */}
      <div style={styles.card}>
        <div style={styles.sectionHeader}>
          <h3 style={{ margin: 0 }}>💼 Ongoing Jobs & Active Situations</h3>
          <span style={styles.countBadge}>
            {player.active_actions?.length || 0} Active
          </span>
        </div>

        {(!player.active_actions || player.active_actions.length === 0) ? (
          <div style={styles.emptyState}>
            No ongoing jobs or active situations. Select a recurring job in Daily Actions & Jobs to begin earning salary!
          </div>
        ) : (
          <div style={styles.actionsList}>
            {player.active_actions.map((active) => {
              const actionData = catalog.actions.find((a) => a.id === active.action_id);
              if (!actionData) return null;

              const { nextDay, daysRemaining } = getNextPayoutInfo(active, actionData);

              return (
                <div key={active.action_id} style={styles.actionItem}>
                  <div style={styles.actionMain}>
                    <div>
                      <div style={styles.actionName}>
                        {actionData.name}{' '}
                        <span style={styles.typeBadge}>{actionData.type}</span>
                      </div>
                      <div style={styles.actionMeta}>
                        Started on Day {active.start_day} • Payout: £
                        {actionData.payout.toLocaleString()} every {actionData.payout_freq}{' '}
                        {actionData.payout_freq_unit}(s)
                      </div>
                    </div>

                    <div style={styles.payoutBadgeBox}>
                      <span style={styles.label}>Next Payout:</span>
                      <div style={styles.payoutDayText}>
                        {daysRemaining === 0 ? (
                          <span style={{ color: '#22c55e', fontWeight: 'bold' }}>
                            Payout Due Today!
                          </span>
                        ) : (
                          <>
                            Day {nextDay}{' '}
                            <span style={styles.countdownText}>
                              ({daysRemaining} {daysRemaining === 1 ? 'day' : 'days'} left)
                            </span>
                          </>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '1.25rem',
  },
  card: {
    backgroundColor: '#1e293b',
    border: '1px solid #334155',
    borderRadius: '8px',
    padding: '1.25rem',
  },
  profileHeader: {
    marginBottom: '1.25rem',
    borderBottom: '1px solid #334155',
    paddingBottom: '0.75rem',
  },
  characterTitle: {
    margin: 0,
    color: '#f8fafc',
    fontSize: '1.5rem',
  },
  subtitle: {
    fontSize: '0.85rem',
    color: '#94a3b8',
  },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
    gap: '1rem',
  },
  label: {
    fontSize: '0.75rem',
    color: '#94a3b8',
    display: 'block',
    marginBottom: '0.2rem',
  },
  statValue: {
    fontSize: '1.1rem',
    fontWeight: 'bold',
    color: '#f1f5f9',
  },
  sectionHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1rem',
    color: '#f8fafc',
  },
  countBadge: {
    backgroundColor: '#3b82f6',
    color: '#ffffff',
    padding: '0.2rem 0.6rem',
    borderRadius: '12px',
    fontSize: '0.8rem',
    fontWeight: 'bold',
  },
  emptyState: {
    padding: '1.5rem',
    backgroundColor: '#0f172a',
    borderRadius: '6px',
    color: '#64748b',
    textAlign: 'center',
    fontSize: '0.9rem',
    border: '1px dashed #334155',
  },
  actionsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.75rem',
  },
  actionItem: {
    backgroundColor: '#0f172a',
    border: '1px solid #334155',
    borderRadius: '6px',
    padding: '1rem',
  },
  actionMain: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    flexWrap: 'wrap',
    gap: '0.75rem',
  },
  actionName: {
    fontSize: '1rem',
    fontWeight: 'bold',
    color: '#38bdf8',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
  },
  typeBadge: {
    fontSize: '0.7rem',
    backgroundColor: '#334155',
    color: '#cbd5e1',
    padding: '0.15rem 0.4rem',
    borderRadius: '4px',
    fontWeight: 'normal',
    textTransform: 'uppercase',
  },
  actionMeta: {
    fontSize: '0.825rem',
    color: '#94a3b8',
    marginTop: '0.25rem',
  },
  payoutBadgeBox: {
    backgroundColor: '#1e293b',
    padding: '0.5rem 0.75rem',
    borderRadius: '6px',
    border: '1px solid #334155',
    textAlign: 'right',
  },
  payoutDayText: {
    fontSize: '0.9rem',
    color: '#f8fafc',
  },
  countdownText: {
    color: '#f59e0b',
    fontSize: '0.8rem',
  },
};
