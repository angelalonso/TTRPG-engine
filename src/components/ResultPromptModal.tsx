import React, { useState } from 'react';

interface ResultPromptModalProps {
  eventName: string;
  damageOptions: Array<{ id: string; name: string }>;
  onSubmit: (result: string, damageType: string) => Promise<void>;
  onClose: () => void;
}

export const ResultPromptModal: React.FC<ResultPromptModalProps> = ({
  eventName,
  damageOptions,
  onSubmit,
  onClose,
}) => {
  const [result, setResult] = useState('');
  const [damageType, setDamageType] = useState('none');
  const [submitting, setSubmitting] = useState(false);

  const submit = async () => {
    if (!result.trim() || submitting) return;
    setSubmitting(true);
    try {
      await onSubmit(result.trim(), damageType);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(event) => event.stopPropagation()}>
        <h2>Enter result</h2>
        <p>How did {eventName} finish?</p>
        <input
          autoFocus
          value={result}
          onChange={(event) => setResult(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') void submit();
          }}
          placeholder="For example: success, 2nd place, or retired"
        />
        {damageOptions.length > 0 && (
          <label>
            Damage from this event
            <select value={damageType} onChange={(event) => setDamageType(event.target.value)}>
              <option value="none">No additional damage</option>
              {damageOptions.map((option) => (
                <option key={option.id} value={option.id}>{option.name}</option>
              ))}
            </select>
          </label>
        )}
        <div style={styles.actions}>
          <button onClick={onClose} disabled={submitting}>Cancel</button>
          <button onClick={() => void submit()} disabled={!result.trim() || submitting}>
            Record result
          </button>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 2100, display: 'grid', placeItems: 'center',
    padding: '1rem', background: 'rgba(2, 6, 23, 0.82)',
  },
  modal: {
    width: 'min(520px, 94vw)', padding: '1.5rem', background: '#1e293b',
    border: '1px solid #475569', borderRadius: '10px', color: '#f8fafc',
  },
  actions: { display: 'flex', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '1rem' },
};

export default ResultPromptModal;
