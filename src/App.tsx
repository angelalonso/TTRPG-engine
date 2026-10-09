import React, { useCallback, useEffect, useRef, useState } from 'react';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import type { ActivityData, ChampionshipCompetitor, EventData, EventEligibility, GameState, ObjectTransactionEligibility, ServiceType, TimeSpeed, EncounterState, EncounterResult, SponsorData } from './types/game';
import { getCharacteristic, getLabel } from './types/game';
import { calendarMode, formatCalendarDay, formatEventDate, formatShortCalendarDay } from './utils/calendar';
import {
  buyObject,
  switchInsurance,
  terminateInsurance,
  dismissAlert,
  enterEvent,
  rentEvent,
  getGameState,
  getEventEligibility,
  getObjectTransactionEligibility,
  getAppConfig,
  saveAppConfig,
  getThemeColors,
  autodetectRaceResultsPlugin,
  selectRaceResultsFile,
  openSponsorNegotiation,
  closeSponsorNegotiation,
  type SponsorNegotiationState,
  listSponsors,
  joinQuest,
  performEvent,
  quitEvent,
  reloadDataset,
  listSaveSlots,
  loadGameFrom,
  loadSaveGameAtOwnRisk,
  migrateSaveGameFrom,
  deleteSaveSlot,
  saveGameAs,
  serviceObject,
  sellObject,
  setTimeSpeed,
  setPopupCategories,
  toggleAlarm,
  submitEventResult,
  resolveEncounterTurn,
  rememberDatasetPath,
  type SaveSlot,
} from './services/tauriApi';
import { AlertModal } from './components/AlertModal';
import { ConfigModal } from './components/ConfigModal';
import { ConfirmModal } from './components/ConfirmModal';
import { DetailModal } from './components/DetailModal';
import { FeedbackModal } from './components/FeedbackModal';
import { ResultPromptModal } from './components/ResultPromptModal';
import { RentalModal, type RentalCarOption } from './components/RentalModal';
import { FightModal } from './components/FightModal';
import { EventLogModal } from './components/EventLogModal';
import { SaveSlotsModal } from './components/SaveSlotsModal';
import { StartupScreen } from './components/StartupScreen';
import { DatasetEditor } from './components/DatasetEditor';
import { SponsorNegotiationModal } from './components/SponsorNegotiationModal';
import { IntroPlugin } from './components/IntroPlugin';
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
type SortDirection = 'asc' | 'desc';

