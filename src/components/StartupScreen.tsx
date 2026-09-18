import React, { useState } from 'react';
import {
  listSaveSlots,
  loadGameFrom,
  getDefaultDatasetDialogPath,
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

  const chooseFolder = async () => selectDatasetFolder(await getDefaultDatasetDialogPath());
  const chooseSaveDataset = async () => {
    setLoading(true);
    setError('');
    try {
      const selected = await chooseFolder();
      if (!selected) return;
      setDatasetPath(selected);
      const available = await listSaveSlots(selected);
      setSlots(available);
      if (!available.length) setError('No saved games were found in that dataset folder.');
    } catch (caught) {
      setError(String(caught));
    } finally {
      setLoading(false);
    }
  };

  const startFromScratch = async () => {
    setLoading(true);
    setError('');
    try {
      const selected = await chooseFolder();
      if (selected) await onStarted(await startNewGame(selected));
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
        <p>Choose how you want to begin.</p>
        <div style={styles.choices}>
          <button style={styles.choice} onClick={() => void chooseSaveDataset()} disabled={loading}>
            <strong>Load saved game</strong>
            <span>Choose a dataset folder, then select one of its save slots.</span>
          </button>
          <button style={styles.choice} onClick={() => void startFromScratch()} disabled={loading}>
            <strong>Start from scratch</strong>
            <span>Choose a dataset folder and create a new game.</span>
          </button>
        </div>
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
                    await onStarted(await loadGameFrom(datasetPath, slot.name));
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
};
