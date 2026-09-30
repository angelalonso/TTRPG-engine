use super::schema::EffectOperation;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PLUGIN_PROTOCOL_VERSION: u32 = 1;
pub const PLUGIN_RESULT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    NormalizeResult,
    EvaluateCustomFact,
    ProvideResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginOperation {
    NormalizeResult,
    EvaluateCustomFact,
    ProvideResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub id: String,
    pub entrypoint: String,
    pub protocol_version: u32,
    pub capabilities: Vec<PluginCapability>,
    pub result_schema: String,
    pub result_schema_version: u32,
    pub required: bool,
    pub dispatch: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginRequest {
    pub protocol_version: u32,
    pub plugin_id: String,
    pub operation: PluginOperation,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginFact {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginResult {
    pub id: String,
    pub value: Value,
    #[serde(default)]
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposedEffect {
    pub operation: EffectOperation,
    pub target: String,
    #[serde(default)]
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginResponse {
    pub protocol_version: u32,
    pub plugin_id: String,
    pub result_schema_version: u32,
    #[serde(default)]
    pub facts: Vec<PluginFact>,
    #[serde(default)]
    pub results: Vec<PluginResult>,
    #[serde(default)]
    pub effects: Vec<ProposedEffect>,
}

impl PluginManifest {
    pub fn supports(&self, capability: &PluginCapability) -> bool {
        self.capabilities.iter().any(|item| item == capability)
    }

    pub fn validate_response(&self, response: &PluginResponse) -> Result<(), String> {
        if response.plugin_id != self.id {
            return Err(format!(
                "plugin response belongs to '{}', expected '{}'",
                response.plugin_id, self.id
            ));
        }
        if response.protocol_version != self.protocol_version
            || response.protocol_version != PLUGIN_PROTOCOL_VERSION
        {
            return Err(format!(
                "plugin '{}' returned unsupported protocol version {}",
                self.id, response.protocol_version
            ));
        }
        if response.result_schema_version != self.result_schema_version
            || response.result_schema_version != PLUGIN_RESULT_SCHEMA_VERSION
        {
            return Err(format!(
                "plugin '{}' returned unsupported result schema version {}",
                self.id, response.result_schema_version
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest() -> PluginManifest {
        PluginManifest {
            id: "cookbook".into(),
            entrypoint: "plugins/cookbook.py".into(),
            protocol_version: PLUGIN_PROTOCOL_VERSION,
            capabilities: vec![PluginCapability::ProvideResult],
            result_schema: "generic_result".into(),
            result_schema_version: PLUGIN_RESULT_SCHEMA_VERSION,
            required: true,
            dispatch: "cooking".into(),
        }
    }

    #[test]
    fn protocol_response_is_checked_at_the_boundary() {
        let response = PluginResponse {
            protocol_version: PLUGIN_PROTOCOL_VERSION,
            plugin_id: "cookbook".into(),
            result_schema_version: PLUGIN_RESULT_SCHEMA_VERSION,
            facts: vec![PluginFact {
                key: "dish.ready".into(),
                value: json!(true),
            }],
            results: Vec::new(),
            effects: Vec::new(),
        };
        assert!(manifest().validate_response(&response).is_ok());
    }

    #[test]
    fn unknown_response_fields_are_rejected() {
        let parsed = serde_json::from_str::<PluginResponse>(
            r#"{"protocol_version":1,"plugin_id":"cookbook","result_schema_version":1,"unexpected":true}"#,
        );
        assert!(parsed.is_err());
    }
}
