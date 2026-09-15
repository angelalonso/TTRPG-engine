import React, { useEffect } from 'react';

interface EventLogEntry {
  id: string;
  day: number;
  event: string;
}

interface EventLogModalProps {
  entries: EventLogEntry[];
  formatDay: (day: number) => string;
  onClose: () => void;
}

export const EventLogModal: React.FC<EventLogModalProps> = ({ entries, formatDay, onClose }) => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} role="dialog" aria-modal="true" aria-labelledby="event-log-title" onClick={(event) => event.stopPropagation()}>
        <div style={styles.header}>
          <h2 id="event-log-title">Event Log</h2>
          <button style={styles.close} onClick={onClose} aria-label="Close event log">×</button>
        </div>
        <div style={styles.content}>
          {entries.length === 0 && <p>No events recorded yet.</p>}
          {entries.slice().reverse().map((entry) => (
            <div key={entry.id} style={styles.row}>
              <span style={styles.day}>{formatDay(entry.day)}</span>
              <span>{entry.event}</span>
            </div>
          ))}
        </div>
        <footer style={styles.footer}>
          <button onClick={onClose}>Close</button>
        </footer>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 2100, display: 'grid', placeItems: 'center',
    padding: '1rem', background: 'var(--modal-overlay)', backdropFilter: 'blur(4px)',
  },
  modal: {
    width: 'min(720px, 94vw)', maxHeight: '84vh', display: 'flex', flexDirection: 'column',
    background: 'var(--surface-background)', border: '1px solid var(--control-border)',
    borderRadius: '12px', color: 'var(--primary-text)', boxShadow: '0 24px 50px var(--modal-overlay-dark)',
  },
  header: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    padding: '1rem 1.25rem', borderBottom: '1px solid var(--surface-border)',
  },
  close: { border: 0, background: 'transparent', color: 'var(--subtle-text)', fontSize: '1.8rem', cursor: 'pointer' },
  content: { overflowY: 'auto', padding: '0.75rem 1.25rem' },
  row: {
    display: 'grid', gridTemplateColumns: 'minmax(120px, 0.35fr) 1fr', gap: '1rem',
    padding: '0.75rem 0', borderBottom: '1px solid var(--surface-border)',
  },
  day: { color: 'var(--subtle-text)', fontSize: '0.9rem' },
  footer: { display: 'flex', justifyContent: 'flex-end', padding: '1rem 1.25rem', borderTop: '1px solid var(--surface-border)' },
};

export default EventLogModal;
