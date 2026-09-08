import React, { useEffect, useState } from 'react';
import type { GameState, ServiceType, TimeSpeed } from './types/game';
import { getCharacteristic, getLabel } from './types/game';
import {
  buyObject,
  dismissAlert,
  enterEvent,
  getGameState,
  performAction,
  payCost,
  reloadDataset,
  serviceObject,
  setTimeSpeed,
  submitEventResult,
  tickGameDay,
} from './services/tauriApi';
import { AlertModal } from './components/AlertModal';
import { ConfigModal } from './components/ConfigModal';
import { DetailModal } from './components/DetailModal';

const speeds: TimeSpeed[] = ['Paused', 'OneDayEveryFiveSec', 'OneDayPerSec', 'OneWeekPerSec'];

export const App: React.FC = () => {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [tab, setTab] = useState<'dashboard' | 'inventory' | 'dealer' | 'events' | 'actions'>('dashboard');
  const [configOpen, setConfigOpen] = useState(false);
  const [message, setMessage] = useState('');
  const [selectedDetail, setSelectedDetail] = useState<{
    title: string;
    descriptionPath: string;
    footer: React.ReactNode;
  } | null>(null);
  const [eventResults, setEventResults] = useState<Record<string, string>>({});

  useEffect(() => {
    getGameState().then(setGameState).catch((error) => setMessage(String(error)));
  }, []);

  useEffect(() => {
    if (!gameState || gameState.time_speed === 'Paused' || gameState.pending_alerts.length > 0) return;
    const interval = gameState.time_speed === 'OneDayEveryFiveSec' ? 5000
      : gameState.time_speed === 'OneWeekPerSec' ? 1000 / 7 : 1000;
    const timer = setInterval(() => tickGameDay().then(setGameState).catch((error) => setMessage(String(error))), interval);
    return () => clearInterval(timer);
  }, [gameState?.time_speed, gameState?.pending_alerts.length]);

  if (!gameState) {
    return <div style={styles.loading}>Loading economy engine...</div>;
  }

  const { catalog, player } = gameState;
  const objectName = getLabel(catalog, 'object_name', 'Object');
  const objectPlural = getLabel(catalog, 'object_plural', `${objectName}s`);
  const inventoryName = getLabel(catalog, 'inventory_name', 'Inventory');
  const dealerName = getLabel(catalog, 'dealer_name', 'Dealer');
  const eventName = getLabel(catalog, 'event_name', 'Event');
  const eventPlural = getLabel(catalog, 'event_plural', `${eventName}s`);
  const currency = getLabel(catalog, 'currency_symbol', '$');
  const overviewName = getLabel(catalog, 'overview_name', 'Overview');
  const ageLabel = getLabel(catalog, 'age_name', 'Age');
  const budgetLabel = getLabel(catalog, 'budget_name', 'Budget');
  const dayLabel = getLabel(catalog, 'day_name', 'Day');
  const activeLabel = getLabel(catalog, 'active_name', 'Active');
  const budget = getCharacteristic(player, 'budget');
  const costFor = (object: (typeof player.inventory)[number], index: number) => {
    const id = object[`cost_${index}` as keyof typeof object] as string;
    return catalog.costs.find((cost) => cost.id === id);
  };
  const needsService = (object: (typeof player.inventory)[number], index: number) =>
    object[`service_${index}_needed` as keyof typeof object] as boolean;

  const run = async (operation: () => Promise<unknown>, success: string) => {
    try {
      const result = await operation();
      if (result && typeof result === 'object' && 'message' in result) {
        setMessage(String(result.message));
        if ('event_name' in result) {
          setGameState(await getGameState());
          return;
        }
      } else if (result && typeof result === 'object' && 'player' in result) {
        setGameState(result as GameState);
      }
      setMessage(success);
    } catch (error) {
      setMessage(String(error));
    }
  };

  const renderDashboard = () => (
    <div style={styles.grid}>
      <section style={styles.card}>
        <h2>{overviewName}</h2>
        <p>{ageLabel}: {Math.floor(player.age_days / gameState.days_per_year)} years</p>
        <p>{budgetLabel}: {currency}{budget.toLocaleString()}</p>
        {catalog.player_characteristics
          .filter((characteristic) => characteristic.id !== 'budget')
          .map((characteristic) => (
            <p key={characteristic.id}>{characteristic.name}: {getCharacteristic(player, characteristic.id)}</p>
          ))}
        <p>{inventoryName}: {player.inventory.length}</p>
        <p>{activeLabel} {getLabel(catalog, 'action_name', 'Actions')}: {player.active_actions.length}</p>
      </section>
      <section style={styles.card}>
        <h2>Cost Ledger</h2>
        {gameState.cost_ledger.length === 0 && <p>No generated costs yet.</p>}
        {gameState.cost_ledger.slice(-8).reverse().map((occurrence) => {
          const cost = catalog.costs.find((entry) => entry.id === occurrence.cost_id);
          return (
            <div key={occurrence.id} style={styles.row}>
              <span>
                {cost?.name || occurrence.cost_id}: {currency}{occurrence.amount.toLocaleString()}
                {' '}({occurrence.status})
              </span>
              {occurrence.status === 'pending' && (
                <button
                  disabled={budget < occurrence.amount}
                  onClick={() => run(() => payCost(occurrence.id), 'Cost paid.')}
                >
                  Pay
                </button>
              )}
            </div>
          );
        })}
      </section>
    </div>
  );

  const renderInventory = () => (
    <div style={styles.grid}>
      {player.inventory.length === 0 && <p>No {objectPlural.toLowerCase()} in {inventoryName.toLowerCase()}.</p>}
      {player.inventory.map((object) => (
        <section key={object.id} style={styles.card}>
          <button
            style={styles.linkButton}
            onClick={() => setSelectedDetail({
              title: object.name,
              descriptionPath: object.description_html,
              footer: (
                <button onClick={() => run(
                  () => serviceObject(object.id, { BuyUnits: 1 }),
                  'Units acquired.',
                )}>
                  Buy 1 {getLabel(catalog, 'units_name', 'Unit')}
                </button>
              ),
            })}
          >
            <span style={styles.inventoryTitle}>{object.name}</span>
          </button>
          <p>{getLabel(catalog, 'units_name', 'Units')}: {object.units_available}</p>
          {[1, 2, 3, 4].map((index) => (
            <div key={index} style={styles.row}>
              <span>
                {costFor(object, index)?.name || `Cost ${index}`}:
                {' '}{needsService(object, index) ? 'Required' : 'Ready'}
              </span>
              <button
                disabled={!needsService(object, index) || !costFor(object, index) || budget < (costFor(object, index)?.amount || 0)}
                onClick={() => run(
                  () => serviceObject(object.id, `Service${index}` as ServiceType),
                  `${costFor(object, index)?.name || `Cost ${index}`} completed.`,
                )}
              >
                {currency}{costFor(object, index)?.amount ?? 0}
              </button>
            </div>
          ))}
          <button onClick={() => run(() => serviceObject(object.id, { BuyUnits: 1 }), 'Units acquired.')}>
            Buy 1 {getLabel(catalog, 'units_name', 'Unit')} ({currency}{costFor(object, 4)?.amount ?? 0})
          </button>
        </section>
      ))}
    </div>
  );

  const renderDealer = () => (
    <div style={styles.grid}>
      {catalog.objects.map((object) => (
        <button
          key={object.id}
          style={styles.nameCard}
          onClick={() => setSelectedDetail({
            title: object.name,
            descriptionPath: object.description_html,
            footer: (
              <button onClick={() => run(() => buyObject(object.id), `${objectName} acquired.`)}>
                Acquire ({currency}{object.price.toLocaleString()})
              </button>
            ),
          })}
        >
          {object.name}
        </button>
      ))}
    </div>
  );

  const renderEvents = () => {
    const currentDay = ((gameState.current_day - 1) % gameState.days_per_year) + 1;
    return (
      <div style={styles.grid}>
        {gameState.pending_events.length > 0 && (
          <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <h2>Pending {eventPlural}</h2>
            {gameState.pending_events.map((pending) => {
              const event = catalog.events.find((entry) => entry.id === pending.event_id);
              const object = player.inventory.find((entry) => entry.id === pending.object_id);
              if (!event) return null;
              return (
                <div key={pending.id} style={styles.pendingEvent}>
                  <div>
                    <strong>{event.name}</strong>
                    <span style={styles.muted}> with {object?.name || pending.object_id}</span>
                  </div>
                  <div style={styles.resultControls}>
                    <input
                      value={eventResults[pending.id] || ''}
                      onChange={(input) => setEventResults((current) => ({
                        ...current,
                        [pending.id]: input.target.value,
                      }))}
                      placeholder="Enter result"
                    />
                    <button
                      disabled={!eventResults[pending.id]?.trim()}
                      onClick={() => run(
                        async () => {
                          const result = await submitEventResult(pending.id, eventResults[pending.id]);
                          setEventResults((current) => {
                            const next = { ...current };
                            delete next[pending.id];
                            return next;
                          });
                          setGameState(await getGameState());
                          return result;
                        },
                        'Event result recorded.',
                      )}
                    >
                      Record result
                    </button>
                  </div>
                </div>
              );
            })}
          </section>
        )}
        {catalog.events.map((event) => (
          <button
            key={event.id}
            style={styles.nameCard}
            onClick={() => setSelectedDetail({
              title: event.name,
              descriptionPath: event.description_html,
              footer: (
                <>
                  <span>{dayLabel}: {event.day_of_year} | Entry: {currency}{event.entry_fee}</span>
                  {player.inventory.map((object) => (
                    <button
                      key={object.id}
                      disabled={currentDay !== event.day_of_year}
                      onClick={() => run(
                        () => enterEvent(object.id, event.id),
                        `${eventName} entered. Record its result below.`,
                      )}
                    >
                      Enter with {object.name}
                    </button>
                  ))}
                </>
              ),
            })}
          >
            {event.name}
          </button>
        ))}
        {gameState.event_history.length > 0 && (
          <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <h2>Completed {eventPlural}</h2>
            {gameState.event_history.slice().reverse().map((history) => {
              const event = catalog.events.find((entry) => entry.id === history.event_id);
              const object = player.inventory.find((entry) => entry.id === history.object_id);
              return (
                <div key={history.id} style={styles.row}>
                  <span>
                    {event?.name || history.event_id} with {object?.name || history.object_id}:
                    {' '}{history.result} ({history.outcome})
                  </span>
                  {history.reward_awarded > 0 && (
                    <span>{currency}{history.reward_awarded.toLocaleString()}</span>
                  )}
                </div>
              );
            })}
          </section>
        )}
      </div>
    );
  };

  const renderActions = () => (
    <div style={styles.grid}>
      {catalog.actions.map((action) => (
        <button
          key={action.id}
          style={styles.nameCard}
          onClick={() => setSelectedDetail({
            title: action.name,
            descriptionPath: action.description_html,
            footer: (
              <>
                <span>Cost: {currency}{action.base_cost} | Success: {(action.success_rate * 100).toFixed(0)}%</span>
                <button onClick={() => run(
                  async () => {
                    const result = await performAction(action.id);
                    setGameState(await getGameState());
                    return result;
                  },
                  'Action completed.',
                )}>
                  Start {action.name}
                </button>
              </>
            ),
          })}
        >
          {action.name}
        </button>
      ))}
    </div>
  );

  return (
    <div style={styles.app}>
      <header style={styles.header}>
        <div>
          <h1>{getLabel(catalog, 'application_name', 'Economy Engine')}</h1>
          <span>Day {gameState.current_day} | {currency}{budget.toLocaleString()}</span>
        </div>
        <div>
          {speeds.map((speed) => (
            <button key={speed} onClick={() => setTimeSpeed(speed).then(setGameState)}>
              {speed === 'Paused' ? 'Pause' : speed}
            </button>
          ))}
          <button onClick={() => setConfigOpen(true)}>Settings</button>
        </div>
      </header>
      {message && <div style={styles.banner} onClick={() => setMessage('')}>{message}</div>}
      <nav style={styles.nav}>
        {([
          ['dashboard', 'Dashboard'],
          ['inventory', inventoryName],
          ['dealer', dealerName],
          ['events', eventPlural],
          ['actions', getLabel(catalog, 'action_name', 'Actions')],
        ] as const).map(([key, title]) => (
          <button key={key} onClick={() => setTab(key)}>{title}</button>
        ))}
      </nav>
      <main>
        {tab === 'dashboard' && renderDashboard()}
        {tab === 'inventory' && renderInventory()}
        {tab === 'dealer' && renderDealer()}
        {tab === 'events' && renderEvents()}
        {tab === 'actions' && renderActions()}
      </main>
      <AlertModal alerts={gameState.pending_alerts} onDismiss={setGameState} />
      <ConfigModal
        isOpen={configOpen}
        currentPath={gameState.dataset_path}
        onClose={() => setConfigOpen(false)}
        onReloadDataset={(path) => reloadDataset(path).then(setGameState).then(() => setConfigOpen(false))}
      />
      {selectedDetail && (
        <DetailModal
          title={selectedDetail.title}
          descriptionPath={selectedDetail.descriptionPath}
          onClose={() => setSelectedDetail(null)}
        >
          {selectedDetail.footer}
          <button onClick={() => setSelectedDetail(null)}>Close</button>
        </DetailModal>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  app: { minHeight: '100vh', padding: '1.5rem', background: '#0f172a', color: '#f8fafc', fontFamily: 'sans-serif' },
  loading: { minHeight: '100vh', display: 'grid', placeItems: 'center', background: '#0f172a', color: '#f8fafc' },
  header: { display: 'flex', justifyContent: 'space-between', gap: '1rem', borderBottom: '1px solid #334155', paddingBottom: '1rem' },
  nav: { display: 'flex', gap: '0.5rem', margin: '1rem 0' },
  grid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1rem' },
  card: { background: '#1e293b', border: '1px solid #334155', borderRadius: '8px', padding: '1rem' },
  row: { display: 'flex', justifyContent: 'space-between', gap: '1rem', alignItems: 'center', padding: '0.5rem 0', borderBottom: '1px solid #334155' },
  nameCard: { background: '#1e293b', border: '1px solid #334155', borderRadius: '8px', padding: '1.25rem', color: '#f8fafc', fontSize: '1.1rem', fontWeight: 700, textAlign: 'left', cursor: 'pointer' },
  linkButton: { background: 'none', border: 0, color: '#bfdbfe', fontSize: '1rem', cursor: 'pointer', padding: 0 },
  inventoryTitle: { fontSize: '1.5rem', fontWeight: 700 },
  pendingEvent: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '1rem', flexWrap: 'wrap', padding: '0.75rem 0', borderBottom: '1px solid #334155' },
  resultControls: { display: 'flex', gap: '0.5rem', flexWrap: 'wrap' },
  muted: { color: '#94a3b8' },
  banner: { background: '#1e3a8a', padding: '0.75rem', margin: '1rem 0', cursor: 'pointer' },
};

export default App;
