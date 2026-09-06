import React, { useState } from 'react';

interface ConfigModalProps {
  isOpen: boolean;
  currentPath: string;
  onClose: () => void;
  onReloadDataset: (newPath: string) => Promise<void>;
}

export const ConfigModal: React.FC<ConfigModalProps> = ({
  isOpen,
  currentPath,
  onClose,
  onReloadDataset,
}) => {
  const [datasetPath, setDatasetPath] = useState(currentPath);
  const [loading, setLoading] = useState(false);

  if (!isOpen) return null;

  const handleSaveAndReload = async () => {
    setLoading(true);
    try {
      await onReloadDataset(datasetPath);
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
          <input
            type="text"
            value={datasetPath}
            onChange={(e) => setDatasetPath(e.target.value)}
            style={styles.input}
            placeholder="e.g. dataset or /path/to/dataset"
          />
          <p style={styles.hint}>
            Path containing <code>cars.csv</code>, <code>actions.csv</code>, and <code>races.csv</code>.
          </p>
        </div>
        <div style={styles.footer}>
          <button style={styles.cancelBtn} onClick={onClose} disabled={loading}>
            Cancel
          </button>
          <button style={styles.saveBtn} onClick={handleSaveAndReload} disabled={loading}>
            {loading ? 'Reloading...' : 'Load Dataset'}
          </button>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.75)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  modal: {
    backgroundColor: '#1e293b',
    color: '#f8fafc',
    borderRadius: '8px',
    border: '1px solid #334155',
    width: '90%',
    maxWidth: '500px',
    padding: '1.5rem',
    boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.5)',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    borderBottom: '1px solid #334155',
    paddingBottom: '0.75rem',
    marginBottom: '1rem',
  },
  closeBtn: {
    background: 'none',
    border: 'none',
    color: '#94a3b8',
    fontSize: '1.2rem',
    cursor: 'pointer',
  },
  body: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
  },
  label: {
    color: '#f8fafc',
    fontSize: '0.95rem',
  },
  input: {
    backgroundColor: '#0f172a',
    border: '1px solid #334155',
    borderRadius: '4px',
    color: '#f8fafc',
    padding: '0.6rem 0.8rem',
    fontSize: '0.95rem',
    outline: 'none',
  },
  hint: {
    fontSize: '0.8rem',
    color: '#94a3b8',
    marginTop: '0.25rem',
    marginBottom: 0,
  },
  footer: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '0.75rem',
    marginTop: '1.5rem',
    paddingTop: '0.75rem',
    borderTop: '1px solid #334155',
  },
  cancelBtn: {
    backgroundColor: '#334155',
    color: '#ffffff',
    border: 'none',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
  saveBtn: {
    backgroundColor: '#2563eb',
    color: '#ffffff',
    border: 'none',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 'bold',
  },
};
