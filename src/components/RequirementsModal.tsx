import React, { useEffect, useState } from 'react';
import { loadDatasetText, openExternalUrl } from '../services/tauriApi';

interface RequirementsModalProps {
  onClose: () => void;
}

const URL_PATTERN = /(https?:\/\/[^\s<]+)/g;

function renderLinkedText(text: string): React.ReactNode[] {
  return text.split(URL_PATTERN).map((part, index) => {
    if (!/^https?:\/\//i.test(part)) return <React.Fragment key={index}>{part}</React.Fragment>;
    const href = part.replace(/[),.;!?]+$/, '');
    const trailing = part.slice(href.length);
    return (
      <React.Fragment key={index}>
        <a
          href={href}
          target="_blank"
          rel="noreferrer"
          onClick={(event) => {
            event.preventDefault();
            event.stopPropagation();
            void openExternalUrl(href);
          }}
          style={styles.link}
        >
          {href}
        </a>
        {trailing}
      </React.Fragment>
    );
  });
}

export const RequirementsModal: React.FC<RequirementsModalProps> = ({ onClose }) => {
  const [text, setText] = useState<string | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    void loadDatasetText('requirements.txt')
      .then(setText)
      .catch((caught: unknown) => setError(caught instanceof Error ? caught.message : String(caught)));
  }, []);

  return (
    <div style={styles.overlay}>
      <div style={styles.modal} role="dialog" aria-modal="true" aria-labelledby="requirements-title">
        <h1 id="requirements-title" style={styles.title}>GTR2 CAREER</h1>
        <p>Files required to reproduce the GTR2 races:</p>
        {error ? (
          <p style={styles.error}>Unable to read requirements.txt: {error}</p>
        ) : (
          <pre style={styles.requirementsText}>{renderLinkedText(text ?? 'Loading requirements.txt...')}</pre>
        )}
        <div style={styles.actions}>
          <button type="button" style={styles.button} onClick={onClose}>Back</button>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    inset: 0,
    zIndex: 7000,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '1rem',
    background: 'rgba(0, 0, 0, .82)',
  },
  modal: {
    width: 'min(1080px, 94vw)',
    maxHeight: '92vh',
    padding: '2rem',
    overflow: 'auto',
    background: 'rgba(15, 23, 42, .98)',
    border: '1px solid #64748b',
    borderRadius: 12,
    color: '#fff',
  },
  title: { marginTop: 0 },
  requirementsText: {
    maxHeight: '65vh',
    overflow: 'auto',
    margin: 0,
    whiteSpace: 'pre-wrap',
    font: 'inherit',
    lineHeight: 1.45,
  },
  error: { color: '#fca5a5' },
  actions: { display: 'flex', justifyContent: 'flex-end', marginTop: '1.5rem' },
  button: {
    padding: '0.7rem 1.5rem',
    border: '1px solid #777',
    background: '#111',
    color: '#fff',
    cursor: 'pointer',
  },
  link: { color: '#93c5fd', textDecoration: 'underline' },
};
