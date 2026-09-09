import React, { useEffect, useState } from 'react';
import { loadDescription } from '../services/tauriApi';

interface DetailModalProps {
  title: string;
  descriptionPath: string;
  onClose: () => void;
  message?: string;
  children: React.ReactNode;
}

export const DetailModal: React.FC<DetailModalProps> = ({
  title,
  descriptionPath,
  onClose,
  message,
  children,
}) => {
  const [html, setHtml] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    setHtml('');
    setError('');
    if (!descriptionPath) return;
    loadDescription(descriptionPath)
      .then(setHtml)
      .catch((reason) => setError(String(reason)));
  }, [descriptionPath]);

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(event) => event.stopPropagation()}>
        <header style={styles.header}>
          <h2 style={styles.title}>{title}</h2>
          <button style={styles.close} onClick={onClose} aria-label="Close">×</button>
        </header>
        <div style={styles.content}>
          {error && <p style={styles.error}>{error}</p>}
          {message && <p style={styles.error}>{message}</p>}
          {!descriptionPath && <p style={styles.empty}>No description has been configured for this entry.</p>}
          {descriptionPath && !html && !error && <p style={styles.empty}>Loading description...</p>}
          {html && (
            <iframe
              title={`${title} description`}
              srcDoc={html}
              sandbox=""
              style={styles.frame}
            />
          )}
        </div>
        <footer style={styles.footer}>{children}</footer>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 2000, display: 'flex',
    alignItems: 'center', justifyContent: 'center', padding: '1rem',
    background: 'rgba(2, 6, 23, 0.82)', backdropFilter: 'blur(4px)',
  },
  modal: {
    width: 'min(900px, 96vw)', height: 'min(720px, 92vh)', display: 'flex',
    flexDirection: 'column', background: '#1e293b', border: '1px solid #475569',
    borderRadius: '10px', boxShadow: '0 24px 50px rgba(0, 0, 0, 0.45)',
  },
  header: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    padding: '1rem 1.25rem', borderBottom: '1px solid #334155',
  },
  title: { margin: 0, fontSize: '1.25rem' },
  close: {
    border: 0, background: 'transparent', color: '#cbd5e1',
    fontSize: '1.8rem', lineHeight: 1, cursor: 'pointer',
  },
  content: { flex: 1, minHeight: 0, padding: '1rem', background: '#f8fafc' },
  frame: { width: '100%', height: '100%', border: 0, background: '#fff' },
  empty: { color: '#475569', margin: 0 },
  error: { color: '#b91c1c', margin: 0, whiteSpace: 'pre-wrap' },
  footer: {
    display: 'flex', flexWrap: 'wrap', gap: '0.75rem', justifyContent: 'flex-end',
    padding: '1rem 1.25rem', borderTop: '1px solid #334155',
  },
};

export default DetailModal;
