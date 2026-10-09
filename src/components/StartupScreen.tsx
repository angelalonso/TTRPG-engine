import React, { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  getDefaultDatasetDialogPath,
  getAppConfig,
  getRememberedDatasetPaths,
  forgetDatasetPath,
  isDatasetPath,
  listSaveSlots,
  deleteSaveSlot,
  loadGameFrom,
  loadSaveGameAtOwnRisk,
  migrateSaveGameFrom,
  type SaveSlot,
  rememberDatasetPath,
  selectDatasetFolder,
  startNewGame,
} from '../services/tauriApi';
import type { GameState } from '../types/game';
import { SaveSlotsModal } from './SaveSlotsModal';

interface StartupScreenProps {
  onStarted: (state: GameState, showIntro: boolean) => Promise<void>;
  onEditor: (datasetPath: string) => void;
}

type Mode = 'menu' | 'new' | 'editor';

export const StartupScreen: React.FC<StartupScreenProps> = ({ onStarted, onEditor }) => {
  const [mode, setMode] = useState<Mode>('menu');
  const [datasets, setDatasets] = useState<string[]>([]);
  const [selectedDataset, setSelectedDataset] = useState('');
  const [playerName, setPlayerName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [loadSlots, setLoadSlots] = useState<SaveSlot[]>([]);
  const [loadModalOpen, setLoadModalOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const loadDatasets = async () => {
      const config = await getAppConfig();
      const remembered = getRememberedDatasetPaths();
      const fallback = await getDefaultDatasetDialogPath();
      const candidates = Array.from(new Set(
        [config.dataset_path, ...remembered, fallback]
          .filter((path): path is string => typeof path === 'string' && path.trim().length > 0)
          .map((path) => path.trim()),
      ));
      const validity = await Promise.all(candidates.map(async (path) => ({
        path,
        valid: await isDatasetPath(path),
      })));
      const paths = validity.filter((entry) => entry.valid).map((entry) => entry.path);
      remembered
        .filter((path) => !paths.includes(path))
        .forEach(forgetDatasetPath);
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

  const openLoadGame = async () => {
    const dataset = datasets[0];
    if (!dataset) {
      setError('No dataset has been configured yet.');
      return;
    }
    setLoading(true);
    setError('');
    try {
      const slots = await listSaveSlots(dataset);
      if (!slots.length) {
        setError('No saved game was found for the latest dataset.');
        return;
      }
      setLoadSlots(slots);
      setLoadModalOpen(true);
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
      await onStarted(state, true);
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
            <button style={styles.choice} onClick={() => void openLoadGame()} disabled={loading}>
              Load Game
            </button>
            <button style={styles.choice} onClick={() => { setMode('new'); setError(''); }} disabled={loading}>
              New Game
            </button>
            <button style={styles.choice} onClick={() => { setMode('editor'); setError(''); }} disabled={loading}>
              Dataset Editor
            </button>
            <button style={styles.choice} onClick={() => void getCurrentWindow().close()} disabled={loading}>
              Quit Game
            </button>
          </div>
        ) : mode === 'new' ? (
          <div style={styles.newGame}>
            <label style={styles.nameField}>
              <span style={styles.nameLabel}>Player Name</span>
              <input
                style={styles.nameInput}
                autoFocus
                value={playerName}
                onChange={(event) => setPlayerName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter') {
                    event.preventDefault();
                    if (!loading) void createGame();
                  }
                }}
                placeholder="Your name"
                disabled={loading}
              />
            </label>
            <h2>Choose a dataset</h2>
            <div style={styles.datasetList}>
              {datasets.map((path) => (
                <div key={path} style={styles.datasetRow}>
                  <button
                    style={path === selectedDataset ? styles.datasetSelected : styles.dataset}
                    onClick={() => chooseNewDataset(path)}
                    disabled={loading}
                  >
                    {path}
                  </button>
                  <button
                    type="button"
                    style={styles.removeDataset}
                    onClick={() => {
                      forgetDatasetPath(path);
                      setDatasets((current) => {
                        const next = current.filter((entry) => entry !== path);
                        if (selectedDataset === path) setSelectedDataset(next[0] || '');
                        return next;
                      });
                    }}
                    disabled={loading}
                    aria-label={`Remove ${path} from the dataset list`}
                  >
                    Remove
                  </button>
                </div>
              ))}
            </div>
            <button style={styles.secondaryButton} onClick={() => void addDataset()} disabled={loading}>
              Add dataset from file browser
            </button>
            <div style={styles.newGameActions}>
              <button style={styles.secondaryButton} onClick={() => setMode('menu')} disabled={loading}>
                Back
              </button>
              <button style={styles.primaryButton} onClick={() => void createGame()} disabled={loading}>
                Create Game
              </button>
            </div>
          </div>
        ) : (
          <div style={styles.newGame}>
            <h2>Edit a dataset</h2>
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
            <div style={styles.newGameActions}>
              <button style={styles.secondaryButton} onClick={() => setMode('menu')} disabled={loading}>
                Back
              </button>
              <button
                style={styles.editorButton}
                onClick={() => selectedDataset && onEditor(selectedDataset)}
                disabled={loading || !selectedDataset}
              >
                Open Editor
              </button>
            </div>
          </div>
        )}
        {loading && <p>Loading...</p>}
        {error && <p role="alert" style={styles.error}>{error}</p>}
      </div>
      {loadModalOpen && datasets[0] && (
        <SaveSlotsModal
          mode="load"
          slots={loadSlots}
          onSave={async () => {}}
          onLoad={async (slot) => {
            const state = await loadGameFrom(datasets[0], slot);
            rememberDatasetPath(datasets[0]);
            setLoadModalOpen(false);
            await onStarted(state, false);
          }}
          onMigrate={async (slot) => {
            const state = await migrateSaveGameFrom(datasets[0], slot);
            rememberDatasetPath(datasets[0]);
            setLoadModalOpen(false);
            await onStarted(state, false);
          }}
          onLoadAtOwnRisk={async (slot) => {
            const state = await loadSaveGameAtOwnRisk(datasets[0], slot);
            rememberDatasetPath(datasets[0]);
            setLoadModalOpen(false);
            await onStarted(state, false);
          }}
          onDelete={async (slot) => {
            await deleteSaveSlot(datasets[0], slot);
            setLoadSlots(await listSaveSlots(datasets[0]));
          }}
          onClose={() => setLoadModalOpen(false)}
        />
      )}
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
  datasetRow: { display: 'flex', gap: '0.5rem', alignItems: 'center' },
  dataset: { padding: '0.75rem', textAlign: 'left', cursor: 'pointer' },
  datasetSelected: { padding: '0.75rem', textAlign: 'left', cursor: 'pointer', border: '2px solid var(--primary-accent)' },
  removeDataset: { padding: '0.35rem 0.5rem', cursor: 'pointer', color: 'var(--danger-text)' },
  nameField: { display: 'grid', gap: '0.4rem' },
  nameLabel: { fontWeight: 700, fontSize: '1.1rem' },
  nameInput: { height: '3.5rem', padding: '0.5rem 0.65rem', boxSizing: 'border-box' },
  newGameActions: { display: 'flex', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '0.75rem' },
  secondaryButton: { padding: '0.6rem 0.9rem', cursor: 'pointer' },
  primaryButton: { padding: '0.6rem 0.9rem', cursor: 'pointer', fontWeight: 'bold' },
  editorButton: { padding: '0.6rem 0.9rem', cursor: 'pointer', fontWeight: 'bold', color: 'var(--white-text)', background: 'var(--danger-action)', border: 0, borderRadius: 5 },
  error: { background: 'var(--error-background)', border: '1px solid var(--error-border)', borderRadius: '8px', padding: '0.75rem', color: 'var(--error-light-text)' },
};
