import React, { useEffect, useState } from 'react';
import { loadDatasetText, openExternalUrl } from '../services/tauriApi';

interface IntroPluginProps {
  onClose: () => void;
  datasetPath: string;
  moneyAmount: number;
}

type Scene = 'welcome' | 'requirements' | 'disclaimers' | 'story-1' | 'story-2';

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

export const IntroPlugin: React.FC<IntroPluginProps> = ({ onClose, datasetPath, moneyAmount }) => {
  const [scene, setScene] = useState<Scene>('welcome');
  const [requirementsText, setRequirementsText] = useState<string | null>(null);
  const [requirementsError, setRequirementsError] = useState<string | null>(null);

  useEffect(() => {
    if (scene !== 'requirements') return undefined;
    setRequirementsText(null);
    setRequirementsError(null);
    void loadDatasetText('requirements.txt')
      .then(setRequirementsText)
      .catch((error: unknown) => {
        setRequirementsError(error instanceof Error ? error.message : String(error));
      });
    return () => undefined;
  }, [datasetPath, scene]);

  useEffect(() => {
    const closeOnKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape' || event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        if (event.key === 'Escape') onClose();
        else if (scene === 'welcome') setScene('story-1');
        else if (scene === 'requirements') setScene('story-1');
        else if (scene === 'disclaimers' || scene === 'story-2') onClose();
        else setScene('story-2');
      }
    };
    window.addEventListener('keydown', closeOnKey);
    return () => {
      window.removeEventListener('keydown', closeOnKey);
    };
  }, [onClose, scene]);

  const continueScene = () => {
    if (scene === 'welcome' || scene === 'requirements') setScene('story-1');
    else if (scene === 'story-1') setScene('story-2');
    else onClose();
  };
  const image = scene === 'story-1' ? '/img/gtr2_career_1.jpeg' : '/img/gtr2_career_2.jpeg';
  const text = scene === 'story-1'
    ? `Walking away from family expectations, the journey starts here. Armed with £${moneyAmount.toFixed(0)} and a key to a modest flat, the road ahead is uncertain, but the destination is clear.`
    : 'Inside, the focus shifts straight to the plan. Scanning used car listings, balancing a tight budget, and lining up work—every step toward the starting line begins in this room.';

  return (
    <div style={styles.overlay}>
      {scene === 'welcome' ? (
        <div style={styles.card}>
          <h1>Welcome</h1>
          <p style={styles.welcomeText}>
            This game is a Proof of concept on how to simulate a progression race driver career.
          </p>
          <p style={styles.welcomeText}>
            To get the best experience, you should &quot;play the game&quot; here, then move on to your sim of choice for races, then come back to add the results and continue.
          </p>
          <p style={styles.welcomeText}>
            Of course this mean you can cheat but what&apos;s the point?
          </p>
          <p style={styles.welcomeText}>
            I used GTR2 because I found the mods fit what this game is about, but feel free to use other sims.
          </p>
          <div style={styles.welcomeActions}>
            <button type="button" style={styles.secondaryButton} onClick={() => setScene('requirements')}>
              List of GTR2 mods
            </button>
            <button type="button" style={styles.secondaryButton} onClick={() => setScene('disclaimers')}>
              Disclaimers
            </button>
            <button type="button" style={styles.continue} onClick={continueScene}>
              Continue
            </button>
            <button type="button" style={styles.skip} onClick={onClose}>
              Skip
            </button>
          </div>
        </div>
      ) : scene === 'requirements' ? (
        <div style={styles.card}>
          <h1>GTR2 CAREER</h1>
          <p>Files required to reproduce the GTR2 races:</p>
          {requirementsError ? (
            <p style={styles.error}>Unable to read requirements.txt: {requirementsError}</p>
          ) : (
            <pre style={styles.requirementsText}>{renderLinkedText(requirementsText ?? 'Loading requirements.txt...')}</pre>
          )}
          <div style={styles.actions}>
            <button type="button" style={styles.continue} onClick={() => setScene('welcome')}>Back</button>
          </div>
        </div>
      ) : scene === 'disclaimers' ? (
        <div style={styles.card}>
          <h1>Disclaimers</h1>
          <div style={styles.emptyDisclaimers} />
          <div style={styles.actions}>
            <button type="button" style={styles.continue} onClick={() => setScene('welcome')}>Back</button>
          </div>
        </div>
      ) : (
        <>
          <div style={styles.storyImageFrame}>
            <img src={image} alt="" style={styles.storyImage} />
          </div>
          <div style={styles.storyShade} />
          <div style={styles.story}><p>{renderLinkedText(text)}</p></div>
        </>
      )}
      {(scene === 'story-1' || scene === 'story-2') && (
        <div style={styles.actions}>
          <button type="button" style={styles.skip} onClick={onClose}>Skip</button>
          <button type="button" style={styles.continue} onClick={continueScene}>
            Continue
          </button>
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    inset: 0,
    zIndex: 5000,
    display: 'grid',
    placeItems: 'center',
    background: '#000',
    color: '#fff',
  },
  storyImageFrame: {
    position: 'absolute',
    inset: 0,
    display: 'grid',
    placeItems: 'center',
  },
  storyImage: {
    width: 'auto',
    height: 'auto',
    maxWidth: '100%',
    maxHeight: '100%',
    margin: 'auto',
  },
  storyShade: {
    position: 'absolute',
    inset: 0,
    background: 'linear-gradient(0deg, rgba(0,0,0,.82), rgba(0,0,0,.12))',
    pointerEvents: 'none',
  },
  card: { width: 'min(1080px, 94vw)', padding: '2rem', background: 'rgba(15, 23, 42, .94)', border: '1px solid #64748b', borderRadius: 12 },
  welcomeText: { maxWidth: '72ch', margin: '1rem 0', lineHeight: 1.5 },
  welcomeActions: { display: 'flex', flexWrap: 'wrap', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '2rem' },
  secondaryButton: {
    padding: '0.7rem 1rem',
    border: '1px solid #777',
    background: '#111',
    color: '#fff',
    cursor: 'pointer',
  },
  emptyDisclaimers: { minHeight: '10rem' },
  requirementsText: {
    maxHeight: '55vh',
    overflow: 'auto',
    margin: 0,
    whiteSpace: 'pre-wrap',
    font: 'inherit',
    lineHeight: 1.45,
  },
  error: { color: '#fca5a5' },
  story: { position: 'absolute', zIndex: 1, bottom: '6rem', width: 'min(1080px, 94vw)', padding: '1rem 2rem', fontSize: 'clamp(1.1rem, 2vw, 1.6rem)', lineHeight: 1.5, textShadow: '0 2px 8px #000', boxSizing: 'border-box' },
  actions: {
    position: 'absolute',
    zIndex: 2,
    bottom: '2rem',
    display: 'flex',
    gap: '0.75rem',
  },
  continue: {
    padding: '0.7rem 1.5rem',
    border: '1px solid #777',
    background: '#111',
    color: '#fff',
    cursor: 'pointer',
  },
  skip: {
    padding: '0.7rem 1.5rem',
    border: '1px solid #777',
    background: 'transparent',
    color: '#fff',
    cursor: 'pointer',
  },
  link: { color: '#93c5fd', textDecoration: 'underline' },
};
