import React, { useEffect, useState } from 'react';
import { loadDescription } from '../services/tauriApi';

interface DetailModalProps {
  title: string;
  descriptionPath: string;
  onClose: () => void;
  message?: string;
  hideDescription?: boolean;
  children: React.ReactNode;
}

export const DetailModal: React.FC<DetailModalProps> = ({
  title,
  descriptionPath,
  onClose,
  message,
  hideDescription = false,
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

  const styledHtml = html.includes('</head>')
    ? html.replace('</head>', `${embeddedStyles()}</head>`)
    : `${embeddedStyles()}${html}`;

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
        <header style={styles.header}>
          <h2 style={styles.title}>{title}</h2>
          <button style={styles.close} onClick={onClose} aria-label="Close">×</button>
        </header>
        <div style={styles.content}>
          {error && <p style={styles.error}>{error}</p>}
          {message && <p style={styles.error}>{message}</p>}
          {!hideDescription && !descriptionPath && <p style={styles.empty}>No description has been configured for this entry.</p>}
          {!hideDescription && descriptionPath && !html && !error && <p style={styles.empty}>Loading description...</p>}
          {html && (
            <iframe
              title={`${title} description`}
              srcDoc={styledHtml}
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

const embeddedStyles = () => {
  const themeVariableNames = [
    'app-background', 'surface-background', 'primary-text', 'secondary-text', 'surface-border',
    'control-border', 'control-background', 'white-text', 'muted-text',
    'primary-accent', 'primary-accent-border',
  ];
  const themeVariables = themeVariableNames
    .map((name) => `--${name}: ${getComputedStyle(document.documentElement).getPropertyValue(`--${name}`).trim()};`)
    .join('');
  return `<style>
:root {
  color-scheme: dark;
  ${themeVariables}
  --gtr-black: #08090b;
  --gtr-panel: #111316;
  --gtr-panel-light: #1c2024;
  --gtr-line: #3b4045;
  --gtr-red: #e5232b;
  --gtr-white: #f3f4f5;
  --gtr-muted: #aeb4b9;
  --gtr-yellow: #fff59d;
}
@font-face {
  font-family: 'Rajdhani';
  src: url('/fonts/rajdhani-400.ttf') format('truetype');
  font-weight: 400;
}
@font-face {
  font-family: 'Rajdhani';
  src: url('/fonts/rajdhani-700.ttf') format('truetype');
  font-weight: 700;
}
* { box-sizing: border-box; }
body {
  position: relative; min-height: 100vh; margin: 0; padding: 2rem 1rem;
  background:
    radial-gradient(ellipse at 50% 40%, rgb(45 48 51 / 35%), transparent 62%),
    repeating-linear-gradient(135deg, rgb(255 255 255 / 3%) 0 2px, transparent 2px 8px),
    repeating-linear-gradient(45deg, rgb(0 0 0 / 28%) 0 3px, transparent 3px 8px),
    var(--gtr-black);
  color: var(--gtr-white); font: 16px/1.45 'Rajdhani', 'Arial Narrow', sans-serif;
  letter-spacing: .025em;
}
body::before {
  position: absolute; top: .8rem; right: 0; left: 0; height: 2px;
  background: var(--gtr-red); box-shadow: 0 5px 0 rgb(229 35 43 / 45%); content: '';
}
h1, h2, h3, h4, dt, th, strong {
  color: var(--gtr-white); font-family: 'Rajdhani', 'Arial Narrow', sans-serif;
  font-weight: 700; letter-spacing: .05em; text-transform: uppercase;
}
h1 { margin-top: 0; font-size: 1.7rem; }
h2 { margin-top: 1.35rem; border-bottom: 1px solid var(--gtr-red); padding-bottom: .35rem; font-size: 1.1rem; }
p, ul, ol, dl { max-width: 72ch; }
a, a:visited { color: var(--gtr-yellow); }
button, .button, a.button {
  display: inline-block; border: 1px solid #737b82; border-radius: 0;
  background: linear-gradient(#272c31, #15181b); color: var(--gtr-white);
  padding: .55rem 1rem; font: 600 1rem 'Rajdhani', 'Arial Narrow', sans-serif;
  letter-spacing: .055em; text-transform: uppercase; cursor: pointer; text-decoration: none;
}
button:hover, .button:hover, a.button:hover {
  border-color: #ff5960; background: linear-gradient(#3a2023, #221417);
}
button:disabled, .button:disabled { cursor: not-allowed; opacity: .55; }
select, input, textarea {
  border: 1px solid #596168; border-radius: 0; background: rgb(8 10 12 / 88%);
  color: var(--gtr-white); padding: .55rem .7rem; font: inherit;
}
.tag, .pill {
  display: inline-block; border: 1px solid #737b82; border-radius: 0;
  background: var(--gtr-panel-light); color: var(--gtr-white); padding: .15rem .55rem; font-size: .85rem;
}
</style>`;
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 2000, display: 'flex',
    alignItems: 'center', justifyContent: 'center', padding: '1rem',
    background: 'var(--modal-overlay)', backdropFilter: 'blur(4px)',
  },
  modal: {
    width: 'min(900px, 96vw)', height: 'min(720px, 92vh)', display: 'flex',
    flexDirection: 'column', background: 'var(--surface-background)', border: '1px solid var(--control-border)',
    borderRadius: '10px', boxShadow: '0 24px 50px var(--modal-overlay-dark)',
  },
  header: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    padding: '1rem 1.25rem', borderBottom: '1px solid var(--surface-border)',
  },
  title: { margin: 0, fontSize: '1.25rem' },
  close: {
    border: 0, background: 'transparent', color: 'var(--subtle-text)',
    fontSize: '1.8rem', lineHeight: 1, cursor: 'pointer',
  },
  content: { flex: 1, minHeight: 0, padding: '1rem', background: 'var(--surface-background)' },
  frame: { width: '100%', height: '100%', border: 0, background: 'var(--surface-background)' },
  empty: { color: 'var(--control-border)', margin: 0 },
  error: { color: 'var(--danger-text)', margin: 0, whiteSpace: 'pre-wrap' },
  footer: {
    display: 'flex', flexWrap: 'wrap', gap: '0.75rem', justifyContent: 'flex-end',
    padding: '1rem 1.25rem', borderTop: '1px solid var(--surface-border)',
  },
};

export default DetailModal;
