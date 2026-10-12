import React, { useEffect, useState } from 'react';
import { loadDatasetText } from '../services/tauriApi';

interface IntroPluginProps {
  onClose: () => void;
  datasetPath: string;
  moneyAmount: number;
}

type Scene = 'requirements' | 'story-1' | 'story-2';

export const IntroPlugin: React.FC<IntroPluginProps> = ({ onClose, datasetPath, moneyAmount }) => {
  const isGtr2Career = datasetPath.replaceAll('\\', '/').endsWith('/gtr2career')
    || datasetPath.replaceAll('\\', '/') === 'gtr2career';
  const [scene, setScene] = useState<Scene>(isGtr2Career ? 'requirements' : 'story-1');
  const [countdown, setCountdown] = useState(3);
  const [requirementsText, setRequirementsText] = useState<string | null>(null);
  const [requirementsError, setRequirementsError] = useState<string | null>(null);

  useEffect(() => {
    if (isGtr2Career) return undefined;
    const timer = window.setInterval(() => {
      setCountdown((current) => {
        if (current <= 1) {
          window.clearInterval(timer);
          onClose();
          return 0;
        }
        return current - 1;
      });
    }, 1000);
    return () => window.clearInterval(timer);
  }, [isGtr2Career, onClose]);

  useEffect(() => {
    if (!isGtr2Career) return undefined;
    void loadDatasetText('requirements.txt')
      .then(setRequirementsText)
      .catch((error: unknown) => {
        setRequirementsError(error instanceof Error ? error.message : String(error));
      });
    return () => undefined;
  }, [datasetPath, isGtr2Career]);

  useEffect(() => {
    const closeOnKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape' || event.key === 'Enter') {
        event.preventDefault();
        if (scene === 'requirements' || scene === 'story-2') onClose();
        else setScene(scene === 'story-1' ? 'story-2' : 'story-1');
      }
    };
    window.addEventListener('keydown', closeOnKey);
    return () => {
      window.removeEventListener('keydown', closeOnKey);
    };
  }, [onClose, scene]);

  if (!isGtr2Career) {
    return <div style={styles.overlay} onClick={onClose}><div style={styles.countdown}>{countdown || ''}</div></div>;
  }

  const continueScene = () => {
    if (scene === 'requirements') setScene('story-1');
    else if (scene === 'story-1') setScene('story-2');
    else onClose();
  };
  const image = scene === 'story-1' ? '/img/gtr2_career_1.jpeg' : '/img/gtr2_career_2.jpeg';
  const text = scene === 'story-1'
    ? `Walking away from family expectations, the journey starts here. Armed with £${moneyAmount.toFixed(0)} and a key to a modest flat, the road ahead is uncertain, but the destination is clear.`
    : 'Inside, the focus shifts straight to the plan. Scanning used car listings, balancing a tight budget, and lining up work—every step toward the starting line begins in this room.';

  return (
    <div key={scene} style={{ ...styles.overlay, ...(scene === 'requirements' ? {} : { animation: 'introFade 900ms ease both' }) }}>
      <style>{'@keyframes introFade { from { opacity: 0; } to { opacity: 1; } }'}</style>
      {scene === 'requirements' ? (
        <div style={styles.card}>
          <h1>GTR2 CAREER</h1>
          <p>Files required to reproduce the GTR2 races:</p>
          {requirementsError ? (
            <p style={styles.error}>Unable to read requirements.txt: {requirementsError}</p>
          ) : (
            <pre style={styles.requirementsText}>{requirementsText ?? 'Loading requirements.txt…'}</pre>
          )}
        </div>
      ) : (
        <>
          <div style={styles.storyImageFrame}>
            <img src={image} alt="" style={styles.storyImage} />
          </div>
          <div style={styles.storyShade} />
          <div style={styles.story}><p>{text}</p></div>
        </>
      )}
      <button type="button" style={styles.continue} onClick={continueScene}>
        {scene === 'story-2' ? 'Continue' : scene === 'requirements' ? 'Start' : 'Continue'}
      </button>
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
  countdown: { fontSize: 'min(24vw, 16rem)', fontWeight: 700, lineHeight: 1 },
  card: { width: 'min(620px, 88vw)', padding: '2rem', background: 'rgba(15, 23, 42, .94)', border: '1px solid #64748b', borderRadius: 12 },
  requirementsText: {
    maxHeight: '55vh',
    overflow: 'auto',
    margin: 0,
    whiteSpace: 'pre-wrap',
    font: 'inherit',
    lineHeight: 1.45,
  },
  error: { color: '#fca5a5' },
  story: { position: 'relative', zIndex: 1, width: 'min(760px, 88vw)', marginTop: 'auto', padding: '2rem', fontSize: 'clamp(1.1rem, 2vw, 1.6rem)', lineHeight: 1.5, textShadow: '0 2px 8px #000' },
  continue: {
    position: 'absolute',
    zIndex: 2,
    bottom: '2rem',
    padding: '0.7rem 1.5rem',
    border: '1px solid #777',
    background: '#111',
    color: '#fff',
    cursor: 'pointer',
  },
};
