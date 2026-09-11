import React, { useState } from 'react';
import type { ChampionshipCompetitor } from '../types/game';

interface ResultPromptModalProps {
  eventName: string;
  damageOptions: Array<{ id: string; name: string }>;
  onSubmit: (result: string, damageType: string) => Promise<void>;
  championship?: boolean;
  onSubmitChampionship?: (
    result: string,
    damageType: string,
    playerPosition: number,
    competitors: ChampionshipCompetitor[],
  ) => Promise<void>;
  previousCompetitors?: ChampionshipCompetitor[];
  onClose: () => void;
}

export const ResultPromptModal: React.FC<ResultPromptModalProps> = ({
  eventName,
  damageOptions,
  onSubmit,
  onClose,
  championship = false,
  onSubmitChampionship,
  previousCompetitors = [],
}) => {
  const [result, setResult] = useState('');
  const [damageType, setDamageType] = useState('none');
  const [submitting, setSubmitting] = useState(false);
  const [playerPosition, setPlayerPosition] = useState('');
  const [competitors, setCompetitors] = useState<ChampionshipCompetitor[]>(previousCompetitors);

  const submit = async () => {
    if ((!championship && !result.trim()) || (championship && (!playerPosition || Number(playerPosition) < 1)) || submitting) return;
    setSubmitting(true);
    try {
      if (championship && onSubmitChampionship) {
        await onSubmitChampionship(
          'success',
          damageType,
          Number(playerPosition),
          competitors.filter((entry) => entry.name.trim() && entry.position > 0),
        );
      } else {
        await onSubmit(result.trim(), damageType);
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(event) => event.stopPropagation()}>
        <h2>Enter result</h2>
        {championship ? (
          <p>Your finishing position in {eventName}</p>
        ) : (
          <>
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
          </>
        )}
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
        {championship && (
          <div>
            <label>
              Your finishing position
              <input type="number" min="1" value={playerPosition}
                onChange={(event) => setPlayerPosition(event.target.value)} />
            </label>
            <p>Other point-scoring drivers</p>
            {competitors.map((competitor, index) => (
              <div key={index} style={styles.competitorRow}>
                <input list="championship-drivers" placeholder="Driver name" value={competitor.name}
                  onChange={(event) => setCompetitors((current) => current.map((entry, entryIndex) =>
                    entryIndex === index ? { ...entry, name: event.target.value } : entry))} />
                <input type="number" min="1" placeholder="Position" value={competitor.position || ''}
                  onChange={(event) => setCompetitors((current) => current.map((entry, entryIndex) =>
                    entryIndex === index ? { ...entry, position: Number(event.target.value) } : entry))} />
              </div>
            ))}
            <datalist id="championship-drivers">
              {Array.from(new Set(previousCompetitors.map((competitor) => competitor.name).filter(Boolean))).map((name) => (
                <option key={name} value={name} />
              ))}
            </datalist>
            <button type="button" onClick={() => setCompetitors((current) => [...current, { name: '', position: current.length + 1 }])}>
              Add driver
            </button>
          </div>
        )}
        <div style={styles.actions}>
          <button onClick={onClose} disabled={submitting}>Cancel</button>
          <button onClick={() => void submit()} disabled={(!championship && !result.trim()) || submitting || (championship && (!playerPosition || Number(playerPosition) < 1))}>
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
  competitorRow: { display: 'flex', gap: '0.5rem', marginBottom: '0.4rem' },
};

export default ResultPromptModal;
