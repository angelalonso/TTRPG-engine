use crate::engine::plugin::{
    PluginCapability, PluginManifest, PLUGIN_PROTOCOL_VERSION, PLUGIN_RESULT_SCHEMA_VERSION,
};
use crate::engine::schema::capability_metadata;
pub use crate::engine::schema::TransferPolicy;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static CATALOG_CACHE: OnceLock<Mutex<HashMap<PathBuf, GameCatalog>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectData {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub name: String,
    pub price: f64,
    #[serde(
        default = "default_object_policy_version",
        alias = "ownership_policy_version"
    )]
    pub policy_version: u32,
    #[serde(default = "default_true")]
    pub buyable: bool,
    #[serde(default = "default_true")]
    pub sellable: bool,
    #[serde(default)]
    pub reward_only: bool,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub max_owned: u32,
    #[serde(default = "default_use_policy")]
    pub use_policy: String,
    #[serde(default = "default_consume_policy")]
    pub consume_policy: String,
    #[serde(default)]
    pub transfer_policy: String,
    #[serde(default)]
    pub rental_duration_days: u32,
    #[serde(default)]
    pub rental_cost: f64,
    #[serde(default)]
    pub return_required: bool,
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
    #[serde(default)]
    pub cosmetic: bool,
    #[serde(default)]
    pub event_tags: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResourceRoles {
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub recovery: Option<String>,
    #[serde(default)]
    pub age: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalConditions {
    #[serde(default)]
    pub currency_below_zero: bool,
    #[serde(default)]
    pub recovery_at_or_below_zero: bool,
}

impl Default for TerminalConditions {
    fn default() -> Self {
        Self {
            currency_below_zero: true,
            recovery_at_or_below_zero: true,
        }
    }
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

fn default_object_policy_version() -> u32 {
    1
}

fn default_use_policy() -> String {
    "unrestricted".to_string()
}

fn default_consume_policy() -> String {
    "never".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectPolicy {
    pub version: u32,
    pub buyable: bool,
    pub sellable: bool,
    pub reward_only: bool,
    pub unique: bool,
    pub max_owned: u32,
    pub use_policy: String,
    pub consume_policy: String,
}

impl ObjectData {
    pub fn transfer_policy(&self) -> TransferPolicy {
        match self.transfer_policy.trim().to_ascii_lowercase().as_str() {
            "loan" => TransferPolicy::Loan,
            "rental" => TransferPolicy::Rental,
            "returnable" => TransferPolicy::Returnable,
            _ => TransferPolicy::None,
        }
    }

    pub fn has_explicit_transfer_policy(&self) -> bool {
        !self.transfer_policy.trim().is_empty()
    }

    pub fn validate_transfer_terms(&self) -> Result<(), String> {
        if !self.rental_cost.is_finite() || self.rental_cost < 0.0 {
            return Err(format!(
                "Object '{}' requires a finite, non-negative rental cost",
                self.id
            ));
        }
        if matches!(
            self.transfer_policy(),
            TransferPolicy::Rental | TransferPolicy::Loan | TransferPolicy::Returnable
        ) && self.rental_duration_days == 0
        {
            return Err(format!(
                "Object '{}' requires a positive rental duration for transfer policy '{}'",
                self.id, self.transfer_policy
            ));
        }
        if self.return_required && matches!(self.transfer_policy(), TransferPolicy::None) {
            return Err(format!(
                "Object '{}' requires a transfer policy when return_required is set",
                self.id
            ));
        }
        Ok(())
    }

    pub fn policy(&self) -> ObjectPolicy {
        let use_legacy_defaults = self.policy_version == 1;
        // These id/type-derived defaults are a version-1 compatibility adapter.
        // Versioned datasets must use the explicit policy fields below instead.
        let legacy_reward_only = use_legacy_defaults
            && (self.id.starts_with("trophy_")
                || matches!(
                    self.id.as_str(),
                    "trophies" | "business_proposal" | "lower_cost"
                ));
        let legacy_non_sellable = use_legacy_defaults
            && self.object_type.eq_ignore_ascii_case("license")
            || legacy_reward_only;
        let legacy_unique = use_legacy_defaults && self.object_type.eq_ignore_ascii_case("license");
        let unique = self.unique
            || legacy_unique
            || self.object_type.eq_ignore_ascii_case("insurance");

        ObjectPolicy {
            version: self.policy_version,
            buyable: self.buyable && !self.reward_only && !legacy_reward_only,
            sellable: self.sellable && !legacy_non_sellable,
            reward_only: self.reward_only || legacy_reward_only,
            unique,
            max_owned: if unique { 1 } else { self.max_owned },
            use_policy: self.use_policy.trim().to_ascii_lowercase(),
            consume_policy: self.consume_policy.trim().to_ascii_lowercase(),
        }
    }
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
    #[serde(default)]
    pub requirement_group: String,
    #[serde(default, alias = "championship_id")]
    pub quest_id: String,
    #[serde(default = "default_quest_event_required", alias = "required_for_quest")]
    pub quest_event_required: bool,
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
    #[serde(default)]
    pub plugin_id: String,
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
pub struct EffectData {
    pub id: String,
    pub operation: String,
    pub target: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub quantity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectBindingData {
    pub id: String,
    pub effect_id: String,
    pub trigger_type: String,
    #[serde(default)]
    pub trigger_ref: String,
    #[serde(default)]
    pub reported_result: String,
    #[serde(default = "default_probability")]
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequirementBindingData {
    pub id: String,
    pub operation: String,
    pub requirement_group: String,
    #[serde(default)]
    pub target_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumericModifierData {
    pub id: String,
    pub target: String,
    pub operation: String,
    pub value: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub condition_group: String,
    #[serde(default)]
    pub minimum: Option<f64>,
    #[serde(default)]
    pub maximum: Option<f64>,
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
    #[serde(default = "default_quest_enrollment_policy")]
    pub enrollment_policy: String,
    #[serde(default = "default_quest_completion_mode")]
    pub completion_mode: String,
    #[serde(default = "default_quest_repeat_policy")]
    pub repeat_policy: String,
    #[serde(default)]
    pub required_event_ids: String,
    #[serde(default)]
    pub optional_event_ids: String,
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
fn default_quest_event_required() -> bool {
    true
}
fn default_quest_enrollment_policy() -> String {
    "manual".into()
}
fn default_quest_completion_mode() -> String {
    "all_required".into()
}
fn default_quest_repeat_policy() -> String {
    "once".into()
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
    #[serde(default)]
    pub active_group: String,
    #[serde(default)]
    pub active_group_limit: u32,
    #[serde(default)]
    pub active_exclusive: bool,
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
    #[serde(default)]
    pub service_slot: Option<usize>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionGroupData {
    pub id: String,
    pub operator: String,
    #[serde(default)]
    pub children: String,
    #[serde(default)]
    pub source_row: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionData {
    pub id: String,
    pub group_id: String,
    pub subject_type: String,
    pub subject_ref: String,
    pub operator: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub source_row: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameLabels {
    #[serde(default)]
    pub values: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextVariants {
    pub variable: String,
    pub values: Vec<String>,
}

impl GameLabels {
    pub fn get(&self, key: &str, fallback: &str) -> String {
        self.values
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifestData {
    pub id: String,
    #[serde(default)]
    pub entrypoint: String,
    #[serde(default)]
    pub protocol_version: u32,
    #[serde(default, alias = "capabilities")]
    pub capability: String,
    #[serde(default, alias = "result_schema_id")]
    pub result_schema: String,
    #[serde(default, alias = "schema_version")]
    pub result_schema_version: u32,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub dispatch: String,
}

impl PluginManifestData {
    pub fn typed(&self) -> Result<PluginManifest, String> {
        let capabilities = self
            .capability
            .split(';')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| match value {
                "normalize_result" => Ok(PluginCapability::NormalizeResult),
                "evaluate_custom_fact" => Ok(PluginCapability::EvaluateCustomFact),
                "provide_result" => Ok(PluginCapability::ProvideResult),
                _ => Err(format!("unknown capability '{value}'")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if capabilities.is_empty() {
            return Err("at least one capability is required".into());
        }
        if capabilities
            .iter()
            .enumerate()
            .any(|(index, capability)| capabilities[..index].contains(capability))
        {
            return Err("duplicate capability".into());
        }
        Ok(PluginManifest {
            id: self.id.trim().to_string(),
            entrypoint: self.entrypoint.trim().to_string(),
            protocol_version: self.protocol_version,
            capabilities,
            result_schema: self.result_schema.clone(),
            result_schema_version: self.result_schema_version,
            required: self.required,
            dispatch: self.dispatch.trim().to_string(),
        })
    }
}

fn configured_role(labels: &GameLabels, role: &str) -> Option<String> {
    [
        format!("resource_role_{role}"),
        format!("{role}_role"),
        format!("{role}_characteristic"),
    ]
    .iter()
    .find_map(|key| labels.values.get(key))
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
}

fn configured_bool(labels: &GameLabels, key: &str) -> Option<bool> {
    labels.values.get(key).and_then(|value| match value.trim() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameCatalog {
    pub player_characteristics: Vec<PlayerCharacteristicData>,
    #[serde(default)]
    pub resource_roles: ResourceRoles,
    #[serde(default)]
    pub terminal_conditions: TerminalConditions,
    pub objects: Vec<ObjectData>,
    pub costs: Vec<CostData>,
    pub cost_rules: Vec<CostRule>,
    pub cost_conditions: Vec<CostCondition>,
    #[serde(default)]
    pub condition_groups: Vec<ConditionGroupData>,
    #[serde(default)]
    pub conditions: Vec<ConditionData>,
    pub events: Vec<EventData>,
    #[serde(default)]
    pub activities: Vec<ActivityData>,
    #[serde(default)]
    pub obligations: Vec<ObligationData>,
    #[serde(default)]
    pub event_outcomes: Vec<EventOutcomeData>,
    #[serde(default)]
    pub event_results: Vec<EventResultData>,
    pub effects: Vec<EffectData>,
    pub effect_bindings: Vec<EffectBindingData>,
    #[serde(default)]
    pub requirement_bindings: Vec<RequirementBindingData>,
    #[serde(default)]
    pub numeric_modifiers: Vec<NumericModifierData>,
    #[serde(default, alias = "championships")]
    pub quests: Vec<QuestData>,
    pub labels: GameLabels,
    #[serde(default)]
    pub texts: Vec<TextVariants>,
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
    pub plugins: Vec<PluginManifestData>,
    #[serde(default)]
    pub dataset_warnings: Vec<String>,
}

impl GameCatalog {
    pub fn plugin_manifest(&self, plugin_id: &str) -> Result<Option<PluginManifest>, String> {
        let plugin_id = plugin_id.trim();
        if plugin_id.is_empty() {
            return Ok(None);
        }
        let matches = self
            .plugins
            .iter()
            .filter(|manifest| manifest.id.trim() == plugin_id)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Ok(None),
            [manifest] => manifest.typed().map(Some),
            _ => Err(format!("plugin '{plugin_id}' is declared more than once")),
        }
    }

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
        macro_rules! load_if_present {
            ($name:literal, $type:ty) => {{
                if base.join($name).is_file() {
                    load!($name, $type)
                } else {
                    Vec::<$type>::new()
                }
            }};
        }

        let player_characteristics = load!("player.csv", PlayerCharacteristicData);
        let configured_resource_roles = ResourceRoles {
            currency: configured_role(&labels, "currency"),
            recovery: configured_role(&labels, "recovery"),
            age: configured_role(&labels, "age"),
        };
        // These are compatibility mappings for the version-1 racing dataset only.
        let resource_roles = ResourceRoles {
            currency: configured_resource_roles.currency.or_else(|| {
                player_characteristics
                    .iter()
                    .any(|entry| entry.id == "budget")
                    .then(|| "budget".to_string())
            }),
            recovery: configured_resource_roles.recovery.or_else(|| {
                player_characteristics
                    .iter()
                    .any(|entry| entry.id == "stamina")
                    .then(|| "stamina".to_string())
            }),
            age: configured_resource_roles.age.or_else(|| {
                player_characteristics
                    .iter()
                    .any(|entry| entry.id == "age")
                    .then(|| "age".to_string())
            }),
        };
        let terminal_conditions = TerminalConditions {
            currency_below_zero: configured_bool(&labels, "terminal_currency_below_zero")
                .unwrap_or(resource_roles.currency.is_some()),
            recovery_at_or_below_zero: configured_bool(
                &labels,
                "terminal_recovery_at_or_below_zero",
            )
            .unwrap_or(resource_roles.recovery.is_some()),
        };
        let objects = load!("objects.csv", ObjectData);
        let costs = load!("costs.csv", CostData);
        let cost_rules = load!("cost_rules.csv", CostRule);
        let cost_conditions = load!("cost_rule_conditions.csv", CostCondition);
        let condition_groups = load_if_present!("condition_groups.csv", ConditionGroupData);
        let conditions = load_if_present!("conditions.csv", ConditionData);
        let events = load!("events.csv", EventData);
        let event_outcomes = load!("event_outcomes.csv", EventOutcomeData);
        let event_results = load!("event_results.csv", EventResultData);
        let effects = load_if_present!("effects.csv", EffectData);
        let effect_bindings = load_if_present!("effect_bindings.csv", EffectBindingData);
        let requirement_bindings =
            load_if_present!("requirement_bindings.csv", RequirementBindingData);
        let numeric_modifiers = load_if_present!("numeric_modifiers.csv", NumericModifierData);
        let quests = load!("quests.csv", QuestData);
        let obligations = load!("obligations.csv", ObligationData);
        let encounter_attributes = load!("encounter_attributes.csv", EncounterAttributeData);
        let encounter_actions = load!("encounter_actions.csv", EncounterActionData);
        let encounter_objects = load!("encounter_objects.csv", EncounterObjectData);
        let encounter_opponents = load!("encounter_opponents.csv", EncounterOpponentData);
        let encounter_outcomes = load!("encounter_outcomes.csv", EncounterOutcomeData);
        let encounter_configs = load!("encounter_config.csv", EncounterConfigData);
        let plugins = load!("plugins.csv", PluginManifestData);
        let (texts, text_warnings) = parse_texts_file_with_diagnostics(base.join("texts.csv"));
        dataset_warnings.extend(text_warnings);
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
            resource_roles,
            terminal_conditions,
            objects,
            costs,
            cost_rules,
            cost_conditions,
            condition_groups,
            conditions,
            events,
            activities,
            obligations,
            event_outcomes,
            event_results,
            effects,
            effect_bindings,
            requirement_bindings,
            numeric_modifiers,
            quests,
            labels,
            texts,
            encounter_attributes,
            encounter_actions,
            encounter_objects,
            encounter_opponents,
            encounter_outcomes,
            encounter_configs,
            plugins,
            dataset_warnings,
        }
    }

    pub fn load_from_directory_cached<P: AsRef<Path>>(dir: P) -> Self {
        let path = dir.as_ref();
        let key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let cache = CATALOG_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        if let Ok(catalogs) = cache.lock() {
            if let Some(catalog) = catalogs.get(&key) {
                return catalog.clone();
            }
        }
        let catalog = Self::load_from_directory(path);
        if let Ok(mut catalogs) = cache.lock() {
            catalogs.insert(key, catalog.clone());
        }
        catalog
    }

    pub fn clear_cached_directory<P: AsRef<Path>>(dir: P) {
        let path = dir.as_ref();
        let key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if let Some(cache) = CATALOG_CACHE.get() {
            if let Ok(mut catalogs) = cache.lock() {
                catalogs.remove(&key);
            }
        }
    }
}

fn parse_texts_file_with_diagnostics<P: AsRef<Path>>(path: P) -> (Vec<TextVariants>, Vec<String>) {
    let path = path.as_ref();
    if !path.exists() {
        return (Vec::new(), Vec::new());
    }
    let mut warnings = Vec::new();
    let mut reader = match csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)
    {
        Ok(reader) => reader,
        Err(error) => return (Vec::new(), vec![format!("{}: {}", path.display(), error)]),
    };
    let mut values = HashMap::<String, Vec<String>>::new();
    for (index, result) in reader.records().enumerate() {
        match result {
            Ok(record) => {
                let variable = record.get(0).unwrap_or_default().trim();
                if index == 0 && variable.eq_ignore_ascii_case("variable") {
                    continue;
                }
                if variable.is_empty() {
                    warnings.push(format!(
                        "{} line {}: missing text variable",
                        path.display(),
                        index + 1
                    ));
                    continue;
                }
                let entries = values.entry(variable.to_string()).or_default();
                entries.extend(
                    record
                        .iter()
                        .skip(1)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string),
                );
            }
            Err(error) => warnings.push(format_csv_warning(path, &error)),
        }
    }
    (
        values
            .into_iter()
            .map(|(variable, values)| TextVariants { variable, values })
            .collect(),
        warnings,
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnsupportedField {
    pub file: String,
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DatasetValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub unsupported_fields: Vec<UnsupportedField>,
}

impl DatasetValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

fn supported_numeric_modifier_target(target: &str) -> bool {
    const STATIC_TARGETS: &[&str] = &[
        "action_success_probability",
        "sponsor_success_probability",
        "event_success_probability",
        "action_payout",
        "event_reward",
        "event_charisma_reward",
        "object_acquisition_cost",
        "service_cost",
    ];
    STATIC_TARGETS.contains(&target)
        || target
            .strip_prefix("effect_quantity:")
            .is_some_and(|value| !value.trim().is_empty())
        || target
            .strip_prefix("encounter_action_success_probability:")
            .is_some_and(|value| !value.trim().is_empty())
        || target
            .strip_prefix("encounter_action_effect_success:")
            .is_some_and(|value| !value.trim().is_empty())
        || target
            .strip_prefix("encounter_action_effect_failure:")
            .is_some_and(|value| !value.trim().is_empty())
        || target
            .strip_prefix("encounter_action_resource_cost:")
            .is_some_and(|value| !value.trim().is_empty())
}

pub fn validate_dataset_directory<P: AsRef<Path>>(dir: P) -> DatasetValidationReport {
    let base = dir.as_ref();
    let catalog = GameCatalog::load_from_directory(base);
    let mut report = DatasetValidationReport {
        unsupported_fields: find_unsupported_fields(base),
        ..DatasetValidationReport::default()
    };
    report
        .warnings
        .extend(report.unsupported_fields.iter().map(|field| {
            format!(
                "{} column '{}' is loaded but unsupported; it will be ignored",
                field.file, field.field
            )
        }));
    report.errors.extend(
        catalog
            .dataset_warnings
            .iter()
            .map(|warning| format!("Dataset parse error: {warning}")),
    );
    if !catalog.condition_groups.is_empty() || !catalog.conditions.is_empty() {
        match crate::engine::conditions::ConditionSet::from_rows(
            &catalog.condition_groups,
            &catalog.conditions,
        ) {
            Ok(_) => {}
            Err(errors) => report.errors.extend(
                errors
                    .into_iter()
                    .map(|error| format!("Condition contract error: {error:?}")),
            ),
        }
    }

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
    validate_unique_ids(
        &mut report,
        "plugin",
        catalog.plugins.iter().map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "effect",
        catalog.effects.iter().map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "effect binding",
        catalog.effect_bindings.iter().map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "requirement binding",
        catalog
            .requirement_bindings
            .iter()
            .map(|row| row.id.as_str()),
    );
    validate_unique_ids(
        &mut report,
        "numeric modifier",
        catalog.numeric_modifiers.iter().map(|row| row.id.as_str()),
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
    let plugin_ids: HashSet<&str> = catalog.plugins.iter().map(|row| row.id.as_str()).collect();
    let effect_ids: HashSet<&str> = catalog.effects.iter().map(|row| row.id.as_str()).collect();
    let encounter_action_ids: HashSet<&str> = catalog
        .encounter_actions
        .iter()
        .map(|row| row.action_id.as_str())
        .collect();
    let condition_group_ids: HashSet<&str> = catalog
        .condition_groups
        .iter()
        .map(|row| row.id.as_str())
        .collect();

    for effect in &catalog.effects {
        let operation = effect.operation.trim().to_ascii_lowercase();
        if !matches!(
            operation.as_str(),
            "add_characteristic"
                | "set_characteristic"
                | "multiply_characteristic"
                | "grant_object"
                | "remove_object"
                | "consume_object"
                | "set_object_availability"
                | "set_object_service"
        ) {
            report.errors.push(format!(
                "Effect '{}' uses unsupported operation '{}'",
                effect.id, effect.operation
            ));
            continue;
        }
        let target = effect.target.trim();
        if target.is_empty() {
            report
                .errors
                .push(format!("Effect '{}' has an empty target", effect.id));
        } else if matches!(
            operation.as_str(),
            "add_characteristic" | "set_characteristic" | "multiply_characteristic"
        ) {
            if !characteristic_ids.contains(target) {
                report.errors.push(format!(
                    "Effect '{}' references missing characteristic '{}'",
                    effect.id, target
                ));
            }
            if let Err(error) = crate::engine::expressions::Expression::parse(effect.value.trim()) {
                report.errors.push(format!(
                    "Effect '{}' has an invalid value expression '{}': {}",
                    effect.id, effect.value, error
                ));
            }
        } else {
            if !object_ids.contains(target) {
                report.errors.push(format!(
                    "Effect '{}' references missing object '{}'",
                    effect.id, target
                ));
            }
            if operation == "set_object_service" {
                let slot = effect.value.trim();
                let valid_slot = slot
                    .strip_prefix("service_")
                    .and_then(|value| value.strip_suffix("_needed").or(Some(value)))
                    .and_then(|value| value.parse::<usize>().ok())
                    .is_some_and(|value| (1..=15).contains(&value));
                if !valid_slot {
                    report.errors.push(format!(
                        "Effect '{}' has an invalid service slot '{}'",
                        effect.id, effect.value
                    ));
                }
            } else {
                let quantity = if effect.quantity.trim().is_empty() {
                    Ok(1.0)
                } else {
                    effect.quantity.trim().parse::<f64>()
                };
                match quantity {
                    Ok(value) if value.is_finite() && value >= 0.0 && value.fract() == 0.0 => {}
                    _ => report.errors.push(format!(
                        "Effect '{}' has an invalid non-negative integral quantity '{}'",
                        effect.id, effect.quantity
                    )),
                }
            }
        }
    }

    for binding in &catalog.effect_bindings {
        if !effect_ids.contains(binding.effect_id.trim()) {
            report.errors.push(format!(
                "Effect binding '{}' references missing effect '{}'",
                binding.id, binding.effect_id
            ));
        }

        for binding in &catalog.requirement_bindings {
            if binding.operation.trim().is_empty() {
                report.errors.push(format!(
                    "Requirement binding '{}' has an empty operation",
                    binding.id
                ));
            } else if !matches!(
                binding.operation.trim().to_ascii_lowercase().as_str(),
                "acquire"
                    | "sell"
                    | "use"
                    | "consume"
                    | "rent"
                    | "loan"
                    | "return"
                    | "event_start"
                    | "event_entry"
                    | "encounter_action"
                    | "quest_join"
            ) {
                report.errors.push(format!(
                    "Requirement binding '{}' uses unsupported operation '{}'",
                    binding.id, binding.operation
                ));
            }
            if binding.requirement_group.trim().is_empty() {
                report.errors.push(format!(
                    "Requirement binding '{}' has an empty requirement group",
                    binding.id
                ));
            } else if !condition_group_ids.contains(binding.requirement_group.trim()) {
                report.errors.push(format!(
                    "Requirement binding '{}' references missing condition group '{}'",
                    binding.id, binding.requirement_group
                ));
            }
            if binding.target_ref.trim().is_empty() {
                report.errors.push(format!(
                    "Requirement binding '{}' has an empty target reference",
                    binding.id
                ));
            } else {
                let target = binding.target_ref.trim();
                let operation = binding.operation.trim().to_ascii_lowercase();
                let known = match operation.as_str() {
                    "acquire" | "sell" | "use" | "consume" | "rent" | "loan" | "return" => {
                        object_ids.contains(target)
                    }
                    "event_start" | "event_entry" => event_ids.contains(target),
                    "encounter_action" => encounter_action_ids.contains(target),
                    "quest_join" => quest_ids.contains(target),
                    _ => true,
                };
                if !known {
                    report.errors.push(format!(
                        "Requirement binding '{}' references unknown {} target '{}'",
                        binding.id, operation, target
                    ));
                }
            }
        }

        for modifier in &catalog.numeric_modifiers {
            if modifier.target.trim().is_empty() {
                report.errors.push(format!(
                    "Numeric modifier '{}' has an empty target",
                    modifier.id
                ));
            } else if !supported_numeric_modifier_target(modifier.target.trim()) {
                report.errors.push(format!(
                    "Numeric modifier '{}' uses unsupported target '{}'",
                    modifier.id, modifier.target
                ));
            }
            if !matches!(
                modifier.operation.trim().to_ascii_lowercase().as_str(),
                "set" | "add" | "multiply"
            ) {
                report.errors.push(format!(
                    "Numeric modifier '{}' uses unsupported operation '{}'",
                    modifier.id, modifier.operation
                ));
            }

            if modifier.value.trim().is_empty() {
                report.errors.push(format!(
                    "Numeric modifier '{}' has an empty value expression",
                    modifier.id
                ));
            } else if let Err(error) =
                crate::engine::expressions::Expression::parse(modifier.value.trim())
            {
                report.errors.push(format!(
                    "Numeric modifier '{}' has an invalid value expression '{}': {}",
                    modifier.id, modifier.value, error
                ));
            }
            for (name, value) in [("minimum", modifier.minimum), ("maximum", modifier.maximum)] {
                if value.is_some_and(|number| !number.is_finite()) {
                    report.errors.push(format!(
                        "Numeric modifier '{}' has a non-finite {} bound",
                        modifier.id, name
                    ));
                }
            }
            if let (Some(minimum), Some(maximum)) = (modifier.minimum, modifier.maximum) {
                if minimum > maximum {
                    report.errors.push(format!(
                        "Numeric modifier '{}' has minimum greater than maximum",
                        modifier.id
                    ));
                }
            }
            if !modifier.condition_group.trim().is_empty()
                && !condition_group_ids.contains(modifier.condition_group.trim())
            {
                report.errors.push(format!(
                    "Numeric modifier '{}' references missing condition group '{}'",
                    modifier.id, modifier.condition_group
                ));
            }
        }
        if binding.trigger_type.trim().is_empty() {
            report.errors.push(format!(
                "Effect binding '{}' has an empty trigger type",
                binding.id
            ));
        } else if !matches!(
            binding.trigger_type.trim().to_ascii_lowercase().as_str(),
            "event_started"
                | "event_entered"
                | "event_completed"
                | "object_acquired"
                | "object_sold"
                | "encounter_completed"
                | "quest_joined"
                | "quest_completed"
                | "day_elapsed"
        ) {
            report.errors.push(format!(
                "Effect binding '{}' uses unsupported trigger type '{}'",
                binding.id, binding.trigger_type
            ));
        }
        if !(0.0..=1.0).contains(&binding.probability) {
            report.errors.push(format!(
                "Effect binding '{}' has probability outside 0..1",
                binding.id
            ));
        }
        if !binding.reported_result.trim().is_empty()
            && !matches!(
                binding.reported_result.trim().to_ascii_lowercase().as_str(),
                "success" | "failure"
            )
        {
            report.errors.push(format!(
                "Effect binding '{}' uses unsupported reported result '{}'",
                binding.id, binding.reported_result
            ));
        }
    }

    for plugin in &catalog.plugins {
        let typed = match plugin.typed() {
            Ok(value) => value,
            Err(error) => {
                report
                    .errors
                    .push(format!("Plugin '{}' is malformed: {error}", plugin.id));
                continue;
            }
        };
        if typed.protocol_version != PLUGIN_PROTOCOL_VERSION {
            report.errors.push(format!(
                "Plugin '{}' uses unsupported protocol version {}; supported version is {}",
                plugin.id, typed.protocol_version, PLUGIN_PROTOCOL_VERSION
            ));
        }
        if typed.result_schema.trim().is_empty() {
            report
                .errors
                .push(format!("Plugin '{}' has an empty result schema", plugin.id));
        }
        if typed.result_schema_version != PLUGIN_RESULT_SCHEMA_VERSION {
            report.errors.push(format!(
                "Plugin '{}' uses unsupported result schema version {}; supported version is {}",
                plugin.id, typed.result_schema_version, PLUGIN_RESULT_SCHEMA_VERSION
            ));
        }
        if let Err(error) = typed.normalized_dispatch() {
            report.errors.push(error);
        }
        if typed.entrypoint.trim().is_empty() {
            let message = format!("Plugin '{}' has an empty entrypoint", plugin.id);
            if typed.required {
                report.errors.push(message);
            } else {
                report.warnings.push(message);
            }
        } else {
            let entrypoint = Path::new(typed.entrypoint.trim());
            if entrypoint.is_absolute()
                || entrypoint
                    .components()
                    .any(|component| component == std::path::Component::ParentDir)
            {
                report.errors.push(format!(
                    "Plugin '{}' has an invalid dataset-relative entrypoint '{}'",
                    plugin.id, typed.entrypoint
                ));
            } else if !base.join(entrypoint).is_file() {
                let message = format!(
                    "Plugin '{}' references missing entrypoint '{}'",
                    plugin.id, typed.entrypoint
                );
                if typed.required {
                    report.errors.push(message);
                } else {
                    report.warnings.push(message);
                }
            }
        }
    }

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
        if !event.plugin_id.trim().is_empty() && !plugin_ids.contains(event.plugin_id.trim()) {
            report.errors.push(format!(
                "Event '{}' references missing plugin '{}'",
                event.id, event.plugin_id
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
        validate_object_policy(&mut report, object);
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
        if rule
            .service_slot
            .is_some_and(|slot| !(1..=15).contains(&slot))
        {
            report.errors.push(format!(
                "Cost rule '{}' has service_slot outside 1..15",
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

fn find_unsupported_fields(base: &Path) -> Vec<UnsupportedField> {
    capability_metadata()
        .tables
        .iter()
        .filter_map(|table| {
            let path = base.join(&table.file);
            if !path.is_file() {
                return None;
            }
            let mut reader = csv::ReaderBuilder::new()
                .trim(csv::Trim::All)
                .flexible(true)
                .from_path(path)
                .ok()?;
            let headers = reader.headers().ok()?.clone();
            Some(unsupported_fields_for_headers(table, headers.iter()))
        })
        .flatten()
        .collect()
}

fn unsupported_fields_for_headers<'a, I>(
    table: &crate::engine::schema::TableMetadata,
    headers: I,
) -> Vec<UnsupportedField>
where
    I: IntoIterator<Item = &'a str>,
{
    let supported = table
        .fields
        .iter()
        .flat_map(|field| {
            std::iter::once(field.name.as_str()).chain(field.aliases.iter().map(String::as_str))
        })
        .collect::<HashSet<_>>();
    headers
        .into_iter()
        .filter_map(|header| {
            let field = header.trim();
            if !supported.contains(field) {
                Some(UnsupportedField {
                    file: table.file.clone(),
                    field: field.to_string(),
                    reason: "no engine capability is declared for this column".to_string(),
                })
            } else if table
                .fields
                .iter()
                .any(|metadata| metadata.name == field && metadata.unsupported)
            {
                Some(UnsupportedField {
                    file: table.file.clone(),
                    field: field.to_string(),
                    reason: "loaded for compatibility but not interpreted by the engine"
                        .to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn validate_object_policy(report: &mut DatasetValidationReport, object: &ObjectData) {
    if !matches!(object.policy_version, 1 | 2) {
        report.errors.push(format!(
            "Object '{}' uses unsupported policy version {}; supported versions are 1 and 2",
            object.id, object.policy_version
        ));
    }
    if object.reward_only && object.buyable {
        report.errors.push(format!(
            "Object '{}' cannot be both reward_only and buyable",
            object.id
        ));
    }
    if object.unique && object.max_owned > 1 {
        report.errors.push(format!(
            "Object '{}' cannot be unique with max_owned greater than 1",
            object.id
        ));
    }
    if !matches!(
        object.use_policy.trim().to_ascii_lowercase().as_str(),
        "unrestricted" | "usable" | "not_usable"
    ) {
        report.errors.push(format!(
            "Object '{}' has unsupported use_policy '{}'",
            object.id, object.use_policy
        ));
    }
    if !matches!(
        object.consume_policy.trim().to_ascii_lowercase().as_str(),
        "never" | "on_use" | "on_acquire"
    ) {
        report.errors.push(format!(
            "Object '{}' has unsupported consume_policy '{}'",
            object.id, object.consume_policy
        ));
    }
    if let Err(error) = object.validate_transfer_terms() {
        report.errors.push(error);
    }
    for (name, value) in [
        ("resale_initial_percent", object.resale_initial_percent),
        ("resale_annual_percent", object.resale_annual_percent),
        ("resale_min_percent", object.resale_min_percent),
    ] {
        if !(0.0..=1.0).contains(&value) {
            report
                .errors
                .push(format!("Object '{}' has {} outside 0..1", object.id, name));
        }
    }
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
    use super::{
        parse_csv_file, unsupported_fields_for_headers, validate_dataset_directory, ObjectData,
        PluginManifestData, TransferPolicy,
    };
    use crate::engine::plugin::{PLUGIN_PROTOCOL_VERSION, PLUGIN_RESULT_SCHEMA_VERSION};
    use crate::engine::schema::capability_metadata;

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
    fn legacy_object_policy_defaults_preserve_racing_behavior() {
        let catalog = super::GameCatalog::load_from_directory(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dataset"
        ));
        let license = catalog
            .objects
            .iter()
            .find(|object| object.id == "license_basic")
            .expect("sample licence should exist")
            .policy();
        assert!(license.buyable);
        assert!(license.unique);
        assert!(!license.sellable);

        let trophy = catalog
            .objects
            .iter()
            .find(|object| object.id == "trophy_formula_ford")
            .expect("sample trophy should exist")
            .policy();
        assert!(trophy.reward_only);
        assert!(!trophy.buyable);
        assert!(!trophy.sellable);
    }

    #[test]
    fn explicit_object_policy_is_loaded_and_validated() {
        let catalog = super::GameCatalog::load_from_directory(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dataset"
        ));
        let object = catalog
            .objects
            .iter()
            .find(|object| object.id == "gloves")
            .expect("sample object should exist");
        assert_eq!(object.policy_version, 1);
        assert!(object.buyable);
        assert!(object.sellable);
        assert_eq!(object.policy().max_owned, 0);
        assert_eq!(object.policy().consume_policy, "never");
        assert!(
            validate_dataset_directory(concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset"))
                .is_valid()
        );
    }

    #[test]
    fn name_derived_object_policy_isolated_to_legacy_adapter() {
        let legacy_trophy: ObjectData = serde_json::from_value(serde_json::json!({
            "id": "trophy_custom",
            "type": "achievement",
            "name": "Custom award",
            "price": 0.0,
            "policy_version": 1
        }))
        .expect("legacy object should deserialize");
        let legacy_license: ObjectData = serde_json::from_value(serde_json::json!({
            "id": "permit_custom",
            "type": "license",
            "name": "Custom permit",
            "price": 10.0,
            "policy_version": 1
        }))
        .expect("legacy object should deserialize");
        assert!(legacy_trophy.policy().reward_only);
        assert!(legacy_license.policy().unique);
        assert!(!legacy_license.policy().sellable);

        let insurance: ObjectData = serde_json::from_value(serde_json::json!({
            "id": "insurance_custom",
            "type": "insurance",
            "name": "Custom insurance",
            "price": 10.0,
            "policy_version": 2
        }))
        .expect("insurance object should deserialize");
        assert!(insurance.policy().unique);
        assert_eq!(insurance.policy().max_owned, 1);

        let explicit_trophy: ObjectData = serde_json::from_value(serde_json::json!({
            "id": "trophy_custom",
            "type": "achievement",
            "name": "Custom award",
            "price": 0.0,
            "policy_version": 2
        }))
        .expect("version-2 object should deserialize");
        let explicit_license: ObjectData = serde_json::from_value(serde_json::json!({
            "id": "permit_custom",
            "type": "license",
            "name": "Custom permit",
            "price": 10.0,
            "policy_version": 2
        }))
        .expect("version-2 object should deserialize");
        assert!(!explicit_trophy.policy().reward_only);
        assert!(explicit_trophy.policy().buyable);
        assert!(!explicit_license.policy().unique);
        assert!(explicit_license.policy().sellable);
    }

    #[test]
    fn transfer_terms_are_shared_and_validated() {
        let rental: ObjectData = serde_json::from_value(serde_json::json!({
            "id": "oven",
            "type": "tool",
            "name": "Oven",
            "price": 100.0,
            "transfer_policy": "rental",
            "rental_duration_days": 3,
            "rental_cost": 12.5
        }))
        .expect("minimal object should deserialize");
        assert_eq!(rental.transfer_policy(), TransferPolicy::Rental);
        assert_eq!(rental.rental_cost, 12.5);
        assert!(rental.has_explicit_transfer_policy());
        assert!(rental.validate_transfer_terms().is_ok());

        let invalid = ObjectData {
            transfer_policy: "loan".into(),
            rental_duration_days: 0,
            ..rental
        };
        assert!(invalid.validate_transfer_terms().is_err());
        let invalid_return = ObjectData {
            transfer_policy: "returnable".into(),
            rental_duration_days: 0,
            ..serde_json::from_value(serde_json::json!({
                "id": "loaned-oven",
                "type": "tool",
                "name": "Loaned Oven",
                "price": 100.0,
                "rental_cost": 0.0
            }))
            .expect("minimal returnable object should deserialize")
        };
        assert!(invalid_return.validate_transfer_terms().is_err());

        let invalid_cost = ObjectData {
            rental_cost: -1.0,
            ..invalid
        };
        assert!(invalid_cost.validate_transfer_terms().is_err());
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

    #[test]
    fn unsupported_csv_columns_are_reported_explicitly() {
        let table = capability_metadata()
            .tables
            .into_iter()
            .find(|table| table.file == "objects.csv")
            .expect("object metadata should exist");
        let fields = unsupported_fields_for_headers(
            &table,
            ["id", "name", "future_engine_field", "description"],
        );
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].file, "objects.csv");
        assert_eq!(fields[0].field, "future_engine_field");
        assert!(fields[0].reason.contains("no engine capability"));
    }

    #[test]
    fn loaded_but_unsupported_encounter_fields_are_reported_explicitly() {
        let metadata = capability_metadata();
        let unsupported = [
            ("encounter_attributes.csv", "visible_to_player"),
            ("encounter_actions.csv", "ai_weight"),
            ("encounter_objects.csv", "consumable_in_encounter"),
            ("encounter_opponents.csv", "action_weights"),
            ("encounter_opponents.csv", "scripted_actions"),
            ("encounter_config.csv", "rng_mode"),
        ];

        for (file, field_name) in unsupported {
            let table = metadata
                .tables
                .iter()
                .find(|table| table.file == file)
                .unwrap_or_else(|| panic!("{file} metadata should exist"));
            let field = table
                .fields
                .iter()
                .find(|field| field.name == field_name)
                .unwrap_or_else(|| panic!("{file}.{field_name} metadata should exist"));
            assert!(
                field.unsupported,
                "{file}.{field_name} must be marked unsupported"
            );
            assert!(
                field.description.contains("not interpreted"),
                "{file}.{field_name} must explain its unsupported status"
            );

            let diagnostics = unsupported_fields_for_headers(table, [field_name]);
            assert_eq!(diagnostics.len(), 1, "{file}.{field_name} should diagnose");
            assert_eq!(diagnostics[0].field, field_name);
            assert!(diagnostics[0].reason.contains("not interpreted"));
        }
    }

    #[test]
    fn plugin_manifest_is_typed_and_versioned() {
        let manifest = PluginManifestData {
            id: "example".into(),
            entrypoint: "plugins/example.py".into(),
            protocol_version: PLUGIN_PROTOCOL_VERSION,
            capability: "provide_result;evaluate_custom_fact".into(),
            result_schema: "generic_result".into(),
            result_schema_version: PLUGIN_RESULT_SCHEMA_VERSION,
            required: false,
            dispatch: "example".into(),
        }
        .typed()
        .expect("manifest should be valid");
        assert_eq!(manifest.id, "example");
        assert_eq!(manifest.capabilities.len(), 2);
    }

    #[test]
    fn catalog_lookup_returns_the_dataset_selected_manifest() {
        let mut catalog = super::GameCatalog::default();
        catalog.plugins.push(PluginManifestData {
            id: " selected ".into(),
            entrypoint: "plugin_echo.py".into(),
            protocol_version: PLUGIN_PROTOCOL_VERSION,
            capability: "provide_result".into(),
            result_schema: "generic_result".into(),
            result_schema_version: PLUGIN_RESULT_SCHEMA_VERSION,
            required: false,
            dispatch: "Cooking-Results".into(),
        });

        let manifest = catalog
            .plugin_manifest("selected")
            .expect("lookup should succeed")
            .expect("manifest should exist");
        assert_eq!(manifest.id, "selected");
        assert_eq!(manifest.normalized_dispatch().unwrap(), "cooking_results");
        assert!(catalog.plugin_manifest("missing").unwrap().is_none());
    }

    #[test]
    fn optional_authoring_tables_deserialize_with_defaults() {
        let binding: super::RequirementBindingData = serde_json::from_value(serde_json::json!({
            "id": "rent_requires_license",
            "operation": "rent",
            "requirement_group": "licensed",
            "target_ref": "race_event"
        }))
        .expect("requirement binding should deserialize");
        assert_eq!(binding.operation, "rent");
        assert_eq!(binding.target_ref, "race_event");

        let modifier: super::NumericModifierData = serde_json::from_value(serde_json::json!({
            "id": "ingredient_bonus",
            "target": "success_probability",
            "operation": "add",
            "value": "0.05",
            "priority": 10
        }))
        .expect("numeric modifier should deserialize");
        assert_eq!(modifier.priority, 10);
        assert!(modifier.condition_group.is_empty());
        assert!(modifier.minimum.is_none());
    }

    #[test]
    fn numeric_modifier_targets_are_explicitly_supported() {
        assert!(super::supported_numeric_modifier_target("event_reward"));
        assert!(super::supported_numeric_modifier_target(
            "effect_quantity:reward"
        ));
        assert!(super::supported_numeric_modifier_target(
            "encounter_action_effect_success:strike"
        ));
        assert!(!super::supported_numeric_modifier_target("unknown_value"));
    }

    #[test]
    fn malformed_plugin_capability_is_rejected() {
        let manifest = PluginManifestData {
            id: "broken".into(),
            entrypoint: "plugins/broken.py".into(),
            protocol_version: PLUGIN_PROTOCOL_VERSION,
            capability: "not_a_capability".into(),
            result_schema: "generic_result".into(),
            result_schema_version: PLUGIN_RESULT_SCHEMA_VERSION,
            required: true,
            dispatch: String::new(),
        };
        assert!(manifest.typed().is_err());
    }
}
