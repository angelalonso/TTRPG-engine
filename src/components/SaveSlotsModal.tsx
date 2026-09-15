import React, { useState } from 'react';
import type { SaveSlot } from '../services/tauriApi';

interface SaveSlotsModalProps {
  mode: 'save' | 'load';
  slots: SaveSlot[];
  onSave: (slot: string) => Promise<void>;
  onLoad: (slot: string) => Promise<void>;
  onClose: () => void;
}

export const SaveSlotsModal: React.FC<SaveSlotsModalProps> = ({
  mode, slots, onSave, onLoad, onClose,
}) => {
  const [slot, setSlot] = useState('');
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
        <div style={styles.slots}>
          {slots.map((save) => (
            <button
              key={save.name}
              style={styles.slot}
              disabled={busy}
              onClick={() => void run(() => mode === 'save' ? onSave(save.name) : onLoad(save.name))}
            >
              {mode === 'save' ? `Overwrite ${save.name}` : save.name}
            </button>
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
  modal: { width: 'min(520px, 94vw)', padding: '1.5rem', background: 'var(--surface-background)', border: '1px solid var(--control-border)', borderRadius: '10px', color: 'var(--primary-text)' },
  saveRow: { display: 'flex', gap: '0.5rem' },
  label: { color: 'var(--subtle-text)' },
  slots: { display: 'grid', gap: '0.5rem', maxHeight: '16rem', overflowY: 'auto' },
  slot: { padding: '0.75rem', textAlign: 'left', cursor: 'pointer' },
  error: { color: 'var(--error-text)' },
  footer: { display: 'flex', justifyContent: 'flex-end', marginTop: '1rem' },
};

export default SaveSlotsModal;
