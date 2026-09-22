import React, { useState } from 'react';
import type { ChampionshipCompetitor, GameState, ServiceType, TimeSpeed, EncounterState, EncounterResult } from './types/game';
import { getCharacteristic, getLabel } from './types/game';
import {
  buyObject,
  dismissAlert,
  enterEvent,
  getGameState,
  getThemeColors,
  joinQuest,
  performEvent,
  quitEvent,
  reloadDataset,
  listSaveSlots,
  loadGameFrom,
  saveGameAs,
  serviceObject,
  sellObject,
  setTimeSpeed,
  setPopupCategories,
  toggleAlarm,
  submitEventResult,
  resolveEncounterTurn,
  rememberDatasetPath,
} from './services/tauriApi';
import { AlertModal } from './components/AlertModal';
import { ConfigModal } from './components/ConfigModal';
import { ConfirmModal } from './components/ConfirmModal';
import { DetailModal } from './components/DetailModal';
import { FeedbackModal } from './components/FeedbackModal';
import { ResultPromptModal } from './components/ResultPromptModal';
import { FightModal } from './components/FightModal';
import { EventLogModal } from './components/EventLogModal';
import { SaveSlotsModal } from './components/SaveSlotsModal';
import { StartupScreen } from './components/StartupScreen';
import { useGameEffects } from './hooks/useGameEffects';

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
    closeLabel?: string;
  } | null>(null);
  const [encounter, setEncounter] = useState<EncounterState | EncounterResult | null>(null);
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
  const [eventVisibility, setEventVisibility] = useState<'all' | 'alarms'>('all');
  const [eventSort, setEventSort] = useState<'name' | 'days'>('days');
  const [championshipFilter, setChampionshipFilter] = useState('');
  const [championshipSort, setChampionshipSort] = useState<'name' | 'races' | 'status'>('name');
  const [inventoryTab, setInventoryTab] = useState('service_bay');
  const [marketCategory, setMarketCategory] = useState<string | null>(null);
  const [marketFilter, setMarketFilter] = useState('');
  const [marketSort, setMarketSort] = useState<'name' | 'price'>('name');
  const [marketImages, setMarketImages] = useState<Record<string, string>>({});
  const [playerImage, setPlayerImage] = useState('/img/player.jpeg');
  const [marketError, setMarketError] = useState('');
  const [confirmation, setConfirmation] = useState<{
    title: string;
    message: string;
    confirmLabel: string;
    onConfirm: () => void;
  } | null>(null);
  const [eventLogOpen, setEventLogOpen] = useState(false);
  const [saveModal, setSaveModal] = useState<'save' | 'load' | null>(null);
  const [saveSlots, setSaveSlots] = useState<{ name: string }[]>([]);
  const [activityTab, setActivityTab] = useState<'work' | 'trade' | 'sponsor'>('work');

  useGameEffects({
    gameState,
    setGameState,
    setMessage,
    setMarketImages,
    setMarketSort,
    setPlayerImage,
  });

  React.useEffect(() => {
    if (!gameState) return;
    const applicationName = getLabel(gameState.catalog, 'application_name', 'TTRPG Engine');
    document.title = applicationName
      .replaceAll('%player_name%', gameState.player.name)
      .replaceAll('{player_name}', gameState.player.name);
  }, [gameState?.player.name, gameState?.catalog]);

  const applyLoadedState = async (state: GameState) => {
    rememberDatasetPath(state.dataset_path);
    setGameState(state);
    setEncounter(state.active_encounter || state.last_encounter_result || null);
    const colors = await getThemeColors();
    for (const [elementId, color] of Object.entries(colors)) {
      document.documentElement.style.setProperty(`--${elementId.replaceAll('_', '-')}`, color);
    }
  };

  if (!gameState) {
    return <StartupScreen onStarted={applyLoadedState} />;
  }

  const { catalog, player } = gameState;
  const applicationTitle = getLabel(catalog, 'application_name', 'TTRPG Engine')
    .replaceAll('%player_name%', player.name)
    .replaceAll('{player_name}', player.name);
  const objectName = getLabel(catalog, 'object_name', 'Object');
  const objectPlural = getLabel(catalog, 'object_plural', `${objectName}s`);
  const inventoryName = getLabel(catalog, 'inventory_name', 'Inventory');
  const dealerName = getLabel(catalog, 'dealer_name', 'Dealer');
  const marketHiddenObjectIds = new Set(
    getLabel(catalog, 'market_hidden_object_ids', '')
      .split(';')
      .map((id) => id.trim())
      .filter(Boolean),
  );
  const eventName = getLabel(catalog, 'event_name', 'Event');
  const eventPlural = getLabel(catalog, 'event_plural', `${eventName}s`);
  const questName = getLabel(catalog, 'quest_name', 'Quest');
  const questPlural = getLabel(catalog, 'quest_plural', `${questName}s`);
  const currency = getLabel(catalog, 'currency_symbol', '$');
  const overviewName = getLabel(catalog, 'overview_name', 'Overview');
  const ageLabel = getLabel(catalog, 'age_name', 'Age');
  const budgetLabel = getLabel(catalog, 'budget_name', 'Budget');
  const dayLabel = getLabel(catalog, 'day_name', 'Day');
  const incomeSourcesLabel = getLabel(catalog, 'income_sources_name', 'Income sources');
  const eventLogLabel = getLabel(catalog, 'event_log_name', 'Event Log');
  const characterSheetLabel = getLabel(catalog, 'character_sheet_name', 'Character Sheet');
  const highestLicenseLabel = getLabel(catalog, 'highest_license_name', 'Highest license');
  const equipmentReadinessLabel = getLabel(catalog, 'equipment_readiness_name', 'Equipment readiness');
  const equipmentReadyMessage = getLabel(catalog, 'equipment_ready_message', 'Ready');
  const equipmentNotReadyMessage = getLabel(catalog, 'equipment_not_ready_message', 'Not ready');
  const ownedEquipmentLabel = getLabel(catalog, 'owned_equipment_name', 'Owned equipment');
  const noneLabel = getLabel(catalog, 'none_name', 'None');
  const objectLabel = getLabel(catalog, 'object_name', 'Object');
  const objectsLabel = getLabel(catalog, 'object_plural', 'Objects');
  const competitorLabel = getLabel(catalog, 'competitor_name', 'Competitor');
  const scoreLabel = getLabel(catalog, 'score_name', 'Score');
  const eventCountLabel = getLabel(catalog, 'event_count_name', eventPlural);
  const objectMatchesId = (object: { id: string }, id: string) =>
    object.id === id || object.id.startsWith(`${id}_`);
  const catalogObjectType = (object: { object_type?: string; type?: string }) =>
    (object.object_type || object.type || '').trim();
  const prizePositions = (event: (typeof catalog.events)[number]) => {
    const configured = (event.position_rewards || '')
      .split(';')
      .map((entry) => Number(entry.split(':')[0]?.trim()))
      .filter((position) => Number.isInteger(position) && position > 0);
    return Math.max(1, configured.length > 0 ? Math.max(...configured) : event.reward_pool > 0 ? 3 : 1);
  };
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
        name: getLabel(catalog, 'inventory_service_bay_name', inventoryName),
        types: getLabel(catalog, 'inventory_service_bay_types', 'vehicle')
          .split(';')
          .map((type) => type.trim())
          .filter(Boolean),
      },
      {
        id: 'drivers_room',
        name: getLabel(catalog, 'inventory_tab_drivers_room_name', inventoryName),
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
  const dashboardInventoryTabId = getLabel(catalog, 'dashboard_inventory_tab', 'service_bay');
  const dashboardInventoryTab = inventoryTabs.find((entry) => entry.id === dashboardInventoryTabId);
  const dashboardInventoryCount = dashboardInventoryTab
    ? player.inventory.filter((object) => dashboardInventoryTab.types.includes(object.object_type)).length
    : player.inventory.length;
  const dashboardInventoryName = getLabel(catalog, 'dashboard_inventory_name', inventoryName);
  const activeInventoryTab = inventoryTabs.find((entry) => entry.id === inventoryTab) || inventoryTabs[0];
  const inventoryObjects = player.inventory.filter((object) =>
    activeInventoryTab.types.includes(object.object_type),
  );
  const readinessGroups = getLabel(catalog, 'dashboard_readiness_object_groups', '')
    .split(';')
    .map((group) => group.split('|').map((id) => id.trim()).filter(Boolean))
    .filter((group) => group.length > 0);
  const equipmentReady = readinessGroups.length > 0 && readinessGroups.every((group) =>
    group.some((id) => player.inventory.some((owned) => objectMatchesId(owned, id))),
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
      let hasResultMessage = false;
      if (result && typeof result === 'object' && 'message' in result) {
        setMessage(String(result.message));
        hasResultMessage = true;
        setGameState(await getGameState());
      } else if (result && typeof result === 'object' && 'player' in result) {
        setGameState(result as GameState);
      }
      if (!hasResultMessage) setMessage(success);
    } catch (error) {
      setMessage(String(error));
    }
  };

  const openSaveModal = async (mode: 'save' | 'load') => {
    try {
      setSaveSlots(await listSaveSlots(gameState?.dataset_path || './dataset'));
      setSaveModal(mode);
    } catch (error) {
      setMessage(String(error));
    }
  };

  const renderDashboard = () => (
    <div style={styles.dashboardGrid}>
      <section style={styles.dashboardCard}>
        <div style={styles.portraitPanel}>
          <h2>{overviewName}</h2>
          <img src={playerImage} alt={getLabel(catalog, 'player_image_alt', 'Player')} style={styles.dashboardImage} onError={() => setPlayerImage('/img/player.jpeg')} />
          <button style={styles.logButton} onClick={() => setEventLogOpen(true)}>{getLabel(catalog, 'view_event_log_label', `View ${eventLogLabel}`)}</button>
        </div>
        <div style={styles.characterSheet}>
          <h2>{characterSheetLabel}</h2>
          <table style={styles.characterTable}>
            <tbody>
              <tr><th style={styles.characterLabel}>{ageLabel}</th><td style={styles.characterValue}>{Math.floor(player.age_days / gameState.days_per_year)} years</td></tr>
              <tr><th style={styles.characterLabel}>{budgetLabel}</th><td style={styles.characterValue}>{currency}{budget.toLocaleString()}</td></tr>
              {catalog.player_characteristics
                .filter((characteristic) => !['budget', 'age', 'picture_file'].includes(characteristic.id))
                .map((characteristic) => (
                  <tr key={characteristic.id}>
                    <th style={styles.characterLabel}>{characteristic.name}</th>
                    <td style={styles.characterValue}>{getCharacteristic(player, characteristic.id)} {statBar(characteristic.id, getCharacteristic(player, characteristic.id))}</td>
                  </tr>
                ))}
              <tr><th style={styles.characterLabel}>{dashboardInventoryName}</th><td style={styles.characterValue}>{dashboardInventoryCount}</td></tr>
              <tr><th style={styles.characterLabel}>{highestLicenseLabel}</th><td style={styles.characterValue}>{ownedLicenses[0]?.name || noneLabel}</td></tr>
              {readinessGroups.length > 0 && (
                <tr><th style={styles.characterLabel}>{equipmentReadinessLabel}</th><td style={{ ...styles.characterValue, color: equipmentReady ? 'var(--success-text)' : 'var(--error-text)' }}>{equipmentReady ? equipmentReadyMessage : equipmentNotReadyMessage}</td></tr>
              )}
              <tr><th style={styles.characterLabel}>{ownedEquipmentLabel}</th><td style={styles.characterValue}>{ownedEquipment.length > 0 ? ownedEquipment.map((object) => object.name).join(', ') : noneLabel}</td></tr>
              <tr><th style={styles.characterLabel}>{incomeSourcesLabel}</th><td style={styles.characterValue}>{player.active_events.length}</td></tr>
            </tbody>
          </table>
        </div>
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
      {inventoryObjects.length === 0 && <p>{getLabel(catalog, 'empty_inventory_message', `No ${objectsLabel.toLowerCase()} in ${activeInventoryTab.name.toLowerCase()}.`)}</p>}
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
              {getLabel(catalog, 'loaned_object_message', `Loaned ${objectLabel.toLowerCase()}; returned on day ${object.expires_day}.`)
                .replace('{object_type}', object.object_type)
                .replace('{day}', String(object.expires_day))}
            </p>
          )}
          {definedCosts(object).map(({ index, id, cost }) => (
            <div key={index} style={styles.row}>
              <span>
                {cost?.name || id} {cost ? `(${currency}${cost.amount.toLocaleString()})` : '(missing cost definition)'}:
                {' '}{needsService(object, index)
                  ? getLabel(catalog, 'required_status_name', 'Required')
                  : getLabel(catalog, 'ready_status_name', 'Ready')}
              </span>
              <button
                disabled={!needsService(object, index) || !cost || budget < cost.amount}
                onClick={() => run(
                  () => serviceObject(object.id, `Service${index}` as ServiceType),
                  `${cost?.name || `Cost ${index}`} completed.`,
                )}
              >
                {getLabel(catalog, 'service_action_name', 'Service')}
              </button>
            </div>
          ))}
          {object.loaned ? (
            <p style={styles.muted}>Loaned sponsor objects cannot be sold.</p>
          ) : object.object_type === 'license' ? (
            <p style={styles.muted}>{getLabel(catalog, 'license_not_resellable_message', 'Licenses cannot be resold.')}</p>
          ) : (
            <button onClick={() => setConfirmation({
              title: getLabel(catalog, 'sell_object_title', `Sell ${objectLabel.toLowerCase()}?`),
              message: `Sell ${object.name}? This action cannot be undone.`,
              confirmLabel: getLabel(catalog, 'sell_action_name', 'Sell'),
              onConfirm: () => {
                setConfirmation(null);
                void run(() => sellObject(object.id), 'Object sold.');
              },
            })}>{getLabel(catalog, 'sell_action_name', 'Sell')} ({currency}{(
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
      .filter((object) => !marketHiddenObjectIds.has(object.id))
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
            {(() => {
              const price = catalogObjectType(object) === 'license' && object.license_fee > 0 ? object.license_fee : object.price;
              const missing = [
                object.license_previous_id && !player.inventory.some((owned) => objectMatchesId(owned, object.license_previous_id))
                  ? object.license_previous_id : '',
                ...object.requires_object_ids.split(';').filter((id) => id && !player.inventory.some((owned) => objectMatchesId(owned, id))),
              ].filter(Boolean);
              const unavailable = missing.length > 0 || budget < price || (object.lifetime_days === 0 && player.inventory.some((owned) => objectMatchesId(owned, object.id)));
              return (
                <div style={unavailable ? undefined : styles.marketAvailable}>
                  {!unavailable && marketImages[object.id] && (
                    <img src={marketImages[object.id]} alt={object.name} style={styles.marketThumbnail} />
                  )}
                  <div style={styles.marketDetails}>
                    <div style={unavailable ? styles.marketUnavailableTitle : undefined}>
                      <span style={styles.inventoryTitle}>{object.name}</span>
                    </div>
                    <p style={{ color: budget < price ? 'var(--danger-text)' : undefined }}>
                      {currency}{price.toLocaleString()}
                    </p>
                    {missing.length > 0 && <p style={styles.marketUnavailableTitle}>Requires: {missing.join(', ')}</p>}
                    {!unavailable && <button onClick={async () => {
                      setMarketError('');
                      try {
                        setGameState(await buyObject(object.id));
                        setFeedback({ title: 'Purchase complete', message: `You have purchased ${object.name}.` });
                      } catch (error) {
                        setMarketError(String(error));
                      }
                    }}>Buy</button>}
                  </div>
                </div>
              );
            })()}
          </section>
        ))}
      </div>
    </div>
    );
  };

  const renderEvents = () => {
    const currentDay = ((gameState.current_day - 1) % gameState.days_per_year) + 1;
    const eligibleObjectsFor = (event: (typeof catalog.events)[number]) => player.inventory.filter((object) => {
      if (object.unavailable_until_day > gameState.current_day) return false;
      if ([1, 2, 3, 4].some((index) => object[`service_${index}_needed` as keyof typeof object])) return false;
      if (event.required_license_id && !player.inventory.some((owned) => objectMatchesId(owned, event.required_license_id))) return false;
      const requiredObjects = event.required_object_ids.split(';').map((id) => id.trim()).filter(Boolean);
      return requiredObjects.length === 0 || requiredObjects.some((id) => objectMatchesId(object, id));
    });
    const visibleEvents = catalog.events
      .map((event) => ({
        event,
        daysLeft: (event.day_of_year - currentDay + gameState.days_per_year) % gameState.days_per_year,
      }))
      .filter(({ event }) => event.name.toLowerCase().includes(eventFilter.toLowerCase()))
      .filter(({ event }) => eventVisibility === 'all' || gameState.alarm_event_ids.includes(event.id))
      .sort((left, right) => eventSort === 'name'
        ? left.event.name.localeCompare(right.event.name)
        : left.daysLeft - right.daysLeft);
    const isMember = (questId: string) => (gameState.quest_memberships || [])
      .some((membership) => membership.quest_id === questId);
    const questForEvent = (event: (typeof catalog.events)[number]) =>
      event.quest_id ? catalog.quests.find((quest) => quest.id === event.quest_id) : undefined;
    const eventKind = (event: (typeof catalog.events)[number]) => {
      if (event.type.trim()) {
        return event.type.replaceAll('_', ' ').replace(/\b\w/g, (letter) => letter.toUpperCase());
      }
      const tags = event.tags.split(';').map((tag) => tag.trim().toLowerCase());
      if (tags.includes('social')) return getLabel(catalog, 'social_event_type_name', 'Social event');
      if (tags.includes('track_day')) return getLabel(catalog, 'track_event_type_name', 'Track day');
      if (tags.includes('race')) return eventName;
      return eventName;
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
    return (
      <div style={styles.grid}>
        <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
          <label>
            Filter events:{' '}
            <input value={eventFilter} onChange={(event) => setEventFilter(event.target.value)} />
          </label>
          <label style={{ marginLeft: '1rem' }}>
            Show:{' '}
            <select value={eventVisibility} onChange={(event) => setEventVisibility(event.target.value as 'all' | 'alarms')}>
              <option value="all">All events</option>
              <option value="alarms">Alarm list only</option>
            </select>
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
            <h2>{getLabel(catalog, 'pending_name', 'Pending')} {eventPlural}</h2>
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
                        scoringPositions: event.quest_id ? prizePositions(event) : 1,
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
                    {' '}| {getLabel(catalog, 'type_name', 'Type')}: {eventKind(event)}
                    {' '}| {getLabel(catalog, 'resolution_name', 'Resolution')}: {event.resolution_method}
                    {' '}| {getLabel(catalog, 'entry_fee_name', 'Entry')}: {currency}{event.entry_fee}
                    {' '}| {getLabel(catalog, 'duration_name', 'Duration')}: {event.duration_value} {event.duration_unit}
                    {' '}| {getLabel(catalog, 'reward_name', 'Reward')}: {currency}{event.reward_pool} + {event.charisma_reward} {getLabel(catalog, 'secondary_reward_name', 'reward')}
                  </span>
                  {event.quest_id && !isMember(event.quest_id) && (
                    <span style={styles.muted}>
                      Join {questForEvent(event)?.name || 'the quest'} to enter this event.
                    </span>
                  )}
                  {(!event.quest_id || isMember(event.quest_id)) && eligibleObjectsFor(event).map((object) => (
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
                              scoringPositions: event.quest_id ? prizePositions(event) : 1,
                            });
                          }
                        } catch (error) {
                          setDetailMessage(String(error));
                        }
                      }}
                    >
                      {getLabel(catalog, 'enter_with_object_label', 'Enter with')} {object.name}
                    </button>
                  ))}
                  {eligibleObjectsFor(event).length === 0 && (
                    <span style={styles.muted}>{getLabel(catalog, 'no_eligible_objects_message', `No eligible ${objectsLabel.toLowerCase()} available.`)}</span>
                  )}
                </>
              ),
                });
            }}
          >
            <span>{event.name}</span>
            <small style={styles.eventDays}>
              {daysLeft} days left{' '}
              <button
                type="button"
                onClick={(click) => {
                  click.stopPropagation();
                  void toggleAlarm(event.id).then(setGameState).catch((error) => setMessage(String(error)));
                }}
              >
                {gameState.alarm_event_ids.includes(event.id) ? 'Alarm on' : 'Add alarm'}
              </button>
            </small>
          </button>
        ))}
        {gameState.event_history.length > 0 && (
          <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <h2>{getLabel(catalog, 'completed_name', 'Completed')} {eventPlural}</h2>
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

  const renderActivities = () => (
    <div style={styles.grid}>
      <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
        <div style={styles.subnav}>
          {(['work', 'trade', 'sponsor'] as const).map((type) => (
            <button key={type} style={activityTab === type ? styles.selectedPill : styles.pillButton} onClick={() => setActivityTab(type)}>
              {getLabel(catalog, `activity_type_${type}_name`, type)}
            </button>
          ))}
        </div>
      </section>
      {player.active_events.map((active) => {
        const action = catalog.events.find((entry) => entry.id === active.event_id);
        if (!action) return null;
        if (action.type.toLowerCase() !== activityTab) return null;
        const activityType = getLabel(
          catalog,
          `activity_type_${action.type.toLowerCase()}_name`,
          action.type,
        );
        const sponsorObject = action.sponsor_object_id
          ? catalog.objects.find((object) => object.id === action.sponsor_object_id)
          : undefined;
        return (
          <section key={`active-${active.event_id}`} style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <strong>{incomeSourcesLabel}: {action.name}</strong>
            <span style={styles.activityType}>{activityType}</span>
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
                  await quitEvent(action.id).then(setGameState);
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
      {catalog.activities.filter((activity) => !activity.scheduled).map((activity) => {
        const action = catalog.events.find((entry) => entry.id === activity.id);
        if (!action) return null;
        if (action.type.toLowerCase() !== activityTab) return null;
        const activityType = getLabel(
          catalog,
          `activity_type_${activity.activity_type.toLowerCase()}_name`,
          activity.activity_type,
        );
        return (
        <button
          key={activity.id}
          style={styles.nameCard}
          onClick={() => setSelectedDetail({
            title: action.name,
            descriptionPath: action.description_html,
            closeLabel: action.type.toLowerCase() === 'sponsor' ? 'Cancel' : undefined,
            footer: (
              <>
                <span>Type: {activity.activity_type} | Resolution: {activity.resolution_method}</span>
                <span> | Cost: {currency}{activity.base_cost} | Success: {(activity.success_rate * 100).toFixed(0)}%</span>
                <span> | Stamina: {Math.round(activity.stamina_cost)}</span>
                <button
                  disabled={getCharacteristic(player, 'stamina') < action.stamina_cost}
                  onClick={() => {
                    setSelectedDetail(null);
                    return run(
                  async () => {
                    const result = await performEvent(action.id);
                    const nextState = await getGameState();
                    setGameState(nextState);
                    if (action.encounter_id) {
                      setEncounter(nextState.active_encounter || nextState.last_encounter_result || null);
                    }
                    return result;
                  },
                  'Action completed.',
                    );

                  }}
                >
                  {action.type.toLowerCase() === 'sponsor' ? 'Try luck with Sponsor' : `Start ${action.name}`}
                </button>
              </>
            ),
          })}
        >
          <span>{action.name}</span>
          <span style={styles.activityType}>{activityType}</span>
        </button>
        );
      })}
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
    return (
      <div style={styles.grid}>
        <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
          <label>
            {getLabel(catalog, 'filter_name', 'Filter')} {questPlural.toLowerCase()}: {' '}
            <input
              value={championshipFilter}
              onChange={(event) => setChampionshipFilter(event.target.value)}
              placeholder={`${getLabel(catalog, 'search_name', 'Search by')} ${questName.toLowerCase()} ${getLabel(catalog, 'name_name', 'name')}`}
            />
          </label>
          <label style={{ marginLeft: '1rem' }}>
            {getLabel(catalog, 'sort_by_name', 'Sort by')}: {' '}
            <select value={championshipSort} onChange={(event) => setChampionshipSort(event.target.value as 'name' | 'races' | 'status')}>
              <option value="name">Name</option>
              <option value="races">{getLabel(catalog, 'event_count_sort_name', `Number of ${eventPlural.toLowerCase()}`)}</option>
              <option value="status">{getLabel(catalog, 'joined_status_name', 'Joined status')}</option>
            </select>
          </label>
        </section>
        {championships.map((quest) => {
          const races = catalog.events.filter((event) => event.quest_id === quest.id);
          const points = new Map<string, number>();
          const score = (race: (typeof catalog.events)[number], position: number) => (
            position > 0 && position <= prizePositions(race)
              ? Math.max(1, races.length - position + 1)
              : 0
          );
          (gameState.championship_results || [])
            .filter((result) => races.some((race) => race.id === result.event_id))
            .forEach((result) => {
              const race = races.find((entry) => entry.id === result.event_id);
              if (!race) return;
              points.set('You', (points.get('You') || 0) + score(race, result.player_position));
              result.competitors.forEach((competitor) => points.set(
                competitor.name,
                (points.get(competitor.name) || 0) + score(race, competitor.position),
              ));
            });
          const missingRequirements = [
            quest.required_license_id && !player.inventory.some((object) => objectMatchesId(object, quest.required_license_id))
              ? `Licence: ${catalog.objects.find((object) => object.id === quest.required_license_id)?.name || quest.required_license_id}` : '',
            quest.level > 1 && !player.inventory.some((object) => object.object_type === 'achievements' && object.trophy_level === quest.level - 1)
              ? `Level ${quest.level - 1} trophy` : '',
            budget < quest.join_fee ? `Funds: ${currency}${quest.join_fee.toLocaleString()}` : '',
            ...races
              .filter((race) => race.required_object_ids.trim())
              .map((race) => {
                const alternatives = race.required_object_ids.split(';').map((id) => id.trim()).filter(Boolean);
                if (alternatives.some((id) => player.inventory.some((object) => objectMatchesId(object, id)))) return '';
                return `Car: ${alternatives.map((id) => catalog.objects.find((object) => object.id === id)?.name || id).join(' or ')}`;
              }),
          ].filter(Boolean);
          const joined = gameState.quest_memberships.some((membership) => membership.quest_id === quest.id);
          const missingText = missingRequirements.join(' | ');
          return (
            <div
              key={quest.id}
              style={{ ...styles.nameCard, gridColumn: '1 / -1', width: '100%' }}
              onClick={() => setSelectedDetail({
                title: quest.name,
                descriptionPath: quest.description_html,
                footer: (
                  <>
                    <span>{races.length} {eventCountLabel.toLowerCase()} | {points.get('You') || 0} {scoreLabel.toLowerCase()}</span>
                    <strong>{getLabel(catalog, 'event_prizes_name', `${eventName} prizes by position`)}</strong>
                    {races.map((race) => (
                      <span key={race.id}>
                        {race.name}: {rewards(race.position_rewards, race.reward_pool).map((prize) =>
                          `${prize.position}${prize.position === 1 ? 'st' : prize.position === 2 ? 'nd' : prize.position === 3 ? 'rd' : 'th'} ${currency}${prize.amount.toLocaleString()}`,
                        ).join(' | ')}
                      </span>
                    ))}
                    <strong>{getLabel(catalog, 'quest_prizes_name', `${questName} prizes by final position`)}</strong>
                    <span>{rewards(quest.championship_rewards, quest.join_fee).map((prize) =>
                      `${prize.position}${prize.position === 1 ? 'st' : prize.position === 2 ? 'nd' : prize.position === 3 ? 'rd' : 'th'} ${currency}${prize.amount.toLocaleString()}`,
                    ).join(' | ')}</span>
                    {!joined && (
                      <button
                        disabled={missingRequirements.length > 0}
                        style={missingRequirements.length > 0 ? styles.disabledJoinButton : undefined}
                        onClick={() => {
                          void joinQuest(quest.id)
                            .then((state) => {
                              setGameState(state);
                              setSelectedDetail(null);
                            })
                            .catch((error) => setMessage(String(error)));
                        }}
                      >
                        {getLabel(catalog, 'join_action_name', 'Join')} for {currency}{quest.join_fee.toLocaleString()}
                        {missingText ? ` - Missing: ${missingText}` : ''}
                      </button>
                    )}
                  </>
                ),
              })}
            >
              <div style={styles.championshipSummary}>
                <span>{quest.name}</span>
                <span style={joined ? styles.championshipStatus : styles.championshipMissing}>
                  {joined ? 'Joined' : 'Not enrolled'}
                </span>
              </div>
            </div>
          );
        })}
        {championships.length === 0 && (
          <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
            {championshipFilter
              ? `${getLabel(catalog, 'no_matching_name', 'No matching')} ${questPlural.toLowerCase()}.`
              : `No ${questPlural.toLowerCase()} configured.`}
          </section>
        )}
      </div>
    );
  };

  return (
    <div style={styles.app}>
      <header style={styles.header}>
        <div>
          <h1>{applicationTitle}</h1>
          <span>{formatGameDay(gameState.current_day)} | {currency}{budget.toLocaleString()}</span>
        </div>
        <div style={styles.headerActions}>
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
          <button
            title="Save"
            aria-label="Save"
            style={styles.headerButton}
            onClick={() => void openSaveModal('save')}
          >
            <img src={getLabel(catalog, headerIconKeys.save, '/img/save.svg')} alt="" style={styles.headerIcon} />
          </button>
          <button
            title="Load"
            aria-label="Load"
            style={styles.headerButton}
            onClick={() => void openSaveModal('load')}
          >
            <img src={getLabel(catalog, headerIconKeys.load, '/img/load.svg')} alt="" style={styles.headerIcon} />
          </button>
          <button
            title="Settings"
            aria-label="Settings"
            style={styles.headerButton}
            onClick={() => setConfigOpen(true)}
          >
            <img src={getLabel(catalog, headerIconKeys.settings, '/img/settings.svg')} alt="" style={styles.headerIcon} />
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
          ['championships', questPlural],
          ['activities', getLabel(catalog, 'activity_name', 'Activities') ],
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
      <main style={styles.main}>
        {tab === 'dashboard' && renderDashboard()}
        {tab === 'inventory' && renderInventory()}
        {tab === 'dealer' && renderDealer()}
        {tab === 'events' && renderEvents()}
        {tab === 'championships' && renderChampionships()}
        {tab === 'activities' && renderActivities()}
      </main>
      {eventLogOpen && (
        <EventLogModal
          entries={gameState.event_log || []}
          formatDay={formatGameDay}
          onClose={() => setEventLogOpen(false)}
        />
      )}
      <AlertModal alerts={gameState.pending_alerts} onDismiss={setGameState} />
      <ConfigModal
        isOpen={configOpen}
        currentPath={gameState.dataset_path}
        onClose={() => setConfigOpen(false)}
        onReloadDataset={async (path) => {
          const state = await reloadDataset(path);
          rememberDatasetPath(path);
          setGameState(state);
          setConfigOpen(false);
        }}
        popupCategories={gameState.popup_categories}
        onPopupCategoriesChange={async (categories) => setGameState(await setPopupCategories(categories))}
      />
      {saveModal && (
        <SaveSlotsModal
          mode={saveModal}
          slots={saveSlots}
          onClose={() => setSaveModal(null)}
          onSave={async (slot) => {
            const path = await saveGameAs(slot);
            setSaveSlots(await listSaveSlots(gameState.dataset_path));
            setSaveModal(null);
            setFeedback({ title: 'Game saved', message: `Game saved to ${path}.` });
          }}
          onLoad={async (slot) => {
            const loaded = await loadGameFrom(gameState.dataset_path, slot);
            await applyLoadedState(loaded);
            setSaveModal(null);
            setFeedback({ title: 'Game loaded', message: `Save slot '${slot}' has been loaded.` });
          }}
        />
      )}
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
          }}>{selectedDetail.closeLabel || 'Close'}</button>
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
          competitorLabel={competitorLabel}
          competitorPluralLabel={getLabel(catalog, 'competitor_plural', 'Competitors')}
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
            setFeedback({ title: `${questName} result recorded`, message: eventResult.message });
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
      {encounter && (
        <FightModal
          encounter={encounter}
          catalog={catalog}
          inventory={player.inventory}
          onAction={async (actionId) => {
            try {
              const nextEncounter = await resolveEncounterTurn(actionId);
              const nextState = await getGameState();
              setGameState(nextState);
              setEncounter(nextEncounter);
            } catch (error) {
              setMessage(String(error));
            }
          }}
          onClose={() => setEncounter(null)}
        />
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  app: { height: '100vh', boxSizing: 'border-box', display: 'flex', flexDirection: 'column', overflow: 'hidden', padding: '1.5rem', background: 'var(--app-background)', color: 'var(--primary-text)', fontFamily: 'sans-serif' },
  loading: { minHeight: '100vh', display: 'grid', placeItems: 'center', background: 'var(--app-background)', color: 'var(--primary-text)' },
  startup: { minHeight: '100vh', display: 'grid', placeItems: 'center', padding: '1.5rem', boxSizing: 'border-box', background: 'var(--app-background)', color: 'var(--primary-text)' },
  startupCard: { width: 'min(680px, 94vw)', padding: '2rem', background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: '14px', boxShadow: '0 18px 45px var(--modal-overlay)' },
  startupChoices: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '1rem', marginTop: '1.5rem' },
  startupChoice: { display: 'flex', flexDirection: 'column', gap: '0.6rem', minHeight: 140, padding: '1.25rem', textAlign: 'left', border: '1px solid var(--control-border)', borderRadius: '10px', background: 'var(--control-background)', color: 'var(--primary-text)', cursor: 'pointer' },
  startupSlots: { display: 'grid', gap: '0.5rem', marginTop: '1.5rem', paddingTop: '1rem', borderTop: '1px solid var(--surface-border)' },
  saveSlotButton: { padding: '0.8rem 1rem', textAlign: 'left', cursor: 'pointer' },
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
  headerActions: { display: 'flex', alignItems: 'center', gap: 4 },
  headerButton: {
    width: 36,
    height: 32,
    padding: 5,
    marginLeft: 4,
    border: '1px solid var(--muted-text)',
    borderRadius: 4,
    background: 'var(--white-text)',
    color: 'var(--dark-text)',
    cursor: 'pointer',
  },
  headerIcon: { width: 18, height: 18, display: 'block' },
  dashboardTab: {
    fontSize: '1.15rem',
    fontWeight: 800,
    padding: '0.75rem 1rem',
  },
  header: { flexShrink: 0, display: 'flex', justifyContent: 'space-between', gap: '1rem', borderBottom: '1px solid var(--surface-border)', paddingBottom: '1rem' },
  nav: { flexShrink: 0, display: 'flex', gap: '0.5rem', margin: '1rem 0' },
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
  marketAvailable: { display: 'flex', alignItems: 'center', gap: '1rem' },
  marketDetails: { display: 'grid', gap: '0.25rem', minWidth: 0 },
  activityType: {
    alignSelf: 'flex-start',
    display: 'inline-block',
    marginTop: '0.35rem',
    padding: '0.15rem 0.45rem',
    borderRadius: 999,
    background: 'var(--info-background)',
    color: 'var(--info-text)',
    fontSize: '0.75rem',
    fontWeight: 600,
  },
  marketControls: { display: 'flex', gap: '0.75rem', flexWrap: 'wrap' },
  row: { display: 'flex', justifyContent: 'space-between', gap: '1rem', alignItems: 'center', padding: '0.5rem 0', borderBottom: '1px solid var(--surface-border)' },
  nameCard: { background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: '8px', padding: '1.25rem', color: 'var(--primary-text)', fontSize: '1.1rem', fontWeight: 700, textAlign: 'left', cursor: 'pointer' },
  marketItemButton: { display: 'flex', width: '100%', alignItems: 'center', gap: '1rem', background: 'transparent', border: 0, color: 'var(--primary-text)', textAlign: 'left', cursor: 'pointer', padding: 0 },
  marketThumbnail: { flex: '0 0 120px', width: 120, height: 90, objectFit: 'contain', borderRadius: 6, background: 'var(--app-background)' },
  marketUnavailableTitle: { color: 'var(--danger-text)' },
  championshipMissing: { display: 'block', marginTop: '0.4rem', color: 'var(--danger-text)', fontSize: '0.85rem', fontWeight: 600 },
  championshipStatus: { display: 'block', marginTop: '0.4rem', color: 'var(--success-text)', fontSize: '0.85rem', fontWeight: 600 },
  championshipSummary: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '1rem' },
  disabledJoinButton: { color: 'var(--danger-text)', cursor: 'not-allowed' },
  eventDays: { display: 'block', marginTop: '0.35rem', color: 'var(--subtle-text)', fontSize: '0.85rem', fontWeight: 400 },
  unavailableNotice: { color: 'var(--warning-text)', fontWeight: 700 },
  linkButton: { background: 'none', border: 0, color: 'var(--link-text)', fontSize: '1rem', cursor: 'pointer', padding: 0 },
  pillButton: { border: '1px solid var(--control-border)', borderRadius: '999px', background: 'var(--surface-background)', color: 'var(--secondary-text)', padding: '0.55rem 1rem', cursor: 'pointer' },
  main: { flex: 1, minHeight: 0, overflowY: 'auto', overflowX: 'hidden', paddingRight: '0.25rem' },
  dashboardGrid: { display: 'grid', gridTemplateColumns: 'minmax(0, 1fr)', gap: '1rem' },
  dashboardCard: { display: 'grid', gridTemplateColumns: 'minmax(150px, 0.35fr) minmax(0, 1fr)', gap: '1.5rem', background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: '12px', padding: '1.25rem', boxShadow: '0 8px 24px var(--modal-overlay)' },
  portraitPanel: { display: 'flex', flexDirection: 'column', minWidth: 0 },
  characterSheet: { minWidth: 0 },
  dashboardImage: { width: '100%', aspectRatio: '3 / 4', maxHeight: 360, objectFit: 'cover', objectPosition: 'center', borderRadius: '10px', marginBottom: '0.75rem', background: 'var(--app-background)' },
  logButton: { width: '100%', marginTop: 'auto', padding: '0.7rem 0.8rem', border: '1px solid var(--primary-accent-border)', borderRadius: '7px', background: 'var(--primary-accent)', color: 'var(--white-text)', cursor: 'pointer', fontWeight: 700 },
  characterTable: { width: '100%', borderCollapse: 'collapse' },
  characterLabel: { width: '42%', padding: '0.7rem 0.75rem 0.7rem 0', textAlign: 'left', verticalAlign: 'top', borderBottom: '1px solid var(--surface-border)' },
  characterValue: { padding: '0.7rem 0', textAlign: 'right', verticalAlign: 'top', borderBottom: '1px solid var(--surface-border)', overflowWrap: 'anywhere' },
  statLabel: { fontWeight: 700 },
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
