import React from 'react';

interface ConfirmModalProps {
  title: string;
  message: string;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmModal: React.FC<ConfirmModalProps> = ({
  title,
  message,
  confirmLabel,
  onConfirm,
  onCancel,
}) => (
  <div style={styles.overlay} onClick={onCancel}>
    <div style={styles.modal} onClick={(event) => event.stopPropagation()}>
      <h2>{title}</h2>
      <p>{message}</p>
      <div style={styles.actions}>
        <button onClick={onCancel}>Cancel</button>
        <button onClick={onConfirm}>{confirmLabel}</button>
      </div>
    </div>
  </div>
);

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 2200, display: 'grid', placeItems: 'center',
    padding: '1rem', background: 'var(--modal-overlay)',
  },
  modal: {
    width: 'min(440px, 94vw)', padding: '1.5rem', background: 'var(--surface-background)',
    border: '1px solid var(--control-border)', borderRadius: '10px', color: 'var(--primary-text)',
  },
  actions: {
    display: 'flex', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '1.25rem',
  },
};

export default ConfirmModal;
