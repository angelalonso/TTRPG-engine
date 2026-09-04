import React, { useEffect, useState } from 'react';
import type { GameState, TimeSpeed } from './types/game';
import {
  getGameState,
  setTimeSpeed,
  tickGameDay,
  buyCar,
  maintainCar,
  performAction,
  enterRace,
} from './services/tauriApi';
import { AlertModal } from './components/AlertModal';

export function App() {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [activeTab, setActiveTab] = useState<'dashboard' | 'garage' | 'races' | 'activities'>('dashboard');
  const [selectedCarId, setSelectedCarId] = useState<string>('');
  const [feedbackMessage, setFeedbackMessage] = useState<string>('');

  // Initial Game State Load
  useEffect(() => {
    getGameState()
      .then(setGameState)
      .catch((err: unknown) => console.error('Failed to load initial state:', err));
  }, []);

  // Time Engine Loop
  useEffect(() => {
    if (!gameState || gameState.time_speed === 'Paused') return;

    let intervalMs = 1000;
    if (gameState.time_speed === 'OneDayEveryFiveSec') intervalMs = 5000;
    if (gameState.time_speed === 'OneDayPerSec') intervalMs = 1000;
    if (gameState.time_speed === 'OneWeekPerSec') intervalMs = 1000 / 7;

    const timer = setInterval(() => {
      tickGameDay()
        .then(setGameState)
	.catch((err: unknown) => console.error('Tick failed:', err));
    }, intervalMs);

    return () => clearInterval(timer);
  }, [gameState?.time_speed]);

  if (!gameState) {
    return (
      <div style={styles.loading}>
        <h2>🏎️ Loading Engine State...</h2>
      </div>
    );
  }

  const currentDayOfYear = ((gameState.current_day - 1) % 365) + 1;
  const currentYear = Math.floor((gameState.current_day - 1) / 365) + 1;

  const handleSpeedChange = async (speed: TimeSpeed) => {
    try {
      const updated = await setTimeSpeed(speed);
      setGameState(updated);
    } catch (err: any) {
      setFeedbackMessage(`Failed to change speed: ${err}`);
    }
  };

  const handleBuyCar = async (carId: string) => {
    try {
      const updated = await buyCar(carId);
      setGameState(updated);
      setFeedbackMessage('Vehicle purchased successfully!');
    } catch (err: any) {
      setFeedbackMessage(`Purchase failed: ${err}`);
    }
  };

  const handleMaintainCar = async (carId: string, maintType: any) => {
    try {
      const updated = await maintainCar(carId, maintType);
      setGameState(updated);
      setFeedbackMessage('Maintenance completed successfully!');
    } catch (err: any) {
      setFeedbackMessage(`Maintenance failed: ${err}`);
    }
  };

  const handlePerformAction = async (actionId: string) => {
    try {
      const res = await performAction(actionId);
      setFeedbackMessage(res.message);
      const freshState = await getGameState();
      setGameState(freshState);
    } catch (err: any) {
      setFeedbackMessage(`Action failed: ${err}`);
    }
  };

  const handleEnterRace = async (raceId: string) => {
    if (!selectedCarId) {
      setFeedbackMessage('Please select a car from your garage before entering a race.');
      return;
    }
    try {
      const res = await enterRace(selectedCarId, raceId);
      setFeedbackMessage(`${res.message} (Prize: £${res.prize_awarded})`);
      const freshState = await getGameState();
      setGameState(freshState);
    } catch (err: any) {
      setFeedbackMessage(`Race entry failed: ${err}`);
    }
  };

  return (
    <div style={styles.container}>
      {/* Header Bar */}
      <header style={styles.header}>
        <div>
          <h1 style={styles.appTitle}>GTR2 RACEWARS</h1>
          <p style={styles.subTitle}>
            Year {currentYear}, Day {currentDayOfYear} (Total Day {gameState.current_day})
          </p>
        </div>
        <div style={styles.budgetBadge}>
          💰 Budget: £{gameState.player.budget.toLocaleString('en-US', { minimumFractionDigits: 2 })}
        </div>
      </header>

      {/* Time Simulation Controls */}
      <div style={styles.controlsBar}>
        <span style={{ fontWeight: 'bold', color: '#94a3b8' }}>Simulation Speed:</span>
        {(['Paused', 'OneDayEveryFiveSec', 'OneDayPerSec', 'OneWeekPerSec'] as TimeSpeed[]).map((speed) => (
          <button
            key={speed}
            style={{
              ...styles.speedButton,
              backgroundColor: gameState.time_speed === speed ? '#2563eb' : '#334155',
            }}
            onClick={() => handleSpeedChange(speed)}
          >
            {speed === 'Paused' ? '⏸️ Pause' : speed}
          </button>
        ))}
      </div>

      {/* Action Feedback Banner */}
      {feedbackMessage && (
        <div style={styles.feedbackBanner} onClick={() => setFeedbackMessage('')}>
          <span>{feedbackMessage}</span>
          <span style={{ cursor: 'pointer', fontWeight: 'bold' }}>✕</span>
        </div>
      )}

      {/* Tab Navigation */}
      <nav style={styles.navTabs}>
        <button
          style={{ ...styles.tabButton, borderBottom: activeTab === 'dashboard' ? '3px solid #3b82f6' : 'none' }}
          onClick={() => setActiveTab('dashboard')}
        >
          📊 Overview
        </button>
        <button
          style={{ ...styles.tabButton, borderBottom: activeTab === 'garage' ? '3px solid #3b82f6' : 'none' }}
          onClick={() => setActiveTab('garage')}
        >
          🏎️ Garage & Dealership
        </button>
        <button
          style={{ ...styles.tabButton, borderBottom: activeTab === 'races' ? '3px solid #3b82f6' : 'none' }}
          onClick={() => setActiveTab('races')}
        >
          🏁 Race Events
        </button>
        <button
          style={{ ...styles.tabButton, borderBottom: activeTab === 'activities' ? '3px solid #3b82f6' : 'none' }}
          onClick={() => setActiveTab('activities')}
        >
          💼 Jobs & Activities
        </button>
      </nav>

      {/* Tab Main Content */}
      <main style={styles.mainContent}>
        {activeTab === 'dashboard' && (
          <div style={styles.gridTwoColumn}>
            <div style={styles.card}>
              <h3>Active Employment & Contracts</h3>
              {gameState.player.active_actions.length === 0 ? (
                <p style={{ color: '#94a3b8' }}>No active recurring jobs. Visit Jobs & Activities to start one.</p>
              ) : (
                gameState.player.active_actions.map((act: any) => {
                  const spec = gameState.catalog.actions.find((a) => a.id === act.action_id);
                  const daysActive = gameState.current_day - act.start_day;
                  return (
                    <div key={act.action_id} style={styles.itemRow}>
                      <div>
                        <strong>{spec?.name || act.action_id}</strong>
                        <div style={{ fontSize: '0.85rem', color: '#94a3b8' }}>
                          Started Day {act.start_day} ({daysActive} days active)
                        </div>
                      </div>
                      <div style={{ color: '#10b981', fontWeight: 'bold' }}>
                        +£{spec?.payout} / {spec?.payout_freq} {spec?.payout_freq_unit}
                      </div>
                    </div>
                  );
                })
              )}
            </div>

            <div style={styles.card}>
              <h3>Upcoming Race Schedule</h3>
              {gameState.catalog.races.map((race) => {
                let daysLeft = race.day_of_year - currentDayOfYear;
                if (daysLeft < 0) daysLeft += 365;

                return (
                  <div key={race.id} style={styles.itemRow}>
                    <div>
                      <strong>{race.name}</strong>
                      <div style={{ fontSize: '0.85rem', color: '#94a3b8' }}>
                        Day {race.day_of_year} of Year ({daysLeft === 0 ? 'TODAY!' : `${daysLeft} days away`})
                      </div>
                    </div>
                    <div style={{ textAlign: 'right' }}>
                      <div style={{ color: '#3b82f6' }}>Prize: £{race.prize_pool}</div>
                      <div style={{ fontSize: '0.8rem', color: '#94a3b8' }}>Fee: £{race.entry_fee}</div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {activeTab === 'garage' && (
          <div>
            <h3>Your Garage</h3>
            {gameState.player.cars.length === 0 ? (
              <p style={{ color: '#94a3b8' }}>Your garage is empty. Purchase a car below.</p>
            ) : (
              <div style={styles.gridTwoColumn}>
                {gameState.player.cars.map((car) => (
                  <div
                    key={car.id}
                    style={{
                      ...styles.card,
                      border: selectedCarId === car.id ? '2px solid #3b82f6' : '1px solid #334155',
                    }}
                    onClick={() => setSelectedCarId(car.id)}
                  >
                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
                      <h4>{car.name}</h4>
                      {selectedCarId === car.id && <span style={{ color: '#3b82f6' }}>Selected for Race</span>}
                    </div>
                    <p style={{ margin: '0.2rem 0', fontSize: '0.9rem' }}>🛞 Tires Available: {car.tire_sets_available} sets</p>
                    <p style={{ margin: '0.2rem 0', fontSize: '0.9rem', color: car.needs_oil_change ? '#ef4444' : '#10b981' }}>
                      🛢️ Oil Status: {car.needs_oil_change ? 'Needs Service' : 'Good'}
                    </p>
                    <p style={{ margin: '0.2rem 0', fontSize: '0.9rem', color: car.needs_gearbox_maint ? '#ef4444' : '#10b981' }}>
                      ⚙️ Gearbox Status: {car.needs_gearbox_maint ? 'Needs Service' : 'Good'}
                    </p>
                    <div style={{ display: 'flex', gap: '0.5rem', marginTop: '1rem', flexWrap: 'wrap' }}>
                      {car.needs_oil_change && (
                        <button style={styles.actionBtn} onClick={() => handleMaintainCar(car.id, 'OilChange')}>
                          Oil Change (£{car.oil_change_cost})
                        </button>
                      )}
                      {car.needs_gearbox_maint && (
                        <button style={styles.actionBtn} onClick={() => handleMaintainCar(car.id, 'GearboxService')}>
                          Gearbox Service (£{car.gearbox_maint_cost})
                        </button>
                      )}
                      <button style={styles.actionBtn} onClick={() => handleMaintainCar(car.id, { BuyTires: 4 })}>
                        Buy 4 Tires (£{car.tire_set_cost * 4})
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}

            <h3 style={{ marginTop: '2rem' }}>Car Dealership</h3>
            <div style={styles.gridTwoColumn}>
              {gameState.catalog.cars.map((car) => (
                <div key={car.id} style={styles.card}>
                  <h4>{car.name}</h4>
                  <p style={{ color: '#10b981', fontWeight: 'bold' }}>Price: £{car.price.toLocaleString()}</p>
                  <button style={styles.primaryBtn} onClick={() => handleBuyCar(car.id)}>
                    Buy Vehicle
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'races' && (
          <div>
            <h3>Scheduled Race Events</h3>
            <p style={{ color: '#94a3b8' }}>
              Selected Vehicle for Race: {selectedCarId ? gameState.player.cars.find((c) => c.id === selectedCarId)?.name : 'None selected (select one in Garage)'}
            </p>
            <div style={styles.gridTwoColumn}>
              {gameState.catalog.races.map((race) => {
                const isRaceDay = race.day_of_year === currentDayOfYear;

                return (
                  <div key={race.id} style={styles.card}>
                    <h4>{race.name}</h4>
                    <p>Day of Year: {race.day_of_year}</p>
                    <p>Entry Fee: £{race.entry_fee}</p>
                    <p style={{ color: '#10b981' }}>Prize Pool: £{race.prize_pool}</p>
                    <button
                      style={{
                        ...styles.primaryBtn,
                        backgroundColor: isRaceDay ? '#22c55e' : '#475569',
                        cursor: isRaceDay ? 'pointer' : 'not-allowed',
                      }}
                      disabled={!isRaceDay}
                      onClick={() => handleEnterRace(race.id)}
                    >
                      {isRaceDay ? '🏁 Enter Race Today!' : 'Race Closed'}
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {activeTab === 'activities' && (
          <div>
            <h3>Available Jobs & Activities</h3>
            <div style={styles.gridTwoColumn}>
              {gameState.catalog.actions.map((act) => (
                <div key={act.id} style={styles.card}>
                  <h4>{act.name}</h4>
                  <p style={{ color: '#cbd5e1', fontSize: '0.9rem' }}>{act.description}</p>
                  <p style={{ color: '#10b981' }}>
                    Payout: £{act.payout} ({act.payout_freq_type} every {act.payout_freq} {act.payout_freq_unit})
                  </p>
                  <p style={{ color: '#94a3b8', fontSize: '0.85rem' }}>Upfront Cost: £{act.base_cost}</p>
                  <button style={styles.primaryBtn} onClick={() => handlePerformAction(act.id)}>
                    Start Activity
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}
      </main>

      {/* Global Alert Pop-up Modal */}
      <AlertModal alerts={gameState.pending_alerts || []} onDismiss={setGameState} />
    </div>
  );
}

export default App;

const styles: Record<string, React.CSSProperties> = {
  container: {
    backgroundColor: '#0f172a',
    color: '#f8fafc',
    minHeight: '100vh',
    padding: '1.5rem',
    fontFamily: 'Segoe UI, Tahoma, Geneva, Verdana, sans-serif',
  },
  loading: {
    backgroundColor: '#0f172a',
    color: '#f8fafc',
    height: '100vh',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '1.25rem',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    borderBottom: '1px solid #334155',
    paddingBottom: '1rem',
  },
  appTitle: {
    margin: 0,
    fontSize: '1.75rem',
    letterSpacing: '1px',
    color: '#3b82f6',
  },
  subTitle: {
    margin: '0.25rem 0 0 0',
    color: '#94a3b8',
    fontSize: '0.9rem',
  },
  budgetBadge: {
    backgroundColor: '#1e293b',
    border: '1px solid #10b981',
    padding: '0.6rem 1.2rem',
    borderRadius: '8px',
    fontWeight: 'bold',
    fontSize: '1.1rem',
    color: '#10b981',
  },
  controlsBar: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    margin: '1rem 0',
    backgroundColor: '#1e293b',
    padding: '0.75rem',
    borderRadius: '8px',
  },
  speedButton: {
    color: '#ffffff',
    border: 'none',
    padding: '0.4rem 0.8rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
  feedbackBanner: {
    backgroundColor: '#3b82f6',
    color: '#ffffff',
    padding: '0.75rem 1rem',
    borderRadius: '6px',
    marginBottom: '1rem',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  navTabs: {
    display: 'flex',
    gap: '1rem',
    borderBottom: '1px solid #334155',
    marginBottom: '1.5rem',
  },
  tabButton: {
    backgroundColor: 'transparent',
    color: '#f8fafc',
    border: 'none',
    padding: '0.75rem 1rem',
    fontSize: '1rem',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
  mainContent: {
    marginTop: '1rem',
  },
  gridTwoColumn: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
    gap: '1rem',
  },
  card: {
    backgroundColor: '#1e293b',
    borderRadius: '8px',
    padding: '1.25rem',
    border: '1px solid #334155',
  },
  itemRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '0.75rem 0',
    borderBottom: '1px solid #334155',
  },
  primaryBtn: {
    backgroundColor: '#2563eb',
    color: '#ffffff',
    border: 'none',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
    marginTop: '0.5rem',
    width: '100%',
  },
  actionBtn: {
    backgroundColor: '#334155',
    color: '#ffffff',
    border: 'none',
    padding: '0.4rem 0.8rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '0.85rem',
  },
};
