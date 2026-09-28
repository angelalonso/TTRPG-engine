import React, { useState } from 'react';
import type { ChampionshipCompetitor } from '../types/game';
import type { RaceResultsPluginResponse } from '../services/tauriApi';

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
  finishingPositions?: number;
  pluginEnabled?: boolean;
  competitorLabel?: string;
  competitorPluralLabel?: string;
  onClose: () => void;
  onOpenPlugin?: () => Promise<RaceResultsPluginResponse>;
  onSubmitPlugin?: (response: RaceResultsPluginResponse) => Promise<void>;
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
  finishingPositions,
  competitorLabel = 'Competitor',
  competitorPluralLabel = 'Competitors',
  pluginEnabled = false,
  onOpenPlugin,
  onSubmitPlugin,
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
  const [pluginStarted, setPluginStarted] = useState(false);
  const [overridePlugin, setOverridePlugin] = useState(false);
  const pluginActive = pluginEnabled && !overridePlugin;
  const positionOptions = Array.from({ length: Math.max(1, scoringPositions) }, (_, index) => index + 1);
  const finishingPositionCount = Math.max(
    positionOptions.length,
    finishingPositions || championshipDrivers.length,
  );
  const finishingPositionOptions = Array.from(
    { length: Math.max(1, finishingPositionCount) },
    (_, index) => index + 1,
  );
  const competitorPositionOptions = Array.from(
    { length: Math.max(finishingPositionOptions.length + competitors.length, competitors.length + 1) },
    (_, index) => index + 1,
  );
  const playerPositionOptions = [0, ...finishingPositionOptions];
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
    if ((!championship && !result.trim()) || (championship && !playerPosition) || submitting) return;
    setSubmitting(true);
    setError('');
    try {
      if (championship) {
        if (!onSubmitChampionship) {
          throw new Error('Championship result handler is unavailable');
        }
        const usedPositions = new Set(
          competitors
            .filter((entry) => entry.name.trim() && entry.position > 0)
            .map((entry) => entry.position),
        );
        let assignedPlayerPosition = Number(playerPosition);
        if (assignedPlayerPosition === 0) {
          assignedPlayerPosition = finishingPositionCount + 1;
          while (usedPositions.has(assignedPlayerPosition)) assignedPlayerPosition += 1;
        }
        await onSubmitChampionship(
          'success',
          damageType,
          assignedPlayerPosition,
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

  React.useEffect(() => {
    if (!pluginActive || pluginStarted || !onOpenPlugin || !onSubmitPlugin) return;
    setPluginStarted(true);
    setSubmitting(true);
    void onOpenPlugin()
      .then(onSubmitPlugin)
      .catch((pluginError) => setError(String(pluginError)))
      .finally(() => setSubmitting(false));
  }, [onOpenPlugin, onSubmitPlugin, pluginActive, pluginStarted]);

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(event) => event.stopPropagation()}>
        {pluginActive && (
          <div style={styles.pluginWaiting}>
            <h2>Race results plugin</h2>
            <p>The green Python results window is open. Save the result there to continue.</p>
            {error && (
              <>
                <p role="alert" style={styles.error}>{error}</p>
                <div style={styles.actions}>
                  <button
                    type="button"
                    onClick={() => {
                      setError('');
                      setPluginStarted(false);
                    }}
                    disabled={submitting}
                  >
                    Retry plugin
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setError('');
                      setOverridePlugin(true);
                    }}
                  >
                    Override in game
                  </button>
                </div>
              </>
            )}
          </div>
        )}
        {!pluginActive && (
        <>
        <div style={pluginActive ? styles.pluginHeader : undefined}>
          {pluginActive && (
            <>
              <h2 style={{ margin: 0 }}>Race results plugin</h2>
              <span>Green input mode</span>
            </>
          )}
        </div>
        {!pluginEnabled && <h2>Enter result</h2>}
        {championship ? (
          <p>Your finishing position in {eventName}</p>
        ) : (
          <>
            <p>How did {eventName} finish?</p>
            <input
              style={styles.resultInput}
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
          <label style={styles.damageField}>
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
            <p>Other {competitorPluralLabel.toLowerCase()} (positions beyond the configured points places do not score)</p>
            {competitors.map((competitor, index) => (
              <div key={index} style={styles.competitorRow}>
                <input list="championship-drivers" placeholder={`${competitorLabel} name`} value={competitor.name}
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
              Add {competitorLabel.toLowerCase()}
            </button>
          </div>
        )}
        {error && <p role="alert" style={styles.error}>{error}</p>}
        <div style={styles.actions}>
          <button onClick={onClose} disabled={submitting}>Cancel</button>
          <button onClick={() => void submit()} disabled={(!championship && !result.trim()) || submitting || (championship && !playerPosition)}>
            Record result
          </button>
        </div>
        </>
        )}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 2100, display: 'grid', placeItems: 'center',
    padding: '1rem', background: 'var(--modal-overlay)',
  },
  modal: {
    width: 'min(520px, 94vw)', padding: '1.5rem', background: 'var(--surface-background)',
    border: '1px solid var(--control-border)', borderRadius: '10px', color: 'var(--primary-text)',
  },
  pluginHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    gap: '1rem',
    marginBottom: '0.75rem',
    padding: '0.65rem 0.8rem',
    borderRadius: '7px',
    background: '#163d27',
    border: '1px solid #36b765',
    color: '#b8f5c8',
  },
  actions: { display: 'flex', justifyContent: 'flex-end', gap: '0.75rem', marginTop: '1rem' },
  resultInput: { display: 'block', width: '100%', boxSizing: 'border-box', marginBottom: '0.75rem' },
  damageField: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.35rem',
    marginTop: '0.75rem',
  },
  competitorRow: { display: 'flex', gap: '0.5rem', marginBottom: '0.4rem' },
  error: { color: 'var(--error-text)', marginBottom: '0.75rem' },
};

export default ResultPromptModal;
