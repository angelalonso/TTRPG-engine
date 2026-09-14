import React, { useEffect } from 'react';

interface FeedbackModalProps {
  title: string;
  message: string;
  onClose: () => void;
}

export const FeedbackModal: React.FC<FeedbackModalProps> = ({ title, message, onClose }) => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' || event.key === 'Enter') {
        event.preventDefault();
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(event) => event.stopPropagation()}>
        <h2>{title}</h2>
        <p>{message}</p>
        <button onClick={onClose}>Continue</button>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 2100, display: 'grid', placeItems: 'center',
    padding: '1rem', background: 'var(--modal-overlay)',
  },
  modal: {
    width: 'min(440px, 94vw)', padding: '1.5rem', background: 'var(--surface-background)',
    border: '1px solid var(--control-border)', borderRadius: '10px', color: 'var(--primary-text)',
  },
};

export default FeedbackModal;
