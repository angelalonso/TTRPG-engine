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
  onAutodetectPlugin?: (resultsFile?: string) => Promise<RaceResultsPluginResponse>;
  onSubmitPlugin?: (response: RaceResultsPluginResponse) => Promise<void>;
  onChooseResultFile?: () => Promise<string | null>;
}

type ImportedRacer = Record<string, string | number>;

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
  onAutodetectPlugin,
  onSubmitPlugin,
  onChooseResultFile,
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
  const [detectedFile, setDetectedFile] = useState('');
  const [polePosition, setPolePosition] = useState(false);
  const [racers, setRacers] = useState<ImportedRacer[]>([]);
  const [selectedRacer, setSelectedRacer] = useState(-1);
  const [standings, setStandings] = useState<RaceResultsPluginResponse['standings']>([]);
  const [trackId, setTrackId] = useState('');
  const pluginActive = pluginEnabled && !overridePlugin;
  const applyPluginResponse = (response: RaceResultsPluginResponse) => {
    setResult(response.result);
    setDamageType(response.damage_type || 'none');
    setPlayerPosition(response.player_position ? String(response.player_position) : '0');
    setCompetitors(response.competitors || []);
    setDetectedFile(response.detected_file || '');
    setPolePosition(Boolean(response.pole_position));
    setRacers(response.racers || []);
    setStandings(response.standings || []);
    setTrackId(response.track_id || '');
    setSelectedRacer((response.racers || []).findIndex((racer) =>
      String(racer.Driver || '').toLowerCase() === String(response.driver_name || '').toLowerCase()));
    setOverridePlugin(true);
  };
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
      if (racers.length > 0 && onSubmitPlugin) {
        if (selectedRacer < 0 || selectedRacer >= racers.length) {
          throw new Error('Select which imported racer is the player');
        }
        const selected = racers[selectedRacer];
        const isDnf = String(selected.RaceTime || '').trim().toUpperCase() === 'DNF';
        await onSubmitPlugin({
          result: isDnf ? 'failure' : 'success',
          player_position: isDnf ? 0 : Number(selected.position || 0),
          competitors: racers
            .filter((_, index) => index !== selectedRacer)
            .map((racer) => ({ name: String(racer.Driver || '').trim(), position: Number(racer.position || 0) }))
            .filter((entry) => entry.name && entry.position > 0),
          damage_type: damageType,
          pole_position: polePosition,
          standings,
          detected_file: detectedFile,
          track_id: trackId,
          racers,
          driver_name: String(selected.Driver || ''),
        });
        return;
      }
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
      .then(applyPluginResponse)
      .catch((pluginError) => setError(String(pluginError)))
      .finally(() => setSubmitting(false));
  }, [onOpenPlugin, onSubmitPlugin, pluginActive, pluginStarted]);

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(event) => event.stopPropagation()}>
        {pluginActive && (
          <div style={styles.pluginWaiting}>
            <h2>Race results plugin</h2>
            <p>Import the latest GTR2 result, choose a result file, or enter the result manually.</p>
            {onAutodetectPlugin && (
              <>
                <button type="button" disabled={submitting} onClick={() => {
                  setSubmitting(true);
                  setError('');
                  void onAutodetectPlugin()
                    .then(applyPluginResponse)
                    .catch((autodetectError) => setError(String(autodetectError)))
                    .finally(() => setSubmitting(false));
                }}>Autodetect latest result</button>
                {onChooseResultFile && (
                  <button type="button" disabled={submitting} onClick={() => {
                    setSubmitting(true);
                    setError('');
                    void onChooseResultFile()
                      .then((file) => file ? onAutodetectPlugin(file) : null)
                      .then((response) => response && applyPluginResponse(response))
                      .catch((fileError) => setError(String(fileError)))
                      .finally(() => setSubmitting(false));
                  }}>Choose result file</button>
                )}
              </>
            )}
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
        {detectedFile && (
          <p style={{ color: 'var(--secondary-text)', fontSize: '0.85rem' }}>
            Review imported data from <code>{detectedFile}</code>, then confirm it below.
          </p>
        )}
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
        {!racers.length && (
          <label style={styles.checkbox}>
            <input type="checkbox" checked={polePosition} onChange={(event) => setPolePosition(event.target.checked)} />
            Pole position
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
        {racers.length > 0 && (
          <section style={styles.importedResults}>
            <strong>Imported racers</strong>
            <p style={styles.muted}>Select the player and correct imported values before saving.</p>
            <div style={styles.racerTableWrap}>
              <table style={styles.racerTable}>
                <thead><tr><th>Player</th><th>Pos</th><th>Driver</th><th>Vehicle</th><th>Race time</th><th>Best lap</th></tr></thead>
                <tbody>
                  {racers.map((racer, index) => (
                    <tr key={index}>
                      <td><input type="radio" name="race-player" checked={selectedRacer === index} onChange={() => setSelectedRacer(index)} /></td>
                      {(['position', 'Driver', 'Vehicle', 'RaceTime', 'BestLap'] as const).map((field) => (
                        <td key={field}>
                          <input
                            value={String(racer[field] || '')}
                            onChange={(event) => setRacers((current) => current.map((entry, entryIndex) =>
                              entryIndex === index
                                ? { ...entry, [field]: field === 'position' ? Number(event.target.value || 0) : event.target.value }
                                : entry))}
                          />
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <label style={styles.checkbox}>
              <input type="checkbox" checked={polePosition} onChange={(event) => setPolePosition(event.target.checked)} />
              Pole position
            </label>
          </section>
        )}
        {error && <p role="alert" style={styles.error}>{error}</p>}
        <div style={styles.actions}>
          <button onClick={onClose} disabled={submitting}>Cancel</button>
          <button onClick={() => void submit()} disabled={(!championship && !result.trim()) || submitting || (championship && !playerPosition)}>
            Confirm & Save
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
    width: 'min(980px, 96vw)', maxHeight: '94vh', overflowY: 'auto', padding: '1.5rem', background: 'var(--surface-background)',
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
  importedResults: { marginTop: '1rem', padding: '0.75rem', border: '1px solid var(--control-border)', background: 'var(--control-background)' },
  muted: { color: 'var(--secondary-text)' },
  racerTableWrap: { overflowX: 'auto', maxHeight: '300px', overflowY: 'auto' },
  racerTable: { width: '100%', borderCollapse: 'collapse', fontSize: '0.82rem' },
  checkbox: { display: 'flex', gap: '0.4rem', alignItems: 'center', marginTop: '0.6rem' },
  error: { color: 'var(--error-text)', marginBottom: '0.75rem' },
};

export default ResultPromptModal;
