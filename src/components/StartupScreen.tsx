import React, { useEffect, useState } from 'react';
import {
  listSaveSlots,
  loadGameFrom,
  getDefaultDatasetDialogPath,
  getRememberedDatasetPath,
  rememberDatasetPath,
  selectDatasetFolder,
  startNewGame,
} from '../services/tauriApi';
import type { GameState } from '../types/game';

interface StartupScreenProps {
  onStarted: (state: GameState) => Promise<void>;
}

export const StartupScreen: React.FC<StartupScreenProps> = ({ onStarted }) => {
  const [datasetPath, setDatasetPath] = useState('');
  const [slots, setSlots] = useState<{ name: string }[]>([]);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [playerName, setPlayerName] = useState('');
  const [newGameDatasetPath, setNewGameDatasetPath] = useState('');

  useEffect(() => {
    let cancelled = false;
    const loadDefaultDataset = async () => {
      setLoading(true);
      try {
        const defaultPath = getRememberedDatasetPath() || await getDefaultDatasetDialogPath();
        const available = await listSaveSlots(defaultPath);
        if (cancelled) return;
        setDatasetPath(defaultPath);
        setNewGameDatasetPath(defaultPath);
        setSlots(available);
      } catch (caught) {
        if (!cancelled) setError(String(caught));
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    void loadDefaultDataset();
    return () => {
      cancelled = true;
    };
  }, []);

  const chooseFolder = async () =>
    selectDatasetFolder(getRememberedDatasetPath() || await getDefaultDatasetDialogPath());
  const chooseSaveDataset = async () => {
    setLoading(true);
    setError('');
    try {
      const selected = await chooseFolder();
      if (!selected) return;
      setDatasetPath(selected);
      const available = await listSaveSlots(selected);
      rememberDatasetPath(selected);
      setSlots(available);
      if (!available.length) setError('No saved games were found in that dataset folder.');
    } catch (caught) {
      setError(String(caught));
    } finally {
      setLoading(false);
    }
  };

  const chooseNewGameDataset = async () => {
    setLoading(true);
    setError('');
    try {
      const selected = await chooseFolder();
      if (selected) {
        setNewGameDatasetPath(selected);
      }
    } catch (caught) {
      setError(String(caught));
    } finally {
      setLoading(false);
    }
  };

  const startFromScratch = async () => {
    const name = playerName.trim();
    if (!newGameDatasetPath) return;
    if (!name) {
      setError('Enter a player name before starting the game.');
      return;
    }
    setLoading(true);
    setError('');
    try {
      const state = await startNewGame(newGameDatasetPath, name);
      rememberDatasetPath(newGameDatasetPath);
      await onStarted(state);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={styles.startup}>
      <div style={styles.card}>
        <h1>Start TTRPG Engine</h1>
        <p>Dataset: {newGameDatasetPath || 'Loading default dataset...'}</p>
        <div style={styles.choices}>
          <button style={styles.choice} onClick={() => void startFromScratch()} disabled={loading || !newGameDatasetPath}>
            <strong>Start new game</strong>
            <span>Use the selected dataset and create a new game.</span>
          </button>
          <button style={styles.choice} onClick={() => void chooseSaveDataset()} disabled={loading}>
            <strong>Choose another dataset for saved games</strong>
            <span>Browse for a dataset folder and load one of its saves below.</span>
          </button>
        </div>
        {newGameDatasetPath && (
          <div style={styles.newGame}>
            <h2>New game</h2>
            <p>Dataset selected: {newGameDatasetPath}</p>
            <label style={styles.nameField}>
              Player name
              <input
                autoFocus
                value={playerName}
                onChange={(event) => setPlayerName(event.target.value)}
                placeholder="Your name"
                disabled={loading}
              />
            </label>
            <div style={styles.newGameActions}>
              <button style={styles.secondaryButton} onClick={() => setNewGameDatasetPath('')} disabled={loading}>
                Choose another dataset
              </button>
              <button style={styles.primaryButton} onClick={() => void startFromScratch()} disabled={loading}>
                Create game
              </button>
            </div>
          </div>
        )}
        {loading && <p>Opening dataset folder...</p>}
        {datasetPath && slots.length > 0 && (
          <div style={styles.slots}>
            <h2>Saved games</h2>
            {slots.map((slot) => (
              <button
                key={slot.name}
                style={styles.slot}
                disabled={loading}
                onClick={async () => {
                  setLoading(true);
                  setError('');
                  try {
                    const state = await loadGameFrom(datasetPath, slot.name);
                    rememberDatasetPath(datasetPath);
                    await onStarted(state);
                  } catch (caught) {
                    setError(String(caught));
                  } finally {
                    setLoading(false);
                  }
                }}
              >
                {slot.name}
              </button>
            ))}
          </div>
        )}
        {error && <p role="alert" style={styles.error}>{error}</p>}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  startup: {
    minHeight: '100vh',
    display: 'grid',
    placeItems: 'center',
    padding: '1.5rem',
    boxSizing: 'border-box',
    background: 'var(--app-background)',
    color: 'var(--primary-text)',
  },
  card: {
    width: 'min(680px, 94vw)',
    padding: '2rem',
    background: 'var(--surface-background)',
    border: '1px solid var(--surface-border)',
    borderRadius: '14px',
    boxShadow: '0 18px 45px var(--modal-overlay)',
  },
  choices: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
    gap: '1rem',
    marginTop: '1.5rem',
  },
  choice: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.6rem',
    minHeight: 140,
    padding: '1.25rem',
    textAlign: 'left',
    border: '1px solid var(--control-border)',
    borderRadius: '10px',
    background: 'var(--control-background)',
    color: 'var(--primary-text)',
    cursor: 'pointer',
  },
  slots: {
    display: 'grid',
    gap: '0.5rem',
    marginTop: '1.5rem',
    paddingTop: '1rem',
    borderTop: '1px solid var(--surface-border)',
  },
  slot: { padding: '0.8rem 1rem', textAlign: 'left', cursor: 'pointer' },
  error: {
    background: 'var(--error-background)',
    border: '1px solid var(--error-border)',
    borderRadius: '8px',
    padding: '0.75rem',
    color: 'var(--error-light-text)',
  },
  nameField: { display: 'grid', gap: '0.4rem', marginTop: '1rem' },
  newGame: {
    display: 'grid',
    gap: '0.5rem',
    marginTop: '1.5rem',
    paddingTop: '1rem',
    borderTop: '1px solid var(--surface-border)',
  },
  newGameActions: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '0.75rem',
    marginTop: '0.75rem',
  },
  secondaryButton: { padding: '0.6rem 0.9rem', cursor: 'pointer' },
  primaryButton: { padding: '0.6rem 0.9rem', cursor: 'pointer', fontWeight: 'bold' },
};
