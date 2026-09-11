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
  championshipDrivers?: string[];
  scoringPositions?: number;
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
  championshipDrivers = [],
  scoringPositions = 1,
}) => {
  const [result, setResult] = useState('');
  const [damageType, setDamageType] = useState('none');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');
  const [playerPosition, setPlayerPosition] = useState('');
  const initialCompetitors = previousCompetitors
    .filter((entry) => entry.position > 0)
    .sort((a, b) => a.position - b.position)
    .reduce<ChampionshipCompetitor[]>((entries, entry) => {
      if (!entries.some((existing) => existing.position === entry.position)) entries.push(entry);
      return entries;
    }, []);
  const [competitors, setCompetitors] = useState<ChampionshipCompetitor[]>(initialCompetitors);
  const positionOptions = Array.from({ length: Math.max(1, scoringPositions) }, (_, index) => index + 1);
  const competitorPositionOptions = Array.from(
    { length: Math.max(positionOptions.length, competitors.length + 1) },
    (_, index) => index + 1,
  );
  const playerPositionOptions = [0, ...positionOptions];
  const setPosition = (value: string) => {
    const nextPosition = Number(value);
    setPlayerPosition(value);
    if (!Number.isInteger(nextPosition) || nextPosition < 0 || nextPosition === 0) return;
    setCompetitors((current) => {
      const remaining = current.filter((entry) => entry.position !== nextPosition);
      let position = 1;
      return remaining.map((entry) => {
        while (position === nextPosition) position += 1;
        const assigned = { ...entry, position };
        position += 1;
        return assigned;
      });
    });
  };
  const changeCompetitorPosition = (index: number, value: string) => {
    const nextPosition = Number(value);
    if (!Number.isInteger(nextPosition) || nextPosition === Number(playerPosition)) return;
    setCompetitors((current) => {
      if (current.some((entry, entryIndex) => entryIndex !== index && entry.position === nextPosition)) {
        return current;
      }
      return current.map((entry, entryIndex) =>
        entryIndex === index ? { ...entry, position: nextPosition } : entry,
      ).sort((a, b) => a.position - b.position);
    });
  };

  const submit = async () => {
    if ((!championship && !result.trim()) || (championship && (!playerPosition || Number(playerPosition) < 1)) || submitting) return;
    setSubmitting(true);
    setError('');
    try {
      if (championship) {
        if (!onSubmitChampionship) {
          throw new Error('Championship result handler is unavailable');
        }
        await onSubmitChampionship(
          'success',
          damageType,
          Number(playerPosition),
          competitors.filter((entry) =>
            entry.name.trim() && entry.position > 0,
          ),
        );
      } else {
        await onSubmit(result.trim(), damageType);
      }
    } catch (submitError) {
      setError(String(submitError));
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
              <select value={playerPosition} onChange={(event) => setPosition(event.target.value)}>
                <option value="">Choose position</option>
                {playerPositionOptions.map((position) => (
                  <option key={position} value={position}>
                    {position === 0 ? 'Not on points' : position}
                  </option>
                ))}
              </select>
            </label>
            <p>Other point-scoring drivers</p>
            {competitors.map((competitor, index) => (
              <div key={index} style={styles.competitorRow}>
                <input list="championship-drivers" placeholder="Driver name" value={competitor.name}
                  onChange={(event) => setCompetitors((current) => current.map((entry, entryIndex) =>
                    entryIndex === index ? { ...entry, name: event.target.value } : entry))} />
                <select value={competitor.position || ''} onChange={(event) => changeCompetitorPosition(index, event.target.value)}>
                  {competitorPositionOptions.map((position) => (
                    <option
                      key={position}
                      value={position}
                      disabled={position === Number(playerPosition)
                        || competitors.some((entry, entryIndex) => entryIndex !== index && entry.position === position)}
                    >
                      {position}
                    </option>
                  ))}
                </select>
                <button type="button" onClick={() => setCompetitors((current) => current.filter((_, entryIndex) => entryIndex !== index))}>
                  Remove
                </button>
              </div>
            ))}
            <datalist id="championship-drivers">
              {Array.from(new Set([...championshipDrivers, ...previousCompetitors.map((competitor) => competitor.name)].filter(Boolean))).map((name) => (
                <option key={name} value={name} />
              ))}
            </datalist>
            <button
              type="button"
              onClick={() => setCompetitors((current) => {
                const used = new Set(current.map((entry) => entry.position));
                const position = competitorPositionOptions.find((candidate) =>
                  candidate !== Number(playerPosition) && !used.has(candidate),
                );
                return position ? [...current, { name: '', position }] : current;
              })}
            >
              Add driver
            </button>
          </div>
        )}
        {error && <p role="alert" style={styles.error}>{error}</p>}
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
  error: { color: '#fca5a5', marginBottom: '0.75rem' },
};

export default ResultPromptModal;
