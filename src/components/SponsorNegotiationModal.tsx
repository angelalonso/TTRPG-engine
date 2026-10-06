import React, { useMemo, useState } from 'react';
import {
  closeSponsorNegotiation,
  sponsorNegotiationAction,
  type SponsorNegotiationState,
} from '../services/tauriApi';
import type { GameState } from '../types/game';

interface SponsorNegotiationModalProps {
  initialState: SponsorNegotiationState;
  onGameState: (state: GameState) => void;
  onClose: () => void;
}

const booleanFields = ['entry_fees', 'gear', 'maintenance'] as const;
const numberFields = ['initial_money', 'monthly_payment', 'result_bonus', 'dnf_penalty'] as const;
const fieldLabels: Record<string, string> = {
  entry_fees: 'Entry fees covered',
  gear: 'Race gear included',
  maintenance: 'Maintenance covered',
  initial_money: 'Initial payment',
  monthly_payment: 'Monthly payment',
  result_bonus: 'Result bonus',
  dnf_penalty: 'DNF penalty',
};

type Proposal = Record<string, number | boolean>;

export const SponsorNegotiationModal: React.FC<SponsorNegotiationModalProps> = ({
  initialState,
  onGameState,
  onClose,
}) => {
  const [state, setState] = useState(initialState);
  const [proposal, setProposal] = useState<Proposal>(initialState.proposal || {});
  const [target, setTarget] = useState(initialState.selected_target || '');
  const [car, setCar] = useState(initialState.selected_car || 'No car included');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const terminal = ['SIGNED', 'REJECTED', 'BANNED'].includes(state.status);
  const proposalSent = state.cold_call ? state.round > 0 : true;
  const targetOptions = state.target_options || [];
  const vehicleOptions = state.vehicle_options || [];

  const proposalValue = useMemo(() => {
    const cashValue = Object.entries(proposal)
      .filter(([, value]) => typeof value === 'number')
      .reduce((total, [key, value]) => (
        total + (key === 'dnf_penalty' ? -Number(value) : Number(value))
      ), 0);
    const vehicleValue = vehicleOptions.find((option) => option.name === car)?.price || 0;
    return cashValue + vehicleValue;
  }, [car, proposal, vehicleOptions]);

  const applyResponse = (nextState: SponsorNegotiationState, gameState: GameState) => {
    const mergedState = { ...state, ...nextState };
    setState(mergedState);
    onGameState(gameState);
    setProposal(nextState.proposal || {});
    setTarget(nextState.selected_target || target);
    setCar(nextState.selected_car || car);
  };

  const act = async (action: Record<string, unknown>) => {
    setBusy(true);
    setError('');
    try {
      const response = await sponsorNegotiationAction(action);
      applyResponse(response.state, response.game_state);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setBusy(false);
    }
  };

  const selectScope = async (scope: string) => {
    await act({ action: 'new_session', scope });
    setTarget('');
    setCar('No car included');
  };

  const selectTarget = (nextTarget: string) => {
    setTarget(nextTarget);
    void act({ action: 'select_target', target_name: nextTarget });
  };

  const selectCar = (nextCar: string) => {
    setCar(nextCar);
    void act({
      action: 'select_car',
      car_name: nextCar,
      car_object_id: vehicleOptions.find((option) => option.name === nextCar)?.id || '',
    });
  };

  const close = async () => {
    try {
      await closeSponsorNegotiation();
    } finally {
      onClose();
    }
  };

  return (
    <div style={styles.overlay} onClick={() => void close()}>
      <section style={styles.modal} onClick={(event) => event.stopPropagation()}>
        <header style={styles.header}>
          <div>
            <span style={styles.eyebrow}>TTRPG ENGINE / DRIVER SERVICES</span>
            <h2 style={styles.title}>Sponsor negotiation</h2>
          </div>
          <div style={styles.tier}>{state.sponsor_tier.toUpperCase()} / {state.sponsor_brand || 'PARTNERSHIP'}</div>
        </header>
        <div style={styles.toolbar}>
          <label style={styles.toolbarLabel}>
            Sponsorship period
            <select value={state.scope} disabled={busy} onChange={(event) => void selectScope(event.target.value)}>
              <option value="race">Race</option>
              <option value="championship">Championship</option>
              <option value="year">Year</option>
            </select>
          </label>
          {state.scope !== 'year' && (
            <label style={styles.toolbarLabel}>
              Target
              <select value={target} disabled={busy} onChange={(event) => selectTarget(event.target.value)}>
                <option value="">Choose target</option>
                {targetOptions.map((option) => <option key={option.id} value={option.name}>{option.name}</option>)}
              </select>
            </label>
          )}
          <label style={styles.toolbarLabel}>
            Vehicle
            <select
              value={car}
              disabled={busy || (state.scope !== 'year' && !target)}
              onChange={(event) => selectCar(event.target.value)}
            >
              <option value="No car included">No car included</option>
              {vehicleOptions.map((option) => <option key={option.id} value={option.name}>{option.name}</option>)}
            </select>
          </label>
          <span style={styles.manager}>{state.manager_level ? `MANAGER LV. ${state.manager_level}` : 'NO MANAGER'}</span>
        </div>
        {error && <p role="alert" style={styles.error}>{error}</p>}
        <div style={styles.columns}>
          <section style={styles.panel}>
            <h3 style={styles.sectionTitle}>{state.sponsor_name}</h3>
            <p style={styles.dialogue}>{state.sponsor_dialogue}</p>
            <div style={styles.metrics}>
              <Metric label="Attraction score" value={Number(state.attraction_score || 0).toFixed(1)} />
              <Metric label="Race tier" value={state.race_tier} />
              <Metric label="Round" value={String(state.round || 0)} />
              <Metric label="Status" value={state.status} />
            </div>
            <p style={styles.log}>{state.final_log || state.last_action_log || ''}</p>
          </section>
          <aside style={styles.panel}>
            <h3 style={styles.sectionTitle}>Negotiation brief</h3>
            <p>Build a package that reflects your results, audience value, and the sponsor&apos;s walkaway limit.</p>
            <p style={styles.muted}>{state.objection}</p>
            {state.agreement && <p style={styles.success}>Agreement signed. Close this panel to return to the game.</p>}
          </aside>
        </div>
        <section style={{ ...styles.panel, ...styles.packagePanel, ...(state.status === 'SIGNED' ? styles.signed : {}) }}>
          <h3 style={styles.sectionTitle}>Proposed package</h3>
          <div style={styles.packageGrid}>
            {booleanFields.map((key) => (
              <label key={key} style={styles.field}>
                <span>{fieldLabels[key]}</span>
                <input
                  type="checkbox"
                  checked={Boolean(proposal[key])}
                  disabled={busy || terminal}
                  onChange={(event) => setProposal((current) => ({ ...current, [key]: event.target.checked }))}
                />
              </label>
            ))}
            {numberFields.map((key) => (
              <label key={key} style={styles.field}>
                <span>{fieldLabels[key]}</span>
                <input
                  type="number"
                  min="0"
                  value={Number(proposal[key] || 0)}
                  disabled={busy || terminal || (key === 'monthly_payment' && state.scope !== 'year')}
                  onChange={(event) => setProposal((current) => ({ ...current, [key]: Number(event.target.value || 0) }))}
                />
                {state.ideal_proposal?.[key] !== undefined && state.ideal_proposal[key] !== proposal[key] && (
                  <small style={styles.counter}>Ideal: {Number(state.ideal_proposal[key]).toLocaleString()}</small>
                )}
              </label>
            ))}
          </div>
          <p style={styles.total}>Package value: {proposalValue.toLocaleString()}</p>
          <div style={styles.actions}>
            <button
              type="button"
              disabled={busy || terminal}
              onClick={() => void act({
                action: 'counter_proposal',
                car_object_id: vehicleOptions.find((option) => option.name === car)?.id || '',
                proposal: { ...proposal, car: car !== 'No car included' },
              })}
            >
              {state.cold_call ? 'Send proposal' : 'Send counter-proposal'}
            </button>
            <button
              type="button"
              disabled={busy || terminal || !proposalSent}
              onClick={() => void act({ action: 'accept_proposal' })}
            >
              Accept sponsor proposal
            </button>
            <button type="button" disabled={busy} onClick={() => void close()}>Close</button>
          </div>
        </section>
      </section>
    </div>
  );
};

