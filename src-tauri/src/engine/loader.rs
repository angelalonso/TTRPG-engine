use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectData {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub name: String,
    pub price: f64,
    #[serde(default)]
    pub requirement_group: String,
    #[serde(default)]
    pub paddock_cred_bonus: f64,
    #[serde(default)]
    pub cost_1: String,
    #[serde(default)]
    pub cost_2: String,
    #[serde(default)]
    pub cost_3: String,
    #[serde(default)]
    pub cost_4: String,
    #[serde(default)]
    pub cost_5: String,
    #[serde(default)]
    pub cost_6: String,
    #[serde(default)]
    pub cost_7: String,
    #[serde(default)]
    pub cost_8: String,
    #[serde(default)]
    pub cost_9: String,
    #[serde(default)]
    pub cost_10: String,
    #[serde(default)]
    pub cost_11: String,
    #[serde(default)]
    pub cost_12: String,
    #[serde(default)]
    pub cost_13: String,
    #[serde(default)]
    pub cost_14: String,
    #[serde(default)]
    pub cost_15: String,
    #[serde(default)]
    pub service_1_interval_days: u32,
    #[serde(default)]
    pub service_2_interval_days: u32,
    #[serde(default)]
    pub service_3_interval_days: u32,
    #[serde(default)]
    pub service_4_interval_days: u32,
    #[serde(default)]
    pub service_5_interval_days: u32,
    #[serde(default)]
    pub service_6_interval_days: u32,
    #[serde(default)]
    pub service_7_interval_days: u32,
    #[serde(default)]
    pub service_8_interval_days: u32,
    #[serde(default)]
    pub service_9_interval_days: u32,
    #[serde(default)]
    pub service_10_interval_days: u32,
    #[serde(default)]
    pub service_11_interval_days: u32,
    #[serde(default)]
    pub service_12_interval_days: u32,
    #[serde(default)]
    pub service_13_interval_days: u32,
    #[serde(default)]
    pub service_14_interval_days: u32,
    #[serde(default)]
    pub service_15_interval_days: u32,
    #[serde(default = "default_resale_initial_percent")]
    pub resale_initial_percent: f64,
    #[serde(default = "default_resale_annual_percent")]
    pub resale_annual_percent: f64,
    #[serde(default = "default_resale_min_percent")]
    pub resale_min_percent: f64,
    #[serde(
        default,
        alias = "description",
        alias = "description_path",
        alias = "html"
    )]
    pub description_html: String,
    #[serde(default)]
    pub license_level: u32,
    #[serde(default)]
    pub license_previous_id: String,
    #[serde(default)]
    pub requires_object_ids: String,
    #[serde(default, deserialize_with = "deserialize_zero_f64")]
    pub license_fee: f64,
    #[serde(default)]
    pub lifetime_days: u32,
    #[serde(default, deserialize_with = "deserialize_zero_u32")]
    pub availability_days: u32,
    #[serde(default)]
    pub image_path: String,
    #[serde(default)]
    pub trophy_championship: String,
    #[serde(default)]
    pub trophy_position: u32,
    #[serde(default = "default_trophy_level")]
    pub trophy_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostData {
    pub id: String,
    pub name: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerCharacteristicData {
    pub id: String,
    pub name: String,
    pub value: f64,
    #[serde(
        default = "default_characteristic_min",
        deserialize_with = "deserialize_min_bound"
    )]
    pub min_value: f64,
    #[serde(
        default = "default_characteristic_max",
        deserialize_with = "deserialize_max_bound"
    )]
    pub max_value: f64,
}

fn default_characteristic_min() -> f64 {
    f64::NEG_INFINITY
}

fn default_characteristic_max() -> f64 {
    f64::INFINITY
}

fn deserialize_min_bound<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<f64>::deserialize(deserializer)?.unwrap_or_else(default_characteristic_min))
}

fn deserialize_max_bound<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<f64>::deserialize(deserializer)?.unwrap_or_else(default_characteristic_max))
}