export const App: React.FC = () => {
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [editorDatasetPath, setEditorDatasetPath] = useState<string | null>(null);
  const [tab, setTab] = useState('dashboard');
  const [configOpen, setConfigOpen] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);
  const [windowWidth, setWindowWidth] = useState(1440);
  const [windowHeight, setWindowHeight] = useState(900);
  const [gameDirectory, setGameDirectory] = useState('');
  const [resultsDirectory, setResultsDirectory] = useState('');
  const [message, setMessage] = useState('');
  const [messageIsWarning, setMessageIsWarning] = useState(false);
  const [detailMessage, setDetailMessage] = useState('');
  const [selectedDetail, setSelectedDetail] = useState<{
    title: string;
    descriptionPath: string;
    footer: React.ReactNode;
    closeLabel?: string;
    hideDescription?: boolean;
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
    finishingPositions?: number;
    pluginEnabled?: boolean;
  } | null>(null);
  const [rentalEventId, setRentalEventId] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<{ title: string; message: string } | null>(null);
  const [eventFilter, setEventFilter] = useState('');
  const [eventVehicleFilter, setEventVehicleFilter] = useState<string[]>([]);
  const [eventVisibility, setEventVisibility] = useState<'all' | 'alarms'>('all');
  const [eventSort, setEventSort] = useState<'name' | 'days' | 'type' | 'entry' | 'reward'>('days');
  const [eventSortDirection, setEventSortDirection] = useState<SortDirection>('asc');
  const [championshipFilter, setChampionshipFilter] = useState('');
  const [championshipVehicleFilter, setChampionshipVehicleFilter] = useState<string[]>([]);
  const [championshipSort, setChampionshipSort] = useState<'name' | 'races' | 'status' | 'points'>('name');
  const [championshipSortDirection, setChampionshipSortDirection] = useState<SortDirection>('asc');
  const [inventoryTab, setInventoryTab] = useState('');
  const [marketCategory, setMarketCategory] = useState<string | null>(null);
  const [marketFilter, setMarketFilter] = useState('');
  const [marketSort, setMarketSort] = useState<'name' | 'price' | 'owned' | 'availability'>('name');
  const [marketSortDirection, setMarketSortDirection] = useState<SortDirection>('asc');
  const [marketImages, setMarketImages] = useState<Record<string, string>>({});
  const [playerImage, setPlayerImage] = useState('/img/player.jpeg');
  const [marketError, setMarketError] = useState('');
  const [eventEligibility, setEventEligibility] = useState<Record<string, EventEligibility[]>>({});
  const [objectEligibility, setObjectEligibility] = useState<Record<string, ObjectTransactionEligibility>>({});
  const [confirmation, setConfirmation] = useState<{
    title: string;
    message: string;
    confirmLabel: string;
    onConfirm: () => void;
  } | null>(null);
  const [eventLogOpen, setEventLogOpen] = useState(false);
  const [saveModal, setSaveModal] = useState<'save' | 'load' | null>(null);
  const [sponsorNegotiation, setSponsorNegotiation] = useState<SponsorNegotiationState | null>(null);
  const [introVisible, setIntroVisible] = useState(false);
  const [saveSlots, setSaveSlots] = useState<SaveSlot[]>([]);
  const [activityTab, setActivityTab] = useState<'work' | 'trade' | 'sponsor'>('work');
  const [activitySort, setActivitySort] = useState<'name' | 'type' | 'cost' | 'success' | 'pay' | 'frequency' | 'return'>('name');
  const [activitySortDirection, setActivitySortDirection] = useState<SortDirection>('asc');
  const [sponsors, setSponsors] = useState<SponsorData[]>([]);
  const previousSpeed = useRef<Exclude<TimeSpeed, 'Paused'>>('OneDayEveryFiveSec');

  const showMessage = useCallback((value: string) => {
    setMessage(value);
    setMessageIsWarning(
      /^(error|failed|cannot|can't|could not|couldn't|no |not enough|insufficient|this .* (?:cannot|can't|is not|is already|has already)|already |required |you (?:cannot|can't|do not|don't|already)|missing |invalid |unable to|not available|unavailable|blocked|expired|rejected|denied|must |requires? |needs? |that attempt did not|the plan fell through|you gave it a try|this opportunity got away|you didn't get|someone better qualified|the position went)/i.test(value.trim()),
    );
  }, []);

  const closeResultPrompt = useCallback(() => {
    setResultPrompt(null);
    void getGameState()
      .then(setGameState)
      .catch((error) => showMessage(`Unable to refresh pending events: ${String(error)}`));
  }, [showMessage]);

  const openConfig = useCallback(() => {
    setConfigOpen(true);
    void getCurrentWindow().innerSize().then(async (size) => {
      const logicalSize = size.toLogical(await getCurrentWindow().scaleFactor());
      setWindowWidth(Math.round(logicalSize.width));
      setWindowHeight(Math.round(logicalSize.height));
    }).catch((error) => {
      showMessage(`Unable to read the current window size: ${String(error)}`);
    });
  }, [showMessage]);

  useGameEffects({
    gameState,
    setGameState,
    setMessage: showMessage,
    setMarketImages,
    setMarketSort,
    setPlayerImage,
  });

  useEffect(() => {
    if (!gameState) return;
    let cancelled = false;
    Promise.all(gameState.catalog.events.map(async (event) => [event.id, await getEventEligibility(event.id)] as const))
      .then((entries) => {
        if (!cancelled) setEventEligibility(Object.fromEntries(entries));
      })
      .catch((error) => {
        if (!cancelled) showMessage(`Unable to load event eligibility: ${String(error)}`);
      });
    return () => {
      cancelled = true;
    };
  }, [gameState, showMessage]);

  useEffect(() => {
    if (!gameState || activityTab !== 'sponsor') return;
    let cancelled = false;
    listSponsors()
      .then((entries) => {
        if (!cancelled) setSponsors(entries);
      })
      .catch((error) => {
        if (!cancelled) showMessage(`Unable to load sponsors: ${String(error)}`);
      });
    return () => {
      cancelled = true;
    };
  }, [activityTab, gameState, showMessage]);

  useEffect(() => {
    if (!gameState) return;
    let cancelled = false;
    getObjectTransactionEligibility()
      .then((entries) => {
        if (!cancelled) setObjectEligibility(Object.fromEntries(entries.map((entry) => [entry.object_id, entry])));
      })
      .catch((error) => {
        if (!cancelled) showMessage(`Unable to load object eligibility: ${String(error)}`);
      });
    return () => {
      cancelled = true;
    };
  }, [gameState, showMessage]);

  React.useEffect(() => {
    if (gameState && gameState.time_speed !== 'Paused') {
      previousSpeed.current = gameState.time_speed;
    }
  }, [gameState?.time_speed]);

  const handleAlertDismiss = (state: GameState) => {
    setGameState(state);
  };

  React.useEffect(() => {
    if (!gameState) return;
    const applicationName = getLabel(gameState.catalog, 'application_name', 'TTRPG Engine');
    document.title = applicationName
      .replaceAll('%player_name%', gameState.player.name)
      .replaceAll('{player_name}', gameState.player.name);
  }, [gameState?.player.name, gameState?.catalog]);

  React.useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (introVisible) return;
      const target = event.target as HTMLElement | null;
      const isTextEntry = target instanceof HTMLInputElement
        || target instanceof HTMLTextAreaElement
        || target?.isContentEditable;
      if (isTextEntry) return;

      if (event.key === ' ') {
        if (gameState.pending_alerts.length > 0) return;
        event.preventDefault();
        const nextSpeed = gameState.time_speed === 'Paused'
          ? previousSpeed.current
          : 'Paused';
        if (gameState.time_speed !== 'Paused') {
          previousSpeed.current = gameState.time_speed;
        }
        void setTimeSpeed(nextSpeed).then(setGameState);
        return;
      }

      if (event.key !== 'Escape') return;
      if (gameState.pending_alerts.length > 0) return;
      event.preventDefault();
      if (confirmation) {
        setConfirmation(null);
      } else if (configOpen) {
        setConfigOpen(false);
      } else if (sponsorNegotiation) {
        void closeSponsorNegotiation().finally(() => setSponsorNegotiation(null));
      } else if (saveModal) {
        setSaveModal(null);
      } else if (selectedDetail) {
        setSelectedDetail(null);
        setDetailMessage('');
      } else if (resultPrompt) {
        closeResultPrompt();
      } else if (eventLogOpen) {
        setEventLogOpen(false);
      } else if (feedback) {
        setFeedback(null);
      } else if (encounter) {
        setEncounter(null);
      } else {
        setConfirmation({
          title: 'Exit game?',
          message: 'Are you sure you want to exit the game?',
          confirmLabel: 'Exit',
          onConfirm: () => void getCurrentWindow().close(),
        });
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [
    configOpen,
    confirmation,
    encounter,
    eventLogOpen,
    feedback,
    gameState,
    resultPrompt,
    saveModal,
    selectedDetail,
    sponsorNegotiation,
    introVisible,
  ]);

  const applyLoadedState = async (state: GameState, showIntro = false) => {
    rememberDatasetPath(state.dataset_path);
    setGameState(state);
    const config = await getAppConfig();
    setFullscreen(config.fullscreen);
    setWindowWidth(config.window_width);
    setWindowHeight(config.window_height);
    setGameDirectory(config.game_directory || '');
    setResultsDirectory(config.results_directory || '');
    setIntroVisible(showIntro);
    setEncounter(state.active_encounter || state.last_encounter_result || null);
    const colors = await getThemeColors();
    for (const [elementId, color] of Object.entries(colors)) {
      document.documentElement.style.setProperty(`--${elementId.replaceAll('_', '-')}`, color);
    }
  };

  if (!gameState) {
    if (editorDatasetPath) {
      return (
        <DatasetEditor
          datasetPath={editorDatasetPath}
          onExit={() => setEditorDatasetPath(null)}
        />
      );
    }
    return <StartupScreen onStarted={applyLoadedState} onEditor={setEditorDatasetPath} />;
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
  const currencyCharacteristicId = catalog.resource_roles?.currency || 'budget';
  const recoveryCharacteristicId = catalog.resource_roles?.recovery || 'stamina';
  const ageCharacteristicId = catalog.resource_roles?.age || 'age';
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
    if (configured.size > 0) {
      return Array.from(configured.entries())
        .filter(([, entry]) => entry.name && entry.types)
        .map(([id, entry]) => ({
          id,
          name: entry.name as string,
          types: (entry.types as string).split(';').map((type) => type.trim()).filter(Boolean),
        }))
        .sort((left, right) => {
          const order = ['service_bay', 'drivers_room', 'achievements'];
          return (order.indexOf(left.id) < 0 ? order.length : order.indexOf(left.id))
            - (order.indexOf(right.id) < 0 ? order.length : order.indexOf(right.id));
        });
    }
    const objectTypes = Array.from(new Set(
      catalog.objects
        .map(catalogObjectType)
        .filter(Boolean),
    ));
    return objectTypes.map((type) => ({
      id: `objects_${type.toLowerCase().replace(/[^a-z0-9]+/g, '_')}`,
      name: getLabel(catalog, `inventory_type_${type}_name`, type),
      types: [type],
    }));
  })();
  const dashboardInventoryTabId = getLabel(catalog, 'dashboard_inventory_tab', inventoryTabs[0]?.id || '');
  const dashboardInventoryTab = inventoryTabs.find((entry) => entry.id === dashboardInventoryTabId);
  const dashboardInventoryCount = dashboardInventoryTab
    ? player.inventory.filter((object) => dashboardInventoryTab.types.includes(object.object_type)).length
    : player.inventory.length;
  const dashboardInventoryName = getLabel(catalog, 'dashboard_inventory_name', inventoryName);
  const activeInventoryTab = inventoryTabs.find((entry) => entry.id === inventoryTab) || inventoryTabs[0];
  const inventoryObjects = player.inventory.filter((object) =>
    activeInventoryTab.types.includes(object.object_type),
  );
  const companionTypes = getLabel(catalog, 'companion_types', 'companion')
    .split(';')
    .map((type) => type.trim().toLowerCase())
    .filter(Boolean);
  const companionObjects = player.inventory.filter((object) =>
    companionTypes.includes(object.object_type.trim().toLowerCase()),
  );
  const hasCompanion = companionObjects.length > 0;
  const managerObjectId = getLabel(catalog, 'manager_object_id', 'manager').trim();
  const hasManager = companionObjects.some((object) => objectMatchesId(object, managerObjectId));
  const companionTabName = getLabel(catalog, 'companion_tab_name', 'Crew');
  const readinessGroups = getLabel(catalog, 'dashboard_readiness_object_groups', '')
    .split(';')
    .map((group) => group.split('|').map((id) => id.trim()).filter(Boolean))
    .filter((group) => group.length > 0);
  const readinessGroupSatisfied = (group: string[]) => group.some((id) => {
    const requirement = catalog.objects.find((object) => objectMatchesId(object, id));
    const requirementGroup = requirement?.requirement_group?.trim();
    return player.inventory.some((owned) =>
      objectMatchesId(owned, id)
      || Boolean(requirementGroup && owned.requirement_group?.trim() === requirementGroup),
    );
  });
  const missingReadinessGroups = readinessGroups.filter((group) =>
    !readinessGroupSatisfied(group),
  );
  const equipmentReady = readinessGroups.length > 0 && missingReadinessGroups.length === 0;
  const missingReadinessNames = missingReadinessGroups.map((group) =>
    group
      .map((id) => catalog.objects.find((object) => objectMatchesId(object, id))?.name || id)
      .join(' / '),
  );
  const ownedVehicles = Array.from(
    new Map(
      player.inventory
        .filter((object) => object.object_type.toLowerCase() === 'vehicle')
        .map((vehicle) => [vehicle.definition_id, vehicle]),
    ).values(),
  );
  const sponsorContractMatchesEvent = (event: (typeof catalog.events)[number]) =>
    (gameState.sponsor_contracts || []).some((contract) => {
      if (contract.scope.toLowerCase() === 'year') return false;
      if (['championship', 'quest'].includes(contract.scope.toLowerCase())) {
        if (!event.quest_id) return false;
        if (contract.target_id === event.quest_id) return true;
        const quest = catalog.quests.find((entry) => entry.id === event.quest_id);
        return Boolean(quest && contract.target_name === quest.name);
      }
      return contract.target_id === event.id;
    });
  const eventRequirementsPending = (event: (typeof catalog.events)[number]) => {
    const options = eventEligibility[event.id];
    if (!sponsorContractMatchesEvent(event) || !options || options.some((option) => option.available)) return false;
    return options.some((option) => !['scheduled', 'duplicate_entry', 'insufficient_resource'].includes(option.reason_code));
  };
  const selectedVehicleMatches = (event: (typeof catalog.events)[number], selected: string[]) => {
    if (selected.length === 0) return true;
    const required = event.required_object_ids.split(';').map((id) => id.trim()).filter(Boolean);
    if (required.length === 0) return true;
    return selected.some((selectedId) => required.some((requiredId) =>
      selectedId === requiredId
      || selectedId.startsWith(`${requiredId}_`)
      || requiredId.startsWith(`${selectedId}_`),
    ));
  };
  const rentalCarsFor = (event: (typeof catalog.events)[number]): RentalCarOption[] =>
    (eventEligibility[event.id] || [])
      .filter((option) => option.rented)
      .map((option) => {
        const car = catalog.objects.find((object) => object.id === option.definition_id);
        return {
          id: option.selection_id,
          name: car?.name || option.definition_id,
          rentalCost: option.rental_cost,
          available: option.available,
          reason: option.reason,
          reasonCode: option.reason_code,
          failedFacts: option.failed_facts,
        };
      });
  const damageOptions = Array.from(new Map(
    catalog.cost_rules
      .filter((rule) => rule.damage_type.trim())
      .map((rule) => [rule.damage_type, catalog.costs.find((cost) => cost.id === rule.cost_id)?.name || rule.damage_type]),
  )).map(([id, name]) => ({ id, name }));
  const ownedLicenses = player.inventory
    .filter((object) => object.license_level > 0)
    .sort((left, right) => right.license_level - left.license_level);
  const budget = getCharacteristic(player, currencyCharacteristicId);
  const livingCostBase = catalog.costs.find((cost) => cost.id === 'living_cost')?.amount || 0;
  const ownedCarValue = player.inventory
    .filter((object) => catalogObjectType(object).toLowerCase() === 'vehicle')
    .reduce((total, object) => total + Math.max(0, object.price), 0);
  const livingCosts = livingCostBase + ownedCarValue * 0.01;
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
    return formatCalendarDay(day, gameState.days_per_year, catalog.labels.values);
  };
  const formatHeaderDay = (day: number) => {
    return formatShortCalendarDay(day, gameState.days_per_year, catalog.labels.values);
  };
  const sortHeader = (
    label: string,
    column: string,
    activeColumn: string,
    direction: SortDirection,
    onSort: (column: string) => void,
  ) => (
    <button
      type="button"
      style={styles.sortHeader}
      aria-label={`Sort by ${label}`}
      onClick={() => onSort(column)}
    >
      <span>{label}</span>
      <span aria-hidden="true" style={styles.sortTriangle}>
        {activeColumn === column ? direction === 'asc' ? '▲' : '▼' : '△'}
      </span>
    </button>
  );

  const run = async (operation: () => Promise<unknown>, success: string) => {
    try {
      const result = await operation();
      let hasResultMessage = false;
      if (result && typeof result === 'object' && 'message' in result) {
        showMessage(String(result.message));
        if ('success' in result && result.success === false) {
          setMessageIsWarning(true);
        }
        hasResultMessage = true;
        setGameState(await getGameState());
      } else if (result && typeof result === 'object' && 'player' in result) {
        setGameState(result as GameState);
      }
      if (!hasResultMessage) showMessage(success);
    } catch (error) {
      showMessage(String(error));
    }
  };

  const openSaveModal = async (mode: 'save' | 'load') => {
    try {
      setSaveSlots(await listSaveSlots(gameState?.dataset_path || './gtr2career'));
      setSaveModal(mode);
    } catch (error) {
      showMessage(String(error));
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
              <tr><th style={styles.characterLabel}>Living costs / month</th><td style={styles.characterValue}>{currency}{livingCosts.toLocaleString()}</td></tr>
              {catalog.player_characteristics
                .filter((characteristic) => ![
                  currencyCharacteristicId,
                  ageCharacteristicId,
                  recoveryCharacteristicId,
                  'picture_file',
                ].includes(characteristic.id))
                .map((characteristic) => (
                  <tr key={characteristic.id}>
                    <th style={styles.characterLabel}>{characteristic.name}</th>
                    <td style={styles.characterValue}>{getCharacteristic(player, characteristic.id)} {statBar(characteristic.id, getCharacteristic(player, characteristic.id))}</td>
                  </tr>
                ))}
              <tr><th style={styles.characterLabel}>{dashboardInventoryName}</th><td style={styles.characterValue}>{dashboardInventoryCount}</td></tr>
              <tr><th style={styles.characterLabel}>{highestLicenseLabel}</th><td style={styles.characterValue}>{ownedLicenses[0]?.name || noneLabel}</td></tr>
              {readinessGroups.length > 0 && (
                <tr>
                  <th style={styles.characterLabel}>{equipmentReadinessLabel}</th>
                  <td style={{ ...styles.characterValue, color: equipmentReady ? 'var(--success-text)' : 'var(--error-text)' }}>
                    {equipmentReady
                      ? equipmentReadyMessage
                      : `${equipmentNotReadyMessage}: ${missingReadinessNames.join(', ')}`}
                  </td>
                </tr>
              )}
              <tr><th style={styles.characterLabel}>{incomeSourcesLabel}</th><td style={styles.characterValue}>{player.active_events.length}</td></tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );

  const renderCompanions = () => {
    const managerOfferIds = new Set(gameState.manager_sponsor_offer_ids || []);
    const sponsorActions = catalog.activities
      .filter((activity) => !activity.scheduled)
      .filter((activity) => managerOfferIds.has(activity.id))
      .map((activity) => catalog.events.find((event) => event.id === activity.id))
      .filter((event): event is (typeof catalog.events)[number] => Boolean(event))
      .filter((event) => event.type.toLowerCase() === 'sponsor');
    return (
      <div style={styles.grid}>
        <section style={{ ...styles.card, ...styles.stickyControls, gridColumn: '1 / -1' }}>
          <h2>{companionTabName}</h2>
          <p style={styles.muted}>
            Companions are dataset-defined. Their costs and benefits are applied
            by the same rules as other game objects.
          </p>
        </section>
        {companionObjects.map((companion) => (
          <section key={companion.id} style={styles.card}>
            <h3>{companion.name}</h3>
            <p>{companion.description_html ? 'Active companion' : 'Companion'}</p>
            {hasManager && objectMatchesId(companion, managerObjectId) && (
              <p style={styles.successMessage}>
                {getLabel(catalog, 'manager_active_message', 'Your manager brings sponsor offers to you.')}
              </p>
            )}
            <button
              style={styles.linkButton}
              onClick={() => setSelectedDetail({
                title: companion.name,
                descriptionPath: companion.description_html,
                footer: null,
              })}
            >
              View details
            </button>
          </section>
        ))}
        {hasManager && (
          <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <h2>{getLabel(catalog, 'manager_offers_name', 'Sponsor offers')}</h2>
            <p style={styles.muted}>
              {getLabel(catalog, 'manager_offer_message', 'Your manager has arranged these offers.')}
            </p>
            {sponsorActions.length === 0
              ? <p>{getLabel(catalog, 'no_manager_offers_message', 'No sponsor offers are currently available.')}</p>
              : sponsorActions.map((action) => (
                <div key={action.id} style={styles.row}>
                  <span>{action.name}</span>
                  <button onClick={() => {
                    setSelectedDetail({
                      title: action.name,
                      descriptionPath: action.description_html,
                      closeLabel: 'Cancel',
                      footer: (
                        <button onClick={() => {
                        void startSponsorNegotiation(action, undefined, 'proposal');
                        }}>
                          Open negotiation
                        </button>
                      ),
                    });
                  }}>
                    Review offer
                  </button>
                </div>
              ))}
          </section>
        )}
      </div>
    );
  };

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
      {activeInventoryTab.id === 'drivers_room' && (
        <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
          <h2>Sponsor agreements</h2>
          {(gameState.sponsor_contracts || []).length === 0 ? (
            <p style={styles.muted}>No active sponsor agreements.</p>
          ) : (
            <div style={styles.grid}>
              {(gameState.sponsor_contracts || []).map((contract) => (
                <div key={contract.id} style={styles.card}>
                  <h3>{contract.sponsor_name}</h3>
                  <p style={styles.muted}>
                    {contract.sponsor_tier} · {contract.scope}
                    {contract.target_name ? ` · ${contract.target_name}` : ''}
                  </p>
                  <p>
                    Active through day {contract.expires_day}
                  </p>
                  <p>
                    Initial: {currency}{contract.initial_money.toLocaleString()} ·
                    Monthly: {currency}{contract.monthly_payment.toLocaleString()}
                  </p>
                  <p>
                    {contract.entry_fees ? 'Entry fees covered' : 'Entry fees not covered'} ·
                    {' '}{contract.maintenance
                      ? `Repairs covered up to ${currency}${contract.repair_coverage.toLocaleString()}`
                      : 'Repairs not covered'}
                  </p>
                  <p>
                    {contract.car ? 'Car included' : 'No car'} ·
                    {' '}{contract.gear ? 'Gear included' : 'No gear'} ·
                    {' '}Bonus: {currency}{contract.result_bonus.toLocaleString()} ·
                    {' '}DNF penalty: {currency}{contract.dnf_penalty.toLocaleString()}
                  </p>
                </div>
              ))}
            </div>
          )}
        </section>
      )}
      {activeInventoryTab.types.some((type) => type.toLowerCase() === 'achievements') && (() => {
        const levels = new Set<number>([0]);
        player.inventory
          .filter((object) => object.object_type.toLowerCase() === 'achievements')
          .forEach((object) => levels.add(object.trophy_level || 0));
        const stats = new Map<number, { races: number; wins: number; podiums: number; poles: number }>();
        gameState.event_history.forEach((history) => {
          const event = catalog.events.find((entry) => entry.id === history.event_id);
          if (!event || !event.tags.split(';').some((tag) => ['race', 'track_day', 'trackday'].includes(tag.trim().toLowerCase()))) return;
          const level = event.quest_id
            ? catalog.quests.find((quest) => quest.id === event.quest_id)?.level || 0
            : 0;
          levels.add(level);
          const current = stats.get(level) || { races: 0, wins: 0, podiums: 0, poles: 0 };
          current.races += 1;
          const position = history.player_position || 0;
          const successful = history.outcome.toLowerCase() === 'success'
            || history.outcome.toLowerCase() === 'successful';
          if (position === 1 || (position === 0 && successful)) current.wins += 1;
          if ((position >= 1 && position <= 3) || (position === 0 && successful)) current.podiums += 1;
          if (history.pole_position) current.poles += 1;
          stats.set(level, current);
        });
        return (
          <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <h2>Trophy results by level</h2>
            <table style={styles.dataTable}>
              <thead>
                <tr>
                  <th style={styles.dataTableHeader}>Level</th>
                  <th style={{ ...styles.dataTableHeader, ...styles.numericHeader }}>Races</th>
                  <th style={{ ...styles.dataTableHeader, ...styles.numericHeader }}>Wins</th>
                  <th style={{ ...styles.dataTableHeader, ...styles.numericHeader }}>Podiums</th>
                  <th style={{ ...styles.dataTableHeader, ...styles.numericHeader }}>Poles</th>
                </tr>
              </thead>
              <tbody>
                {Array.from(levels).sort((left, right) => left - right).map((level) => {
                  const row = stats.get(level) || { races: 0, wins: 0, podiums: 0, poles: 0 };
                  return (
                    <tr key={level}>
                      <td style={styles.dataTableCell}>{level === 0 ? '0 (track days)' : level}</td>
                      <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>{row.races}</td>
                      <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>{row.wins}</td>
                      <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>{row.podiums}</td>
                      <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>{row.poles}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </section>
        );
      })()}
      {inventoryObjects.length === 0
        && activeInventoryTab.id !== 'drivers_room'
        && !activeInventoryTab.types.some((type) => type.toLowerCase() === 'achievements') && (
        <p>{getLabel(catalog, 'empty_inventory_message', `No ${objectsLabel.toLowerCase()} in ${activeInventoryTab.name.toLowerCase()}.`)}</p>
      )}
      {inventoryObjects.map((object) => (
        <section
          key={object.id}
          style={{
            ...styles.card,
            opacity: object.unavailable_until_day > gameState.current_day ? 0.6 : 1,
          }}
        >
          {(() => {
            const transaction = objectEligibility[object.definition_id || object.id];
            const sellBlockedReason = transaction && !transaction.can_sell
              ? transaction.sell_reason
              : '';
            return (
              <>
          {activeInventoryTab.id === 'drivers_room'
            && (marketImages[object.definition_id || object.id] || marketImages[object.id])
            && (
              <img
                src={marketImages[object.definition_id || object.id] || marketImages[object.id]}
                alt=""
                style={styles.inventoryObjectImage}
              />
            )}
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
          {sellBlockedReason ? (
            <p style={styles.muted}>{sellBlockedReason}</p>
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
          {object.object_type.toLowerCase() === 'insurance' && (
            <button
              onClick={() => setConfirmation({
                title: 'Terminate insurance policy?',
                message: `Terminate ${object.name}? You will no longer have this insurance coverage.`,
                confirmLabel: 'Terminate',
                onConfirm: () => {
                  setConfirmation(null);
                  void run(() => terminateInsurance(object.id), 'Insurance policy terminated.');
                },
              })}
            >
              Terminate policy
            </button>
          )}
              </>
            );
          })()}
        </section>
      ))}
    </div>
  );

  const renderDealer = () => {
    const marketTypes = Array.from(new Set(
      catalog.objects
        .map(catalogObjectType)
        .filter((type) => type && type.toLowerCase() !== 'achievements'),
    ));
    const selectedType = marketTypes.includes(marketCategory || '') ? marketCategory || '' : marketTypes[0] || '';
    const marketObjects = catalog.objects
      .filter((object) => !marketHiddenObjectIds.has(object.id))
      .filter((object) => catalogObjectType(object).toLowerCase() !== 'achievements')
      .filter((object) => catalogObjectType(object) === selectedType)
      .filter((object) => object.name.toLowerCase().includes(marketFilter.toLowerCase()))
      .sort((left, right) => {
        const leftPrice = left.license_fee > 0 ? left.license_fee : left.price;
        const rightPrice = right.license_fee > 0 ? right.license_fee : right.price;
        const leftOwned = player.inventory.filter((owned) => objectMatchesId(owned, left.id)).length;
        const rightOwned = player.inventory.filter((owned) => objectMatchesId(owned, right.id)).length;
        const leftAvailability = left.requires_object_ids || left.license_previous_id ? 'Requires' : budget < leftPrice ? 'Insufficient funds' : 'Available';
        const rightAvailability = right.requires_object_ids || right.license_previous_id ? 'Requires' : budget < rightPrice ? 'Insufficient funds' : 'Available';
        const comparison = marketSort === 'name'
          ? left.name.localeCompare(right.name)
          : marketSort === 'price'
            ? leftPrice - rightPrice
            : marketSort === 'owned'
              ? leftOwned - rightOwned
              : leftAvailability.localeCompare(rightAvailability);
        return marketSortDirection === 'asc' ? comparison : -comparison;
      });
    return (
    <div style={styles.market}>
      <div style={styles.stickyControls}>
        <div style={styles.marketToolbar}>
          <div style={styles.marketSubnav}>
          {marketTypes.map((type) => (
            <button
              key={type}
              style={{
                ...(type === selectedType ? styles.selectedPill : styles.pillButton),
                ...styles.marketPill,
              }}
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
              style={styles.marketFilterInput}
            />
          </div>
        </div>
      </div>
      {marketError && <div style={styles.errorBanner}>{marketError}</div>}
      <table style={{ ...styles.dataTable, gridColumn: '1 / -1' }}>
        <thead>
          <tr>
            <th style={styles.dataTableHeader}>{sortHeader('Item', 'name', marketSort, marketSortDirection, (column) => {
              const next = column as 'name' | 'price' | 'owned' | 'availability';
              setMarketSortDirection(marketSort === next ? marketSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setMarketSort(next);
            })}</th>
            <th style={styles.dataTableHeader}>{sortHeader('Price', 'price', marketSort, marketSortDirection, (column) => {
              const next = column as 'name' | 'price' | 'owned' | 'availability';
              setMarketSortDirection(marketSort === next ? marketSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setMarketSort(next);
            })}</th>
            <th style={styles.dataTableHeader}>{sortHeader('Owned', 'owned', marketSort, marketSortDirection, (column) => {
              const next = column as 'name' | 'price' | 'owned' | 'availability';
              setMarketSortDirection(marketSort === next ? marketSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setMarketSort(next);
            })}</th>
            <th style={styles.dataTableHeader}>{sortHeader('Availability', 'availability', marketSort, marketSortDirection, (column) => {
              const next = column as 'name' | 'price' | 'owned' | 'availability';
              setMarketSortDirection(marketSort === next ? marketSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setMarketSort(next);
            })}</th>
            <th style={styles.dataTableHeader}>Action</th>
          </tr>
        </thead>
        <tbody>
        {marketObjects.length === 0 && <tr><td colSpan={5} style={{ ...styles.dataTableCell, ...styles.muted }}>No matching items in this category.</td></tr>}
        {marketObjects.map((object) => (
          <tr key={object.id} className="data-table-row">
            {(() => {
              const price = catalogObjectType(object) === 'license' && object.license_fee > 0 ? object.license_fee : object.price;
              const backendEligibility = objectEligibility[object.id];
              const missing = [
                object.license_previous_id && !player.inventory.some((owned) => objectMatchesId(owned, object.license_previous_id))
                  ? object.license_previous_id : '',
                ...object.requires_object_ids.split(';').filter((id) => id && !player.inventory.some((owned) => objectMatchesId(owned, id))),
              ].filter(Boolean);
              const unavailable = backendEligibility
                ? !backendEligibility.can_acquire
                : missing.length > 0 || budget < price || (object.lifetime_days === 0 && player.inventory.some((owned) => objectMatchesId(owned, object.id)));
              const availabilityReason = backendEligibility?.acquire_reason
                || (missing.length > 0 ? `Requires: ${missing.join(', ')}` : budget < price ? 'Insufficient funds' : 'Available');
              const isInsurance = catalogObjectType(object).toLowerCase() === 'insurance';
              const activeInsurance = player.inventory.find((owned) => owned.object_type.toLowerCase() === 'insurance');
              const activeInsuranceDefinition = activeInsurance
                ? catalog.objects.find((entry) => entry.id === activeInsurance.definition_id)
                : undefined;
              const canSwitchInsurance = isInsurance
                && Boolean(activeInsurance)
                && activeInsurance?.definition_id !== object.id;
              return (
                <>
                  <td style={styles.dataTableCell}>
                    <button style={styles.marketItemButton} onClick={() => setSelectedDetail({
                      title: object.name,
                      descriptionPath: object.description_html,
                      footer: null,
                    })}>
                      {marketImages[object.id] && <img src={marketImages[object.id]} alt="" style={styles.marketThumbnail} />}
                      <span style={unavailable ? styles.marketUnavailableName : styles.linkButton}>{object.name}</span>
                    </button>
                  </td>
                  <td style={{ ...styles.dataTableCell, ...styles.numericCell, ...(unavailable && budget < price ? styles.marketUnavailablePrice : {}) }}>
                    {currency}{price.toLocaleString()}{isInsurance ? '/year' : ''}
                  </td>
                  <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>
                    {player.inventory.filter((owned) => objectMatchesId(owned, object.id)).length}
                  </td>
                  <td style={styles.dataTableCell}>
                    {availabilityReason}
                  </td>
                  <td style={styles.dataTableCell}>
                    {canSwitchInsurance ? (
                      <button onClick={async () => {
                        setMarketError('');
                        try {
                          setGameState(await switchInsurance(object.id));
                          setFeedback({ title: 'Insurance changed', message: `You are now covered by ${object.name}.` });
                        } catch (error) {
                          setMarketError(String(error));
                        }
                      }}>
                        Switch policy
                      </button>
                    ) : !unavailable && <button onClick={async () => {
                        setMarketError('');
                        try {
                          setGameState(await buyObject(object.id));
                          setFeedback({ title: 'Purchase complete', message: `You have purchased ${object.name}.` });
                        } catch (error) {
                          setMarketError(String(error));
                        }
                      }}>Buy</button>}
                    {canSwitchInsurance && activeInsuranceDefinition && price < activeInsuranceDefinition.price
                      && gameState.current_day % gameState.days_per_year !== 0 && (
                        <span style={styles.muted}>Downgrade available at year end</span>
                      )}
                  </td>
                </>
              );
            })()}
          </tr>
        ))}
        </tbody>
      </table>
      </div>
    );
  };

  const renderEvents = () => {
    const currentDay = ((gameState.current_day - 1) % gameState.days_per_year) + 1;
    const currentCalendarMode = calendarMode(catalog.labels.values);
    const eligibleObjectsFor = (event: (typeof catalog.events)[number]) => player.inventory.filter((object) => {
      const backendOption = eventEligibility[event.id]?.find((option) => (
        !option.rented
        && (option.selection_id === object.id
          || objectMatchesId(object, option.selection_id)
          || objectMatchesId(object, option.definition_id))
      ));
      if (backendOption) return backendOption.available;
      if (eventEligibility[event.id]) return false;
      const eventAllowsAnyVehicle = event.tags.split(';').some((tag) => {
        const normalizedTag = tag.trim().toLowerCase();
        return normalizedTag === 'track_day' || normalizedTag === 'trackday';
      }) || event.name.toLowerCase().includes('open race');
      const motorsportEvent = event.tags.split(';').some((tag) => {
        const normalizedTag = tag.trim().toLowerCase();
        return normalizedTag === 'race' || normalizedTag === 'track_day' || normalizedTag === 'trackday';
      });
      if (motorsportEvent && object.object_type !== 'vehicle') return false;
      if (object.unavailable_until_day > gameState.current_day) return false;
      const hasBlockingService = Array.from({ length: 15 }, (_, index) => index + 1).some((index) => {
        const needed = object[`service_${index}_needed` as keyof typeof object] as boolean;
        const costId = (object[`cost_${index}` as keyof typeof object] as string || '').toLowerCase();
        const cost = catalog.costs.find((candidate) => candidate.id.toLowerCase() === costId);
        return needed && !cost?.cosmetic;
      });
      if (hasBlockingService) return false;
      if (event.required_license_id && !player.inventory.some((owned) => objectMatchesId(owned, event.required_license_id))) return false;
      const requiredObjects = event.required_object_ids.split(';').map((id) => id.trim()).filter(Boolean);
      return eventAllowsAnyVehicle || requiredObjects.length === 0 || requiredObjects.some((id) => objectMatchesId(object, id));
    });
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
    const visibleEvents = catalog.events
      .map((event) => ({
        event,
        daysLeft: (event.day_of_year - currentDay + gameState.days_per_year) % gameState.days_per_year,
      }))
      .filter(({ event }) => event.day_of_year > 0 && event.type.toLowerCase() !== 'work')
      .filter(({ event }) => event.name.toLowerCase().includes(eventFilter.toLowerCase()))
      .filter(({ event }) => selectedVehicleMatches(event, eventVehicleFilter))
      .filter(({ event }) => eventVisibility === 'all' || gameState.alarm_event_ids.includes(event.id))
      .sort((left, right) => {
        const comparison = eventSort === 'name'
          ? left.event.name.localeCompare(right.event.name)
          : eventSort === 'days'
            ? left.daysLeft - right.daysLeft
            : eventSort === 'type'
              ? eventKind(left.event).localeCompare(eventKind(right.event))
              : eventSort === 'entry'
                ? left.event.entry_fee - right.event.entry_fee
                : left.event.reward_pool - right.event.reward_pool;
        return eventSortDirection === 'asc' ? comparison : -comparison;
      });
    const isMember = (questId: string) => (gameState.quest_memberships || [])
      .some((membership) => membership.quest_id === questId);
    const questForEvent = (event: (typeof catalog.events)[number]) =>
      event.quest_id ? catalog.quests.find((quest) => quest.id === event.quest_id) : undefined;
    const eventRequirementMessages = (event: (typeof catalog.events)[number]) => {
      const options = eventEligibility[event.id] || [];
      const unavailable = options.filter((option) => !option.available && option.reason && !option.rented);
      const hasAvailableEntry = options.some((option) => option.available);
      const eventWideReasons = new Set(
        unavailable
          .filter((option) => !option.selection_id)
          .map((option) => option.reason),
      );
      const messages = options
        .filter((option) => !option.available && option.reason && !option.rented)
        .map((option) => {
          if (option.reason.includes('Only open races and track days offer car rental')) return '';
          if (
            !option.selection_id
            && hasAvailableEntry
            && (
              option.reason.includes('an eligible object is required')
              || option.reason.includes('an eligible vehicle is required')
              || option.reason.includes('a vehicle owned by the player is required')
            )
          ) return '';
          if (option.selection_id && eventWideReasons.has(option.reason)) return '';
          const selected = option.selection_id
            ? player.inventory.find((object) => object.id === option.selection_id)
              || catalog.objects.find((object) => object.id === option.definition_id)
            : undefined;
          const vehicleRequired = option.reason.includes('a vehicle owned by the player is required')
            || option.reason.includes('an eligible object is required');
          if (vehicleRequired && selected && selected.object_type.toLowerCase() !== 'vehicle') return '';
          return `${selected?.name || 'Event'}: ${option.reason}`;
        });
      return Array.from(new Set(messages.filter(Boolean)));
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
    const openEventDetails = (event: (typeof catalog.events)[number], daysLeft: number) => {
      setDetailMessage('');
      const pendingForEvent = gameState.pending_events
        .filter((pending) => pending.event_id === event.id && pending.entered_day === gameState.current_day)
        .at(-1);
      setSelectedDetail({
        title: event.name,
        descriptionPath: event.description_html,
        footer: (
          <>
            <span>
              {currentCalendarMode === 'monthdays_weekdays'
                ? formatEventDate(event.day_of_year, gameState.days_per_year, catalog.labels.values)
                : `${dayLabel}: ${event.day_of_year} | ${daysLeft === 0 ? 'Today' : `${daysLeft} days left`}`}
              {' '}| {getLabel(catalog, 'resolution_name', 'Resolution')}: {event.resolution_method}
              {' '}| {getLabel(catalog, 'entry_fee_name', 'Entry')}: {currency}{event.entry_fee}
              {' '}| {getLabel(catalog, 'reward_name', 'Reward')}: {currency}{event.reward_pool}
            </span>
            {event.quest_id && !isMember(event.quest_id) && (
              <span style={styles.muted}>Join {questForEvent(event)?.name || 'the quest'} to enter this event.</span>
            )}
            {eventRequirementMessages(event).length > 0 && (
              <div style={styles.requirementWarning}>
                <strong>Requirements not met</strong>
                {eventRequirementMessages(event).map((reason) => (
                  <span key={reason}>{reason}</span>
                ))}
              </div>
            )}
            {(!event.quest_id || isMember(event.quest_id)) && eligibleObjectsFor(event).map((object) => (
              <button
                key={object.id}
                disabled={
                  currentDay !== event.day_of_year
                  || gameState.pending_events.some((pending) =>
                    pending.event_id === event.id && pending.entered_day === gameState.current_day)
                }
                onClick={async () => {
                  try {
                    const nextState = await enterEvent(object.id, event.id);
                    setGameState(nextState);
                    setSelectedDetail(null);
                    const pending = nextState.pending_events
                      .filter((entry) => entry.event_id === event.id && entry.object_id === object.id)
                      .at(-1);
                    if (pending) {
                      setResultPrompt({
                        id: pending.id,
                        eventName: event.name,
                        damageOptions,
                        championship: Boolean(event.quest_id),
                        previousCompetitors: event.quest_id ? previousChampionshipCompetitors(event.quest_id) : [],
                        championshipDrivers: event.quest_id ? championshipDrivers(event.quest_id) : [],
                        scoringPositions: event.quest_id ? prizePositions(event) : 1,
                        finishingPositions: event.quest_id ? Math.max(prizePositions(event), championshipDrivers(event.quest_id).length) : 1,
                        pluginEnabled: event.tags.split(';').some((tag) => tag.trim().toLowerCase() === 'race'),
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
            {gameState.pending_events.some((pending) =>
              pending.event_id === event.id && pending.entered_day === gameState.current_day) && (
              <>
                <span style={styles.muted}>
                  Result pending. Enter the result to continue.
                </span>
                {pendingForEvent && (
                  <button
                    type="button"
                    onClick={() => {
                      setSelectedDetail(null);
                      setResultPrompt({
                        id: pendingForEvent.id,
                        eventName: event.name,
                        damageOptions,
                        championship: Boolean(event.quest_id),
                        previousCompetitors: event.quest_id ? previousChampionshipCompetitors(event.quest_id) : [],
                        championshipDrivers: event.quest_id ? championshipDrivers(event.quest_id) : [],
                        scoringPositions: event.quest_id ? prizePositions(event) : 1,
                        finishingPositions: event.quest_id
                          ? Math.max(prizePositions(event), championshipDrivers(event.quest_id).length)
                          : 1,
                        pluginEnabled: event.tags.split(';').some((tag) => tag.trim().toLowerCase() === 'race'),
                      });
                    }}
                  >
                    Enter result
                  </button>
                )}
              </>
            )}
            {eventEligibility[event.id]?.some((option) => option.rented) && (
              <button type="button" disabled={currentDay !== event.day_of_year} onClick={() => setRentalEventId(event.id)}>
                Rent a car
              </button>
            )}
          </>
        ),
      });
    };
    return (
      <div style={styles.grid}>
        <section style={{ ...styles.card, ...styles.stickyControls, ...styles.raceFilterArea, gridColumn: '1 / -1' }}>
          <div style={styles.filtersRow}>
          <label>
            Filter events:{' '}
            <input style={styles.filterInput} value={eventFilter} onChange={(event) => setEventFilter(event.target.value)} />
          </label>
          <label style={{ marginLeft: '1rem' }}>
            Show:{' '}
            <select
              style={styles.filterInput}
              value={eventVisibility}
              onChange={(event) => setEventVisibility(event.target.value as 'all' | 'alarms')}
            >
              <option value="all">All events</option>
              <option value="alarms">Alarm list only</option>
            </select>
          </label>
          <label>
            Filter by vehicle:{' '}
            <select
              value={eventVehicleFilter[0] || ''}
              onChange={(change) => {
                setEventVehicleFilter(change.target.value ? [change.target.value] : []);
              }}
              style={{ ...styles.filterInput, minWidth: '14rem' }}
            >
              <option value="">No vehicle filter</option>
              {ownedVehicles.map((vehicle) => <option key={vehicle.definition_id} value={vehicle.definition_id}>{vehicle.name}</option>)}
            </select>
          </label>
          </div>
        </section>
        {gameState.pending_events.length > 0 && (
          <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <h2>{getLabel(catalog, 'pending_name', 'Pending')} {eventPlural}</h2>
            {gameState.pending_events.map((pending) => {
              const event = catalog.events.find((entry) => entry.id === pending.event_id);
              const object = player.inventory.find((entry) => entry.id === pending.object_id)
                || catalog.objects.find((entry) => entry.id === pending.object_id);
              if (!event) return null;
              return (
                <div key={pending.id} style={styles.pendingEvent}>
                  <div>
                    <strong>{event.name}</strong>
                    <span style={styles.muted}>
                      {' '}with {object?.name || pending.object_id}{pending.rented ? ' (rented)' : ''}
                    </span>
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
                        finishingPositions: event.quest_id
                          ? Math.max(
                            prizePositions(event),
                            championshipDrivers(event.quest_id).length,
                          )
                          : 1,
                        pluginEnabled: event.tags.split(';').some((tag) => tag.trim().toLowerCase() === 'race'),
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
        <table style={{ ...styles.dataTable, gridColumn: '1 / -1' }}>
          <thead>
            <tr>
              <th style={styles.dataTableHeader}>{sortHeader('Race', 'name', eventSort, eventSortDirection, (column) => {
                const next = column as 'name' | 'days' | 'type' | 'entry' | 'reward';
                setEventSortDirection(eventSort === next ? eventSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
                setEventSort(next);
              })}</th>
              <th style={styles.dataTableHeader}>{sortHeader(currentCalendarMode === 'monthdays_weekdays' ? 'Date' : 'Days left', 'days', eventSort, eventSortDirection, (column) => {
                const next = column as 'name' | 'days' | 'type' | 'entry' | 'reward';
                setEventSortDirection(eventSort === next ? eventSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
                setEventSort(next);
              })}</th>
              <th style={styles.dataTableHeader}>{sortHeader('Type', 'type', eventSort, eventSortDirection, (column) => {
                const next = column as 'name' | 'days' | 'type' | 'entry' | 'reward';
                setEventSortDirection(eventSort === next ? eventSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
                setEventSort(next);
              })}</th>
              <th style={styles.dataTableHeader}>{sortHeader('Entry', 'entry', eventSort, eventSortDirection, (column) => {
                const next = column as 'name' | 'days' | 'type' | 'entry' | 'reward';
                setEventSortDirection(eventSort === next ? eventSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
                setEventSort(next);
              })}</th>
              <th style={styles.dataTableHeader}>{sortHeader('Reward', 'reward', eventSort, eventSortDirection, (column) => {
                const next = column as 'name' | 'days' | 'type' | 'entry' | 'reward';
                setEventSortDirection(eventSort === next ? eventSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
                setEventSort(next);
              })}</th>
              <th style={styles.dataTableHeader}>Action</th>
            </tr>
          </thead>
          <tbody>
            {visibleEvents.map(({ event, daysLeft }) => (
              <tr
                key={event.id}
                className="data-table-row"
                style={{
                  background: gameState.alarm_event_ids.includes(event.id)
                    ? '#214b63'
                    : event.quest_id
                      ? (isMember(event.quest_id) ? 'var(--warning-background)' : 'var(--surface-border)')
                      : undefined,
                }}
              >
                <td style={styles.dataTableCell}>
                  <button style={styles.linkButton} onClick={() => openEventDetails(event, daysLeft)}>{event.name}</button>
                  {eventRequirementsPending(event) && <span style={styles.muted}> (Pending requirements)</span>}
                </td>
                <td style={styles.dataTableCell}>
                  {currentCalendarMode === 'monthdays_weekdays'
                    ? formatEventDate(event.day_of_year, gameState.days_per_year, catalog.labels.values)
                    : (daysLeft === 0 ? 'Today' : daysLeft)}
                </td>
                <td style={styles.dataTableCell}>{eventKind(event)}</td>
                <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>{currency}{event.entry_fee.toLocaleString()}</td>
                <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>{currency}{event.reward_pool.toLocaleString()}</td>
                <td style={styles.dataTableCell}>
                  <button onClick={() => openEventDetails(event, daysLeft)}>Details</button>
                  <button type="button" onClick={() => void toggleAlarm(event.id).then(setGameState).catch((error) => showMessage(String(error)))}>
                    {gameState.alarm_event_ids.includes(event.id) ? '✓' : 'Add alarm'}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {gameState.event_history.length > 0 && (
          <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
            <h2>{getLabel(catalog, 'completed_name', 'Completed')} {eventPlural}</h2>
            {gameState.event_history.slice().reverse().map((history) => {
              const event = catalog.events.find((entry) => entry.id === history.event_id);
              const object = player.inventory.find((entry) => entry.id === history.object_id)
                || catalog.objects.find((entry) => entry.id === history.object_id);
              return (
                <div key={history.id} style={styles.row}>
                  <span>
                    {event?.name || history.event_id} with {object?.name || history.object_id}:
                    {history.object_id && !player.inventory.some((entry) => entry.id === history.object_id) ? ' (rented)' : ''}
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

  const activityColumns = (tabName: string) => {
    const fallback = tabName === 'work' ? ['name', 'pay', 'frequency'] : ['name', 'cost', 'success', 'return'];
    const configured = getLabel(catalog, `activity_${tabName === 'work' ? 'jobs' : 'ventures'}_columns`, fallback.join(';'))
      .split(/[;,]/).map((column) => column.trim().toLowerCase()).filter(Boolean);
    return configured.length ? configured : fallback;
  };
  const activityColumnLabel = (column: string) => getLabel(
    catalog,
    `activity_${column}_name`,
    column[0].toUpperCase() + column.slice(1),
  );
  const activityFrequency = (activity: ActivityData) => {
    if (activity.payout_freq_type.toLowerCase() === 'once' || activity.payout_freq <= 0) {
      return getLabel(catalog, 'activity_once_name', 'once');
    }
    const unit = activity.payout_freq_unit || 'period';
    const singular = activity.payout_freq === 1 ? unit.replace(/s$/, '') : unit;
    return `${getLabel(catalog, 'activity_every_name', 'every')} ${activity.payout_freq === 1 ? '' : `${activity.payout_freq} `}${singular}`.trim();
  };
  const activityCell = (column: string, activity: ActivityData, action: EventData, activityType: string) => {
    if (column === 'name') return action.name;
    if (column === 'type') return activityType;
    if (column === 'pay' || column === 'return') return `${currency}${activity.payout.toLocaleString()}`;
    if (column === 'frequency') return activityFrequency(activity);
    if (column === 'cost') return `${currency}${activity.base_cost.toLocaleString()}`;
    if (column === 'success') return `${(activity.success_rate * 100).toFixed(0)}%`;
    return '';
  };
  const activityReturns = (action: EventData) => {
    const returns: string[] = [];
    if (action.payout !== 0) returns.push(`${currency}${action.payout.toLocaleString()}`);
    if (action.charisma_reward !== 0) {
      returns.push(`${action.charisma_reward > 0 ? '+' : ''}${action.charisma_reward} ${getLabel(catalog, 'charisma_name', 'charisma')}`);
    }
    return returns.join(' | ') || getLabel(catalog, 'none_name', 'None');
  };
  const startSponsorNegotiation = async (
    event?: EventData,
    sponsorId?: string,
    approach: 'cold_call' | 'proposal' = 'cold_call',
  ) => {
    try {
      const started = await openSponsorNegotiation(event?.id ?? '', sponsorId, approach);
      setGameState(started.game_state);
      setSponsorNegotiation(started.state);
      setSelectedDetail(null);
    } catch (error) {
      showMessage(String(error));
    }
  };

  const renderActivities = () => (
    <div style={styles.grid}>
      <section style={{ ...styles.card, ...styles.stickyControls, gridColumn: '1 / -1' }}>
        <div style={styles.subnav}>
          {(['work', 'trade', 'sponsor'] as const).map((type) => (
            <button key={type} style={activityTab === type ? styles.selectedPill : styles.pillButton} onClick={() => setActivityTab(type)}>
              {getLabel(catalog, `activity_type_${type}_name`, type)}
            </button>
          ))}
        </div>
      </section>
      {activityTab === 'sponsor' && (
        <section style={{ ...styles.card, gridColumn: '1 / -1' }}>
          <h2>Available sponsors</h2>
          <p style={styles.muted}>
            Choose a sponsor to open a negotiation. These offers are available for local races and can be cold-called without a manager.
          </p>
          <div style={styles.grid}>
            {sponsors.map((sponsor) => (
              <article key={sponsor.id} style={styles.card}>
                <h3>{sponsor.name}</h3>
                <p style={styles.muted}>
                  {sponsor.tier} sponsor · {sponsor.brand}
                </p>
                <p>
                  Initial money: {currency}{sponsor.base_cash.toLocaleString()} · Monthly: {currency}{sponsor.monthly_payment.toLocaleString()}
                </p>
                {sponsor.repair_value > 0 && (
                  <p>Repairs covered: up to {currency}{sponsor.repair_value.toLocaleString()}</p>
                )}
                <button onClick={() => void startSponsorNegotiation(undefined, sponsor.id)}>
                  Negotiate with {sponsor.name}
                </button>
              </article>
            ))}
            {sponsors.length === 0 && <p>No sponsors are currently configured.</p>}
          </div>
        </section>
      )}
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
                  showMessage(String(error));
                }
              },
            })}>{action.type.toLowerCase() === 'work' ? 'Quit job' : 'Stop action'}</button>
          </section>
        );
      })}
      {activityTab !== 'sponsor' && (
      <table style={{ ...styles.dataTable, gridColumn: '1 / -1' }}>
        <thead>
          <tr>
            <th style={styles.dataTableHeader}>{sortHeader('Name', 'name', activitySort, activitySortDirection, (column) => {
              const next = column as 'name' | 'type' | 'cost' | 'success';
              setActivitySortDirection(activitySort === next ? activitySortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setActivitySort(next);
            })}</th>
            <th style={styles.dataTableHeader}>{sortHeader('Type', 'type', activitySort, activitySortDirection, (column) => {
              const next = column as 'name' | 'type' | 'cost' | 'success';
              setActivitySortDirection(activitySort === next ? activitySortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setActivitySort(next);
            })}</th>
            <th style={styles.dataTableHeader}>{sortHeader(activityColumnLabel(activityTab === 'work' ? 'pay' : 'cost'), activityTab === 'work' ? 'pay' : 'cost', activitySort, activitySortDirection, (column) => {
              const next = column as 'name' | 'type' | 'cost' | 'success';
              setActivitySortDirection(activitySort === next ? activitySortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setActivitySort(next);
            })}</th>
            <th style={styles.dataTableHeader}>{activityTab === 'work'
              ? sortHeader(activityColumnLabel('frequency'), 'frequency', activitySort, activitySortDirection, (column) => {
                const next = column as 'name' | 'type' | 'pay' | 'frequency';
                setActivitySortDirection(activitySort === next ? activitySortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
                setActivitySort(next);
              })
              : sortHeader(activityColumnLabel('success'), 'success', activitySort, activitySortDirection, (column) => {
              const next = column as 'name' | 'type' | 'cost' | 'success';
              setActivitySortDirection(activitySort === next ? activitySortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setActivitySort(next);
            })}</th>
            {activityTab === 'trade' && <th style={styles.dataTableHeader}>{sortHeader(activityColumnLabel('return'), 'return', activitySort, activitySortDirection, (column) => {
              const next = column as 'name' | 'type' | 'cost' | 'success' | 'return';
              setActivitySortDirection(activitySort === next ? activitySortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
              setActivitySort(next);
            })}</th>}
            <th style={styles.dataTableHeader}>Action</th>
          </tr>
        </thead>
        <tbody>
      {catalog.activities.filter((activity) => !activity.scheduled)
        .sort((left, right) => {
          const leftAction = catalog.events.find((entry) => entry.id === left.id);
          const rightAction = catalog.events.find((entry) => entry.id === right.id);
          if (!leftAction || !rightAction) return 0;
          const comparison = activitySort === 'name'
            ? leftAction.name.localeCompare(rightAction.name)
            : activitySort === 'type'
              ? left.activity_type.localeCompare(right.activity_type)
              : activitySort === 'cost'
                ? leftAction.base_cost - rightAction.base_cost
                : leftAction.success_rate - rightAction.success_rate;
          return activitySortDirection === 'asc' ? comparison : -comparison;
        })
        .map((activity) => {
        const action = catalog.events.find((entry) => entry.id === activity.id);
        if (!action) return null;
        if (action.type.toLowerCase() !== activityTab) return null;
        const activityType = getLabel(
          catalog,
          `activity_type_${activity.activity_type.toLowerCase()}_name`,
          activity.activity_type,
        );
        const obligation = catalog.obligations.find((entry) => entry.event_id === action.id);
        const staminaCost = obligation?.amount ?? action.stamina_cost;
        return (
        <tr
          key={activity.id}
          className="data-table-row"
          onClick={() => setSelectedDetail({
            title: action.name,
            descriptionPath: action.description_html,
            closeLabel: action.type.toLowerCase() === 'sponsor' ? 'Cancel' : undefined,
            footer: (
              <>
                <span>Type: {activity.activity_type} | Resolution: {activity.resolution_method}</span>
                <span> | Cost: {currency}{activity.base_cost} | Success: {(activity.success_rate * 100).toFixed(0)}%</span>
                <span> | Stamina: {Math.round(staminaCost)}{obligation ? ' per due day' : ''}</span>
                <button
                  disabled={!obligation && getCharacteristic(player, recoveryCharacteristicId) < staminaCost}
                  onClick={() => {
                    setSelectedDetail(null);
                    if (action.type.toLowerCase() === 'sponsor') {
                      void startSponsorNegotiation(action, undefined, 'proposal');
                      return;
                    }
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
          <td style={styles.dataTableCell}><button style={styles.linkButton} onClick={() => setSelectedDetail({
            title: action.name,
            descriptionPath: action.description_html,
            footer: <span>{activityType} | Cost: {currency}{activity.base_cost} | Success: {(activity.success_rate * 100).toFixed(0)}%</span>,
          })}>{action.name}</button></td>
          <td style={styles.dataTableCell}>{activityType}</td>
          <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>
            {activityTab === 'work'
              ? `${currency}${activity.payout.toLocaleString()}`
              : `${currency}${activity.base_cost.toLocaleString()}`}
          </td>
          <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>
            {activityTab === 'work'
              ? activityFrequency(activity)
              : `${(activity.success_rate * 100).toFixed(0)}%`}
          </td>
          {activityTab === 'trade' && (
            <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>
              {activityReturns(action)}
            </td>
          )}
          <td style={styles.dataTableCell}>
            <button onClick={(event) => {
              event.stopPropagation();
              setSelectedDetail({
                title: action.name,
                descriptionPath: action.description_html,
                closeLabel: action.type.toLowerCase() === 'sponsor' ? 'Cancel' : undefined,
                footer: (
                  <>
                    <span>Type: {activity.activity_type} | Cost: {currency}{activity.base_cost} | Success: {(activity.success_rate * 100).toFixed(0)}%</span>
                    <button
                      disabled={!obligation && getCharacteristic(player, recoveryCharacteristicId) < staminaCost}
                      onClick={() => {
                        setSelectedDetail(null);
                        if (action.type.toLowerCase() === 'sponsor') {
                          void startSponsorNegotiation(action);
                          return;
                        }
                        void run(async () => {
                          const result = await performEvent(action.id);
                          const nextState = await getGameState();
                          setGameState(nextState);
                          if (action.encounter_id) setEncounter(nextState.active_encounter || nextState.last_encounter_result || null);
                          return result;
                        }, 'Action completed.');
                      }}
                    >
                      {action.type.toLowerCase() === 'sponsor' ? 'Try luck with Sponsor' : `Start ${action.name}`}
                    </button>
                  </>
                ),
              });
            }}>Details</button>
          </td>
        </tr>
        );
      })}
        </tbody>
      </table>
      )}
    </div>
  );

  const renderChampionships = () => {
    const championships = catalog.quests
      .filter((quest) => quest.name.toLowerCase().includes(championshipFilter.toLowerCase()))
      .filter((quest) => catalog.events
        .filter((event) => event.quest_id === quest.id)
        .some((event) => selectedVehicleMatches(event, championshipVehicleFilter)))
      .sort((left, right) => {
        let comparison = 0;
        if (championshipSort === 'races') {
          comparison = catalog.events.filter((event) => event.quest_id === left.id).length
            - catalog.events.filter((event) => event.quest_id === right.id).length;
        } else if (championshipSort === 'status') {
          const leftJoined = gameState.quest_memberships.some((membership) => membership.quest_id === left.id);
          const rightJoined = gameState.quest_memberships.some((membership) => membership.quest_id === right.id);
          comparison = Number(rightJoined) - Number(leftJoined) || left.name.localeCompare(right.name);
        } else if (championshipSort === 'points') {
          const questPoints = (questId: string) => {
            const run = gameState.quest_runs
              .filter((entry) => entry.quest_id === questId)
              .sort((a, b) => b.sequence - a.sequence)[0];
            if (run) return run.points;
            return new Set(
              gameState.event_history
                .filter((history) => catalog.events.some((event) => event.id === history.event_id && event.quest_id === questId))
                .map((history) => history.event_id),
            ).size;
          };
          comparison = questPoints(left.id) - questPoints(right.id);
        } else {
          comparison = left.name.localeCompare(right.name);
        }
        return championshipSortDirection === 'asc' ? comparison : -comparison;
      });
    const rewards = (value: string | undefined) => {
      const entries = (value || '').split(';').map((entry) => {
        const [position, amount] = entry.split(':').map((part) => part.trim());
        return { position: Number(position), amount: Number(amount) };
      }).filter((entry) => Number.isInteger(entry.position) && entry.position > 0 && Number.isFinite(entry.amount));
      return entries.sort((a, b) => a.position - b.position);
    };
    return (
      <div style={styles.grid}>
        <section style={{ ...styles.card, ...styles.stickyControls, ...styles.raceFilterArea, gridColumn: '1 / -1' }}>
          <div style={styles.filtersRow}>
          <label>
            {getLabel(catalog, 'filter_name', 'Filter')} {questPlural.toLowerCase()}: {' '}
            <input
              style={styles.filterInput}
              value={championshipFilter}
              onChange={(event) => setChampionshipFilter(event.target.value)}
              placeholder={`${getLabel(catalog, 'search_name', 'Search by')} ${questName.toLowerCase()} ${getLabel(catalog, 'name_name', 'name')}`}
            />
          </label>
          <label>
            Filter by vehicle:{' '}
            <select
              value={championshipVehicleFilter[0] || ''}
              onChange={(change) => {
                setChampionshipVehicleFilter(change.target.value ? [change.target.value] : []);
              }}
              style={{ ...styles.filterInput, minWidth: '14rem' }}
            >
              <option value="">No vehicle filter</option>
              {ownedVehicles.map((vehicle) => <option key={vehicle.definition_id} value={vehicle.definition_id}>{vehicle.name}</option>)}
            </select>
          </label>
          </div>
        </section>
        <table style={{ ...styles.dataTable, gridColumn: '1 / -1' }}>
          <thead>
            <tr>
              {(['name', 'races', 'status', 'points'] as const).map((column) => (
                <th key={column} style={styles.dataTableHeader}>
                  {sortHeader(
                    column === 'name' ? questName : column === 'races' ? `Number of ${eventPlural.toLowerCase()}` : column === 'points' ? getLabel(catalog, 'progress_name', 'Progress') : 'Status',
                    column,
                    championshipSort,
                    championshipSortDirection,
                    (nextColumn) => {
                      const next = nextColumn as 'name' | 'races' | 'status' | 'points';
                      setChampionshipSortDirection(championshipSort === next ? championshipSortDirection === 'asc' ? 'desc' : 'asc' : 'asc');
                      setChampionshipSort(next);
                    },
                  )}
                </th>
              ))}
              <th style={styles.dataTableHeader}>Action</th>
            </tr>
          </thead>
          <tbody>
          {championships.map((quest) => {
          const races = catalog.events.filter((event) => event.quest_id === quest.id);
          const questRun = gameState.quest_runs
            .filter((run) => run.quest_id === quest.id)
            .sort((a, b) => b.sequence - a.sequence)[0];
          const progressByEvent = questRun?.events || {};
          const progressEvents = questRun
            ? Object.keys(progressByEvent).map((eventId) =>
              catalog.events.find((event) => event.id === eventId) || {
                id: eventId,
                name: eventId,
                quest_event_required: progressByEvent[eventId].required,
                position_rewards: '',
              } as typeof catalog.events[number],
            )
            : races;
          const completedEventIds = new Set(
            questRun
              ? Object.values(progressByEvent)
                .filter((progress) => progress.status === 'Recorded')
                .map((progress) => progress.event_id)
              : gameState.event_history
                .filter((history) => races.some((race) => race.id === history.event_id))
                .map((history) => history.event_id),
          );
          const pendingEventIds = new Set(
            gameState.pending_events
              .filter((pending) => races.some((race) => race.id === pending.event_id))
              .map((pending) => pending.event_id),
          );
          const requiredRaces = races.filter((race) => race.quest_event_required !== false);
          const completedRequired = questRun
            ? Object.values(progressByEvent).filter((progress) => progress.required && progress.status === 'Recorded').length
            : requiredRaces.filter((race) => completedEventIds.has(race.id)).length;
          const progressText = questRun
            ? `${completedRequired}/${Object.values(progressByEvent).filter((progress) => progress.required).length} ${getLabel(catalog, 'event_count_name', eventPlural).toLowerCase()} · ${questRun.points} ${getLabel(catalog, 'points_name', 'points')}`
            : `${completedRequired}/${requiredRaces.length} ${getLabel(catalog, 'event_count_name', eventPlural).toLowerCase()}`;
          const questReceipts = questRun
            ? gameState.reward_receipts.filter((receipt) => receipt.source_run_id === questRun.run_id)
            : [];
          const recordedResults = (gameState.championship_results || [])
            .filter((result) => races.some((race) => race.id === result.event_id));
          const championshipStandings = (() => {
            const points = new Map<string, number>();
            const addPoints = (name: string, event: (typeof races)[number], position: number) => {
              if (!name.trim() || position <= 0 || position > prizePositions(event)) return;
              points.set(name, (points.get(name) || 0) + Math.max(1, races.length - position + 1));
            };
            recordedResults.forEach((result) => {
              const event = races.find((candidate) => candidate.id === result.event_id);
              if (!event) return;
              addPoints(player.name || 'You', event, result.player_position);
              result.competitors.forEach((competitor) => addPoints(competitor.name, event, competitor.position));
            });
            return Array.from(points.entries())
              .map(([name, total]) => ({ name, points: total }))
              .sort((left, right) => right.points - left.points || left.name.localeCompare(right.name));
          })();
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
          const sponsorRequirementsPending = !joined && races.some((race) => eventRequirementsPending(race));
          const pendingRequirements = !joined && sponsorRequirementsPending;
          const missingText = missingRequirements.join(' | ');
          const showProgress = () => {
            if (!joined) return;
            setSelectedDetail({
              title: `${quest.name} ${getLabel(catalog, 'progress_name', 'progress').toLowerCase()}`,
              descriptionPath: '',
              hideDescription: true,
              footer: (
                <div style={styles.standingsList}>
                  <strong>{progressText}</strong>
                  {championshipStandings.length > 0 && (
                    <>
                      <strong>Current standings</strong>
                      {championshipStandings.map((standing, index) => (
                        <span
                          key={standing.name}
                          style={{
                            ...styles.standingRow,
                            ...(standing.name === (player.name || 'You') ? { fontWeight: 'bold' } : {}),
                          }}
                        >
                          {index + 1}. {standing.name} — {standing.points} {getLabel(catalog, 'points_name', 'points')}
                        </span>
                      ))}
                    </>
                  )}
                  <strong>Races</strong>
                  {progressEvents.map((race) => {
                    const progress = progressByEvent[race.id];
                    const recordedResult = [...recordedResults]
                      .filter((result) => result.event_id === race.id)
                      .sort((left, right) => right.race_day - left.race_day)[0];
                    const history = progress ? undefined : [...gameState.event_history].reverse().find((entry) => entry.event_id === race.id);
                    const status = recordedResult?.player_position
                      ? `${getLabel(catalog, 'position_name', 'Position')} ${recordedResult.player_position}`
                      : progress
                      ? `${progress.status}${progress.points ? ` (${progress.points} ${getLabel(catalog, 'points_name', 'points')})` : ''}`
                      : history
                        ? `${getLabel(catalog, 'completed_name', 'Completed')}: ${history.result || getLabel(catalog, 'recorded_name', 'Recorded')}`
                        : pendingEventIds.has(race.id)
                          ? getLabel(catalog, 'pending_name', 'Pending result')
                          : getLabel(catalog, 'not_started_name', 'Not started');
                    return (
                      <span key={race.id} style={styles.standingRow}>
                      {race.name}{race.quest_event_required === false ? ` (${getLabel(catalog, 'optional_name', 'optional')})` : ''} — {status}
                    </span>
                  );
                  })}
                  {questRun && (
                    <>
                      <strong>{getLabel(catalog, 'quest_status_name', 'Quest status')}: {questRun.status}</strong>
                      {questReceipts.length > 0 && (
                        <>
                          <strong>{getLabel(catalog, 'reward_receipts_name', 'Reward receipts')}</strong>
                          {questReceipts.map((receipt) => (
                            <span key={receipt.receipt_id} style={styles.standingRow}>
                              {receipt.reward_id}
                              {receipt.standing ? ` — ${getLabel(catalog, 'position_name', 'Position')} ${receipt.standing}` : ''}
                              {receipt.level_or_tier ? ` (${receipt.level_or_tier})` : ''}
                            </span>
                          ))}
                        </>
                      )}
                    </>
                  )}
                </div>
              ),
            });
          };
          return (
            <tr
              key={quest.id}
              className="data-table-row"
              style={{
                background: pendingRequirements
                  ? 'var(--pending-background)'
                  : joined ? 'var(--warning-background)' : undefined,
              }}
              onClick={() => setSelectedDetail({
                title: quest.name,
                descriptionPath: quest.description_html,
                footer: (
                  <>
                    <span>{progressText}</span>
                    <strong>{getLabel(catalog, 'event_prizes_name', `${eventName} prizes by position`)}</strong>
                    {races.map((race) => (
                      <span key={race.id}>
                        {race.name}: {rewards(race.position_rewards).length > 0
                          ? rewards(race.position_rewards).map((prize) =>
                          `${prize.position}${prize.position === 1 ? 'st' : prize.position === 2 ? 'nd' : prize.position === 3 ? 'rd' : 'th'} ${currency}${prize.amount.toLocaleString()}`,
                        ).join(' | ')
                          : getLabel(catalog, 'no_rewards_configured_name', 'No configured rewards')}
                      </span>
                    ))}
                    <strong>{getLabel(catalog, 'quest_prizes_name', `${questName} prizes by final position`)}</strong>
                    <span>{rewards(quest.championship_rewards).length > 0
                      ? rewards(quest.championship_rewards).map((prize) =>
                      `${prize.position}${prize.position === 1 ? 'st' : prize.position === 2 ? 'nd' : prize.position === 3 ? 'rd' : 'th'} ${currency}${prize.amount.toLocaleString()}`,
                    ).join(' | ')
                      : getLabel(catalog, 'no_rewards_configured_name', 'No configured rewards')}</span>
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
                            .catch((error) => showMessage(String(error)));
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
              <td style={styles.dataTableCell}><button style={styles.linkButton} onClick={() => setSelectedDetail({
                title: quest.name,
                descriptionPath: quest.description_html,
                footer: <span>{progressText}</span>,
              })}>{quest.name}</button></td>
              <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>{races.length}</td>
              <td style={styles.dataTableCell}>
                <button
                  type="button"
                  disabled={!joined}
                  style={sponsorRequirementsPending
                    ? { ...styles.championshipMissing, color: 'var(--warning-text)' }
                    : joined ? styles.championshipStatus : styles.championshipMissing}
                  onClick={(event) => {
                    event.stopPropagation();
                    showProgress();
                  }}
                >
                  {sponsorRequirementsPending ? 'Pending requirements' : joined ? 'Joined' : 'Not enrolled'}
                </button>
              </td>
              <td style={{ ...styles.dataTableCell, ...styles.numericCell }}>{progressText}</td>
              <td style={styles.dataTableCell}>
                {!joined && (
                  <button
                    disabled={missingRequirements.length > 0}
                    onClick={(event) => {
                      event.stopPropagation();
                      void joinQuest(quest.id).then(setGameState).catch((error) => showMessage(String(error)));
                    }}
                  >
                    Join
                  </button>
                )}
                {joined && <button onClick={(event) => { event.stopPropagation(); showProgress(); }}>{getLabel(catalog, 'progress_name', 'Progress')}</button>}
              </td>
            </tr>
          );
        })}
          </tbody>
        </table>
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
      <style>{`
        .data-table-row:hover > td {
          background: color-mix(in srgb, var(--surface-background) 88%, white) !important;
        }
      `}</style>
      <header style={styles.header}>
        <div style={styles.headerMain}>
          <h1>{applicationTitle}</h1>
          <div style={styles.headerStatus}>
            <span style={styles.headerDay}>{formatHeaderDay(gameState.current_day)}</span>
            <span style={styles.headerMoney}>{currency}{budget.toLocaleString()}</span>
            {(() => {
              const stamina = getCharacteristic(player, recoveryCharacteristicId);
              const definition = catalog.player_characteristics.find((entry) => entry.id === recoveryCharacteristicId);
              const minimum = definition && Number.isFinite(definition.min_value) ? definition.min_value : 0;
              const maximum = definition && Number.isFinite(definition.max_value) ? definition.max_value : 100;
              const percentage = Math.max(0, Math.min(100, ((stamina - minimum) / Math.max(1, maximum - minimum)) * 100));
              return (
                <span style={styles.headerStamina} title="Current stamina">
                  <span>Stamina: {Math.round(stamina)}</span>
                  <span style={styles.headerStaminaTrack}>
                    <span style={{
                      ...styles.headerStaminaFill,
                      width: `${percentage}%`,
                      background: percentage > 60 ? 'var(--progress-high)' : percentage > 30 ? 'var(--progress-medium)' : 'var(--progress-low)',
                    }} />
                  </span>
                </span>
              );
            })()}
          </div>
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
            onClick={openConfig}
          >
            <img src={getLabel(catalog, headerIconKeys.settings, '/img/settings.svg')} alt="" style={styles.headerIcon} />
          </button>
        </div>
      </header>
      {message && (
        <div
          style={messageIsWarning ? styles.warningBanner : styles.banner}
          onClick={() => showMessage('')}
          role={messageIsWarning ? 'alert' : undefined}
        >
          {message}
        </div>
      )}
      <nav style={styles.nav}>
        {([
          ['dashboard', 'Dashboard'],
          ['inventory', inventoryName],
          ['dealer', dealerName],
          ['events', eventPlural],
          ['championships', questPlural],
          ['activities', getLabel(catalog, 'activity_name', 'Activities') ],
          ...(hasCompanion ? [['companions', companionTabName] as const] : []),
        ] as const).map(([key, title]) => (
          <button
            key={key}
            style={key === tab
              ? { ...styles.navTab, ...styles.selectedTab, ...(key === 'dashboard' ? styles.dashboardTab : {}) }
              : { ...styles.navTab, ...(key === 'dashboard' ? styles.dashboardTab : {}) }}
            onClick={() => {
              if (key === 'inventory') setInventoryTab(inventoryTabs[0]?.id || '');
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
        {tab === 'companions' && hasCompanion && renderCompanions()}
      </main>
      {eventLogOpen && (
        <EventLogModal
          entries={gameState.event_log || []}
          formatDay={formatGameDay}
          onClose={() => setEventLogOpen(false)}
        />
      )}
      <AlertModal
        alerts={gameState.pending_alerts}
        onDismiss={handleAlertDismiss}
        onConfigure={openConfig}
      />
      <ConfigModal
        isOpen={configOpen}
        currentPath={gameState.dataset_path}
        gameDirectory={gameDirectory}
        resultsDirectory={resultsDirectory}
        onClose={() => setConfigOpen(false)}
        onReloadDataset={async (path) => {
          const state = await reloadDataset(path);
          await saveAppConfig({
            dataset_path: path,
            fullscreen,
            window_width: windowWidth,
            window_height: windowHeight,
            game_directory: gameDirectory,
            results_directory: resultsDirectory,
          });
          rememberDatasetPath(path);
          setGameState(state);
          setConfigOpen(false);
        }}
        popupCategories={gameState.popup_categories}
        onPopupCategoriesChange={async (categories) => setGameState(await setPopupCategories(categories))}
        fullscreen={fullscreen}
        onFullscreenChange={async (nextFullscreen) => {
          await getCurrentWindow().setFullscreen(nextFullscreen);
          setFullscreen(nextFullscreen);
        }}
        windowWidth={windowWidth}
        windowHeight={windowHeight}
        onWindowSizeChange={async (width, height) => {
          await getCurrentWindow().setSize(new LogicalSize(width, height));
          setWindowWidth(width);
          setWindowHeight(height);
        }}
        onGameDirectoryChange={setGameDirectory}
        onResultsDirectoryChange={setResultsDirectory}
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
          onMigrate={async (slot) => {
            const loaded = await migrateSaveGameFrom(gameState.dataset_path, slot);
            await applyLoadedState(loaded);
            setSaveModal(null);
            setFeedback({ title: 'Save migrated', message: `Save slot '${slot}' was migrated and loaded.` });
          }}
          onLoadAtOwnRisk={async (slot) => {
            const loaded = await loadSaveGameAtOwnRisk(gameState.dataset_path, slot);
            await applyLoadedState(loaded);
            setSaveModal(null);
            setFeedback({
              title: 'Save loaded at your own risk',
              message: `Save slot '${slot}' was loaded without matching the dataset revision. Future results may differ.`,
            });
          }}
          onDelete={async (slot) => {
            await deleteSaveSlot(gameState.dataset_path, slot);
            setSaveSlots(await listSaveSlots(gameState.dataset_path));
          }}
        />
      )}
      {selectedDetail && (
        <DetailModal
          title={selectedDetail.title}
          descriptionPath={selectedDetail.descriptionPath}
          hideDescription={selectedDetail.hideDescription}
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
      {rentalEventId && (() => {
        const event = catalog.events.find((entry) => entry.id === rentalEventId);
        if (!event) return null;
        return (
          <RentalModal
            eventName={event.name}
            currency={currency}
            cars={rentalCarsFor(event)}
            onClose={() => setRentalEventId(null)}
            onRent={async (objectId) => {
              const nextState = await rentEvent(objectId, event.id);
              setGameState(nextState);
              setRentalEventId(null);
              setSelectedDetail(null);
              const matchingPending = nextState.pending_events
                .filter((entry) => entry.event_id === event.id && entry.object_id === objectId);
              const pending = matchingPending[matchingPending.length - 1];
              if (pending) {
                setResultPrompt({
                  id: pending.id,
                  eventName: event.name,
                  damageOptions: [],
                  championship: Boolean(event.quest_id),
                  pluginEnabled: event.tags.split(';').some((tag) => tag.trim().toLowerCase() === 'race'),
                });
              }
            }}
          />
        );
      })()}
      {resultPrompt && (
        <ResultPromptModal
          eventName={resultPrompt.eventName}
          damageOptions={resultPrompt.damageOptions}
          championship={resultPrompt.championship}
          previousCompetitors={resultPrompt.previousCompetitors}
          championshipDrivers={resultPrompt.championshipDrivers}
          scoringPositions={resultPrompt.scoringPositions}
          pluginEnabled={resultPrompt.pluginEnabled}
          competitorLabel={competitorLabel}
          competitorPluralLabel={getLabel(catalog, 'competitor_plural', 'Competitors')}
          onClose={closeResultPrompt}
          onOpenPlugin={async () => autodetectRaceResultsPlugin(resultPrompt.id)}
          onAutodetectPlugin={async (resultsFile) =>
            autodetectRaceResultsPlugin(resultPrompt.id, resultsFile)
          }
          onChooseResultFile={async () => selectRaceResultsFile(resultsDirectory)}
          onSubmitPlugin={async (pluginResponse) => {
            const eventResult = await submitEventResult(
              resultPrompt.id,
              pluginResponse.result,
              pluginResponse.damage_type,
              pluginResponse.player_position || undefined,
              pluginResponse.competitors,
              pluginResponse,
            );
            setResultPrompt(null);
            setGameState(await getGameState());
            setFeedback({
              title: 'Race result recorded',
              message: `Your finishing position: ${
                pluginResponse.player_position > 0 ? pluginResponse.player_position : 'Not classified'
              }.\n\n${eventResult.message}`,
            });
          }}
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
            setFeedback({
              title: `${questName} result recorded`,
              message: `Your finishing position: ${
                playerPosition > 0 ? playerPosition : 'Not classified'
              }.\n\n${eventResult.message}`,
            });
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
      {introVisible && <IntroPlugin onClose={() => setIntroVisible(false)} />}
      {sponsorNegotiation && (
        <SponsorNegotiationModal
          initialState={sponsorNegotiation}
          onGameState={setGameState}
          onClose={() => setSponsorNegotiation(null)}
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
              showMessage(String(error));
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
  headerMain: { minWidth: 'max-content', flex: 1 },
  headerStatus: { display: 'flex', alignItems: 'center', gap: '1rem', flexWrap: 'nowrap' },
  headerDay: { flex: '0 0 14rem', whiteSpace: 'nowrap' },
  headerMoney: { flex: '0 0 8rem', whiteSpace: 'nowrap' },
  headerStamina: { display: 'inline-flex', alignItems: 'center', gap: '0.45rem', flex: '0 0 14rem', whiteSpace: 'nowrap' },
  headerStaminaTrack: { display: 'inline-block', width: 110, height: 10, background: 'var(--progress-background)', border: '1px solid var(--surface-border)', borderRadius: 999, overflow: 'hidden', verticalAlign: 'middle' },
  headerStaminaFill: { display: 'block', height: '100%', borderRadius: 999 },
  nav: { flexShrink: 0, display: 'flex', gap: '0.5rem', margin: '1rem 0' },
  navTab: { border: '1px solid var(--control-border)', borderRadius: '6px', background: 'var(--surface-background)', color: 'var(--secondary-text)', padding: '0.65rem 0.9rem', cursor: 'pointer' },
  subnav: { display: 'flex', gap: '0.4rem', flexWrap: 'wrap', alignItems: 'center' },
  marketSubnav: { display: 'flex', gap: '0.4rem', flexWrap: 'nowrap', alignItems: 'center', flex: 1, minWidth: 0, overflowX: 'auto' },
  stickyControls: { position: 'sticky', top: 0, zIndex: 10, background: 'var(--surface-background)', paddingTop: '0.5rem', paddingBottom: '0.5rem' },
  raceFilterArea: { paddingTop: '0.85rem', paddingBottom: '0.85rem' },
  filtersRow: { display: 'flex', gap: '1rem', flexWrap: 'wrap', alignItems: 'center' },
  selectedTab: { background: 'var(--primary-accent)', color: 'var(--white-text)' },
  selectedPill: {
    border: '1px solid var(--primary-accent-border)',
    borderRadius: '999px',
    background: 'var(--primary-accent)',
    color: 'var(--white-text)',
    padding: '0.4rem 0.85rem',
    cursor: 'pointer',
    boxShadow: '0 0 0 2px var(--primary-accent-border)',
  },
  grid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1rem' },
  card: { background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: '8px', padding: '1rem' },
  clickableCard: { cursor: 'pointer' },
  market: { display: 'grid', gap: '1rem' },
  marketGrid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '0.75rem' },
  marketCard: { padding: '0.65rem 0.8rem' },
  marketAvailable: { display: 'flex', alignItems: 'center', gap: '0.75rem' },
  marketDetails: { display: 'grid', gap: '0.15rem', minWidth: 0 },
  marketTitleRow: { display: 'flex', alignItems: 'baseline', gap: '1.5rem', minWidth: 0 },
  marketPrice: { flexShrink: 0 },
  marketAvailablePrice: { fontSize: '1.15rem', fontWeight: 600 },
  marketUnavailablePrice: { color: 'color-mix(in srgb, var(--danger-text) 75%, white)' },
  marketPurchaseRow: { display: 'flex', alignItems: 'center', gap: '0.75rem' },
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
  marketToolbar: { display: 'flex', alignItems: 'center', gap: '1rem', width: '100%', minHeight: '3rem', flexWrap: 'nowrap' },
  marketControls: { display: 'flex', gap: '0.75rem', flexWrap: 'wrap', marginLeft: 'auto', flexShrink: 0 },
  marketPill: { paddingTop: '0.7rem', paddingBottom: '0.7rem' },
  marketFilterInput: { height: '3rem', boxSizing: 'border-box' },
  filterInput: { height: '2.5rem', boxSizing: 'border-box' },
  dataTable: { width: '100%', borderCollapse: 'collapse', background: 'var(--surface-background)' },
  dataTableHeader: { padding: '0.65rem 0.75rem', textAlign: 'left', borderBottom: '2px solid var(--surface-border)', whiteSpace: 'nowrap' },
  numericHeader: { textAlign: 'right' },
  dataTableCell: { padding: '0.7rem 0.75rem', textAlign: 'left', verticalAlign: 'middle', borderBottom: '1px solid var(--surface-border)' },
  numericCell: { textAlign: 'right', whiteSpace: 'nowrap' },
  sortHeader: { display: 'inline-flex', alignItems: 'center', gap: '0.35rem', padding: 0, border: 0, background: 'transparent', color: 'inherit', font: 'inherit', fontWeight: 700, cursor: 'pointer' },
  sortTriangle: { fontSize: '0.65rem', lineHeight: 1, color: 'var(--link-text)' },
  row: { display: 'flex', justifyContent: 'space-between', gap: '1rem', alignItems: 'center', padding: '0.5rem 0', borderBottom: '1px solid var(--surface-border)' },
  nameCard: { background: 'var(--surface-background)', border: '1px solid var(--surface-border)', borderRadius: '8px', padding: '1.25rem', color: 'var(--primary-text)', fontSize: '1.1rem', fontWeight: 700, textAlign: 'left', cursor: 'pointer' },
  marketItemButton: { display: 'flex', width: '100%', alignItems: 'center', gap: '1rem', background: 'transparent', border: 0, color: 'var(--primary-text)', textAlign: 'left', cursor: 'pointer', padding: 0 },
  marketThumbnail: { flex: '0 0 96px', width: 96, height: 64, objectFit: 'contain', borderRadius: 6, background: 'var(--app-background)' },
  marketUnavailableName: { color: 'color-mix(in srgb, var(--danger-text) 75%, white)', fontWeight: 700 },
  marketUnavailableTitle: { color: 'color-mix(in srgb, var(--danger-text) 75%, white)' },
  marketOwned: { display: 'block', color: 'var(--muted-text)' },
  standingsList: { display: 'flex', flexDirection: 'column', alignItems: 'stretch', width: '100%', gap: '0.4rem' },
  standingRow: { display: 'block' },
  requirementWarning: { display: 'flex', flexDirection: 'column', gap: '0.2rem', margin: '0.5rem 0', padding: '0.6rem', border: '1px solid var(--warning-text)', borderRadius: '6px', color: 'var(--warning-text)' },
  championshipMissing: { marginTop: '0.4rem', marginLeft: '2rem', marginRight: '3rem', color: 'var(--danger-text)', fontSize: '0.85rem', fontWeight: 600, background: 'transparent', border: 0, padding: 0 },
  championshipStatus: { marginTop: '0.4rem', marginLeft: '2rem', marginRight: '3rem', color: 'var(--success-text)', fontSize: '0.85rem', fontWeight: 600, background: 'transparent', border: 0, padding: 0, cursor: 'pointer' },
  championshipSummary: { display: 'flex', alignItems: 'center', justifyContent: 'flex-start', gap: '1rem' },
  disabledJoinButton: {
    color: 'color-mix(in srgb, var(--danger-text) 65%, white)',
    cursor: 'not-allowed',
    opacity: 1,
  },
  eventDays: { display: 'block', marginTop: '0.35rem', color: 'var(--subtle-text)', fontSize: '0.85rem', fontWeight: 400 },
  unavailableNotice: { color: 'var(--warning-text)', fontWeight: 700 },
  linkButton: { background: 'none', border: 0, color: 'var(--link-text)', fontSize: '1rem', cursor: 'pointer', padding: 0 },
  pillButton: { border: '1px solid var(--control-border)', borderRadius: '999px', background: 'var(--surface-background)', color: 'var(--secondary-text)', padding: '0.4rem 0.85rem', cursor: 'pointer' },
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
  inventoryObjectImage: { display: 'block', width: 180, height: 120, objectFit: 'contain', borderRadius: 8, background: 'var(--app-background)', marginBottom: '0.75rem' },
  pendingEvent: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '1rem', flexWrap: 'wrap', padding: '0.75rem 0', borderBottom: '1px solid var(--surface-border)' },
  resultControls: { display: 'flex', gap: '0.5rem', flexWrap: 'wrap' },
  muted: { color: 'var(--muted-text)' },
  successMessage: { color: 'var(--success-text)' },
  errorBanner: { background: 'var(--error-background)', border: '1px solid var(--error-border)', borderRadius: '8px', padding: '0.75rem', color: 'var(--error-light-text)' },
  banner: { background: 'var(--info-background)', padding: '0.75rem', margin: '1rem 0', cursor: 'pointer' },
  warningBanner: { background: 'var(--warning-background)', border: '1px solid var(--warning-text)', borderRadius: '8px', padding: '0.75rem', margin: '1rem 0', color: 'var(--warning-text)', cursor: 'pointer' },
  standings: { width: '100%', borderCollapse: 'collapse' },
};

export default App;
