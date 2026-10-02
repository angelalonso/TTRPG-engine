use serde::{Deserialize, Serialize};

pub const AUTHORING_CONTRACT_VERSION: u32 = 1;
pub const CAPABILITY_METADATA_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleContractVersion {
    V1,
    V2,
}

impl Default for RuleContractVersion {
    fn default() -> Self {
        Self::V1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanGroup {
    All,
    Any,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
    Contains,
    StartsWith,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementSubject {
    PlayerCharacteristic,
    ObjectDefinition,
    ObjectType,
    ObjectCount,
    EventHistory,
    QuestStatus,
    ActiveEvent,
    Calendar,
    Fact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleTrigger {
    EventStarted,
    EventCompleted,
    EventEntered,
    ObjectAcquired,
    ObjectSold,
    EncounterCompleted,
    QuestJoined,
    QuestCompleted,
    DayElapsed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectOperation {
    AddCharacteristic,
    SetCharacteristic,
    MultiplyCharacteristic,
    GrantObject,
    RemoveObject,
    ConsumeObject,
    SetObjectAvailability,
    SetObjectService,
    StartEvent,
    JoinQuest,
    LeaveQuest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipPolicy {
    Buyable,
    RewardOnly,
    Unique,
    MaxOwned,
    Sellable,
    NonSellable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferPolicy {
    None,
    Loan,
    Rental,
    Returnable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoringContract {
    pub version: u32,
    pub boolean_groups: Vec<BooleanGroup>,
    pub requirement_subjects: Vec<RequirementSubject>,
    pub requirement_operators: Vec<RequirementOperator>,
    pub triggers: Vec<RuleTrigger>,
    pub effects: Vec<EffectOperation>,
    pub ownership_policies: Vec<OwnershipPolicy>,
    pub transfer_policies: Vec<TransferPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldMetadata {
    pub name: String,
    pub value_type: String,
    pub required: bool,
    pub description: String,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub unsupported: bool,
    pub deprecated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableMetadata {
    pub file: String,
    pub identity_fields: Vec<String>,
    pub fields: Vec<FieldMetadata>,
    pub supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityMetadata {
    pub metadata_version: u32,
    pub contract: AuthoringContract,
    pub tables: Vec<TableMetadata>,
    pub expressions: ExpressionMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionMetadata {
    pub grammar: String,
    pub functions: Vec<String>,
    pub max_nodes: usize,
    pub max_depth: usize,
}

fn field(name: &str, value_type: &str, required: bool, description: &str) -> FieldMetadata {
    FieldMetadata {
        name: name.to_string(),
        value_type: value_type.to_string(),
        required,
        description: description.to_string(),
        enum_values: Vec::new(),
        references: Vec::new(),
        unit: String::new(),
        default_value: None,
        aliases: Vec::new(),
        unsupported: false,
        deprecated: false,
    }
}

fn table(
    file: &str,
    identity_fields: &[&str],
    fields: &[(&str, &str, bool, &str)],
) -> TableMetadata {
    TableMetadata {
        file: file.to_string(),
        identity_fields: identity_fields
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        fields: fields
            .iter()
            .map(|(name, value_type, required, description)| {
                field(name, value_type, *required, description)
            })
            .collect(),
        supported: true,
    }
}

pub fn capability_metadata() -> CapabilityMetadata {
    let mut tables = vec![
        table(
            "config.csv",
            &["variable"],
            &[
                ("variable", "string", true, "Configuration key"),
                ("value", "string", true, "Configuration value"),
            ],
        ),
        table(
            "player.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Characteristic identifier"),
                ("name", "string", true, "Display name"),
                ("value", "number", true, "Starting value"),
                ("min_value", "number", false, "Optional lower bound"),
                ("max_value", "number", false, "Optional upper bound"),
            ],
        ),
        table(
            "objects.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Object definition identifier"),
                ("type", "identifier", false, "Dataset-defined object type"),
                ("name", "string", true, "Display name"),
                ("price", "number", false, "Acquisition price"),
                (
                    "policy_version",
                    "integer",
                    false,
                    "Object policy contract version (1 preserves legacy defaults)",
                ),
                (
                    "buyable",
                    "boolean",
                    false,
                    "Whether direct acquisition is allowed",
                ),
                (
                    "sellable",
                    "boolean",
                    false,
                    "Whether owned instances may be sold",
                ),
                (
                    "reward_only",
                    "boolean",
                    false,
                    "Whether the object can only be granted by a rule",
                ),
                (
                    "unique",
                    "boolean",
                    false,
                    "Whether at most one instance may be owned",
                ),
                (
                    "max_owned",
                    "integer",
                    false,
                    "Maximum owned instances; zero means unlimited",
                ),
                (
                    "use_policy",
                    "enum",
                    false,
                    "unrestricted, usable, or not_usable",
                ),
                (
                    "consume_policy",
                    "enum",
                    false,
                    "never, on_use, or on_acquire",
                ),
                (
                    "transfer_policy",
                    "enum",
                    false,
                    "none, loan, rental, or returnable",
                ),
                (
                    "rental_duration_days",
                    "integer",
                    false,
                    "Duration for temporary transfers",
                ),
                (
                    "rental_cost",
                    "number",
                    false,
                    "Configured cost for one rental transfer",
                ),
                (
                    "requires_object_ids",
                    "identifier_list",
                    false,
                    "Required object definitions",
                ),
                (
                    "description_html",
                    "path",
                    false,
                    "Dataset-relative description",
                ),
            ],
        ),
        table(
            "costs.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Cost identifier"),
                ("name", "string", true, "Display name"),
                ("amount", "number", true, "Cost amount"),
            ],
        ),
        table(
            "cost_rules.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Rule identifier"),
                ("cost_id", "identifier", true, "Referenced cost"),
                ("trigger_type", "enum", true, "Rule trigger"),
                ("probability", "number", false, "Application probability"),
                (
                    "service_slot",
                    "integer",
                    false,
                    "Explicit object service slot (1 through 15)",
                ),
            ],
        ),
        table(
            "cost_rule_conditions.csv",
            &["rule_id", "subject_type", "subject_ref"],
            &[
                ("rule_id", "identifier", true, "Referenced cost rule"),
                ("subject_type", "enum", true, "Fact subject"),
                ("subject_ref", "identifier", true, "Subject reference"),
                ("operator", "enum", true, "Comparison operator"),
                ("value", "string", true, "Comparison value"),
            ],
        ),
        table(
            "events.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Event identifier"),
                ("name", "string", true, "Display name"),
                (
                    "day_of_year",
                    "integer",
                    true,
                    "Scheduled day; zero means player-started",
                ),
                ("quest_id", "identifier", false, "Optional quest reference"),
                (
                    "required_object_ids",
                    "identifier_list",
                    false,
                    "Required object definitions",
                ),
                ("entry_fee", "number", false, "Entry cost"),
                ("reward_pool", "number", false, "Base reward pool"),
                ("charisma_reward", "number", false, "Characteristic reward"),
                ("duration_value", "integer", false, "Event duration"),
                ("duration_unit", "enum", false, "Duration unit"),
                ("tags", "string", false, "Dataset-defined event tags"),
                (
                    "required_license_id",
                    "identifier",
                    false,
                    "Required licence object",
                ),
                (
                    "requirement_group",
                    "identifier",
                    false,
                    "Optional requirement group",
                ),
                (
                    "quest_event_required",
                    "boolean",
                    false,
                    "Whether the event is required for its quest",
                ),
                (
                    "position_rewards",
                    "string",
                    false,
                    "Configured position rewards",
                ),
                ("type", "identifier", false, "Dataset-defined event type"),
                ("base_cost", "number", false, "Base event cost"),
                ("stamina_cost", "number", false, "Resource cost"),
                ("risk_factor", "number", false, "Configured event risk"),
                ("success_rate", "number", false, "Base success probability"),
                ("payout", "number", false, "Recurring payout"),
                ("payout_freq_type", "enum", false, "Payout frequency type"),
                ("payout_freq", "integer", false, "Payout frequency"),
                ("payout_freq_unit", "enum", false, "Payout frequency unit"),
                (
                    "sponsor_quest_id",
                    "identifier",
                    false,
                    "Sponsor quest reference",
                ),
                (
                    "sponsor_object_id",
                    "identifier",
                    false,
                    "Sponsor object reference",
                ),
                ("sponsor_payouts", "string", false, "Sponsor payout map"),
                (
                    "sponsor_equipment_ids",
                    "identifier_list",
                    false,
                    "Sponsor equipment references",
                ),
                ("encounter_id", "identifier", false, "Encounter reference"),
                ("resolution_method", "enum", false, "Resolution method"),
                (
                    "plugin_id",
                    "identifier",
                    false,
                    "Optional plugin reference",
                ),
                (
                    "description_html",
                    "path",
                    false,
                    "Dataset-relative description",
                ),
            ],
        ),
        table(
            "quests.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Quest definition identifier"),
                ("type", "identifier", false, "Dataset-defined quest mode"),
                ("name", "string", true, "Display name"),
            ],
        ),
        table(
            "event_outcomes.csv",
            &["event_id", "outcome_id"],
            &[
                ("event_id", "identifier", true, "Event reference"),
                ("outcome_id", "identifier", true, "Outcome identifier"),
                ("probability", "number", false, "Outcome probability"),
                (
                    "reward_pool_delta",
                    "number",
                    false,
                    "Reward-pool adjustment",
                ),
                (
                    "charisma_reward_delta",
                    "number",
                    false,
                    "Characteristic reward adjustment",
                ),
                ("message", "string", false, "Outcome message"),
            ],
        ),
        table(
            "event_results.csv",
            &["event_id", "result_id"],
            &[
                ("result_id", "identifier", true, "Result identifier"),
                ("event_id", "identifier", false, "Event reference"),
                ("event_tags", "string", false, "Result event-tag filter"),
                ("reported_result", "string", true, "Reported result value"),
                ("probability", "number", false, "Result probability"),
                (
                    "reward_pool_delta",
                    "number",
                    false,
                    "Reward-pool adjustment",
                ),
                ("effects", "string", false, "Legacy effect expression"),
                ("message", "string", false, "Result message"),
            ],
        ),
        table(
            "condition_groups.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Condition group identifier"),
                ("operator", "enum", true, "all, any, or not"),
                (
                    "children",
                    "identifier_list",
                    false,
                    "Nested group identifiers",
                ),
                ("source_row", "integer", false, "Source CSV row"),
            ],
        ),
        table(
            "conditions.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Condition identifier"),
                ("group_id", "identifier", true, "Condition group reference"),
                ("subject_type", "enum", true, "Fact subject"),
                ("subject_ref", "identifier", true, "Subject reference"),
                ("operator", "enum", true, "Comparison operator"),
                ("value", "string", false, "Expected value"),
                ("source_row", "integer", false, "Source CSV row"),
            ],
        ),
        table(
            "activities.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Activity identifier"),
                ("name", "string", true, "Display name"),
                (
                    "activity_type",
                    "identifier",
                    true,
                    "Dataset-defined activity type",
                ),
                (
                    "resolution_method",
                    "enum",
                    true,
                    "Activity resolution method",
                ),
                ("base_cost", "number", false, "Base cost"),
                ("stamina_cost", "number", false, "Resource cost"),
                ("success_rate", "number", false, "Base success probability"),
                ("payout", "number", false, "Payout amount"),
                ("payout_freq_type", "enum", false, "Payout frequency type"),
                ("payout_freq", "integer", false, "Payout frequency"),
                ("payout_freq_unit", "enum", false, "Payout frequency unit"),
                (
                    "scheduled",
                    "boolean",
                    false,
                    "Whether the activity is scheduled",
                ),
                (
                    "encounter_id",
                    "identifier",
                    false,
                    "Optional encounter reference",
                ),
            ],
        ),
        table(
            "obligations.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Obligation identifier"),
                ("event_id", "identifier", true, "Source event reference"),
                ("resource", "identifier", true, "Payment resource"),
                ("amount", "number", true, "Payment amount"),
                ("interval", "integer", false, "Payment interval"),
                ("interval_unit", "enum", false, "Payment interval unit"),
                ("max_payments", "integer", false, "Maximum payments"),
                ("fault_limit", "integer", false, "Fault threshold"),
                (
                    "active_group",
                    "identifier",
                    false,
                    "Active obligation group",
                ),
                ("active_group_limit", "integer", false, "Active group limit"),
            ],
        ),
        table(
            "effects.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Effect definition identifier"),
                ("operation", "enum", true, "Typed effect operation"),
                ("target", "identifier", true, "Effect target"),
                ("value", "expression", false, "Numeric characteristic value"),
                ("quantity", "integer", false, "Object quantity"),
            ],
        ),
        table(
            "effect_bindings.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Binding identifier"),
                ("effect_id", "identifier", true, "Effect reference"),
                ("trigger_type", "enum", true, "Lifecycle trigger"),
                (
                    "trigger_ref",
                    "identifier",
                    false,
                    "Optional event reference",
                ),
                ("reported_result", "string", false, "Optional result filter"),
                ("probability", "number", false, "Application probability"),
            ],
        ),
        table(
            "requirement_bindings.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Requirement binding identifier"),
                (
                    "operation",
                    "enum",
                    true,
                    "Operation gated by this requirement",
                ),
                (
                    "requirement_group",
                    "identifier",
                    true,
                    "Condition group reference",
                ),
                (
                    "target_ref",
                    "identifier",
                    true,
                    "Operation target reference",
                ),
            ],
        ),
        table(
            "numeric_modifiers.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Numeric modifier identifier"),
                ("target", "identifier", true, "Modified numeric field"),
                ("operation", "enum", true, "set, add, or multiply"),
                ("value", "expression", true, "Modifier expression"),
                ("priority", "integer", false, "Ordering priority"),
                (
                    "condition_group",
                    "identifier",
                    false,
                    "Optional condition group reference",
                ),
                ("minimum", "number", false, "Optional lower clamp"),
                ("maximum", "number", false, "Optional upper clamp"),
            ],
        ),
        table(
            "texts.csv",
            &["variable"],
            &[
                ("variable", "identifier", true, "Text key"),
                ("value", "string", true, "Text variant"),
            ],
        ),
        table(
            "colors.csv",
            &["element_id"],
            &[
                ("element_id", "identifier", true, "Color token"),
                ("label", "string", true, "Human-readable color label"),
                ("hex_color", "string", true, "Current #RRGGBB color value"),
                ("category", "string", true, "Editor color category"),
                ("default_hex", "string", true, "Default #RRGGBB color value"),
            ],
        ),
        table(
            "plugins.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Plugin identifier"),
                (
                    "entrypoint",
                    "path",
                    true,
                    "Dataset-relative plugin entrypoint",
                ),
                (
                    "protocol_version",
                    "integer",
                    true,
                    "Plugin protocol version",
                ),
                ("capability", "enum_list", true, "Plugin capability"),
                (
                    "result_schema",
                    "identifier",
                    true,
                    "Result schema identifier",
                ),
                (
                    "result_schema_version",
                    "integer",
                    true,
                    "Result schema version",
                ),
                (
                    "required",
                    "boolean",
                    false,
                    "Whether the plugin must be available",
                ),
                ("dispatch", "string", false, "Optional dispatch selector"),
            ],
        ),
        table(
            "encounter_attributes.csv",
            &["attribute_id"],
            &[
                (
                    "attribute_id",
                    "identifier",
                    true,
                    "Encounter attribute identifier",
                ),
                ("display_name", "string", true, "Display name"),
                ("min_value", "number", false, "Minimum value"),
                ("max_value", "number", false, "Maximum value"),
                (
                    "is_loss_condition",
                    "boolean",
                    false,
                    "Whether reaching the minimum loses",
                ),
                (
                    "visible_to_player",
                    "boolean",
                    false,
                    "Whether shown to the player",
                ),
            ],
        ),
        table(
            "encounter_actions.csv",
            &["action_id"],
            &[
                (
                    "action_id",
                    "identifier",
                    true,
                    "Encounter action identifier",
                ),
                ("display_name", "string", true, "Display name"),
                ("usable_by", "identifier", false, "Allowed actor"),
                (
                    "requires_attribute_id",
                    "identifier",
                    false,
                    "Required attribute",
                ),
                (
                    "requires_attribute_min",
                    "number",
                    false,
                    "Minimum required attribute value",
                ),
                ("requires_object_id", "identifier", false, "Required object"),
                (
                    "consumes_object",
                    "boolean",
                    false,
                    "Consume the required object on use",
                ),
                (
                    "target_attribute_id",
                    "identifier",
                    true,
                    "Affected attribute",
                ),
                (
                    "base_success_rate",
                    "number",
                    false,
                    "Base success probability",
                ),
                (
                    "resource_cost_attribute_id",
                    "identifier",
                    false,
                    "Resource cost attribute",
                ),
                (
                    "resource_cost_amount",
                    "number",
                    false,
                    "Resource cost amount",
                ),
                (
                    "success_modifier_attribute_id",
                    "identifier",
                    false,
                    "Attribute modifying success probability",
                ),
                (
                    "success_modifier_scale",
                    "number",
                    false,
                    "Success modifier scale",
                ),
                (
                    "effect_on_success",
                    "number",
                    false,
                    "Success effect amount",
                ),
                (
                    "effect_on_success_target",
                    "enum",
                    false,
                    "Success effect target",
                ),
                (
                    "effect_on_failure",
                    "number",
                    false,
                    "Failure effect amount",
                ),
                (
                    "effect_on_failure_target",
                    "enum",
                    false,
                    "Failure effect target",
                ),
                ("cooldown_turns", "integer", false, "Cooldown duration"),
                ("flavor_text_success", "string", false, "Success log text"),
                ("flavor_text_failure", "string", false, "Failure log text"),
                ("result_max", "number", false, "Maximum result value"),
                (
                    "defense_reduction",
                    "number",
                    false,
                    "Attack-defense reduction",
                ),
                (
                    "ai_weight",
                    "number",
                    false,
                    "Legacy opponent action weight",
                ),
            ],
        ),
        table(
            "encounter_objects.csv",
            &["object_id"],
            &[
                ("object_id", "identifier", true, "Object reference"),
                (
                    "enables_action_id",
                    "identifier",
                    false,
                    "Enabled action reference",
                ),
                (
                    "success_rate_bonus",
                    "number",
                    false,
                    "Success probability bonus",
                ),
                (
                    "consumable_in_encounter",
                    "boolean",
                    false,
                    "Consumed during encounter",
                ),
            ],
        ),
        table(
            "encounter_opponents.csv",
            &["opponent_id"],
            &[
                ("opponent_id", "identifier", true, "Opponent identifier"),
                ("display_name", "string", true, "Display name"),
                (
                    "starting_attributes",
                    "string",
                    false,
                    "Initial attribute values",
                ),
                (
                    "available_action_ids",
                    "identifier_list",
                    false,
                    "Available actions",
                ),
                ("strategy", "enum", false, "Opponent strategy"),
                ("action_weights", "string", false, "Action weights"),
                (
                    "scripted_actions",
                    "string",
                    false,
                    "Scripted action sequence",
                ),
            ],
        ),
        table(
            "encounter_outcomes.csv",
            &["outcome_id"],
            &[
                (
                    "outcome_id",
                    "identifier",
                    true,
                    "Encounter outcome identifier",
                ),
                (
                    "applies_to_encounter_id",
                    "identifier",
                    false,
                    "Encounter reference",
                ),
                ("trigger", "enum", true, "Outcome trigger"),
                ("consequence_type", "enum", true, "Consequence type"),
                (
                    "consequence_target",
                    "identifier",
                    true,
                    "Consequence target",
                ),
                ("consequence_value", "string", false, "Consequence value"),
                ("probability", "number", false, "Outcome probability"),
            ],
        ),
        table(
            "encounter_config.csv",
            &["encounter_id"],
            &[
                (
                    "encounter_id",
                    "identifier",
                    true,
                    "Encounter configuration identifier",
                ),
                ("display_label", "string", true, "Display label"),
                ("turn_order", "enum", false, "Turn order"),
                ("max_turns", "integer", false, "Maximum turns"),
                ("tiebreaker", "enum", false, "Tie handling"),
                (
                    "allow_retreat",
                    "boolean",
                    false,
                    "Whether retreat is allowed",
                ),
                ("rng_mode", "enum", false, "Randomness mode"),
                ("opponent_id", "identifier", false, "Opponent reference"),
                ("mode", "enum", false, "Encounter mode"),
                (
                    "player_starting_attributes",
                    "string",
                    false,
                    "Initial player attributes",
                ),
            ],
        ),
    ];

    for metadata in &mut tables {
        for field in &mut metadata.fields {
            if matches!(
                (metadata.file.as_str(), field.name.as_str()),
                ("encounter_attributes.csv", "visible_to_player")
                    | ("encounter_actions.csv", "ai_weight")
                    | ("encounter_objects.csv", "consumable_in_encounter")
                    | ("encounter_opponents.csv", "action_weights")
                    | ("encounter_opponents.csv", "scripted_actions")
                    | ("encounter_config.csv", "rng_mode")
            ) {
                field.unsupported = true;
                field.description = format!(
                    "{} Loaded for compatibility, but this field is not interpreted by the engine.",
                    field.description
                );
            }
            field.enum_values = match (metadata.file.as_str(), field.name.as_str()) {
                ("effect_bindings.csv", "trigger_type") => vec![
                    "event_started".into(),
                    "event_entered".into(),
                    "event_completed".into(),
                    "object_acquired".into(),
                    "object_sold".into(),
                    "encounter_completed".into(),
                    "quest_joined".into(),
                    "quest_completed".into(),
                    "day_elapsed".into(),
                ],
                ("effect_bindings.csv", "reported_result") => {
                    vec!["success".into(), "failure".into()]
                }
                ("numeric_modifiers.csv", "operation") => {
                    vec!["set".into(), "add".into(), "multiply".into()]
                }
                ("requirement_bindings.csv", "operation") => vec![
                    "acquire".into(),
                    "sell".into(),
                    "use".into(),
                    "consume".into(),
                    "rent".into(),
                    "loan".into(),
                    "return".into(),
                    "event_start".into(),
                    "event_entry".into(),
                    "encounter_action".into(),
                    "quest_join".into(),
                ],
                ("condition_groups.csv", "operator") => {
                    vec!["all".into(), "any".into(), "not".into()]
                }
                ("conditions.csv", "subject_type") => vec![
                    "characteristic".into(),
                    "object".into(),
                    "object_type".into(),
                    "object_count".into(),
                    "event_history".into(),
                    "quest_status".into(),
                    "active_event".into(),
                    "calendar".into(),
                    "fact".into(),
                ],
                ("conditions.csv", "operator") => vec![
                    "equals".into(),
                    "not_equals".into(),
                    "greater_than".into(),
                    "greater_or_equal".into(),
                    "less_than".into(),
                    "less_or_equal".into(),
                    "contains".into(),
                    "starts_with".into(),
                ],
                _ => field.enum_values.clone(),
            };
            field.references = match (metadata.file.as_str(), field.name.as_str()) {
                ("effect_bindings.csv", "effect_id") => vec!["effects.csv".into()],
                ("effect_bindings.csv", "trigger_ref") => vec![
                    "events.csv".into(),
                    "objects.csv".into(),
                    "quests.csv".into(),
                    "encounter_config.csv".into(),
                ],
                ("requirement_bindings.csv", "requirement_group") => {
                    vec!["condition_groups.csv".into()]
                }
                ("numeric_modifiers.csv", "condition_group") => {
                    vec!["condition_groups.csv".into()]
                }
                ("conditions.csv", "group_id") => vec!["condition_groups.csv".into()],
                ("obligations.csv", "event_id") => vec!["events.csv".into()],
                ("events.csv", "required_license_id") | ("events.csv", "required_object_ids") => {
                    vec!["objects.csv".into()]
                }
                ("events.csv", "quest_id") => vec!["quests.csv".into()],
                ("events.csv", "encounter_id") => vec!["encounter_config.csv".into()],
                ("events.csv", "plugin_id") => vec!["plugins.csv".into()],
                ("event_outcomes.csv", "event_id") | ("event_results.csv", "event_id") => {
                    vec!["events.csv".into()]
                }
                ("activities.csv", "encounter_id") => vec!["encounter_config.csv".into()],
                ("encounter_actions.csv", "requires_object_id") => {
                    vec!["objects.csv".into()]
                }
                ("encounter_objects.csv", "object_id") => vec!["objects.csv".into()],
                ("encounter_objects.csv", "enables_action_id") => {
                    vec!["encounter_actions.csv".into()]
                }
                ("encounter_opponents.csv", "available_action_ids") => {
                    vec!["encounter_actions.csv".into()]
                }
                ("encounter_outcomes.csv", "applies_to_encounter_id") => {
                    vec!["encounter_config.csv".into()]
                }
                ("encounter_config.csv", "opponent_id") => {
                    vec!["encounter_opponents.csv".into()]
                }
                _ => field.references.clone(),
            };
            field.unit = match field.name.as_str() {
                "amount"
                | "price"
                | "entry_fee"
                | "reward_pool"
                | "charisma_reward"
                | "base_cost"
                | "payout"
                | "join_fee"
                | "license_fee"
                | "resource_cost_amount"
                | "effect_on_success"
                | "effect_on_failure" => "currency_or_points".into(),
                name if name.contains("probability") || name.contains("rate") => {
                    "probability".into()
                }
                name if name.contains("day") || name.contains("interval") => "days".into(),
                _ => field.unit.clone(),
            };
            field.default_value = match (metadata.file.as_str(), field.name.as_str()) {
                ("objects.csv", "buyable") | ("objects.csv", "sellable") => Some("true".into()),
                ("objects.csv", "use_policy") => Some("unrestricted".into()),
                ("objects.csv", "consume_policy") => Some("never".into()),
                ("objects.csv", "transfer_policy") => Some("none".into()),
                ("events.csv", "day_of_year") => Some("0".into()),
                ("events.csv", "duration_unit") => Some("days".into()),
                ("effect_bindings.csv", "probability") | ("numeric_modifiers.csv", "priority") => {
                    Some("0".into())
                }
                ("plugins.csv", "required") => Some("false".into()),
                _ => field.default_value.clone(),
            };
            field.aliases = match (metadata.file.as_str(), field.name.as_str()) {
                ("objects.csv", "description_html") => vec![
                    "description".into(),
                    "description_path".into(),
                    "html".into(),
                ],
                ("events.csv", "quest_id") => vec!["championship_id".into()],
                ("quests.csv", "type") => vec!["quest_type".into()],
                _ => field.aliases.clone(),
            };
        }
    }

    CapabilityMetadata {
        metadata_version: CAPABILITY_METADATA_VERSION,
        contract: AuthoringContract::default(),
        expressions: ExpressionMetadata {
            grammar: "literals, identifiers, +, -, *, /, parentheses, min, max, clamp".to_string(),
            functions: vec!["min".to_string(), "max".to_string(), "clamp".to_string()],
            max_nodes: 256,
            max_depth: 32,
        },
        tables,
    }
}

