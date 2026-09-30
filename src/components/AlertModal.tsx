import React, { useEffect } from 'react';
import type { GameAlert, GameState } from '../types/game';
import { dismissAlert } from '../services/tauriApi';

interface AlertModalProps {
  alerts: GameAlert[];
  onDismiss: (updatedState: GameState) => void;
  onConfigure: () => void;
}

export const AlertModal: React.FC<AlertModalProps> = ({ alerts, onDismiss, onConfigure }) => {
  const currentAlert = alerts?.[0];

  const handleDismiss = async () => {
    if (!currentAlert) return;
    try {
      const newState = await dismissAlert(currentAlert.id);
      onDismiss(newState);
    } catch (err) {
      console.error('Failed to dismiss alert:', err);
    }
  };

  useEffect(() => {
    if (!currentAlert) return undefined;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' || event.key === 'Enter') {
        event.preventDefault();
        void handleDismiss();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [currentAlert]);

  if (!currentAlert) return null;

  return (
    <div style={styles.overlay}>
      <div style={styles.modal}>
        <div style={styles.header}>
          <h3 style={styles.title}>{currentAlert.title}</h3>
          {alerts.length > 1 && (
            <span style={styles.badge}>{alerts.length} Pending</span>
          )}
        </div>
        <p style={styles.message}>{currentAlert.message}</p>
        <div style={styles.footer}>
          <span style={styles.pauseNote}>⏸️ Time automatically paused</span>
          <div style={styles.actions}>
            <button style={styles.configureButton} onClick={onConfigure}>
              Configure popups
            </button>
            <button style={styles.button} onClick={handleDismiss}>
              Acknowledge & Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AlertModal;

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'var(--modal-overlay)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 9999,
    backdropFilter: 'blur(4px)',
  },
  modal: {
    backgroundColor: 'var(--surface-background)',
    border: '2px solid var(--info-border)',
    borderRadius: '12px',
    padding: '1.5rem',
    maxWidth: '480px',
    width: '90%',
    boxShadow: '0 20px 25px -5px var(--modal-overlay-dark)',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1rem',
  },
  title: {
    margin: 0,
    color: 'var(--primary-text)',
    fontSize: '1.25rem',
  },
  badge: {
    backgroundColor: 'var(--info-border)',
    color: 'var(--white-text)',
    fontSize: '0.75rem',
    padding: '0.2rem 0.5rem',
    borderRadius: '12px',
    fontWeight: 'bold',
  },
  message: {
    color: 'var(--subtle-text)',
    fontSize: '1rem',
    lineHeight: '1.5',
    marginBottom: '1.5rem',
  },
  footer: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: '0.75rem',
    flexWrap: 'wrap',
  },
  actions: { display: 'flex', gap: '0.5rem', flexWrap: 'wrap', justifyContent: 'flex-end' },
  pauseNote: {
    fontSize: '0.8rem',
    color: 'var(--attention-text)',
  },
  button: {
    backgroundColor: 'var(--primary-accent)',
    color: 'var(--white-text)',
    border: 'none',
    padding: '0.6rem 1.2rem',
    borderRadius: '6px',
    fontWeight: 'bold',
    cursor: 'pointer',
  },
  configureButton: {
    backgroundColor: 'var(--warning-background)',
    color: 'var(--warning-text)',
    border: '1px solid var(--warning-text)',
    padding: '0.35rem 0.65rem',
    borderRadius: '6px',
    fontSize: '0.75rem',
    fontWeight: 600,
    cursor: 'pointer',
  },
};
