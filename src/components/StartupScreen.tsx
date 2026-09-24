import React, { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  getDefaultDatasetDialogPath,
  getLatestSaveSlot,
  getRememberedDatasetPaths,
  loadGameFrom,
  rememberDatasetPath,
  selectDatasetFolder,
  startNewGame,
} from '../services/tauriApi';
import type { GameState } from '../types/game';

interface StartupScreenProps {
  onStarted: (state: GameState) => Promise<void>;
}

type Mode = 'menu' | 'new';

export const StartupScreen: React.FC<StartupScreenProps> = ({ onStarted }) => {
  const [mode, setMode] = useState<Mode>('menu');
  const [datasets, setDatasets] = useState<string[]>([]);
  const [selectedDataset, setSelectedDataset] = useState('');
  const [playerName, setPlayerName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const loadDatasets = async () => {
      const remembered = getRememberedDatasetPaths();
      const fallback = await getDefaultDatasetDialogPath();
      const paths = Array.from(new Set([...remembered, fallback].filter(Boolean)));
      if (!cancelled) {
        setDatasets(paths);
        setSelectedDataset(paths[0] || '');
      }
    };
    void loadDatasets().catch((caught) => {
      if (!cancelled) setError(String(caught));
    });
    return () => { cancelled = true; };
  }, []);

  const addDataset = async () => {
    setLoading(true);
    setError('');
    try {
      const selected = await selectDatasetFolder(selectedDataset || await getDefaultDatasetDialogPath());
      if (selected) {
        rememberDatasetPath(selected);
        setDatasets((current) => [selected, ...current.filter((path) => path !== selected)]);
        setSelectedDataset(selected);
      }
    } catch (caught) {
      setError(String(caught));
    } finally {
      setLoading(false);
    }
  };

  const continueGame = async () => {
    const dataset = datasets[0];
    if (!dataset) {
      setError('No dataset has been configured yet.');
      return;
    }
    setLoading(true);
    setError('');
    try {
      const latest = await getLatestSaveSlot(dataset);
      if (!latest) {
        setError('No saved game was found for the latest dataset.');
        return;
      }
      const state = await loadGameFrom(dataset, latest.name);
      rememberDatasetPath(dataset);
      await onStarted(state);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setLoading(false);
    }
  };

  const createGame = async () => {
    const name = playerName.trim();
    if (!selectedDataset) {
      setError('Choose a dataset before starting a new game.');
      return;
    }
    if (!name) {
      setError('Enter a player name before starting the game.');
      return;
    }
    setLoading(true);
    setError('');
    try {
      const state = await startNewGame(selectedDataset, name);
      rememberDatasetPath(selectedDataset);
      await onStarted(state);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setLoading(false);
    }
  };

  const chooseNewDataset = (path: string) => {
    setSelectedDataset(path);
    rememberDatasetPath(path);
    setDatasets((current) => [path, ...current.filter((entry) => entry !== path)]);
  };

  return (
    <div style={styles.startup}>
      <div style={styles.card}>
        <h1>Start Game</h1>
        {mode === 'menu' ? (
          <div style={styles.choices}>
            <button style={styles.choice} onClick={() => void continueGame()} disabled={loading}>
              Continue Game
            </button>
            <button style={styles.choice} onClick={() => { setMode('new'); setError(''); }} disabled={loading}>
              New Game
            </button>
            <button style={styles.choice} onClick={() => void getCurrentWindow().close()} disabled={loading}>
              Quit Game
            </button>
          </div>
        ) : (
          <div style={styles.newGame}>
            <h2>Choose a dataset</h2>
            <div style={styles.datasetList}>
              {datasets.map((path) => (
                <button
                  key={path}
                  style={path === selectedDataset ? styles.datasetSelected : styles.dataset}
                  onClick={() => chooseNewDataset(path)}
                  disabled={loading}
                >
                  {path}
                </button>
              ))}
            </div>
            <button style={styles.secondaryButton} onClick={() => void addDataset()} disabled={loading}>
              Add dataset from file browser
            </button>
            <label style={styles.nameField}>
              Player name
              <input
                style={styles.nameInput}
                autoFocus
                value={playerName}
                onChange={(event) => setPlayerName(event.target.value)}
                placeholder="Your name"
                disabled={loading}
              />
            </label>
            <div style={styles.newGameActions}>
              <button style={styles.secondaryButton} onClick={() => setMode('menu')} disabled={loading}>
                Back
              </button>
              <button style={styles.primaryButton} onClick={() => void createGame()} disabled={loading}>
                Create Game
              </button>
            </div>
          </div>
        )}
        {loading && <p>Loading...</p>}
        {error && <p role="alert" style={styles.error}>{error}</p>}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  startup: { minHeight: '100vh', display: 'grid', placeItems: 'center', padding: '1.5rem', boxSizing: 'border-box', background: 'var(--app-background)', color: 'var(--primary-text)' },
  card: { width: 'min(560px, 94vw)', padding: '2rem', background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: '14px', boxShadow: '0 18px 45px var(--modal-overlay)' },
  choices: { display: 'grid', gap: '1rem', margin: '2rem auto 0', width: 'min(320px, 100%)' },
  choice: { minHeight: 58, padding: '1rem', border: '1px solid var(--control-border)', borderRadius: '10px', background: 'var(--control-background)', color: 'var(--primary-text)', cursor: 'pointer', fontSize: '1.05rem', fontWeight: 700 },
  newGame: { display: 'grid', gap: '0.75rem', marginTop: '1.5rem' },
  datasetList: { display: 'grid', gap: '0.5rem', maxHeight: 220, overflowY: 'auto' },
  dataset: { padding: '0.75rem', textAlign: 'left', cursor: 'pointer' },
  datasetSelected: { padding: '0.75rem', textAlign: 'left', cursor: 'pointer', border: '2px solid var(--primary-accent)' },
  nameField: { display: 'grid', gap: '0.4rem', marginTop: '0.5rem' },
  nameInput: { height: '5.2rem', padding: '0.5rem 0.65rem', boxSizing: 'border-box' },
  newGameActions: { display: 'flex', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '0.75rem' },
  secondaryButton: { padding: '0.6rem 0.9rem', cursor: 'pointer' },
  primaryButton: { padding: '0.6rem 0.9rem', cursor: 'pointer', fontWeight: 'bold' },
  error: { background: 'var(--error-background)', border: '1px solid var(--error-border)', borderRadius: '8px', padding: '0.75rem', color: 'var(--error-light-text)' },
};