fn deserialize_zero_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberOrString {
        Number(f64),
        String(String),
        Null,
    }
    match NumberOrString::deserialize(deserializer)? {
        NumberOrString::Number(value) => Ok(value),
        NumberOrString::String(value) if value.trim().is_empty() => Ok(0.0),
        NumberOrString::String(value) => value.trim().parse().map_err(serde::de::Error::custom),
        NumberOrString::Null => Ok(0.0),
    }
}

fn deserialize_zero_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberOrString {
        Number(u32),
        String(String),
        Null,
    }
    match NumberOrString::deserialize(deserializer)? {
        NumberOrString::Number(value) => Ok(value),
        NumberOrString::String(value) if value.trim().is_empty() => Ok(0),
        NumberOrString::String(value) => value.trim().parse().map_err(serde::de::Error::custom),
        NumberOrString::Null => Ok(0),
    }
}

fn default_resale_initial_percent() -> f64 {
    0.9
}

fn default_resale_annual_percent() -> f64 {
    0.9
}

fn default_resale_min_percent() -> f64 {
    0.1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub day_of_year: u32,
    #[serde(default)]
    pub entry_fee: f64,
    #[serde(default)]
    pub reward_pool: f64,
    #[serde(default)]
    pub charisma_reward: f64,
    #[serde(default)]
    pub duration_value: u32,
    #[serde(default = "default_duration_unit")]
    pub duration_unit: String,
    #[serde(default)]
    pub tags: String,
    #[serde(
        default,
        alias = "description",
        alias = "description_path",
        alias = "html"
    )]
    pub description_html: String,
    #[serde(default)]
    pub required_license_id: String,
    #[serde(default)]
    pub required_object_ids: String,
    #[serde(default, alias = "championship_id")]
    pub quest_id: String,
    #[serde(default)]
    pub position_rewards: String,
    #[serde(rename = "type", default = "default_event_type")]
    pub event_type: String,
    #[serde(default, deserialize_with = "deserialize_zero_f64")]
    pub base_cost: f64,
    #[serde(
        default = "default_event_stamina_cost",
        deserialize_with = "deserialize_zero_f64"
    )]
    pub stamina_cost: f64,
    #[serde(default, deserialize_with = "deserialize_zero_f64")]
    pub risk_factor: f64,
    #[serde(
        default = "default_event_success_rate",
        deserialize_with = "deserialize_zero_f64"
    )]
    pub success_rate: f64,
    #[serde(default, deserialize_with = "deserialize_zero_f64")]
    pub payout: f64,
    #[serde(default = "default_payout_frequency_type")]
    pub payout_freq_type: String,
    #[serde(default, deserialize_with = "deserialize_zero_u32")]
    pub payout_freq: u32,
    #[serde(default = "default_payout_frequency_unit")]
    pub payout_freq_unit: String,
    #[serde(default)]
    pub sponsor_quest_id: String,
    #[serde(default)]
    pub sponsor_object_id: String,
    #[serde(default)]
    pub sponsor_payouts: String,
    #[serde(default)]
    pub sponsor_equipment_ids: String,
    #[serde(default)]
    pub encounter_id: String,
    #[serde(default = "default_event_resolution_method")]
    pub resolution_method: String,
}

fn default_event_stamina_cost() -> f64 {
    0.0
}

fn default_payout_frequency_type() -> String {
    "once".into()
}

