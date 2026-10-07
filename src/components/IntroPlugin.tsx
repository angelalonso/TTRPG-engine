import React, { useEffect, useState } from 'react';

interface IntroPluginProps {
  onClose: () => void;
}

export const IntroPlugin: React.FC<IntroPluginProps> = ({ onClose }) => {
  const [countdown, setCountdown] = useState(3);

  useEffect(() => {
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
    const closeOnKey = (event: KeyboardEvent) => {
      if (['Escape', ' ', 'Enter'].includes(event.key)) {
        event.preventDefault();
        onClose();
      }
    };
    window.addEventListener('keydown', closeOnKey);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener('keydown', closeOnKey);
    };
  }, [onClose]);

  return (
    <div style={styles.overlay}>
      <div style={styles.countdown}>{countdown > 0 ? countdown : ''}</div>
      <button type="button" style={styles.skip} onClick={onClose}>Skip</button>
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
  countdown: { fontSize: 'min(24vw, 16rem)', fontWeight: 700, lineHeight: 1 },
  skip: {
    position: 'absolute',
    bottom: '2rem',
    padding: '0.7rem 1.5rem',
    border: '1px solid #777',
    background: '#111',
    color: '#fff',
    cursor: 'pointer',
  },
};
