import React, { useState, useEffect } from 'react';
import { selectDatasetFolder } from '../services/tauriApi';

interface ConfigModalProps {
  isOpen: boolean;
  currentPath: string;
  onClose: () => void;
  onReloadDataset: (newPath: string) => Promise<void>;
  popupCategories?: string[];
  onPopupCategoriesChange?: (categories: string[]) => Promise<void>;
}

export const ConfigModal: React.FC<ConfigModalProps> = ({
  isOpen,
  currentPath,
  onClose,
  onReloadDataset,
  popupCategories = [],
  onPopupCategoriesChange,
}) => {
  const [datasetPath, setDatasetPath] = useState(currentPath || './dataset');
  const [draftPopupCategories, setDraftPopupCategories] = useState(popupCategories);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setDatasetPath(currentPath || './dataset');
      setDraftPopupCategories(popupCategories);
    }
  }, [isOpen, currentPath, popupCategories]);

  if (!isOpen) return null;

  const handleBrowseFolder = async () => {
    try {
      const selectedFolder = await selectDatasetFolder(datasetPath || './dataset');
      if (selectedFolder) {
        setDatasetPath(selectedFolder);
      }
    } catch (error) {
      console.error('Failed to open dataset picker:', error);
    }
  };

  const handleSaveAndReload = async () => {
    setLoading(true);
    try {
      await onReloadDataset(datasetPath || './dataset');
      onClose();
    } finally {
      setLoading(false);
    }
  };

  const handleSaveSettings = async () => {
    setLoading(true);
    try {
      await onPopupCategoriesChange?.(draftPopupCategories);
      onClose();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={styles.overlay}>
      <div style={styles.modal}>
        <div style={styles.header}>
          <h3 style={{ margin: 0 }}>⚙️ Configuration Settings</h3>
          <button style={styles.closeBtn} onClick={onClose}>
            ✕
          </button>
        </div>
        <div style={styles.body}>
          <label style={styles.label}>
            <strong>Dataset Folder Path:</strong>
          </label>
          <div style={styles.inputGroup}>
            <input
              type="text"
              value={datasetPath}
              onChange={(e) => setDatasetPath(e.target.value)}
              style={styles.input}
              placeholder="./dataset or /path/to/dataset"
            />
            <button
              type="button"
              style={styles.browseBtn}
              onClick={handleBrowseFolder}
              disabled={loading}
            >
              📂 Browse
            </button>
          </div>
          <p style={styles.hint}>
            Path containing <code>objects.csv</code>, <code>events.csv</code>, and optional <code>config.csv</code>.
          </p>
          <button style={styles.saveBtn} onClick={handleSaveAndReload} disabled={loading}>
            {loading ? 'Loading...' : 'Load Dataset'}
          </button>
          <strong>Popup and pause categories</strong>
          {['Income', 'Costs applied', 'Event incoming', 'My Alarms'].map((category) => (
            <label key={category} style={styles.checkbox}>
              <input
                type="checkbox"
                checked={draftPopupCategories.includes(category)}
                onChange={(event) => {
                  const next = event.target.checked
                    ? [...draftPopupCategories, category]
                    : draftPopupCategories.filter((entry) => entry !== category);
                  setDraftPopupCategories(next);
                }}
              />
              {category}
            </label>
          ))}
        </div>
        <div style={styles.footer}>
          <button style={styles.cancelBtn} onClick={onClose} disabled={loading}>
            Cancel
          </button>
          <button style={styles.cancelBtn} onClick={() => void handleSaveSettings()} disabled={loading}>
            Save Changes
          </button>
        </div>
      </div>
    </div>
  );
};

export default ConfigModal;

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'var(--modal-overlay-dark)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  modal: {
    backgroundColor: 'var(--surface-background)',
    color: 'var(--primary-text)',
    borderRadius: '8px',
    border: '1px solid var(--surface-border)',
    width: '90%',
    maxWidth: '520px',
    padding: '1.5rem',
    boxShadow: '0 20px 25px -5px var(--modal-overlay-dark)',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    borderBottom: '1px solid var(--surface-border)',
    paddingBottom: '0.75rem',
    marginBottom: '1rem',
  },
  closeBtn: {
    background: 'none',
    border: 'none',
    color: 'var(--muted-text)',
    fontSize: '1.2rem',
    cursor: 'pointer',
  },
  body: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
  },
  label: {
    color: 'var(--primary-text)',
    fontSize: '0.95rem',
  },
  inputGroup: {
    display: 'flex',
    gap: '0.5rem',
  },
  input: {
    flex: 1,
    backgroundColor: 'var(--app-background)',
    border: '1px solid var(--surface-border)',
    borderRadius: '4px',
    color: 'var(--primary-text)',
    padding: '0.6rem 0.8rem',
    fontSize: '0.95rem',
    outline: 'none',
  },
  browseBtn: {
    backgroundColor: 'var(--control-background)',
    color: 'var(--white-text)',
    border: '1px solid var(--control-border)',
    borderRadius: '4px',
    padding: '0.6rem 0.9rem',
    cursor: 'pointer',
    fontWeight: 'bold',
    fontSize: '0.85rem',
    whiteSpace: 'nowrap',
  },
  hint: {
    fontSize: '0.8rem',
    color: 'var(--muted-text)',
    marginTop: '0.25rem',
    marginBottom: 0,
  },
  checkbox: { display: 'flex', gap: '0.5rem', alignItems: 'center', color: 'var(--secondary-text)' },
  footer: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '0.75rem',
    marginTop: '1.5rem',
    paddingTop: '0.75rem',
    borderTop: '1px solid var(--surface-border)',
  },
  cancelBtn: {
    backgroundColor: 'var(--control-background)',
    color: 'var(--white-text)',
    border: 'none',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
  saveBtn: {
    backgroundColor: 'var(--primary-accent)',
    color: 'var(--white-text)',
    border: 'none',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
};
