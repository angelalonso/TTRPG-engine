import React from 'react';
import type { EncounterResult, EncounterState, GameCatalog, OwnedObject } from '../types/game';

interface FightModalProps {
  encounter: EncounterState | EncounterResult;
  catalog: GameCatalog;
  inventory: OwnedObject[];
  onAction: (actionId: string) => void;
  onClose: () => void;
}

const isState = (encounter: EncounterState | EncounterResult): encounter is EncounterState =>
  'current_actor' in encounter;

export const FightModal: React.FC<FightModalProps> = ({
  encounter,
  catalog,
  inventory,
  onAction,
  onClose,
}) => {
  const config = catalog.encounter_configs.find((entry) => entry.encounter_id === encounter.encounter_id);
  const attributes = isState(encounter) ? encounter.attributes : encounter.final_attribute_values;
  const availableTools = catalog.encounter_actions.filter((action) =>
    (!action.usable_by || action.usable_by === 'player' || action.usable_by === 'both')
    && (!action.requires_object_id || inventory.some((object) =>
      object.id === action.requires_object_id || object.id.startsWith(`${action.requires_object_id}_`)))
    && (!action.requires_attribute_id
      || (attributes.player?.[action.requires_attribute_id]
        ?? 0) >= (action.requires_attribute_min ?? 0)),
  );
  const playerResistance = attributes.player?.resistance ?? 0;
  const opponentResistance = attributes.opponent?.resistance ?? 0;
  const opponent = catalog.encounter_opponents.find((entry) => entry.opponent_id === encounter.opponent_id);
  const opponentTools = (opponent?.available_action_ids || '')
    .split(';')
    .map((id) => catalog.encounter_actions.find((action) => action.action_id === id)?.display_name)
    .filter((name): name is string => Boolean(name));

  return (
    <div style={styles.overlay}>
      <div style={styles.modal} role="dialog" aria-modal="true" aria-labelledby="fight-title">
        <header style={styles.header}>
          <h2 id="fight-title">{config?.display_label || 'Fight mode'}</h2>
          {(!isState(encounter) || encounter.finished) && (
            <button style={styles.close} onClick={onClose} aria-label="Close">×</button>
          )}
        </header>
        <div style={styles.content}>
          <div style={styles.resistance}>
            <div><strong>You</strong><span>{playerResistance} resistance</span><small>Tools: your available tools</small></div>
            <div><strong>Opponent</strong><span>{opponentResistance} resistance</span><small>Tools: {opponentTools.join(', ') || 'configured by opponent'}</small></div>
          </div>
          {isState(encounter) && !encounter.finished ? (
            <>
              <p style={styles.instructions}>
                Choose one tool. The opponent defends automatically after your result.
              </p>
              <div style={styles.tools}>
                {availableTools.map((action) => (
                  <button key={action.action_id} style={styles.tool} onClick={() => onAction(action.action_id)}>
                    <strong>{action.display_name}</strong>
                    <span>Success: {(action.base_success_rate * 100).toFixed(0)}%</span>
                    <span>Positive result: {Math.abs(action.effect_on_success)} resistance</span>
                    <span>Negative result: {Math.abs(action.effect_on_failure)} resistance</span>
                    {action.requires_object_id && <span>Requires: {action.requires_object_id}</span>}
                  </button>
                ))}
              </div>
              {availableTools.length === 0 && <p style={styles.error}>No usable tools are available.</p>}
            </>
          ) : (
            <>
              <strong style={styles.outcome}>
                Outcome: {encounter.outcome || 'draw'}
              </strong>
              {'full_log' in encounter
                ? encounter.full_log.map((entry, index) => <p key={index}>{entry.text}</p>)
                : encounter.log.map((entry, index) => <p key={index}>{entry.text}</p>)}
            </>
          )}
        </div>
        {(!isState(encounter) || encounter.finished) && (
          <footer style={styles.footer}>
            <button onClick={onClose}>Close</button>
          </footer>
        )}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed', inset: 0, zIndex: 2200, display: 'flex',
    alignItems: 'center', justifyContent: 'center', padding: '1rem',
    background: 'var(--modal-overlay)', backdropFilter: 'blur(5px)',
  },
  modal: {
    width: 'min(760px, 96vw)', maxHeight: '92vh', display: 'flex',
    flexDirection: 'column', background: 'var(--surface-background)',
    border: '1px solid var(--control-border)', borderRadius: '10px',
    boxShadow: '0 24px 50px var(--modal-overlay-dark)',
  },
  header: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    padding: '1rem 1.25rem', borderBottom: '1px solid var(--surface-border)',
  },
  close: {
    border: 0, background: 'transparent', color: 'var(--subtle-text)',
    fontSize: '1.8rem', lineHeight: 1, cursor: 'pointer',
  },
  content: { overflow: 'auto', padding: '1.25rem' },
  resistance: { display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem', marginBottom: '1rem' },
  instructions: { color: 'var(--secondary-text)' },
  tools: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(190px, 1fr))', gap: '0.75rem' },
  tool: {
    display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: '0.3rem',
    padding: '0.85rem', textAlign: 'left', cursor: 'pointer',
  },
  error: { color: 'var(--error-text)' },
  outcome: { display: 'block', marginBottom: '1rem', fontSize: '1.15rem' },
  footer: {
    display: 'flex', justifyContent: 'flex-end', padding: '1rem 1.25rem',
    borderTop: '1px solid var(--surface-border)',
  },
};

export default FightModal;
