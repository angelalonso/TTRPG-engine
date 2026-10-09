import React, { useState } from 'react';
import type { SaveSlot } from '../services/tauriApi';

interface SaveSlotsModalProps {
  mode: 'save' | 'load';
  slots: SaveSlot[];
  onSave: (slot: string) => Promise<void>;
  onLoad: (slot: string) => Promise<void>;
  onMigrate: (slot: string) => Promise<void>;
  onLoadAtOwnRisk: (slot: string) => Promise<void>;
  onDelete: (slot: string) => Promise<void>;
  onClose: () => void;
}

export const SaveSlotsModal: React.FC<SaveSlotsModalProps> = ({
  mode, slots, onSave, onLoad, onMigrate, onLoadAtOwnRisk, onClose,
  onDelete,
}) => {
  const [slot, setSlot] = useState('');
  const [deleteSlot, setDeleteSlot] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  const run = async (operation: () => Promise<void>) => {
    setBusy(true);
    setError('');
    try {
      await operation();
    } catch (caught) {
      setError(String(caught));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} role="dialog" aria-modal="true" onClick={(event) => event.stopPropagation()}>
        <h2>{mode === 'save' ? 'Save game' : 'Load saved game'}</h2>
        {mode === 'save' && (
          <div style={styles.saveRow}>
            <input
              autoFocus
              value={slot}
              onChange={(event) => setSlot(event.target.value)}
              placeholder="Save name, for example season-1"
              disabled={busy}
            />
            <button
              disabled={!slot.trim() || busy}
              onClick={() => void run(() => onSave(slot.trim()))}
            >
              Save
            </button>
          </div>
        )}
        <p style={styles.label}>{slots.length ? 'Existing save slots' : 'No save slots yet.'}</p>
        <div style={styles.slots} role="table" aria-label="Saved games">
          {slots.map((save) => (
            <div key={save.name} style={styles.slotRow} role="row">
              <button
                style={{ ...styles.cellButton, ...styles.slotButton }}
                disabled={busy}
                onClick={() => void run(() => mode === 'save' ? onSave(save.name) : onLoad(save.name))}
                role="cell"
              >
                {mode === 'save' ? `Overwrite ${save.name}` : save.name}
              </button>
              {mode === 'load' && (
                deleteSlot === save.name ? (
                  <>
                    <button
                      style={{ ...styles.cellButton, ...styles.deleteConfirmButton }}
                      disabled={busy}
                      onClick={() => void run(async () => {
                        await onDelete(save.name);
                        setDeleteSlot(null);
                      })}
                      aria-label={`Confirm deleting ${save.name}`}
                      role="cell"
                    >
                      Yes
                    </button>
                    <button
                      style={styles.cellButton}
                      disabled={busy}
                      onClick={() => setDeleteSlot(null)}
                      aria-label={`Cancel deleting ${save.name}`}
                      role="cell"
                    >
                      No
                    </button>
                  </>
                ) : (
                  <>
                    {save.can_migrate && (
                      <button
                        style={{ ...styles.cellButton, ...styles.migrateButton }}
                        disabled={busy}
                        onClick={() => void run(() => onMigrate(save.name))}
                        title={`Apply ${save.migration_steps} dataset migration${save.migration_steps === 1 ? '' : 's'} before loading`}
                        role="cell"
                      >
                        Migrate
                      </button>
                    )}
                    {save.can_load_at_own_risk && (
                      <button
                        style={{ ...styles.cellButton, ...styles.riskButton }}
                        disabled={busy}
                        onClick={() => void run(() => onLoadAtOwnRisk(save.name))}
                        title="Load this save without checking its dataset fingerprint. Future results may differ."
                        role="cell"
                      >
                        Just try
                      </button>
                    )}
                    <button
                      style={{ ...styles.cellButton, ...styles.deleteButton }}
                      disabled={busy}
                      onClick={() => setDeleteSlot(save.name)}
                      role="cell"
                    >
                      Delete
                    </button>
                  </>
                )
              )}
            </div>
          ))}
        </div>
        {error && <p role="alert" style={styles.error}>{error}</p>}
        <div style={styles.footer}>
          <button onClick={onClose} disabled={busy}>Cancel</button>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: { position: 'fixed', inset: 0, zIndex: 2200, display: 'grid', placeItems: 'center', padding: '1rem', background: 'var(--modal-overlay)' },
  modal: { width: 'min(600px, 94vw)', padding: '1.5rem', background: 'var(--surface-background)', border: '1px solid var(--control-border)', borderRadius: '10px', color: 'var(--primary-text)' },
  saveRow: { display: 'flex', gap: '0.5rem' },
  label: { color: 'var(--subtle-text)' },
  slots: { display: 'grid', gap: '0.35rem', maxHeight: '16rem', overflowY: 'auto' },
  slotRow: { display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) auto auto auto', gap: '0.35rem', alignItems: 'stretch' },
  cellButton: { minHeight: '2.5rem', padding: '0.55rem 0.75rem', border: '1px solid var(--control-border)', borderRadius: '4px', cursor: 'pointer', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' },
  slotButton: { width: '100%', textAlign: 'left' },
  migrateButton: { borderColor: 'var(--link-text)', color: 'var(--link-text)' },
  riskButton: { borderColor: 'var(--warning-text)', color: 'var(--warning-text)' },
  deleteButton: { borderColor: 'var(--error-text)', color: 'var(--error-text)' },
  deleteConfirmButton: { borderColor: 'var(--error-text)', color: 'var(--error-text)' },
  error: { color: 'var(--error-text)' },
  footer: { display: 'flex', justifyContent: 'flex-end', marginTop: '1rem' },
};

export default SaveSlotsModal;
