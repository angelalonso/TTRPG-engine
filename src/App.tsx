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
  loadGame,
  saveGame,
  serviceObject,
  sellObject,
  setTimeSpeed,
  submitEventResult,
  tickGameDay,
} from './services/tauriApi';
import { AlertModal } from './components/AlertModal';
import { ConfigModal } from './components/ConfigModal';
import { DetailModal } from './components/DetailModal';
import { FeedbackModal } from './components/FeedbackModal';
import { ResultPromptModal } from './components/ResultPromptModal';

const speeds: TimeSpeed[] = ['Paused', 'OneDayEveryFiveSec', 'OneDayPerSec', 'OneWeekPerSec'];
const speedIconKeys: Record<TimeSpeed, string> = {
  Paused: 'speed_icon_paused',
  OneDayEveryFiveSec: 'speed_icon_normal',
  OneDayPerSec: 'speed_icon_fast',
  OneWeekPerSec: 'speed_icon_fastest',
  RealTime: 'speed_icon_normal',
};
const headerIconKeys = {
  settings: 'settings_icon',
  save: 'save_icon',
  load: 'load_icon',
};

export const App: React.FC = () => {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [tab, setTab] = useState<'dashboard' | 'inventory' | 'dealer' | 'events' | 'actions'>('dashboard');
  const [configOpen, setConfigOpen] = useState(false);
  const [message, setMessage] = useState('');
  const [detailMessage, setDetailMessage] = useState('');
  const [selectedDetail, setSelectedDetail] = useState<{
    title: string;
    descriptionPath: string;
    footer: React.ReactNode;
  } | null>(null);
  const [resultPrompt, setResultPrompt] = useState<{
    id: string;
    eventName: string;
    damageOptions: Array<{ id: string; name: string }>;
  } | null>(null);
  const [feedback, setFeedback] = useState<{ title: string; message: string } | null>(null);
  const [eventFilter, setEventFilter] = useState('');
  const [eventSort, setEventSort] = useState<'name' | 'days'>('days');

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
  const damageOptions = Array.from(new Map(
    catalog.cost_rules
      .filter((rule) => rule.damage_type.trim())
      .map((rule) => [rule.damage_type, catalog.costs.find((cost) => cost.id === rule.cost_id)?.name || rule.damage_type]),
  )).map(([id, name]) => ({ id, name }));
  const ownedLicenses = player.inventory
    .filter((object) => object.license_level > 0)
    .sort((left, right) => right.license_level - left.license_level);
  const ownedEquipment = player.inventory.filter((object) => object.object_type === 'equipment');
  const objectMatchesId = (object: { id: string }, id: string) =>
    object.id === id || object.id.startsWith(`${id}_`);
  const budget = getCharacteristic(player, 'budget');
  const costReference = (object: (typeof player.inventory)[number], index: number) =>
    (object[`cost_${index}`] as string || '').trim();
  const costFor = (object: (typeof player.inventory)[number], index: number) =>
    catalog.costs.find((cost) => cost.id === costReference(object, index));
  const definedCosts = (object: (typeof player.inventory)[number]) =>
    Array.from({ length: 15 }, (_, index) => index + 1)
      .map((index) => ({
        index,
        id: costReference(object, index),
        cost: costFor(object, index),
      }))
      .filter(({ id }) => id);
  const needsService = (object: (typeof player.inventory)[number], index: number) =>
    object[`service_${index}_needed` as keyof typeof object] as boolean;

  const run = async (operation: () => Promise<unknown>, success: string) => {
    try {
      const result = await operation();
      if (result && typeof result === 'object' && 'message' in result) {
        setMessage(String(result.message));
        setGameState(await getGameState());
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
        <p>Highest licence: {ownedLicenses[0]?.name || 'None'}</p>
        <p>Required equipment: {ownedEquipment.length > 0 ? ownedEquipment.map((object) => object.name).join(', ') : 'None'}</p>
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
              {occurrence.status === 'pending' && occurrence.source_type === 'object_service' && (
                <span style={styles.muted}>Complete in garage</span>
              )}
              {occurrence.status === 'pending' && occurrence.source_type !== 'object_service' && (
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
              footer: null,
            })}
          >
            <span style={styles.inventoryTitle}>{object.name}</span>
          </button>
          {object.unavailable_until_day > gameState.current_day && (
            <p style={styles.muted}>
              Unavailable for {object.unavailable_until_day - gameState.current_day} more day(s)
            </p>
          )}
          {definedCosts(object).map(({ index, id, cost }) => (
            <div key={index} style={styles.row}>
              <span>
                {cost?.name || id} {cost ? `(${currency}${cost.amount.toLocaleString()})` : '(missing cost definition)'}:
                {' '}{index <= 4 ? (needsService(object, index) ? 'Required' : 'Ready') : 'Defined'}
              </span>
              {index <= 4 && (
                <button
                  disabled={!needsService(object, index) || !cost || budget < cost.amount}
                  onClick={() => run(
                    () => serviceObject(object.id, `Service${index}` as ServiceType),
                    `${cost?.name || `Cost ${index}`} completed.`,
                  )}
                >
                  Service
                </button>
              )}
            </div>
          ))}
          <button onClick={() => {
            if (window.confirm(`Sell ${object.name}? This cannot be undone.`)) {
              run(() => sellObject(object.id), 'Object sold.');
            }
          }}>Sell ({currency}{(
            object.price
            * Math.max(
              object.resale_min_percent,
              object.resale_initial_percent
                * Math.pow(object.resale_annual_percent, Math.floor((gameState.current_day - object.purchase_day) / gameState.days_per_year)),
            )
          ).toLocaleString()})</button>
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
              <button
                onClick={async () => {
                  try {
                    const nextState = await buyObject(object.id);
                    setGameState(nextState);
                    setSelectedDetail(null);
                    setFeedback({
                      title: 'Purchase complete',
                      message: `You have purchased ${object.name}.`,
                    });
                  } catch (error) {
                    setMessage(String(error));
                  }
                }}
              >
                Acquire ({currency}{(
                  object.object_type === 'license' && object.license_fee > 0
                    ? object.license_fee
                    : object.price
                ).toLocaleString()})
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
    const eligibleCarsFor = (event: (typeof catalog.events)[number]) => player.inventory.filter((object) => {
      if (object.object_type !== 'vehicle' || object.unavailable_until_day > gameState.current_day) return false;
      if ([1, 2, 3, 4].some((index) => object[`service_${index}_needed` as keyof typeof object])) return false;
      if (event.required_license_id && !player.inventory.some((owned) => objectMatchesId(owned, event.required_license_id))) return false;
      const requiredCars = event.required_object_ids.split(';').map((id) => id.trim()).filter(Boolean);
      return requiredCars.length === 0 || requiredCars.some((id) => objectMatchesId(object, id));
    });
    const visibleEvents = catalog.events
      .map((event) => ({
        event,
        daysLeft: (event.day_of_year - currentDay + gameState.days_per_year) % gameState.days_per_year,
      }))
      .filter(({ event }) => event.name.toLowerCase().includes(eventFilter.toLowerCase()))
      .sort((left, right) => eventSort === 'name'
        ? left.event.name.localeCompare(right.event.name)
        : left.daysLeft - right.daysLeft);
    return (
      <div style={styles.grid}>
        <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
          <label>
            Filter events:{' '}
            <input value={eventFilter} onChange={(event) => setEventFilter(event.target.value)} />
          </label>
          <label style={{ marginLeft: '1rem' }}>
            Sort by:{' '}
            <select value={eventSort} onChange={(event) => setEventSort(event.target.value as 'name' | 'days')}>
              <option value="days">Days left</option>
              <option value="name">Name</option>
            </select>
          </label>
        </section>
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
                    <button
                      onClick={() => setResultPrompt({
                        id: pending.id,
                        eventName: event.name,
                        damageOptions: event.tags.split(';').map((tag) => tag.trim()).includes('race')
                          ? damageOptions
                          : [],
                      })}
                    >
                      Enter result
                    </button>
                  </div>
                </div>
              );
            })}
          </section>
        )}
        {visibleEvents.map(({ event, daysLeft }) => (
          <button
            key={event.id}
            style={{
              ...styles.nameCard,
              background: daysLeft === 0 ? '#166534' : styles.nameCard.background,
            }}
            onClick={() => {
              setDetailMessage('');
              setSelectedDetail({
                title: event.name,
                descriptionPath: event.description_html,
              footer: (
                <>
                  <span>
                    {dayLabel}: {event.day_of_year} | {daysLeft === 0 ? 'Today' : `${daysLeft} days left`}
                    {' '}| Entry: {currency}{event.entry_fee}
                    {' '}| Duration: {event.duration_value} {event.duration_unit}
                    {' '}| Reward: {currency}{event.reward_pool} + {event.charisma_reward} charisma
                  </span>
                  {eligibleCarsFor(event).map((object) => (
                    <button
                      key={object.id}
                      disabled={currentDay !== event.day_of_year}
                      onClick={async () => {
                        try {
                          const nextState = await enterEvent(object.id, event.id);
                          setGameState(nextState);
                          setSelectedDetail(null);
                          const matchingPending = nextState.pending_events
                            .filter((entry) => entry.event_id === event.id && entry.object_id === object.id);
                          const pending = matchingPending[matchingPending.length - 1];
                          if (pending) {
                            setResultPrompt({
                              id: pending.id,
                              eventName: event.name,
                              damageOptions: event.tags.split(';').map((tag) => tag.trim()).includes('race')
                                ? damageOptions
                                : [],
                            });
                          }
                        } catch (error) {
                          setDetailMessage(String(error));
                        }
                      }}
                    >
                      Enter with {object.name}
                    </button>
                  ))}
                  {eligibleCarsFor(event).length === 0 && (
                    <span style={styles.muted}>No eligible cars available.</span>
                  )}
                </>
              ),
                });
            }}
          >
            <span>{event.name}</span>
            <small style={styles.eventDays}>{daysLeft} days left</small>
          </button>
        ))}
        {catalog.championships.map((championship) => {
          const championshipEvents = catalog.events.filter((event) => event.championship_id === championship.id);
          const completed = gameState.event_history.filter((history) =>
            championshipEvents.some((event) => event.id === history.event_id),
          );
          if (championshipEvents.length === 0) return null;
          const points = completed.reduce((total, history) =>
            total + (history.outcome.toLowerCase() === 'success'
              ? championship.success_points
              : championship.failure_points), 0);
          return (
            <section key={championship.id} style={{ ...styles.card, gridColumn: '1 / -1' }}>
              <h2>{championship.name}</h2>
              <p>
                {completed.length}/{championshipEvents.length} rounds completed | Points: {points}
                {completed.length === championshipEvents.length ? ' | Final result calculated' : ''}
              </p>
            </section>
          );
        })}
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
                  {(history.reward_awarded > 0 || history.charisma_reward_awarded > 0) && (
                    <span>
                      {history.reward_awarded > 0 && `${currency}${history.reward_awarded.toLocaleString()}`}
                      {history.reward_awarded > 0 && history.charisma_reward_awarded > 0 && ' | '}
                      {history.charisma_reward_awarded > 0 && `+${history.charisma_reward_awarded} charisma`}
                    </span>
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
                <span> | Stamina: {action.stamina_cost}</span>
                <button
                  disabled={getCharacteristic(player, 'stamina') < action.stamina_cost}
                  onClick={() => {
                    setSelectedDetail(null);
                    return run(
                  async () => {
                    const result = await performAction(action.id);
                    setGameState(await getGameState());
                    return result;
                  },
                  'Action completed.',
                    );
                  }}
                >
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
            <button
              key={speed}
              title={speed}
              aria-label={speed}
              style={{
                ...styles.speedButton,
                background: gameState.time_speed === speed
                  ? speed === 'Paused' ? '#bfdbfe'
                    : speed === 'OneDayEveryFiveSec' ? '#bbf7d0'
                      : speed === 'OneDayPerSec' ? '#fef08a' : '#fed7aa'
                  : '#ffffff',
              }}
              onClick={() => setTimeSpeed(speed).then(setGameState)}
            >
              <img
                src={getLabel(catalog, speedIconKeys[speed], '')}
                alt=""
                style={{ width: 18, height: 18 }}
                onError={(event) => { event.currentTarget.style.display = 'none'; }}
              />
            </button>
          ))}
          <button title="Save" aria-label="Save" onClick={async () => {
            try {
              const path = await saveGame();
              setFeedback({ title: 'Game saved', message: `Game saved to ${path}.` });
            } catch (error) {
              setMessage(String(error));
            }
          }}>
            <img src={getLabel(catalog, headerIconKeys.save, '/img/save.svg')} alt="" style={{ width: 18, height: 18 }} />
          </button>
          <button title="Load" aria-label="Load" onClick={async () => {
            if (!window.confirm('Load the saved game and overwrite the current game status?')) return;
            try {
              setGameState(await loadGame());
              setFeedback({ title: 'Game loaded', message: 'The saved game has replaced the current game status.' });
            } catch (error) {
              setMessage(String(error));
            }
          }}>
            <img src={getLabel(catalog, headerIconKeys.load, '/img/load.svg')} alt="" style={{ width: 18, height: 18 }} />
          </button>
          <button title="Settings" aria-label="Settings" onClick={() => setConfigOpen(true)}>
            <img src={getLabel(catalog, headerIconKeys.settings, '/img/settings.svg')} alt="" style={{ width: 18, height: 18 }} />
          </button>
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
          <button
            key={key}
            style={key === 'dashboard' ? styles.dashboardTab : undefined}
            onClick={() => setTab(key)}
          >
            {title}
          </button>
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
          message={detailMessage}
          onClose={() => {
            setSelectedDetail(null);
            setDetailMessage('');
          }}
        >
          {selectedDetail.footer}
          <button onClick={() => {
            setSelectedDetail(null);
            setDetailMessage('');
          }}>Close</button>
        </DetailModal>
      )}
      {resultPrompt && (
        <ResultPromptModal
          eventName={resultPrompt.eventName}
          damageOptions={resultPrompt.damageOptions}
          onClose={() => setResultPrompt(null)}
          onSubmit={async (result, damageType) => {
            try {
              const eventResult = await submitEventResult(resultPrompt.id, result, damageType);
              setResultPrompt(null);
              setGameState(await getGameState());
              setFeedback({
                title: 'Event finished',
                message: `Event ${eventResult.event_name} finished with result "${result}".`,
              });
            } catch (error) {
              setMessage(String(error));
            }
          }}
        />
      )}
      {feedback && (
        <FeedbackModal
          title={feedback.title}
          message={feedback.message}
          onClose={() => setFeedback(null)}
        />
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  app: { minHeight: '100vh', padding: '1.5rem', background: '#0f172a', color: '#f8fafc', fontFamily: 'sans-serif' },
  loading: { minHeight: '100vh', display: 'grid', placeItems: 'center', background: '#0f172a', color: '#f8fafc' },
  speedButton: {
    width: 36,
    height: 32,
    padding: 5,
    marginLeft: 4,
    border: '1px solid #94a3b8',
    borderRadius: 4,
    color: '#0f172a',
    cursor: 'pointer',
  },
  dashboardTab: {
    fontSize: '1.15rem',
    fontWeight: 800,
    padding: '0.75rem 1rem',
  },
  header: { display: 'flex', justifyContent: 'space-between', gap: '1rem', borderBottom: '1px solid #334155', paddingBottom: '1rem' },
  nav: { display: 'flex', gap: '0.5rem', margin: '1rem 0' },
  grid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1rem' },
  card: { background: '#1e293b', border: '1px solid #334155', borderRadius: '8px', padding: '1rem' },
  row: { display: 'flex', justifyContent: 'space-between', gap: '1rem', alignItems: 'center', padding: '0.5rem 0', borderBottom: '1px solid #334155' },
  nameCard: { background: '#1e293b', border: '1px solid #334155', borderRadius: '8px', padding: '1.25rem', color: '#f8fafc', fontSize: '1.1rem', fontWeight: 700, textAlign: 'left', cursor: 'pointer' },
  eventDays: { display: 'block', marginTop: '0.35rem', color: '#cbd5e1', fontSize: '0.85rem', fontWeight: 400 },
  linkButton: { background: 'none', border: 0, color: '#bfdbfe', fontSize: '1rem', cursor: 'pointer', padding: 0 },
  inventoryTitle: { fontSize: '1.5rem', fontWeight: 700 },
  pendingEvent: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '1rem', flexWrap: 'wrap', padding: '0.75rem 0', borderBottom: '1px solid #334155' },
  resultControls: { display: 'flex', gap: '0.5rem', flexWrap: 'wrap' },
  muted: { color: '#94a3b8' },
  banner: { background: '#1e3a8a', padding: '0.75rem', margin: '1rem 0', cursor: 'pointer' },
};

export default App;
