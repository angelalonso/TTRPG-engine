import React, { useEffect } from 'react';

export interface TutorialHintData {
  title: string;
  message: string;
  showRequirementsButton?: boolean;
}

interface TutorialHintProps {
  hint: TutorialHintData;
  onContinue: () => void;
  onSkipHints: () => void;
  onOpenRequirements?: () => void;
}

export const TutorialHint: React.FC<TutorialHintProps> = ({
  hint,
  onContinue,
  onSkipHints,
  onOpenRequirements,
}) => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onContinue();
      } else if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        onContinue();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onContinue]);

  return (
    <div style={styles.overlay}>
      <div style={styles.modal} role="dialog" aria-modal="true" aria-labelledby="tutorial-hint-title">
        <h2 id="tutorial-hint-title" style={styles.title}>{hint.title}</h2>
        <p style={styles.message}>{hint.message}</p>
        <div style={styles.actions}>
          <button type="button" style={styles.skip} onClick={onSkipHints}>
            Skip hints
          </button>
          {onOpenRequirements && (
            <button type="button" style={styles.secondary} onClick={onOpenRequirements}>
              GTR2 requirements
            </button>
          )}
          <button type="button" style={styles.continue} onClick={onContinue} autoFocus>
            Continue
          </button>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    inset: 0,
    zIndex: 6000,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '1rem',
    backgroundColor: 'var(--modal-overlay)',
    backdropFilter: 'blur(4px)',
  },
  modal: {
    width: 'min(560px, 94vw)',
    padding: '1.5rem',
    backgroundColor: 'var(--surface-background)',
    color: 'var(--primary-text)',
    border: '1px solid var(--info-border)',
    borderRadius: '10px',
    boxShadow: '0 20px 45px var(--modal-overlay-dark)',
  },
  title: {
    margin: '0 0 1rem',
    color: 'var(--primary-text)',
  },
  message: {
    margin: 0,
    color: 'var(--subtle-text)',
    lineHeight: 1.5,
    whiteSpace: 'pre-line',
  },
  actions: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '0.75rem',
    marginTop: '1.5rem',
  },
  skip: {
    padding: '0.65rem 1rem',
    border: '1px solid var(--control-border)',
    borderRadius: '6px',
    backgroundColor: 'transparent',
    color: 'var(--muted-text)',
    cursor: 'pointer',
  },
  continue: {
    padding: '0.65rem 1.1rem',
    border: 'none',
    borderRadius: '6px',
    backgroundColor: 'var(--primary-accent)',
    color: 'var(--white-text)',
    fontWeight: 700,
    cursor: 'pointer',
  },
  secondary: {
    padding: '0.65rem 1rem',
    border: '1px solid var(--control-border)',
    borderRadius: '6px',
    backgroundColor: 'transparent',
    color: 'var(--primary-text)',
    cursor: 'pointer',
  },
};
