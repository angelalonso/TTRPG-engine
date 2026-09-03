import React, { useEffect, useState } from 'react';
import { getGameState } from './services/tauriApi';
import type { GameState } from './types/game';

import { TimeControl } from './components/TimeControl';
import { ActionsView } from './components/ActionsView';
import { Garage } from './components/Garage';
import { RaceCalendar } from './components/RaceCalendar';

type ActiveTab = 'actions' | 'garage' | 'races';

export const App: React.FC = () => {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [activeTab, setActiveTab] = useState<ActiveTab>('actions');
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // Fetch initial game state from Tauri backend on mount
  useEffect(() => {
    const fetchInitialState = async () => {
      try {
        setLoading(true);
        const state = await getGameState();
        setGameState(state);
      } catch (err) {
        setError(String(err));
      } finally {
        setLoading(false);
      }
    };

    fetchInitialState();
  }, []);

  if (loading) {
    return (
      <div style={styles.centerContainer}>
        <h2>🏎️ Loading Game Engine...</h2>
      </div>
    );
  }

  if (error || !gameState) {
    return (
      <div style={styles.centerContainer}>
        <h2 style={{ color: '#ef4444' }}>⚠️ Engine Initialization Failed</h2>
        <p style={{ color: '#94a3b8' }}>{error ?? 'Unknown error occurred.'}</p>
        <button
          style={styles.retryBtn}
          onClick={() => window.location.reload()}
        >
          Retry Connection
        </button>
      </div>
    );
  }

  return (
    <div style={styles.appContainer}>
      {/* Top Engine & Time Dashboard */}
      <header style={styles.header}>
        <div style={styles.brandRow}>
          <h1 style={styles.brandTitle}>🏎️ Sim Racing Manager</h1>
          <span style={styles.versionBadge}>v2.0</span>
        </div>
        <TimeControl gameState={gameState} onStateUpdate={setGameState} />
      </header>

      {/* Primary Tab Navigation */}
      <nav style={styles.navTabs}>
        <button
          style={activeTab === 'actions' ? styles.activeTabBtn : styles.tabBtn}
          onClick={() => setActiveTab('actions')}
        >
          ⚡ Daily Actions & Jobs
        </button>
        <button
          style={activeTab === 'garage' ? styles.activeTabBtn : styles.tabBtn}
          onClick={() => setActiveTab('garage')}
        >
          🚗 Garage & Maintenance ({gameState.player.cars.length})
        </button>
        <button
          style={activeTab === 'races' ? styles.activeTabBtn : styles.tabBtn}
          onClick={() => setActiveTab('races')}
        >
          🏁 Race Calendar ({gameState.catalog.races.length})
        </button>
      </nav>

      {/* Tab Viewport */}
      <main style={styles.mainContent}>
        {activeTab === 'actions' && (
          <ActionsView gameState={gameState} onStateUpdate={setGameState} />
        )}
        {activeTab === 'garage' && (
          <Garage gameState={gameState} onStateUpdate={setGameState} />
        )}
        {activeTab === 'races' && (
          <RaceCalendar gameState={gameState} onStateUpdate={setGameState} />
        )}
      </main>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  appContainer: {
    minHeight: '100vh',
    backgroundColor: '#0f172a',
    color: '#f8fafc',
    fontFamily: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    padding: '1.5rem',
    boxSizing: 'border-box',
    maxWidth: '1200px',
    margin: '0 auto',
  },
  centerContainer: {
    minHeight: '100vh',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#0f172a',
    color: '#f8fafc',
    fontFamily: 'system-ui, sans-serif',
  },
  header: {
    marginBottom: '1rem',
  },
  brandRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    marginBottom: '0.75rem',
  },
  brandTitle: {
    margin: 0,
    fontSize: '1.5rem',
    fontWeight: 'bold',
  },
  versionBadge: {
    backgroundColor: '#3b82f6',
    color: '#ffffff',
    fontSize: '0.75rem',
    fontWeight: 'bold',
    padding: '0.15rem 0.5rem',
    borderRadius: '12px',
  },
  navTabs: {
    display: 'flex',
    gap: '0.5rem',
    borderBottom: '2px solid #334155',
    paddingBottom: '0.5rem',
    marginBottom: '1.5rem',
    overflowX: 'auto',
  },
  tabBtn: {
    padding: '0.6rem 1.2rem',
    borderRadius: '6px 6px 0 0',
    border: 'none',
    backgroundColor: '#1e293b',
    color: '#94a3b8',
    fontWeight: 'bold',
    fontSize: '0.9rem',
    cursor: 'pointer',
    transition: 'all 0.15s ease',
  },
  activeTabBtn: {
    padding: '0.6rem 1.2rem',
    borderRadius: '6px 6px 0 0',
    border: 'none',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    fontWeight: 'bold',
    fontSize: '0.9rem',
    cursor: 'pointer',
  },
  mainContent: {
    display: 'flex',
    flexDirection: 'column',
  },
  retryBtn: {
    marginTop: '1rem',
    padding: '0.5rem 1rem',
    backgroundColor: '#2563eb',
    color: '#ffffff',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
};

export default App;