fn default_payout_frequency_unit() -> String {
    "day".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventOutcomeData {
    pub event_id: String,
    pub outcome_id: String,
    #[serde(default)]
    pub probability: f64,
    #[serde(default)]
    pub reward_pool_delta: f64,
    #[serde(default)]
    pub charisma_reward_delta: f64,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventResultData {
    pub result_id: String,
    #[serde(default)]
    pub event_id: String,
    #[serde(default)]
    pub event_tags: String,
    pub reported_result: String,
    #[serde(default = "default_probability")]
    pub probability: f64,
    #[serde(default)]
    pub reward_pool_delta: f64,
    #[serde(default)]
    pub effects: String,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestData {
    pub id: String,
    #[serde(rename = "type", default = "default_quest_type")]
    pub quest_type: String,
    pub name: String,
    #[serde(default = "default_quest_success_points")]
    pub success_points: f64,
    #[serde(default)]
    pub failure_points: f64,
    #[serde(default)]
    pub join_fee: f64,
    #[serde(default)]
    pub required_license_id: String,
    #[serde(
        default,
        alias = "description",
        alias = "description_path",
        alias = "html"
    )]
    pub description_html: String,
    #[serde(default)]
    pub championship_rewards: String,
    #[serde(default)]
    pub driver_names: String,
    #[serde(default = "default_trophy_level")]
    pub level: u32,
}

fn default_trophy_level() -> u32 {
    1
}

fn default_quest_success_points() -> f64 {
    10.0
}

fn default_quest_type() -> String {
    "generic".into()
}
fn default_duration_unit() -> String {
    "days".into()
}

fn default_event_type() -> String {
    "event".into()
}

fn default_event_resolution_method() -> String {
    "manual".into()
}

fn default_event_success_rate() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityData {
    pub id: String,
    pub name: String,
    pub activity_type: String,
    pub resolution_method: String,
    pub description_html: String,
    pub base_cost: f64,
    pub stamina_cost: f64,
    pub success_rate: f64,
    pub payout: f64,
    pub payout_freq_type: String,
    pub payout_freq: u32,
    pub payout_freq_unit: String,
    pub scheduled: bool,
    pub encounter_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObligationData {
    pub id: String,
    pub event_id: String,
    pub resource: String,
    pub amount: f64,
    pub interval: u32,
    pub interval_unit: String,
    #[serde(default)]
    pub due_days: String,
    #[serde(default)]
    pub max_payments: u32,
    #[serde(default)]
    pub fault_limit: u32,
    #[serde(default)]
    pub fault_consequence: String,
    #[serde(default)]
    pub completion_consequence: String,
    #[serde(default)]
    pub skip_when_sick: bool,
    #[serde(default)]
    pub fault_blocks_payout: bool,
    #[serde(default)]
    pub fault_title: String,
    #[serde(default)]
    pub fault_message: String,
    #[serde(default)]
    pub fault_log: String,
    #[serde(default)]
    pub limit_title: String,
    #[serde(default)]
    pub limit_message: String,
    #[serde(default)]
    pub limit_log: String,
    #[serde(default)]
    pub required_event_type: String,
    #[serde(default)]
    pub max_active: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostRule {
    pub id: String,
    pub cost_id: String,
    pub trigger_type: String,
    #[serde(default)]
    pub trigger_ref: String,
    #[serde(default = "default_multiplier")]
    pub amount_multiplier: f64,
    #[serde(default = "default_probability")]
    pub probability: f64,
    #[serde(default)]
    pub interval_days: u32,
    #[serde(default = "default_charge_mode")]
    pub charge_mode: String,
    #[serde(default = "default_resolution_mode")]
    pub resolution_mode: String,
    #[serde(default)]
    pub pending_message: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub damage_type: String,
    #[serde(default)]
    pub unavailable_days: u32,
    #[serde(default)]
    pub event_interval: u32,
    #[serde(default)]
    pub no_event_days: u32,
}

fn default_multiplier() -> f64 {
    1.0
}

fn default_probability() -> f64 {
    1.0
}

fn default_charge_mode() -> String {
    "immediate".into()
}

fn default_resolution_mode() -> String {
    "charge".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostCondition {
    pub rule_id: String,
    pub subject_type: String,
    pub subject_ref: String,
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameLabels {
    #[serde(default)]
    pub values: HashMap<String, String>,
}

impl GameLabels {
    pub fn get(&self, key: &str, fallback: &str) -> String {
        self.values
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameCatalog {
    pub player_characteristics: Vec<PlayerCharacteristicData>,
    pub objects: Vec<ObjectData>,
    pub costs: Vec<CostData>,
    pub cost_rules: Vec<CostRule>,
    pub cost_conditions: Vec<CostCondition>,
    pub events: Vec<EventData>,
    #[serde(default)]
    pub activities: Vec<ActivityData>,
    #[serde(default)]
    pub obligations: Vec<ObligationData>,
    #[serde(default)]
    pub event_outcomes: Vec<EventOutcomeData>,
    #[serde(default)]
    pub event_results: Vec<EventResultData>,
    #[serde(default, alias = "championships")]
    pub quests: Vec<QuestData>,
    pub labels: GameLabels,
    #[serde(default)]
    pub encounter_attributes: Vec<EncounterAttributeData>,
    #[serde(default)]
    pub encounter_actions: Vec<EncounterActionData>,
    #[serde(default)]
    pub encounter_objects: Vec<EncounterObjectData>,
    #[serde(default)]
    pub encounter_opponents: Vec<EncounterOpponentData>,
    #[serde(default)]
    pub encounter_outcomes: Vec<EncounterOutcomeData>,
    #[serde(default)]
    pub encounter_configs: Vec<EncounterConfigData>,
    #[serde(default)]
    pub dataset_warnings: Vec<String>,
}

impl GameCatalog {
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Self {
        let base = dir.as_ref();
        let mut dataset_warnings = Vec::new();
        let (labels, label_warnings) = parse_config_file_with_diagnostics(base.join("config.csv"));
        dataset_warnings.extend(label_warnings);
        macro_rules! load {
            ($name:literal, $type:ty) => {{
                let (rows, warnings) =
                    parse_csv_file_with_diagnostics::<$type, _>(base.join($name));
                dataset_warnings.extend(warnings);
                rows
            }};
        }

        let player_characteristics = load!("player.csv", PlayerCharacteristicData);
        let objects = load!("objects.csv", ObjectData);
        let costs = load!("costs.csv", CostData);
        let cost_rules = load!("cost_rules.csv", CostRule);
        let cost_conditions = load!("cost_rule_conditions.csv", CostCondition);
        let events = load!("events.csv", EventData);
        let event_outcomes = load!("event_outcomes.csv", EventOutcomeData);
        let event_results = load!("event_results.csv", EventResultData);
        let quests = load!("quests.csv", QuestData);
        let obligations = load!("obligations.csv", ObligationData);
        let encounter_attributes = load!("encounter_attributes.csv", EncounterAttributeData);
        let encounter_actions = load!("encounter_actions.csv", EncounterActionData);
        let encounter_objects = load!("encounter_objects.csv", EncounterObjectData);
        let encounter_opponents = load!("encounter_opponents.csv", EncounterOpponentData);
        let encounter_outcomes = load!("encounter_outcomes.csv", EncounterOutcomeData);
        let encounter_configs = load!("encounter_config.csv", EncounterConfigData);
        let activities = events
            .iter()
            .map(|event| ActivityData {
                id: event.id.clone(),
                name: event.name.clone(),
                activity_type: event.event_type.clone(),
                resolution_method: event.resolution_method.clone(),
                description_html: event.description_html.clone(),
                base_cost: event.base_cost,
                stamina_cost: event.stamina_cost,
                payout: event.payout,
                payout_freq_type: event.payout_freq_type.clone(),
                payout_freq: event.payout_freq,
                payout_freq_unit: event.payout_freq_unit.clone(),
                scheduled: event.day_of_year > 0,
                success_rate: event.success_rate,
                encounter_id: event.encounter_id.clone(),
            })
            .collect();

        Self {
            player_characteristics,
            objects,
            costs,
            cost_rules,
            cost_conditions,
            events,
            activities,
            obligations,
            event_outcomes,
            event_results,
            quests,
            labels,
            encounter_attributes,
            encounter_actions,
            encounter_objects,
            encounter_opponents,
            encounter_outcomes,
            encounter_configs,
            dataset_warnings,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DatasetValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl DatasetValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn validate_dataset_directory<P: AsRef<Path>>(dir: P) -> DatasetValidationReport {
    let base = dir.as_ref();
    let catalog = GameCatalog::load_from_directory(base);
    let mut report = DatasetValidationReport::default();
    report.warnings.extend(catalog.dataset_warnings.clone());

    validate_unique_ids(
        &mut report,
        "player characteristic",
        catalog
            .player_characteristics
            .iter()
            .map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "object",
        catalog.objects.iter().map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "event",
        catalog.events.iter().map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "quest",
        catalog.quests.iter().map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "cost",
        catalog.costs.iter().map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "obligation",
        catalog.obligations.iter().map(|row| row.id.as_str()),
    );

    let object_ids: HashSet<&str> = catalog.objects.iter().map(|row| row.id.as_str()).collect();
    let event_ids: HashSet<&str> = catalog.events.iter().map(|row| row.id.as_str()).collect();
    let quest_ids: HashSet<&str> = catalog.quests.iter().map(|row| row.id.as_str()).collect();
    let cost_ids: HashSet<&str> = catalog.costs.iter().map(|row| row.id.as_str()).collect();
    let characteristic_ids: HashSet<&str> = catalog
        .player_characteristics
        .iter()
        .map(|row| row.id.as_str())
        .collect();

    for event in &catalog.events {
        if !event.required_license_id.trim().is_empty()
            && !object_ids.contains(event.required_license_id.trim())
        {
            report.errors.push(format!(
                "Event '{}' references missing licence '{}'",
                event.id, event.required_license_id
            ));
        }
        for object_id in split_ids(&event.required_object_ids) {
            if !object_ids.contains(object_id) {
                report.errors.push(format!(
                    "Event '{}' references missing object '{}'",
                    event.id, object_id
                ));
            }
        }
        if !event.quest_id.trim().is_empty() && !quest_ids.contains(event.quest_id.trim()) {
            report.errors.push(format!(
                "Event '{}' references missing quest '{}'",
                event.id, event.quest_id
            ));
        }
        validate_asset_path(&mut report, base, &event.id, &event.description_html);
    }

    for obligation in &catalog.obligations {
        if !event_ids.contains(obligation.event_id.trim()) {
            report.errors.push(format!(
                "Obligation '{}' references missing event '{}'",
                obligation.id, obligation.event_id
            ));
        }
        if !characteristic_ids.contains(obligation.resource.trim()) {
            report.errors.push(format!(
                "Obligation '{}' references missing player characteristic '{}'",
                obligation.id, obligation.resource
            ));
        }
        if obligation.amount < 0.0 {
            report.errors.push(format!(
                "Obligation '{}' has a negative amount",
                obligation.id
            ));
        }
        if obligation.interval == 0 {
            report.errors.push(format!(
                "Obligation '{}' must have a positive interval",
                obligation.id
            ));
        }
    }

    for quest in &catalog.quests {
        if !quest.required_license_id.trim().is_empty()
            && !object_ids.contains(quest.required_license_id.trim())
        {
            report.errors.push(format!(
                "Quest '{}' references missing licence '{}'",
                quest.id, quest.required_license_id
            ));
        }
        validate_asset_path(&mut report, base, &quest.id, &quest.description_html);
    }

    for object in &catalog.objects {
        if !object.license_previous_id.trim().is_empty()
            && !object_ids.contains(object.license_previous_id.trim())
        {
            report.errors.push(format!(
                "Object '{}' references missing prerequisite licence '{}'",
                object.id, object.license_previous_id
            ));
        }
        for required in split_ids(&object.requires_object_ids) {
            if !object_ids.contains(required) {
                report.errors.push(format!(
                    "Object '{}' references missing prerequisite '{}'",
                    object.id, required
                ));
            }
        }
        validate_asset_path(&mut report, base, &object.id, &object.description_html);
    }

    for rule in &catalog.cost_rules {
        if !cost_ids.contains(rule.cost_id.trim()) {
            report.errors.push(format!(
                "Cost rule '{}' references missing cost '{}'",
                rule.id, rule.cost_id
            ));
        }
        if !(0.0..=1.0).contains(&rule.probability) {
            report.errors.push(format!(
                "Cost rule '{}' has probability outside 0..1",
                rule.id
            ));
        }
    }

    for outcome in &catalog.event_outcomes {
        if !event_ids.contains(outcome.event_id.trim()) {
            report.errors.push(format!(
                "Event outcome references missing event '{}'",
                outcome.event_id
            ));
        }
        if !(0.0..=1.0).contains(&outcome.probability) {
            report.errors.push(format!(
                "Event outcome '{}' has probability outside 0..1",
                outcome.outcome_id
            ));
        }
    }

    report
}

fn split_ids(value: &str) -> impl Iterator<Item = &str> {
    value.split(';').map(str::trim).filter(|id| !id.is_empty())
}

fn validate_unique_ids<'a, I>(report: &mut DatasetValidationReport, kind: &str, ids: I)
where
    I: Iterator<Item = &'a str>,
{
    let mut seen = HashSet::new();
    for id in ids {
        if id.trim().is_empty() {
            report.errors.push(format!("{kind} has an empty id"));
        } else if !seen.insert(id) {
            report.errors.push(format!("Duplicate {kind} id '{id}'"));
        }
    }
}

fn validate_asset_path(
    report: &mut DatasetValidationReport,
    base: &Path,
    id: &str,
    asset_path: &str,
) {
    let path = asset_path.trim();
    if !path.is_empty() && !base.join(path).is_file() {
        report.warnings.push(format!(
            "Entry '{}' references missing asset '{}'",
            id, path
        ));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncounterAttributeData {
    pub attribute_id: String,
    pub display_name: String,
    #[serde(default)]
    pub min_value: f64,
    #[serde(default = "default_encounter_max")]
    pub max_value: f64,
    #[serde(default)]
    pub is_loss_condition: bool,
    #[serde(default = "default_true")]
    pub visible_to_player: bool,
}
fn default_encounter_max() -> f64 {
    f64::INFINITY
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncounterActionData {
    pub action_id: String,
    pub display_name: String,
    #[serde(default)]
    pub usable_by: String,
    #[serde(default)]
    pub requires_attribute_id: String,
    #[serde(default)]
    pub requires_attribute_min: Option<f64>,
    #[serde(default)]
    pub requires_object_id: String,
    #[serde(default)]
    pub consumes_object: bool,
    #[serde(default)]
    pub resource_cost_attribute_id: String,
    #[serde(default)]
    pub resource_cost_amount: f64,
    #[serde(default)]
    pub base_success_rate: f64,
    #[serde(default)]
    pub success_modifier_attribute_id: String,
    #[serde(default)]
    pub success_modifier_scale: f64,
    pub target_attribute_id: String,
    #[serde(default)]
    pub effect_on_success: f64,
    #[serde(default = "default_opponent")]
    pub effect_on_success_target: String,
    #[serde(default)]
    pub effect_on_failure: f64,
    #[serde(default = "default_self")]
    pub effect_on_failure_target: String,
    #[serde(default)]
    pub cooldown_turns: u32,
    #[serde(default)]
    pub flavor_text_success: String,
    #[serde(default)]
    pub flavor_text_failure: String,
    #[serde(default)]
    pub ai_weight: f64,
    #[serde(default = "default_result_max")]
    pub result_max: f64,
    #[serde(default)]
    pub defense_reduction: f64,
}
fn default_opponent() -> String {
    "opponent".into()
}
fn default_self() -> String {
    "self".into()
}
fn default_result_max() -> f64 {
    10.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncounterObjectData {
    pub object_id: String,
    #[serde(default)]
    pub enables_action_id: String,
    #[serde(default)]
    pub success_rate_bonus: f64,
    #[serde(default)]
    pub consumable_in_encounter: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncounterOpponentData {
    pub opponent_id: String,
    pub display_name: String,
    #[serde(default)]
    pub starting_attributes: String,
    #[serde(default)]
    pub available_action_ids: String,
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default)]
    pub action_weights: String,
    #[serde(default)]
    pub scripted_actions: String,
}
fn default_strategy() -> String {
    "random".into()
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncounterOutcomeData {
    pub outcome_id: String,
    #[serde(default)]
    pub applies_to_encounter_id: String,
    pub trigger: String,
    pub consequence_type: String,
    pub consequence_target: String,
    #[serde(default)]
    pub consequence_value: String,
    #[serde(default = "default_probability")]
    pub probability: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncounterConfigData {
    pub encounter_id: String,
    pub display_label: String,
    #[serde(default = "default_turn_order")]
    pub turn_order: String,
    #[serde(default)]
    pub max_turns: u32,
    #[serde(default = "default_tiebreaker")]
    pub tiebreaker: String,
    #[serde(default)]
    pub allow_retreat: bool,
    #[serde(default)]
    pub rng_mode: String,
    #[serde(default)]
    pub opponent_id: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub player_starting_attributes: String,
}
fn default_turn_order() -> String {
    "player_first".into()
}
fn default_tiebreaker() -> String {
    "draw".into()
}

#[derive(Debug, Deserialize)]
struct ConfigRecord {
    variable: String,
    value: String,
}

pub fn parse_csv_file<T, P: AsRef<Path>>(path: P) -> Result<Vec<T>, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    if !path.as_ref().exists() {
        return Ok(Vec::new());
    }

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;
    let mut records = Vec::new();
    for result in reader.deserialize() {
        records.push(result?);
    }
    Ok(records)
}

fn parse_config_file_with_diagnostics<P: AsRef<Path>>(path: P) -> (GameLabels, Vec<String>) {
    let path = path.as_ref();
    if !path.exists() {
        return (GameLabels::default(), Vec::new());
    }
    let mut warnings = Vec::new();
    let mut reader = match csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)
    {
        Ok(reader) => reader,
        Err(error) => {
            return (
                GameLabels::default(),
                vec![format!("{}: {}", path.display(), error)],
            )
        }
    };
    let mut values = HashMap::new();
    for result in reader.deserialize::<ConfigRecord>() {
        match result {
            Ok(record) => {
                values.insert(record.variable, record.value);
            }
            Err(error) => warnings.push(format_csv_warning(path, &error)),
        }
    }
    (GameLabels { values }, warnings)
}

fn parse_csv_file_with_diagnostics<T, P: AsRef<Path>>(path: P) -> (Vec<T>, Vec<String>)
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    if !path.exists() {
        return (Vec::new(), Vec::new());
    }
    let mut warnings = Vec::new();
    let mut reader = match csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)
    {
        Ok(reader) => reader,
        Err(error) => return (Vec::new(), vec![format!("{}: {}", path.display(), error)]),
    };
    let mut records = Vec::new();
    for result in reader.deserialize::<T>() {
        match result {
            Ok(record) => records.push(record),
            Err(error) => warnings.push(format_csv_warning(path, &error)),
        }
    }
    (records, warnings)
}

fn format_csv_warning(path: &Path, error: &csv::Error) -> String {
    let location = error
        .position()
        .map(|position| format!(" line {}", position.line()))
        .unwrap_or_default();
    format!("{}{}: {}", path.display(), location, error)
}

#[cfg(test)]
mod tests {
    use super::{parse_csv_file, validate_dataset_directory, ObjectData};

    #[test]
    fn dataset_objects_are_loadable() {
        let objects: Vec<ObjectData> = parse_csv_file(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dataset/objects.csv"
        ))
        .expect("dataset/objects.csv should match ObjectData");
        assert!(!objects.is_empty());
    }

    #[test]
    fn encounter_schema_is_loadable() {
        let catalog = super::GameCatalog::load_from_directory(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dataset"
        ));
        assert_eq!(catalog.encounter_configs.len(), 1);
        assert!(!catalog.encounter_actions.is_empty());
        assert!(
            catalog
                .events
                .iter()
                .filter(|event| !event.encounter_id.is_empty())
                .count()
                >= 1
        );
    }

    #[test]
    fn activities_unify_scheduled_and_player_started_entries() {
        let catalog = super::GameCatalog::load_from_directory(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dataset"
        ));
        assert_eq!(catalog.activities.len(), catalog.events.len());
        assert!(catalog
            .activities
            .iter()
            .any(|activity| activity.scheduled && activity.resolution_method == "manual"));
        assert!(catalog
            .activities
            .iter()
            .any(|activity| !activity.scheduled && activity.resolution_method == "encounter"));
    }

    #[test]
    fn default_dataset_passes_validation() {
        let report = validate_dataset_directory(concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset"));
        assert!(report.is_valid(), "{:?}", report.errors);
    }
}
