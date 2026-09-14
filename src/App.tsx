import React, { useEffect, useState } from 'react';
import type { ChampionshipCompetitor, GameState, ServiceType, TimeSpeed } from './types/game';
import { getCharacteristic, getLabel } from './types/game';
import {
  buyObject,
  dismissAlert,
  enterEvent,
  getGameState,
  getThemeColors,
  joinQuest,
  loadDatasetAsset,
  performAction,
  quitAction,
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
import { ConfirmModal } from './components/ConfirmModal';
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
const defaultSpeedIcons: Record<TimeSpeed, string> = {
  Paused: '/img/pause.svg',
  OneDayEveryFiveSec: '/img/normal.svg',
  OneDayPerSec: '/img/fast.svg',
  OneWeekPerSec: '/img/fastest.svg',
  RealTime: '/img/normal.svg',
};
const headerIconKeys = {
  settings: 'settings_icon',
  save: 'save_icon',
  load: 'load_icon',
};

export const App: React.FC = () => {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [tab, setTab] = useState('dashboard');
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
    championship?: boolean;
    previousCompetitors?: ChampionshipCompetitor[];
    championshipDrivers?: string[];
    scoringPositions?: number;
  } | null>(null);
  const [feedback, setFeedback] = useState<{ title: string; message: string } | null>(null);
  const [eventFilter, setEventFilter] = useState('');
  const [eventSort, setEventSort] = useState<'name' | 'days'>('days');
  const [championshipFilter, setChampionshipFilter] = useState('');
  const [championshipSort, setChampionshipSort] = useState<'name' | 'races' | 'status'>('name');
  const [inventoryTab, setInventoryTab] = useState('service_bay');
  const [marketCategory, setMarketCategory] = useState<string | null>(null);
  const [marketFilter, setMarketFilter] = useState('');
  const [marketSort, setMarketSort] = useState<'name' | 'price'>('name');
  const [marketImages, setMarketImages] = useState<Record<string, string>>({});
  const [marketError, setMarketError] = useState('');
  const [confirmation, setConfirmation] = useState<{
    title: string;
    message: string;
    confirmLabel: string;
    onConfirm: () => void;
  } | null>(null);

  useEffect(() => {
    Promise.all([getGameState(), getThemeColors()])
      .then(([state, colors]) => {
        setGameState(state);
        for (const [elementId, color] of Object.entries(colors)) {
          document.documentElement.style.setProperty(`--${elementId.replaceAll('_', '-')}`, color);
        }
      })
      .catch((error) => setMessage(String(error)));
  }, []);

  useEffect(() => {
    if (!gameState || gameState.time_speed === 'Paused' || gameState.pending_alerts.length > 0) return;
    const interval = gameState.time_speed === 'OneDayEveryFiveSec' ? 5000
      : gameState.time_speed === 'OneWeekPerSec' ? 1000 / 7 : 1000;
    const timer = setInterval(() => tickGameDay().then(setGameState).catch((error) => setMessage(String(error))), interval);
    return () => clearInterval(timer);
  }, [gameState?.time_speed, gameState?.pending_alerts.length]);

  useEffect(() => {
    if (!gameState) return;
    const imageObjects = gameState.catalog.objects
      .map((object) => ({ object, path: (object.image_path || '').trim() || `./img/${object.id}.jpeg` }));
    Promise.all(imageObjects.map(async ({ object, path }) => {
      try {
        return [object.id, await loadDatasetAsset(path)] as const;
      } catch {
        return null;
      }
    })).then((entries) => {
      setMarketImages(Object.fromEntries(entries.filter((entry): entry is readonly [string, string] => entry !== null)));
    });
    const configuredSort = getLabel(gameState.catalog, 'market_default_sort', 'price');
    if (configuredSort === 'name' || configuredSort === 'price') setMarketSort(configuredSort);
  }, [gameState?.catalog]);

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
  const incomeSourcesLabel = getLabel(catalog, 'income_sources_name', 'Income sources');
  const objectMatchesId = (object: { id: string }, id: string) =>
    object.id === id || object.id.startsWith(`${id}_`);
  const catalogObjectType = (object: { object_type?: string; type?: string }) =>
    (object.object_type || object.type || '').trim();
  const inventoryTabs = (() => {
    const configured = new Map<string, { name?: string; types?: string }>();
    Object.entries(catalog.labels.values).forEach(([key, value]) => {
      const match = key.match(/^inventory_tab_(.+)_(name|types)$/);
      if (!match) return;
      const entry = configured.get(match[1]) || {};
      entry[match[2] as 'name' | 'types'] = value;
      configured.set(match[1], entry);
    });
    return [
      {
        id: 'service_bay',
        name: getLabel(catalog, 'inventory_service_bay_name', 'Service Bay'),
        types: ['vehicle'],
      },
      {
        id: 'drivers_room',
        name: getLabel(catalog, 'inventory_tab_drivers_room_name', "Driver's Room"),
        types: ['equipment', 'license'],
      },
      ...Array.from(configured.entries())
        .filter(([id, entry]) => !['service_bay', 'drivers_room'].includes(id) && entry.name && entry.types)
        .map(([id, entry]) => ({
          id,
          name: entry.name as string,
          types: (entry.types as string).split(';').map((type) => type.trim()).filter(Boolean),
        })),
    ];
  })();
  const activeInventoryTab = inventoryTabs.find((entry) => entry.id === inventoryTab) || inventoryTabs[0];
  const inventoryObjects = player.inventory.filter((object) =>
    activeInventoryTab.types.includes(object.object_type),
  );
  const raceGear = catalog.objects.filter((object) => catalogObjectType(object) === 'equipment');
  const raceGearReady = raceGear.length > 0 && raceGear.every((required) =>
    player.inventory.some((owned) => objectMatchesId(owned, required.id)),
  );
  const damageOptions = Array.from(new Map(
    catalog.cost_rules
      .filter((rule) => rule.damage_type.trim())
      .map((rule) => [rule.damage_type, catalog.costs.find((cost) => cost.id === rule.cost_id)?.name || rule.damage_type]),
  )).map(([id, name]) => ({ id, name }));
  const ownedLicenses = player.inventory
    .filter((object) => object.license_level > 0)
    .sort((left, right) => right.license_level - left.license_level);
  const ownedEquipment = player.inventory.filter((object) => object.object_type === 'equipment');
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
  const statBar = (id: string, value: number) => {
    const definition = catalog.player_characteristics.find((entry) => entry.id === id);
    if (!definition || !Number.isFinite(definition.max_value)) return null;
    const minimum = Number.isFinite(definition.min_value) ? definition.min_value : 0;
    const percentage = Math.max(0, Math.min(100,
      ((value - minimum) / Math.max(1, definition.max_value - minimum)) * 100,
    ));
    return (
      <span style={styles.statBar}>
        <span style={{ ...styles.statBarFill, width: `${percentage}%`, background: percentage > 60 ? 'var(--progress-high)' : percentage > 30 ? 'var(--progress-medium)' : 'var(--progress-low)' }} />
      </span>
    );
  };
  const formatGameDay = (day: number) => {
    const year = Math.floor((day - 1) / gameState.days_per_year) + 1;
    const dayOfYear = ((day - 1) % gameState.days_per_year) + 1;
    return `Year ${year}, Day ${dayOfYear}`;
  };

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
        <img src="/img/race_driver_grey.jpeg" alt="Race driver" style={styles.dashboardImage} />
        <p>{ageLabel}: {Math.floor(player.age_days / gameState.days_per_year)} years</p>
        <p>{budgetLabel}: {currency}{budget.toLocaleString()}</p>
        {catalog.player_characteristics
          .filter((characteristic) => characteristic.id !== 'budget')
          .filter((characteristic) => characteristic.id !== 'age')
          .map((characteristic) => (
            <p key={characteristic.id}>
              {characteristic.name}: {getCharacteristic(player, characteristic.id)}
              {statBar(characteristic.id, getCharacteristic(player, characteristic.id))}
            </p>
          ))}
        <p>{inventoryName}: {player.inventory.length}</p>
        <p>Highest licence: {ownedLicenses[0]?.name || 'None'}</p>
        <p style={{ color: raceGearReady ? 'var(--success-text)' : 'var(--error-text)' }}>
          Race gear: {raceGearReady ? 'Ready for racing' : 'Not ready - buy all required gear'}
        </p>
        <p>Owned equipment: {ownedEquipment.length > 0 ? ownedEquipment.map((object) => object.name).join(', ') : 'None'}</p>
        <p>{incomeSourcesLabel}: {player.active_actions.length}</p>
      </section>
      <section style={styles.card}>
        <h2>Event Log</h2>
        {(gameState.event_log || []).length === 0 && <p>No events recorded yet.</p>}
        {(gameState.event_log || []).slice(-8).reverse().map((entry) => {
          return (
            <div key={entry.id} style={styles.row}>
              <span>{formatGameDay(entry.day)}</span>
              <span>{entry.event}</span>
            </div>
          );
        })}
      </section>
    </div>
  );

  const renderInventory = () => (
    <div style={styles.grid}>
      <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
        <div style={styles.subnav}>
          {inventoryTabs.map((entry) => (
            <button
              key={entry.id}
              onClick={() => setInventoryTab(entry.id)}
              style={entry.id === activeInventoryTab.id ? styles.selectedPill : styles.pillButton}
            >
              {entry.name}
            </button>
          ))}
        </div>
      </section>
      {inventoryObjects.length === 0 && <p>No objects in {activeInventoryTab.name.toLowerCase()}.</p>}
      {inventoryObjects.map((object) => (
        <section
          key={object.id}
          style={{
            ...styles.card,
            opacity: object.unavailable_until_day > gameState.current_day ? 0.6 : 1,
          }}
        >
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
            <p style={styles.unavailableNotice}>
              Not yet available: {object.unavailable_until_day - gameState.current_day} more day(s)
            </p>
          )}
          {object.lifetime_days > 0 && object.expires_day > gameState.current_day && (
            <p style={styles.muted}>
              Replace in {object.expires_day - gameState.current_day} day(s)
            </p>
          )}
          {object.loaned && (
            <p style={styles.unavailableNotice}>
              Loaned sponsor car; returned on day {object.expires_day}.
            </p>
          )}
          {definedCosts(object).map(({ index, id, cost }) => (
            <div key={index} style={styles.row}>
              <span>
                {cost?.name || id} {cost ? `(${currency}${cost.amount.toLocaleString()})` : '(missing cost definition)'}:
                {' '}{needsService(object, index) ? 'Required' : 'Ready'}
              </span>
              <button
                disabled={!needsService(object, index) || !cost || budget < cost.amount}
                onClick={() => run(
                  () => serviceObject(object.id, `Service${index}` as ServiceType),
                  `${cost?.name || `Cost ${index}`} completed.`,
                )}
              >
                Service
              </button>
            </div>
          ))}
          {object.loaned ? (
            <p style={styles.muted}>Loaned sponsor cars cannot be sold.</p>
          ) : object.object_type === 'license' ? (
            <p style={styles.muted}>Licences cannot be resold.</p>
          ) : (
            <button onClick={() => setConfirmation({
              title: 'Sell object?',
              message: `Sell ${object.name}? This action cannot be undone.`,
              confirmLabel: 'Sell',
              onConfirm: () => {
                setConfirmation(null);
                void run(() => sellObject(object.id), 'Object sold.');
              },
            })}>Sell ({currency}{(
              object.price
              * Math.max(
                object.resale_min_percent,
                object.resale_initial_percent
                  * Math.pow(object.resale_annual_percent, Math.floor((gameState.current_day - object.purchase_day) / gameState.days_per_year)),
              )
            ).toLocaleString()})</button>
          )}
        </section>
      ))}
    </div>
  );

  const renderDealer = () => {
    const marketTypes = Array.from(new Set(catalog.objects.map(catalogObjectType).filter(Boolean)));
    const selectedType = marketCategory || marketTypes[0] || '';
    const marketObjects = catalog.objects
      .filter((object) => catalogObjectType(object) === selectedType)
      .filter((object) => object.name.toLowerCase().includes(marketFilter.toLowerCase()))
      .sort((left, right) => marketSort === 'name'
        ? left.name.localeCompare(right.name)
        : (left.license_fee > 0 ? left.license_fee : left.price)
          - (right.license_fee > 0 ? right.license_fee : right.price));
    return (
    <div style={styles.market}>
      <div style={styles.subnav}>
        {marketTypes.map((type) => (
          <button
            key={type}
            style={type === selectedType ? styles.selectedPill : styles.pillButton}
            onClick={() => {
              setMarketCategory(type);
              setMarketError('');
            }}
          >
            {getLabel(catalog, `market_category_${type}`, `${type.charAt(0).toUpperCase()}${type.slice(1)}`)}
          </button>
        ))}
      </div>
      <div style={styles.marketControls}>
        <input
          value={marketFilter}
          onChange={(event) => setMarketFilter(event.target.value)}
          placeholder={`Filter ${getLabel(catalog, `market_category_${selectedType}`, selectedType)}`}
          aria-label="Filter market items"
        />
        <select value={marketSort} onChange={(event) => setMarketSort(event.target.value as 'name' | 'price')}>
          <option value="name">Sort by name</option>
          <option value="price">Sort by price</option>
        </select>
      </div>
      {marketError && <div style={styles.errorBanner}>{marketError}</div>}
      <div style={styles.grid}>
        {marketObjects.length === 0 && <p style={styles.muted}>No matching items in this category.</p>}
        {marketObjects.map((object) => (
          <section key={object.id} style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <button
              style={styles.marketItemButton}
              onClick={() => setSelectedDetail({
                title: object.name,
                descriptionPath: object.description_html,
                footer: null,
              })}
            >
              {marketImages[object.id] && (
                <img src={marketImages[object.id]} alt="" style={styles.marketThumbnail} />
              )}
              <span style={styles.inventoryTitle}>{object.name}</span>
            </button>
            <p>{currency}{(catalogObjectType(object) === 'license' && object.license_fee > 0
              ? object.license_fee : object.price).toLocaleString()}</p>
            <button onClick={async () => {
              setMarketError('');
              try {
                setGameState(await buyObject(object.id));
                setFeedback({ title: 'Purchase complete', message: `You have purchased ${object.name}.` });
              } catch (error) {
                setMarketError(String(error));
              }
            }}>
              Buy
            </button>
          </section>
        ))}
      </div>
    </div>
    );
  };

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
    const isMember = (questId: string) => (gameState.quest_memberships || [])
      .some((membership) => membership.quest_id === questId);
    const questForEvent = (event: (typeof catalog.events)[number]) =>
      event.quest_id ? catalog.quests.find((quest) => quest.id === event.quest_id) : undefined;
    const eventKind = (event: (typeof catalog.events)[number]) => {
      const tags = event.tags.split(';').map((tag) => tag.trim().toLowerCase());
      if (tags.includes('social')) return 'Social event';
      if (tags.includes('track_day')) return 'Track day';
      if (tags.includes('race')) return 'Race';
      return 'Event';
    };
    const previousChampionshipCompetitors = (questId: string) => {
      const previousResults = (gameState.championship_results || [])
        .filter((result) => catalog.events.find((event) => event.id === result.event_id)?.quest_id === questId)
        .sort((left, right) => right.race_day - left.race_day);
      const byName = new Map<string, ChampionshipCompetitor>();
      previousResults.forEach((result) => result.competitors.forEach((competitor) => {
        if (competitor.name.trim() && !byName.has(competitor.name.trim())) {
          byName.set(competitor.name.trim(), competitor);
        }
      }));
      return Array.from(byName.values()).sort((left, right) => left.position - right.position);
    };
    const championshipDrivers = (questId: string) => {
      const quest = catalog.quests.find((entry) => entry.id === questId);
      const configured = (quest?.driver_names || '').split(';').map((name) => name.trim()).filter(Boolean);
      return Array.from(new Set([
        ...configured,
        ...previousChampionshipCompetitors(questId).map((competitor) => competitor.name),
      ]));
    };
    const scoringPositions = (questId: string) => Math.max(
      1,
      (catalog.quests.find((quest) => quest.id === questId)?.championship_rewards || '')
        .split(';')
        .map((entry) => Number(entry.split(':')[0]?.trim()))
        .filter((position) => Number.isInteger(position) && position > 0)
        .reduce((highest, position) => Math.max(highest, position), 3),
    );
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
                        damageOptions,
                        championship: Boolean(event.quest_id),
                        previousCompetitors: event.quest_id ? previousChampionshipCompetitors(event.quest_id) : [],
                        championshipDrivers: event.quest_id ? championshipDrivers(event.quest_id) : [],
                        scoringPositions: event.quest_id ? scoringPositions(event.quest_id) : 1,
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
              gridColumn: '1 / -1',
              width: '100%',
              background: event.quest_id
                ? isMember(event.quest_id) ? 'var(--warning-background)' : 'var(--surface-border)'
                : daysLeft === 0 ? 'var(--success-background)' : styles.nameCard.background,
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
                    {' '}| Type: {eventKind(event)}
                    {' '}| Entry: {currency}{event.entry_fee}
                    {' '}| Duration: {event.duration_value} {event.duration_unit}
                    {' '}| Reward: {currency}{event.reward_pool} + {event.charisma_reward} charisma
                  </span>
                  {event.quest_id && !isMember(event.quest_id) && (
                    <span style={styles.muted}>
                      Join {questForEvent(event)?.name || 'the quest'} to enter this event.
                    </span>
                  )}
                  {(!event.quest_id || isMember(event.quest_id)) && eligibleCarsFor(event).map((object) => (
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
                              damageOptions,
                              championship: Boolean(event.quest_id),
                              previousCompetitors: event.quest_id ? previousChampionshipCompetitors(event.quest_id) : [],
                              championshipDrivers: event.quest_id ? championshipDrivers(event.quest_id) : [],
                              scoringPositions: event.quest_id ? scoringPositions(event.quest_id) : 1,
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
                  {(history.reward_awarded !== 0 || history.charisma_reward_awarded !== 0) && (
                    <span>
                      {history.reward_awarded > 0 && `${currency}${history.reward_awarded.toLocaleString()}`}
                      {history.reward_awarded !== 0 && history.charisma_reward_awarded !== 0 && ' | '}
                      {history.charisma_reward_awarded !== 0
                        && `${history.charisma_reward_awarded > 0 ? '+' : ''}${history.charisma_reward_awarded} charisma`}
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
      {player.active_actions.map((active) => {
        const action = catalog.actions.find((entry) => entry.id === active.action_id);
        if (!action) return null;
        const sponsorObject = action.sponsor_object_id
          ? catalog.objects.find((object) => object.id === action.sponsor_object_id)
          : undefined;
        return (
          <section key={`active-${active.action_id}`} style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <strong>{incomeSourcesLabel}: {action.name}</strong>
            {action.type.toLowerCase() === 'sponsor' && (
              <p style={styles.muted}>
                {sponsorObject?.name || action.sponsor_object_id} loaned until the end of the year.
                {' '}Payments: {action.sponsor_payouts || 'configured by sponsor'}.
              </p>
            )}
            <button onClick={() => setConfirmation({
              title: action.type.toLowerCase() === 'work' ? 'Quit job?' : 'Stop action?',
              message: `Stop ${action.name}? You will no longer receive its future payments.`,
              confirmLabel: action.type.toLowerCase() === 'work' ? 'Quit job' : 'Stop action',
              onConfirm: async () => {
                setConfirmation(null);
                try {
                  await quitAction(action.id).then(setGameState);
                  const messages = action.type.toLowerCase() === 'work'
                    ? [
                      `You quit ${action.name}. The next paycheque will not arrive.`,
                      `You handed in your notice for ${action.name}. Time to look for something new.`,
                      `You left ${action.name}. Your income source has been removed.`,
                    ]
                    : [`You stopped ${action.name}.`];
                  setFeedback({ title: action.type.toLowerCase() === 'work' ? 'Job quit' : 'Action stopped', message: messages[Math.floor(Math.random() * messages.length)] });
                } catch (error) {
                  setMessage(String(error));
                }
              },
            })}>{action.type.toLowerCase() === 'work' ? 'Quit job' : 'Stop action'}</button>
          </section>
        );
      })}
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
                <span> | Stamina: {Math.round(action.stamina_cost)}</span>
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

  const renderChampionships = () => {
    const championships = catalog.quests
      .filter((quest) => quest.type.toLowerCase() === 'championship')
      .filter((quest) => quest.name.toLowerCase().includes(championshipFilter.toLowerCase()))
      .sort((left, right) => {
        if (championshipSort === 'races') {
          return catalog.events.filter((event) => event.quest_id === left.id).length
            - catalog.events.filter((event) => event.quest_id === right.id).length;
        }
        if (championshipSort === 'status') {
          const leftJoined = gameState.quest_memberships.some((membership) => membership.quest_id === left.id);
          const rightJoined = gameState.quest_memberships.some((membership) => membership.quest_id === right.id);
          return Number(rightJoined) - Number(leftJoined) || left.name.localeCompare(right.name);
        }
        return left.name.localeCompare(right.name);
      });
    return (
    <div style={styles.grid}>
      <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
        <label>
          Filter championships:{' '}
          <input
            value={championshipFilter}
            onChange={(event) => setChampionshipFilter(event.target.value)}
            placeholder="Search by championship name"
          />
        </label>
        <label style={{ marginLeft: '1rem' }}>
          Sort by:{' '}
          <select value={championshipSort} onChange={(event) => setChampionshipSort(event.target.value as 'name' | 'races' | 'status')}>
            <option value="name">Name</option>
            <option value="races">Number of races</option>
            <option value="status">Joined status</option>
          </select>
        </label>
      </section>
      {championships.map((quest) => {
        const races = catalog.events.filter((event) => event.quest_id === quest.id);
        const rewards = (value: string | undefined, fallback: number) => {
          const entries = (value || '').split(';').map((entry) => {
            const [position, amount] = entry.split(':').map((part) => part.trim());
            return { position: Number(position), amount: Number(amount) };
          }).filter((entry) => Number.isInteger(entry.position) && entry.position > 0 && Number.isFinite(entry.amount));
          return entries.length > 0 ? entries.sort((a, b) => a.position - b.position) : [
            { position: 1, amount: fallback },
            { position: 2, amount: Math.round(fallback * 0.6) },
            { position: 3, amount: Math.round(fallback * 0.4) },
          ];
        };
        const points = new Map<string, number>();
        const score = (position: number) => position > 0 ? Math.max(0, races.length - position + 1) : 0;
        (gameState.championship_results || [])
          .filter((result) => races.some((race) => race.id === result.event_id))
          .forEach((result) => {
            points.set('You', (points.get('You') || 0) + score(result.player_position));
            result.competitors.forEach((competitor) => points.set(
              competitor.name,
              (points.get(competitor.name) || 0) + score(competitor.position),
            ));
          });
        return (
          <section key={quest.id} style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <h2>
              <button
                style={styles.linkButton}
                onClick={() => setSelectedDetail({
                  title: quest.name,
                  descriptionPath: quest.description_html,
                  footer: (
                    <>
                      <strong>Race prizes by position</strong>
                      {races.map((race) => (
                        <span key={race.id}>
                          {race.name}: {rewards(race.position_rewards, race.reward_pool).map((prize) =>
                            `${prize.position}${prize.position === 1 ? 'st' : prize.position === 2 ? 'nd' : prize.position === 3 ? 'rd' : 'th'} ${currency}${prize.amount.toLocaleString()}`,
                          ).join(' | ')}
                        </span>
                      ))}
                      <strong>Championship prizes by final position</strong>
                      <span>{rewards(quest.championship_rewards, quest.join_fee).map((prize) =>
                        `${prize.position}${prize.position === 1 ? 'st' : prize.position === 2 ? 'nd' : prize.position === 3 ? 'rd' : 'th'} ${currency}${prize.amount.toLocaleString()}`,
                      ).join(' | ')}</span>
                    </>
                  ),
                })}
              >
                {quest.name}
              </button>
            </h2>
            <p>{races.length} races | {points.get('You') || 0} points</p>
            <p><strong>Race prizes by position</strong></p>
            {races.map((race) => (
              <div key={race.id} style={styles.row}>
                <span>{race.name}</span>
                <span>{rewards(race.position_rewards, race.reward_pool).map((prize) =>
                  `${prize.position}${prize.position === 1 ? 'st' : prize.position === 2 ? 'nd' : prize.position === 3 ? 'rd' : 'th'} ${currency}${prize.amount.toLocaleString()}`,
                ).join(' | ')}</span>
              </div>
            ))}
            <p><strong>Championship prizes by final position</strong></p>
            <div style={styles.row}>
              {rewards(quest.championship_rewards, quest.join_fee).map((prize) =>
                <span key={prize.position}>{prize.position}{prize.position === 1 ? 'st' : prize.position === 2 ? 'nd' : prize.position === 3 ? 'rd' : 'th'} {currency}{prize.amount.toLocaleString()}</span>,
              )}
            </div>
            {!gameState.quest_memberships.some((membership) => membership.quest_id === quest.id) && (
              <button onClick={() => joinQuest(quest.id).then(setGameState).catch((error) => setMessage(String(error)))}>
                Join for {currency}{quest.join_fee.toLocaleString()}
              </button>
            )}
            <table style={styles.standings}>
              <thead><tr><th>Driver</th><th>Points</th></tr></thead>
              <tbody>{Array.from(points.entries()).sort((a, b) => b[1] - a[1]).map(([name, value]) => (
                <tr key={name}><td>{name}</td><td>{value}</td></tr>
              ))}</tbody>
            </table>
          </section>
        );
      })}
      {championships.length === 0 && (
        <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
          {championshipFilter ? 'No championships match the current filter.' : 'No championships configured.'}
        </section>
      )}
    </div>
    );
  };

  return (
    <div style={styles.app}>
      <header style={styles.header}>
        <div>
          <h1>{getLabel(catalog, 'application_name', 'Economy Engine')}</h1>
          <span>{formatGameDay(gameState.current_day)} | {currency}{budget.toLocaleString()}</span>
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
                  ? speed === 'Paused' ? 'var(--link-text)'
                    : speed === 'OneDayEveryFiveSec' ? 'var(--speed-normal-text)'
                      : speed === 'OneDayPerSec' ? 'var(--speed-fast-text)' : 'var(--speed-fastest-text)'
                  : 'var(--white-text)',
              }}
              onClick={() => setTimeSpeed(speed).then(setGameState)}
            >
              <img
              src={getLabel(catalog, speedIconKeys[speed], defaultSpeedIcons[speed])}
              alt={speed}
              style={{ width: 18, height: 18, display: 'block' }}
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
          <button title="Load" aria-label="Load" onClick={() => setConfirmation({
            title: 'Load saved game?',
            message: 'Loading will overwrite the current game status. Continue?',
            confirmLabel: 'Load',
            onConfirm: async () => {
              setConfirmation(null);
              try {
                setGameState(await loadGame());
                setFeedback({ title: 'Game loaded', message: 'The saved game has replaced the current game status.' });
              } catch (error) {
                setMessage(String(error));
              }
            },
          })}>
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
          ['championships', 'Championships'],
          ['actions', getLabel(catalog, 'action_name', 'Actions')],
        ] as const).map(([key, title]) => (
          <button
            key={key}
            style={key === tab
              ? { ...styles.navTab, ...styles.selectedTab, ...(key === 'dashboard' ? styles.dashboardTab : {}) }
              : { ...styles.navTab, ...(key === 'dashboard' ? styles.dashboardTab : {}) }}
            onClick={() => {
              if (key === 'inventory') setInventoryTab('service_bay');
              setTab(key);
            }}
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
        {tab === 'championships' && renderChampionships()}
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
          championship={resultPrompt.championship}
          previousCompetitors={resultPrompt.previousCompetitors}
          championshipDrivers={resultPrompt.championshipDrivers}
          scoringPositions={resultPrompt.scoringPositions}
          onClose={() => setResultPrompt(null)}
          onSubmit={async (result, damageType) => {
            const eventResult = await submitEventResult(resultPrompt.id, result, damageType);
            setResultPrompt(null);
            setGameState(await getGameState());
            setFeedback({
              title: 'Event finished',
              message: eventResult.message,
            });
          }}
          onSubmitChampionship={async (
            result: string,
            damageType: string,
            playerPosition: number,
            competitors: ChampionshipCompetitor[],
          ) => {
            const eventResult = await submitEventResult(
              resultPrompt.id,
              result,
              damageType,
              playerPosition,
              competitors,
            );
            setResultPrompt(null);
            setGameState(await getGameState());
            setFeedback({ title: 'Championship result recorded', message: eventResult.message });
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
      {confirmation && (
        <ConfirmModal
          title={confirmation.title}
          message={confirmation.message}
          confirmLabel={confirmation.confirmLabel}
          onConfirm={confirmation.onConfirm}
          onCancel={() => setConfirmation(null)}
        />
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  app: { minHeight: '100vh', padding: '1.5rem', background: 'var(--app-background)', color: 'var(--primary-text)', fontFamily: 'sans-serif' },
  loading: { minHeight: '100vh', display: 'grid', placeItems: 'center', background: 'var(--app-background)', color: 'var(--primary-text)' },
  speedButton: {
    width: 36,
    height: 32,
    padding: 5,
    marginLeft: 4,
    border: '1px solid var(--muted-text)',
    borderRadius: 4,
    color: 'var(--dark-text)',
    cursor: 'pointer',
  },
  dashboardTab: {
    fontSize: '1.15rem',
    fontWeight: 800,
    padding: '0.75rem 1rem',
  },
  header: { display: 'flex', justifyContent: 'space-between', gap: '1rem', borderBottom: '1px solid var(--surface-border)', paddingBottom: '1rem' },
  nav: { display: 'flex', gap: '0.5rem', margin: '1rem 0' },
  navTab: { border: '1px solid var(--control-border)', borderRadius: '6px', background: 'var(--surface-background)', color: 'var(--secondary-text)', padding: '0.65rem 0.9rem', cursor: 'pointer' },
  subnav: { display: 'flex', gap: '0.5rem', flexWrap: 'wrap' },
  selectedTab: { background: 'var(--primary-accent)', color: 'var(--white-text)' },
  selectedPill: {
    border: '1px solid var(--primary-accent-border)',
    borderRadius: '999px',
    background: 'var(--primary-accent)',
    color: 'var(--white-text)',
    padding: '0.55rem 1rem',
    cursor: 'pointer',
    boxShadow: '0 0 0 2px var(--primary-accent-border)',
  },
  grid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1rem' },
  card: { background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: '8px', padding: '1rem' },
  clickableCard: { cursor: 'pointer' },
  market: { display: 'grid', gap: '1rem' },
  marketControls: { display: 'flex', gap: '0.75rem', flexWrap: 'wrap' },
  row: { display: 'flex', justifyContent: 'space-between', gap: '1rem', alignItems: 'center', padding: '0.5rem 0', borderBottom: '1px solid var(--surface-border)' },
  nameCard: { background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: '8px', padding: '1.25rem', color: 'var(--primary-text)', fontSize: '1.1rem', fontWeight: 700, textAlign: 'left', cursor: 'pointer' },
  marketItemButton: { display: 'flex', width: '100%', alignItems: 'center', gap: '1rem', background: 'transparent', border: 0, color: 'var(--primary-text)', textAlign: 'left', cursor: 'pointer', padding: 0 },
  marketThumbnail: { width: 96, height: 64, objectFit: 'contain', borderRadius: 6, background: 'var(--app-background)' },
  eventDays: { display: 'block', marginTop: '0.35rem', color: 'var(--subtle-text)', fontSize: '0.85rem', fontWeight: 400 },
  unavailableNotice: { color: 'var(--warning-text)', fontWeight: 700 },
  linkButton: { background: 'none', border: 0, color: 'var(--link-text)', fontSize: '1rem', cursor: 'pointer', padding: 0 },
  pillButton: { border: '1px solid var(--control-border)', borderRadius: '999px', background: 'var(--surface-background)', color: 'var(--secondary-text)', padding: '0.55rem 1rem', cursor: 'pointer' },
  dashboardImage: { width: '100%', maxHeight: 220, objectFit: 'cover', borderRadius: '10px', marginBottom: '0.75rem' },
  statBar: { display: 'inline-block', verticalAlign: 'middle', width: 120, height: 8, marginLeft: 8, background: 'var(--progress-background)', borderRadius: 999, overflow: 'hidden' },
  statBarFill: { display: 'block', height: '100%', borderRadius: 999 },
  inventoryTitle: { fontSize: '1.5rem', fontWeight: 700 },
  pendingEvent: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '1rem', flexWrap: 'wrap', padding: '0.75rem 0', borderBottom: '1px solid var(--surface-border)' },
  resultControls: { display: 'flex', gap: '0.5rem', flexWrap: 'wrap' },
  muted: { color: 'var(--muted-text)' },
  errorBanner: { background: 'var(--error-background)', border: '1px solid var(--error-border)', borderRadius: '8px', padding: '0.75rem', color: 'var(--error-light-text)' },
  banner: { background: 'var(--info-background)', padding: '0.75rem', margin: '1rem 0', cursor: 'pointer' },
  standings: { width: '100%', borderCollapse: 'collapse' },
};

export default App;
