import React, { useEffect, useState } from 'react';
import type { GameState, ServiceType, TimeSpeed } from './types/game';
import { getLabel } from './types/game';
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
  tickGameDay,
} from './services/tauriApi';
import { AlertModal } from './components/AlertModal';
import { ConfigModal } from './components/ConfigModal';

const speeds: TimeSpeed[] = ['Paused', 'OneDayEveryFiveSec', 'OneDayPerSec', 'OneWeekPerSec'];

export const App: React.FC = () => {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [tab, setTab] = useState<'dashboard' | 'inventory' | 'events' | 'actions'>('dashboard');
  const [configOpen, setConfigOpen] = useState(false);
  const [message, setMessage] = useState('');

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
  const eventName = getLabel(catalog, 'event_name', 'Event');
  const eventPlural = getLabel(catalog, 'event_plural', `${eventName}s`);
  const currency = getLabel(catalog, 'currency_symbol', '$');
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
        <h2>Overview</h2>
        <p>Age: {Math.floor(player.age_days / 365)} years</p>
        <p>Budget: {currency}{player.budget.toLocaleString()}</p>
        <p>{inventoryName}: {player.inventory.length}</p>
        <p>Active {getLabel(catalog, 'action_name', 'Actions')}: {player.active_actions.length}</p>
      </section>
      <section style={styles.card}>
        <h2>Available {objectPlural}</h2>
        {catalog.objects.map((object) => (
          <div key={object.id} style={styles.row}>
            <span>{object.name} ({currency}{object.price.toLocaleString()})</span>
            <button onClick={() => run(() => buyObject(object.id), `${objectName} acquired.`)}>Acquire</button>
          </div>
        ))}
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
                  disabled={player.budget < occurrence.amount}
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
          <h2>{object.name}</h2>
          <p>{getLabel(catalog, 'units_name', 'Units')}: {object.units_available}</p>
          {[1, 2, 3, 4].map((index) => (
            <div key={index} style={styles.row}>
              <span>
                {costFor(object, index)?.name || `Cost ${index}`}:
                {' '}{needsService(object, index) ? 'Required' : 'Ready'}
              </span>
              <button
                disabled={!needsService(object, index) || !costFor(object, index) || player.budget < (costFor(object, index)?.amount || 0)}
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

  const renderEvents = () => {
    const currentDay = ((gameState.current_day - 1) % 365) + 1;
    return (
      <div style={styles.grid}>
        {catalog.events.map((event) => (
          <section key={event.id} style={styles.card}>
            <h2>{event.name}</h2>
            <p>Scheduled day: {event.day_of_year}</p>
            <p>Entry: {currency}{event.entry_fee}</p>
            <p>Reward: {currency}{event.reward_pool.toLocaleString()}</p>
            {player.inventory.map((object) => (
              <button
                key={object.id}
                disabled={currentDay !== event.day_of_year}
                onClick={() => run(
                  () => enterEvent(object.id, event.id),
                  `${eventName} completed.`,
                )}
              >
                Participate with {object.name}
              </button>
            ))}
          </section>
        ))}
      </div>
    );
  };

  const renderActions = () => (
    <div style={styles.grid}>
      {catalog.actions.map((action) => (
        <section key={action.id} style={styles.card}>
          <h2>{action.name}</h2>
          <p>Cost: {currency}{action.base_cost}</p>
          <p>Success: {(action.success_rate * 100).toFixed(0)}%</p>
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
        </section>
      ))}
    </div>
  );

  return (
    <div style={styles.app}>
      <header style={styles.header}>
        <div>
          <h1>{getLabel(catalog, 'application_name', 'Economy Engine')}</h1>
          <span>Day {gameState.current_day} | {currency}{player.budget.toLocaleString()}</span>
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
          ['events', eventPlural],
          ['actions', getLabel(catalog, 'action_name', 'Actions')],
        ] as const).map(([key, title]) => (
          <button key={key} onClick={() => setTab(key)}>{title}</button>
        ))}
      </nav>
      <main>
        {tab === 'dashboard' && renderDashboard()}
        {tab === 'inventory' && renderInventory()}
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
  banner: { background: '#1e3a8a', padding: '0.75rem', margin: '1rem 0', cursor: 'pointer' },
};

export default App;