const Metric: React.FC<{ label: string; value: string }> = ({ label, value }) => (
  <div style={styles.metric}>
    <small>{label}</small>
    <strong>{value}</strong>
  </div>
);

const styles: Record<string, React.CSSProperties> = {
  overlay: { position: 'fixed', inset: 0, zIndex: 3000, display: 'grid', placeItems: 'center', padding: '1rem', background: 'rgb(0 0 0 / 76%)', backdropFilter: 'blur(4px)' },
  modal: { width: 'min(980px, 96vw)', maxHeight: '94vh', overflowY: 'auto', padding: '1.25rem', background: 'rgb(15 17 20 / 98%)', border: '1px solid #737b82', color: '#f3f4f5', boxShadow: '0 24px 60px rgb(0 0 0 / 55%)' },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'end', gap: '1rem', borderBottom: '1px solid #e5232b', paddingBottom: '0.8rem' },
  eyebrow: { color: '#aeb4b9', fontSize: '0.85rem', letterSpacing: '0.14em' },
  title: { margin: '0.15rem 0 0', fontSize: '1.8rem' },
  tier: { color: '#fff59d', fontSize: '1.05rem' },
  toolbar: { display: 'flex', flexWrap: 'wrap', alignItems: 'end', gap: '0.75rem', margin: '1rem 0', padding: '0.8rem', border: '1px solid #3b4045', background: '#111316' },
  toolbarLabel: { display: 'grid', gap: '0.3rem', color: '#d4d7da' },
  manager: { marginLeft: 'auto', color: '#aeb4b9' },
  columns: { display: 'grid', gridTemplateColumns: 'minmax(0, 1.3fr) minmax(240px, 0.7fr)', gap: '1rem' },
  panel: { padding: '1rem', background: '#111316', border: '1px solid #3b4045' },
  sectionTitle: { marginTop: 0 },
  dialogue: { minHeight: '3.5rem', color: '#d4d7da' },
  metrics: { display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '0.65rem' },
  metric: { borderLeft: '3px solid #e5232b', background: '#1c2024', padding: '0.65rem 0.75rem' },
  log: { minHeight: '2rem', marginBottom: 0, borderTop: '1px solid #3b4045', paddingTop: '0.75rem', color: '#aeb4b9', whiteSpace: 'pre-wrap' },
  muted: { color: '#aeb4b9' },
  success: { color: '#b9f0ab' },
  error: { border: '1px solid #ff5960', background: '#3a1518', padding: '0.7rem', color: '#fff' },
  packagePanel: { marginTop: '1rem' },
  signed: { borderColor: '#8ed081', boxShadow: '0 0 22px rgb(71 170 85 / 18%)' },
  packageGrid: { display: 'grid', gridTemplateColumns: 'repeat(2, minmax(0, 1fr))', gap: '0.7rem 1rem' },
  field: { display: 'grid', gridTemplateColumns: '1fr auto', gap: '0.35rem', alignItems: 'center', color: '#d4d7da' },
  counter: { gridColumn: '1 / -1', color: '#fff59d' },
  total: { color: '#aeb4b9' },
  actions: { display: 'flex', flexWrap: 'wrap', gap: '0.7rem' },
};
