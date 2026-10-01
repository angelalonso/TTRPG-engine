use serde::{Deserialize, Serialize};

pub const AUTHORING_CONTRACT_VERSION: u32 = 1;
pub const CAPABILITY_METADATA_VERSION: u32 = 1;

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
            ],
        ),
        table(
            "cost_rule_conditions.csv",
            &["rule_id", "subject_type", "subject_ref"],
            &[
                ("rule_id", "identifier", true, "Referenced cost rule"),
                ("subject_type", "enum", true, "Fact subject"),
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
                (
                    "description_html",
                    "path",
                    false,
                    "Dataset-relative description",
                ),
            ],
        ),
        table(
            "obligations.csv",
            &["id"],
            &[
                ("id", "identifier", true, "Obligation identifier"),
                ("event_id", "identifier", true, "Source event"),
                ("resource", "identifier", true, "Characteristic to pay"),
                ("amount", "number", true, "Payment amount"),
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
            &["event_id", "id"],
            &[
                ("event_id", "identifier", true, "Event reference"),
                ("id", "identifier", true, "Outcome identifier"),
            ],
        ),
        table(
            "event_results.csv",
            &["event_id", "id"],
            &[
                ("event_id", "identifier", true, "Event reference"),
                ("id", "identifier", true, "Result identifier"),
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
            &["variable"],
            &[
                ("variable", "identifier", true, "Color token"),
                ("value", "string", true, "Color value"),
            ],
        ),
        table(
            "encounter_attributes.csv",
            &["id"],
            &[("id", "identifier", true, "Encounter attribute identifier")],
        ),
        table(
            "encounter_actions.csv",
            &["id"],
            &[("id", "identifier", true, "Encounter action identifier")],
        ),
        table(
            "encounter_objects.csv",
            &["id"],
            &[("id", "identifier", true, "Encounter object rule identifier")],
        ),
        table(
            "encounter_opponents.csv",
            &["id"],
            &[("id", "identifier", true, "Opponent identifier")],
        ),
        table(
            "encounter_outcomes.csv",
            &["id"],
            &[("id", "identifier", true, "Encounter outcome identifier")],
        ),
        table(
            "encounter_config.csv",
            &["id"],
            &[(
                "id",
                "identifier",
                true,
                "Encounter configuration identifier",
            )],
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
    ];

    for metadata in &mut tables {
        for field in &mut metadata.fields {
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
                _ => field.references.clone(),
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
        assert_eq!(metadata.metadata_version, 1);
        for file in [
            "config.csv",
            "player.csv",
            "objects.csv",
            "events.csv",
            "quests.csv",
            "effects.csv",
            "effect_bindings.csv",
            "requirement_bindings.csv",
            "numeric_modifiers.csv",
            "texts.csv",
            "colors.csv",
            "plugins.csv",
        ] {
            assert!(metadata.tables.iter().any(|table| table.file == file));
        }
    }

    #[test]
    fn rule_table_metadata_exposes_contract_values_and_references() {
        let metadata = capability_metadata();
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
    }
}
