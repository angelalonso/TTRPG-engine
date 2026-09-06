import React, { useState, useEffect } from 'react';
import type { GameState, TimeSpeed } from './types/game';
import {
  getGameState,
  setTimeSpeed,
  tickGameDay,
  buyCar,
  maintainCar,
  performAction,
  enterRace,
  reloadDataset,
} from './services/tauriApi';
import { AlertModal } from './components/AlertModal';
import { ConfigModal } from './components/ConfigModal';

export const App: React.FC = () => {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [activeTab, setActiveTab] = useState<'dashboard' | 'garage' | 'races' | 'activities'>('dashboard');
  const [isConfigOpen, setIsConfigOpen] = useState<boolean>(false);
  const [actionMessage, setActionMessage] = useState<string>('');

  // Initial load
  useEffect(() => {
    getGameState()
      .then(setGameState)
      .catch((err) => console.error('Failed to load initial state:', err));
  }, []);

  // Time ticker loop
  useEffect(() => {
    if (
      !gameState ||
      gameState.time_speed === 'Paused' ||
      (gameState.pending_alerts && gameState.pending_alerts.length > 0)
    ) {
      return;
    }

    let intervalMs = 1000;
    if (gameState.time_speed === 'OneDayEveryFiveSec') intervalMs = 5000;
    if (gameState.time_speed === 'OneDayPerSec') intervalMs = 1000;
    if (gameState.time_speed === 'OneWeekPerSec') intervalMs = 1000 / 7;
    if (gameState.time_speed === 'RealTime') intervalMs = 1000;

    const timer = setInterval(async () => {
      try {
        const updatedState = await tickGameDay();
        setGameState(updatedState);
      } catch (err) {
        console.error('Tick failed:', err);
      }
    }, intervalMs);

    return () => clearInterval(timer);
  }, [gameState?.time_speed, gameState?.pending_alerts?.length]);

  const handleSpeedChange = async (speed: TimeSpeed) => {
    try {
      const updated = await setTimeSpeed(speed);
      setGameState(updated);
    } catch (err) {
      console.error('Failed to update speed:', err);
    }
  };

  const handleBuyCar = async (carId: string) => {
    try {
      const updated = await buyCar(carId);
      setGameState(updated);
      setActionMessage('Car purchased successfully!');
    } catch (err: any) {
      setActionMessage(`Error purchasing car: ${err}`);
    }
  };

  const handleMaintainCar = async (carId: string, maintType: string) => {
    try {
      const updated = await maintainCar(carId, maintType);
      setGameState(updated);
      setActionMessage(`Maintenance (${maintType}) completed.`);
    } catch (err: any) {
      setActionMessage(`Maintenance failed: ${err}`);
    }
  };

  const handlePerformAction = async (actionId: string) => {
    try {
      const result = await performAction(actionId);
      setActionMessage(`${result.message} (Payout: $${result.payout_received}, Cost: $${result.cost_paid})`);
      const updated = await getGameState();
      setGameState(updated);
    } catch (err: any) {
      setActionMessage(`Action failed: ${err}`);
    }
  };

  const handleEnterRace = async (carId: string, raceId: string) => {
    try {
      const result = await enterRace(carId, raceId);
      setActionMessage(`${result.message} (Position: ${result.position}, Prize: $${result.prize_awarded})`);
      const updated = await getGameState();
      setGameState(updated);
    } catch (err: any) {
      setActionMessage(`Race entry failed: ${err}`);
    }
  };

  const handleReloadDataset = async (newPath: string) => {
    try {
      const newState = await reloadDataset(newPath);
      setGameState(newState);
      setActionMessage('Dataset reloaded successfully.');
    } catch (err: any) {
      setActionMessage(`Failed to reload dataset: ${err}`);
    }
  };

  if (!gameState) {
    return (
      <div style={styles.loadingContainer}>
        <h2>Loading GTR2 RACEWARS...</h2>
      </div>
    );
  }

  return (
    <div style={styles.appContainer}>
      {/* Header */}
      <header style={styles.header}>
        <div>
          <h1 style={styles.title}>🏎️ GTR2 RACEWARS</h1>
          <span style={styles.subtext}>
            Day {gameState.current_day} | Budget: ${gameState.player.budget.toLocaleString()}
          </span>
        </div>

        <div style={styles.controlsGroup}>
          <div style={styles.speedGroup}>
            {(['Paused', 'OneDayEveryFiveSec', 'OneDayPerSec', 'OneWeekPerSec'] as TimeSpeed[]).map((speed) => (
              <button
                key={speed}
                onClick={() => handleSpeedChange(speed)}
                style={{
                  ...styles.speedBtn,
                  backgroundColor: gameState.time_speed === speed ? '#2563eb' : '#334155',
                }}
              >
                {speed === 'Paused' && '⏸️ Pause'}
                {speed === 'OneDayEveryFiveSec' && '⏳ 5s/d'}
                {speed === 'OneDayPerSec' && '▶️ 1s/d'}
                {speed === 'OneWeekPerSec' && '⏩ 1s/wk'}
              </button>
            ))}
          </div>

          <button onClick={() => setIsConfigOpen(true)} style={styles.configBtn}>
            ⚙️ Settings
          </button>
        </div>
      </header>

      {/* Banner */}
      {actionMessage && (
        <div style={styles.banner}>
          <span>{actionMessage}</span>
          <button onClick={() => setActionMessage('')} style={styles.bannerClose}>
            ✕
          </button>
        </div>
      )}

      {/* Tabs */}
      <nav style={styles.navTabs}>
        <button
          onClick={() => setActiveTab('dashboard')}
          style={{ ...styles.tabBtn, ...(activeTab === 'dashboard' ? styles.activeTabBtn : {}) }}
        >
          📊 Dashboard
        </button>
        <button
          onClick={() => setActiveTab('garage')}
          style={{ ...styles.tabBtn, ...(activeTab === 'garage' ? styles.activeTabBtn : {}) }}
        >
          🚘 Garage ({gameState.player.cars.length})
        </button>
        <button
          onClick={() => setActiveTab('races')}
          style={{ ...styles.tabBtn, ...(activeTab === 'races' ? styles.activeTabBtn : {}) }}
        >
          🏁 Races
        </button>
        <button
          onClick={() => setActiveTab('activities')}
          style={{ ...styles.tabBtn, ...(activeTab === 'activities' ? styles.activeTabBtn : {}) }}
        >
          💼 Activities
        </button>
      </nav>

      {/* Content */}
      <main style={styles.content}>
        {activeTab === 'dashboard' && (
          <div style={styles.grid}>
            <div style={styles.card}>
              <h3>Player Overview</h3>
              <p><strong>Age (Days):</strong> {gameState.player.age_days}</p>
              <p><strong>Budget:</strong> ${gameState.player.budget.toLocaleString()}</p>
              <p><strong>Cars Owned:</strong> {gameState.player.cars.length}</p>
              <p><strong>Active Activities:</strong> {gameState.player.active_actions.length}</p>
            </div>
            <div style={styles.card}>
              <h3>Car Showroom</h3>
              {gameState.catalog.cars.map((car) => (
                <div key={car.id} style={styles.itemRow}>
                  <div>
                    <strong>{car.name}</strong> - ${car.price.toLocaleString()}
                  </div>
                  <button onClick={() => handleBuyCar(car.id)} style={styles.actionBtn}>
                    Buy
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'garage' && (
          <div style={styles.grid}>
            {gameState.player.cars.length === 0 ? (
              <p style={{ color: '#94a3b8' }}>No cars owned yet. Visit the Dashboard to buy one!</p>
            ) : (
              gameState.player.cars.map((car) => (
                <div key={car.id} style={styles.card}>
                  <h3>{car.name}</h3>
                  <p><strong>Tires Available:</strong> {car.tire_sets_available}</p>
                  <p><strong>Needs Oil Change:</strong> {car.needs_oil_change ? '⚠️ Yes' : '✅ No'}</p>
                  <p><strong>Needs Engine Rebuild:</strong> {car.needs_engine_rebuild ? '⚠️ Yes' : '✅ No'}</p>
                  <p><strong>Needs Gearbox Maintenance:</strong> {car.needs_gearbox_maint ? '⚠️ Yes' : '✅ No'}</p>
                  <div style={styles.maintGroup}>
                    <button onClick={() => handleMaintainCar(car.id, 'oil')} style={styles.maintBtn}>
                      Oil (${car.oil_change_cost})
                    </button>
                    <button onClick={() => handleMaintainCar(car.id, 'engine')} style={styles.maintBtn}>
                      Engine (${car.engine_rebuild_cost})
                    </button>
                    <button onClick={() => handleMaintainCar(car.id, 'gearbox')} style={styles.maintBtn}>
                      Gearbox (${car.gearbox_maint_cost})
                    </button>
                    <button onClick={() => handleMaintainCar(car.id, 'tires')} style={styles.maintBtn}>
                      Tires (${car.tire_set_cost})
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'races' && (
          <div style={styles.grid}>
            {gameState.catalog.races.map((race) => (
              <div key={race.id} style={styles.card}>
                <h3>{race.name}</h3>
                <p><strong>Day of Year:</strong> {race.day_of_year}</p>
                <p><strong>Entry Fee:</strong> ${race.entry_fee}</p>
                <p><strong>Prize Pool:</strong> ${race.prize_pool.toLocaleString()}</p>
                {gameState.player.cars.map((car) => (
                  <button
                    key={car.id}
                    onClick={() => handleEnterRace(car.id, race.id)}
                    style={styles.actionBtn}
                  >
                    Enter with {car.name}
                  </button>
                ))}
              </div>
            ))}
          </div>
        )}

        {activeTab === 'activities' && (
          <div style={styles.grid}>
            {gameState.catalog.actions.map((act) => (
              <div key={act.id} style={styles.card}>
                <h3>{act.name}</h3>
                <p>{act.description || 'No description provided.'}</p>
                <p><strong>Cost:</strong> ${act.base_cost}</p>
                <p><strong>Success Rate:</strong> {act.success_rate * 100}%</p>
                <button onClick={() => handlePerformAction(act.id)} style={styles.actionBtn}>
                  Start Activity
                </button>
              </div>
            ))}
          </div>
        )}
      </main>

      {/* Alert Modal */}
      <AlertModal
        alerts={gameState.pending_alerts}
        onDismiss={(updatedState) => setGameState(updatedState)}
      />

      {/* Dataset Settings Modal */}
      <ConfigModal
        isOpen={isConfigOpen}
        currentPath={gameState.dataset_path || ''}
        onClose={() => setIsConfigOpen(false)}
        onReloadDataset={handleReloadDataset}
      />
    </div>
  );
};

export default App;

const styles: Record<string, React.CSSProperties> = {
  loadingContainer: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    height: '100vh',
    backgroundColor: '#0f172a',
    color: '#f8fafc',
    fontFamily: 'sans-serif',
  },
  appContainer: {
    backgroundColor: '#0f172a',
    color: '#f8fafc',
    minHeight: '100vh',
    fontFamily: 'sans-serif',
    padding: '1.5rem',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1rem',
    borderBottom: '1px solid #334155',
    paddingBottom: '1rem',
  },
  title: {
    margin: 0,
    fontSize: '1.75rem',
  },
  subtext: {
    color: '#94a3b8',
    fontSize: '0.95rem',
  },
  controlsGroup: {
    display: 'flex',
    gap: '1rem',
    alignItems: 'center',
  },
  speedGroup: {
    display: 'flex',
    gap: '0.3rem',
  },
  speedBtn: {
    color: '#ffffff',
    border: 'none',
    padding: '0.4rem 0.7rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '0.85rem',
    fontWeight: 'bold',
  },
  configBtn: {
    backgroundColor: '#334155',
    color: '#ffffff',
    border: '1px solid #475569',
    padding: '0.4rem 0.8rem',
    borderRadius: '6px',
    cursor: 'pointer',
    fontWeight: 'bold',
    display: 'flex',
    alignItems: 'center',
    gap: '0.4rem',
  },
  banner: {
    backgroundColor: '#1e293b',
    borderLeft: '4px solid #3b82f6',
    padding: '0.75rem 1rem',
    marginBottom: '1rem',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    borderRadius: '4px',
  },
  bannerClose: {
    background: 'none',
    border: 'none',
    color: '#94a3b8',
    cursor: 'pointer',
  },
  navTabs: {
    display: 'flex',
    gap: '0.5rem',
    marginBottom: '1.5rem',
  },
  tabBtn: {
    backgroundColor: '#1e293b',
    color: '#94a3b8',
    border: '1px solid #334155',
    padding: '0.6rem 1.2rem',
    borderRadius: '6px',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
  activeTabBtn: {
    backgroundColor: '#2563eb',
    color: '#ffffff',
    borderColor: '#2563eb',
  },
  content: {
    marginTop: '1rem',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
    gap: '1rem',
  },
  card: {
    backgroundColor: '#1e293b',
    borderRadius: '8px',
    border: '1px solid #334155',
    padding: '1.25rem',
  },
  itemRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '0.5rem 0',
    borderBottom: '1px solid #334155',
  },
  actionBtn: {
    backgroundColor: '#2563eb',
    color: '#ffffff',
    border: 'none',
    padding: '0.4rem 0.8rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
    marginTop: '0.5rem',
  },
  maintGroup: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: '0.5rem',
    marginTop: '0.75rem',
  },
  maintBtn: {
    backgroundColor: '#334155',
    color: '#ffffff',
    border: 'none',
    padding: '0.4rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '0.8rem',
  },
};