impl Default for AuthoringContract {
    fn default() -> Self {
        Self {
            version: AUTHORING_CONTRACT_VERSION,
            boolean_groups: vec![BooleanGroup::All, BooleanGroup::Any, BooleanGroup::Not],
            requirement_subjects: vec![
                RequirementSubject::PlayerCharacteristic,
                RequirementSubject::ObjectDefinition,
                RequirementSubject::ObjectType,
                RequirementSubject::ObjectCount,
                RequirementSubject::EventHistory,
                RequirementSubject::QuestStatus,
                RequirementSubject::ActiveEvent,
                RequirementSubject::Calendar,
                RequirementSubject::Fact,
            ],
            requirement_operators: vec![
                RequirementOperator::Equals,
                RequirementOperator::NotEquals,
                RequirementOperator::GreaterThan,
                RequirementOperator::GreaterOrEqual,
                RequirementOperator::LessThan,
                RequirementOperator::LessOrEqual,
                RequirementOperator::Contains,
                RequirementOperator::StartsWith,
            ],
            triggers: vec![
                RuleTrigger::EventStarted,
                RuleTrigger::EventCompleted,
                RuleTrigger::EventEntered,
                RuleTrigger::ObjectAcquired,
                RuleTrigger::ObjectSold,
                RuleTrigger::EncounterCompleted,
                RuleTrigger::QuestJoined,
                RuleTrigger::QuestCompleted,
                RuleTrigger::DayElapsed,
            ],
            effects: vec![
                EffectOperation::AddCharacteristic,
                EffectOperation::SetCharacteristic,
                EffectOperation::MultiplyCharacteristic,
                EffectOperation::GrantObject,
                EffectOperation::RemoveObject,
                EffectOperation::ConsumeObject,
                EffectOperation::SetObjectAvailability,
                EffectOperation::SetObjectService,
                EffectOperation::StartEvent,
                EffectOperation::JoinQuest,
                EffectOperation::LeaveQuest,
            ],
            ownership_policies: vec![
                OwnershipPolicy::Buyable,
                OwnershipPolicy::RewardOnly,
                OwnershipPolicy::Unique,
                OwnershipPolicy::MaxOwned,
                OwnershipPolicy::Sellable,
                OwnershipPolicy::NonSellable,
            ],
            transfer_policies: vec![
                TransferPolicy::None,
                TransferPolicy::Loan,
                TransferPolicy::Rental,
                TransferPolicy::Returnable,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{capability_metadata, AuthoringContract, AUTHORING_CONTRACT_VERSION};

    #[test]
    fn default_contract_is_versioned_and_complete() {
        let contract = AuthoringContract::default();
        assert_eq!(contract.version, AUTHORING_CONTRACT_VERSION);
        assert!(contract.boolean_groups.len() >= 3);
        assert!(contract
            .requirement_subjects
            .iter()
            .any(|subject| { matches!(subject, super::RequirementSubject::ObjectCount) }));
        assert!(contract
            .effects
            .iter()
            .any(|effect| matches!(effect, super::EffectOperation::ConsumeObject)));
    }

    #[test]
    fn capability_metadata_covers_loaded_and_extension_tables() {
        let metadata = capability_metadata();
        assert_eq!(metadata.metadata_version, 2);
        for file in [
            "config.csv",
            "player.csv",
            "objects.csv",
            "events.csv",
            "quests.csv",
            "condition_groups.csv",
            "conditions.csv",
            "activities.csv",
            "obligations.csv",
            "effects.csv",
            "effect_bindings.csv",
            "requirement_bindings.csv",
            "numeric_modifiers.csv",
            "texts.csv",
            "colors.csv",
            "encounter_attributes.csv",
            "encounter_actions.csv",
            "encounter_objects.csv",
            "encounter_opponents.csv",
            "encounter_outcomes.csv",
            "encounter_config.csv",
            "plugins.csv",
        ] {
            assert!(metadata.tables.iter().any(|table| table.file == file));
        }
    }

    #[test]
    fn capability_metadata_has_unique_tables_and_declares_identity_fields() {
        let metadata = capability_metadata();
        let mut files = std::collections::HashSet::new();
        for table in &metadata.tables {
            assert!(
                files.insert(&table.file),
                "duplicate metadata table: {}",
                table.file
            );
            assert!(
                !table.identity_fields.is_empty(),
                "table {} must declare an identity",
                table.file
            );
            for identity in &table.identity_fields {
                assert!(
                    table.fields.iter().any(|field| &field.name == identity),
                    "table {} identity {} is not declared as a field",
                    table.file,
                    identity
                );
            }
        }
    }

    #[test]
    fn rule_table_metadata_exposes_contract_values_and_references() {
        let metadata = capability_metadata();
        let colors = metadata
            .tables
            .iter()
            .find(|table| table.file == "colors.csv")
            .expect("color metadata should exist");
        assert_eq!(colors.identity_fields, vec!["element_id"]);
        assert_eq!(
            colors
                .fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "element_id",
                "label",
                "hex_color",
                "category",
                "default_hex"
            ]
        );

        let effects = metadata
            .tables
            .iter()
            .find(|table| table.file == "effect_bindings.csv")
            .expect("effect binding metadata should exist");
        let trigger = effects
            .fields
            .iter()
            .find(|field| field.name == "trigger_type")
            .expect("trigger metadata should exist");
        assert!(trigger.enum_values.contains(&"event_started".into()));
        let effect_reference = effects
            .fields
            .iter()
            .find(|field| field.name == "effect_id")
            .expect("effect reference metadata should exist");
        assert_eq!(effect_reference.references, vec!["effects.csv"]);

        let modifiers = metadata
            .tables
            .iter()
            .find(|table| table.file == "numeric_modifiers.csv")
            .expect("modifier metadata should exist");
        let operation = modifiers
            .fields
            .iter()
            .find(|field| field.name == "operation")
            .expect("modifier operation metadata should exist");
        assert_eq!(operation.enum_values, vec!["set", "add", "multiply"]);
        let objects = metadata
            .tables
            .iter()
            .find(|table| table.file == "objects.csv")
            .expect("object metadata should exist");
        let buyable = objects
            .fields
            .iter()
            .find(|field| field.name == "buyable")
            .expect("buyable metadata should exist");
        assert_eq!(buyable.default_value.as_deref(), Some("true"));
        assert!(!buyable.unsupported);

        let event_results = metadata
            .tables
            .iter()
            .find(|table| table.file == "event_results.csv")
            .expect("event result metadata should exist");
        assert_eq!(event_results.identity_fields, vec!["event_id", "result_id"]);
        let event_reference = event_results
            .fields
            .iter()
            .find(|field| field.name == "event_id")
            .expect("event result reference metadata should exist");
        assert_eq!(event_reference.references, vec!["events.csv"]);
    }
}
