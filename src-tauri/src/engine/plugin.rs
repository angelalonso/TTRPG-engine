use super::schema::EffectOperation;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const PLUGIN_PROTOCOL_VERSION: u32 = 1;
pub const PLUGIN_RESULT_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_PLUGIN_TIMEOUT: Duration = Duration::from_secs(2);
pub const DEFAULT_PLUGIN_OUTPUT_LIMIT: usize = 1024 * 1024;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginExecutionLimits {
    pub timeout: Duration,
    pub max_stdout_bytes: usize,
    pub max_stderr_bytes: usize,
}

impl Default for PluginExecutionLimits {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_PLUGIN_TIMEOUT,
            max_stdout_bytes: DEFAULT_PLUGIN_OUTPUT_LIMIT,
            max_stderr_bytes: DEFAULT_PLUGIN_OUTPUT_LIMIT,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PluginExecutionError {
    InvalidEntrypoint(String),
    EntrypointNotFound(PathBuf),
    Spawn(String),
    WriteRequest(String),
    TimedOut(Duration),
    OutputLimitExceeded { stream: &'static str, limit: usize },
    NonZeroExit { status: String, stderr: String },
    InvalidUtf8(&'static str),
    InvalidResponse(String),
}

impl fmt::Display for PluginExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEntrypoint(message) => {
                write!(formatter, "invalid plugin entrypoint: {message}")
            }
            Self::EntrypointNotFound(path) => {
                write!(formatter, "plugin entrypoint not found: {}", path.display())
            }
            Self::Spawn(message) => write!(formatter, "could not start plugin: {message}"),
            Self::WriteRequest(message) => {
                write!(formatter, "could not send plugin request: {message}")
            }
            Self::TimedOut(timeout) => write!(
                formatter,
                "plugin timed out after {} ms",
                timeout.as_millis()
            ),
            Self::OutputLimitExceeded { stream, limit } => {
                write!(formatter, "plugin {stream} exceeded the {limit}-byte limit")
            }
            Self::NonZeroExit { status, stderr } => {
                if stderr.is_empty() {
                    write!(
                        formatter,
                        "plugin exited unsuccessfully with status {status}"
                    )
                } else {
                    write!(
                        formatter,
                        "plugin exited unsuccessfully with status {status}: {stderr}"
                    )
                }
            }
            Self::InvalidUtf8(stream) => {
                write!(formatter, "plugin returned invalid UTF-8 on {stream}")
            }
            Self::InvalidResponse(message) => {
                write!(formatter, "invalid plugin response: {message}")
            }
        }
    }
}

impl std::error::Error for PluginExecutionError {}

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

    pub fn normalized_dispatch(&self) -> Result<String, String> {
        let dispatch = self.dispatch.trim().to_ascii_lowercase().replace('-', "_");
        if dispatch.is_empty() {
            return Err(format!("plugin '{}' has an empty dispatch", self.id));
        }
        if !dispatch
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return Err(format!(
                "plugin '{}' has an invalid dispatch '{}'",
                self.id, self.dispatch
            ));
        }
        Ok(dispatch)
    }

    pub fn execute_operation(
        &self,
        dataset_root: &Path,
        operation: PluginOperation,
        payload: Value,
        limits: PluginExecutionLimits,
    ) -> Result<PluginResponse, PluginExecutionError> {
        self.normalized_dispatch()
            .map_err(PluginExecutionError::InvalidResponse)?;
        let capability = match operation {
            PluginOperation::NormalizeResult => PluginCapability::NormalizeResult,
            PluginOperation::EvaluateCustomFact => PluginCapability::EvaluateCustomFact,
            PluginOperation::ProvideResult => PluginCapability::ProvideResult,
        };
        if !self.supports(&capability) {
            return Err(PluginExecutionError::InvalidResponse(format!(
                "plugin '{}' does not declare the '{}' capability",
                self.id,
                serde_json::to_string(&capability).unwrap_or_else(|_| "requested".into())
            )));
        }
        self.execute(
            dataset_root,
            &PluginRequest {
                protocol_version: PLUGIN_PROTOCOL_VERSION,
                plugin_id: self.id.clone(),
                operation,
                payload,
            },
            limits,
        )
    }

    pub fn resolve_entrypoint(&self, dataset_root: &Path) -> Result<PathBuf, PluginExecutionError> {
        let raw = self.entrypoint.trim();
        if raw.is_empty() {
            return Err(PluginExecutionError::InvalidEntrypoint(
                "entrypoint is empty".into(),
            ));
        }
        let relative = Path::new(raw);
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| component == std::path::Component::ParentDir)
        {
            return Err(PluginExecutionError::InvalidEntrypoint(format!(
                "'{raw}' must be a dataset-relative path"
            )));
        }
        let path = dataset_root.join(relative);
        if !path.is_file() {
            return Err(PluginExecutionError::EntrypointNotFound(path));
        }
        Ok(path)
    }

    pub fn execute(
        &self,
        dataset_root: &Path,
        request: &PluginRequest,
        limits: PluginExecutionLimits,
    ) -> Result<PluginResponse, PluginExecutionError> {
        if request.plugin_id != self.id {
            return Err(PluginExecutionError::InvalidResponse(format!(
                "request belongs to '{}', expected '{}'",
                request.plugin_id, self.id
            )));
        }
        if request.protocol_version != self.protocol_version
            || request.protocol_version != PLUGIN_PROTOCOL_VERSION
        {
            return Err(PluginExecutionError::InvalidResponse(format!(
                "plugin '{}' received unsupported protocol version {}",
                self.id, request.protocol_version
            )));
        }
        let entrypoint = self.resolve_entrypoint(dataset_root)?;
        let mut command = if entrypoint
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("py")
        {
            let mut command = Command::new("python3");
            command.arg(&entrypoint);
            command
        } else {
            Command::new(&entrypoint)
        };
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| PluginExecutionError::Spawn(error.to_string()))?;

        let payload = serde_json::to_vec(request)
            .map_err(|error| PluginExecutionError::WriteRequest(error.to_string()))?;
        child
            .stdin
            .take()
            .ok_or_else(|| PluginExecutionError::WriteRequest("stdin was unavailable".into()))?
            .write_all(&payload)
            .map_err(|error| PluginExecutionError::WriteRequest(error.to_string()))?;

        let stdout = read_bounded(
            child
                .stdout
                .take()
                .ok_or_else(|| PluginExecutionError::Spawn("stdout was unavailable".into()))?,
            limits.max_stdout_bytes,
        );
        let stderr = read_bounded(
            child
                .stderr
                .take()
                .ok_or_else(|| PluginExecutionError::Spawn("stderr was unavailable".into()))?,
            limits.max_stderr_bytes,
        );
        let status = wait_bounded(&mut child, limits.timeout)?;
        let stdout = stdout
            .join()
            .map_err(|_| PluginExecutionError::Spawn("stdout reader panicked".into()))?;
        let stderr = stderr
            .join()
            .map_err(|_| PluginExecutionError::Spawn("stderr reader panicked".into()))?;
        if stdout.1 {
            return Err(PluginExecutionError::OutputLimitExceeded {
                stream: "stdout",
                limit: limits.max_stdout_bytes,
            });
        }
        if stderr.1 {
            return Err(PluginExecutionError::OutputLimitExceeded {
                stream: "stderr",
                limit: limits.max_stderr_bytes,
            });
        }
        let stderr_text =
            String::from_utf8(stderr.0).map_err(|_| PluginExecutionError::InvalidUtf8("stderr"))?;
        if !status.success() {
            return Err(PluginExecutionError::NonZeroExit {
                status: status.to_string(),
                stderr: stderr_text,
            });
        }
        let stdout_text =
            String::from_utf8(stdout.0).map_err(|_| PluginExecutionError::InvalidUtf8("stdout"))?;
        let response = serde_json::from_str::<PluginResponse>(&stdout_text)
            .map_err(|error| PluginExecutionError::InvalidResponse(error.to_string()))?;
        self.validate_response(&response)
            .map_err(PluginExecutionError::InvalidResponse)?;
        Ok(response)
    }
}

fn read_bounded<R: Read + Send + 'static>(
    mut reader: R,
    limit: usize,
) -> thread::JoinHandle<(Vec<u8>, bool)> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut buffer = [0; 4096];
        let mut exceeded = false;
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    let previous_len = bytes.len();
                    if bytes.len() < limit {
                        let remaining = limit - bytes.len();
                        bytes.extend_from_slice(&buffer[..size.min(remaining)]);
                    }
                    if size > limit.saturating_sub(previous_len) {
                        exceeded = true;
                    }
                }
                Err(_) => break,
            }
        }
        (bytes, exceeded)
    })
}

fn wait_bounded(
    child: &mut Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, PluginExecutionError> {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(PluginExecutionError::TimedOut(timeout));
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(error) => return Err(PluginExecutionError::Spawn(error.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest() -> PluginManifest {
        PluginManifest {
            id: "cookbook".into(),
            entrypoint: "plugin_echo.py".into(),
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
    fn dispatch_is_normalized_without_accepting_arbitrary_tokens() {
        let mut manifest = manifest();
        manifest.dispatch = " Cooking-Results ".into();
        assert_eq!(manifest.normalized_dispatch().unwrap(), "cooking_results");

        manifest.dispatch = "cooking/results".into();
        assert!(manifest.normalized_dispatch().is_err());
    }

    #[test]
    fn operation_dispatch_requires_declared_capability() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
        let error = manifest()
            .execute_operation(
                &root,
                PluginOperation::NormalizeResult,
                Value::Null,
                PluginExecutionLimits::default(),
            )
            .expect_err("undeclared operation should be rejected before spawn");
        assert!(matches!(error, PluginExecutionError::InvalidResponse(_)));
    }

    #[test]
    fn unknown_response_fields_are_rejected() {
        let parsed = serde_json::from_str::<PluginResponse>(
            r#"{"protocol_version":1,"plugin_id":"cookbook","result_schema_version":1,"unexpected":true}"#,
        );
        assert!(parsed.is_err());
    }

    #[test]
    fn entrypoint_resolution_is_dataset_relative_and_bounded() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
        let path = manifest()
            .resolve_entrypoint(&root)
            .expect("fixture entrypoint should resolve");
        assert!(path.ends_with("plugin_echo.py"));

        let mut unsafe_manifest = manifest();
        unsafe_manifest.entrypoint = "../plugin_echo.py".into();
        assert!(matches!(
            unsafe_manifest.resolve_entrypoint(&root),
            Err(PluginExecutionError::InvalidEntrypoint(_))
        ));
    }

    #[test]
    fn plugin_request_round_trip_uses_structured_response_validation() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
        let response = manifest()
            .execute(
                &root,
                &PluginRequest {
                    protocol_version: PLUGIN_PROTOCOL_VERSION,
                    plugin_id: "cookbook".into(),
                    operation: PluginOperation::ProvideResult,
                    payload: json!({"dish": "stew"}),
                },
                PluginExecutionLimits::default(),
            )
            .expect("fixture plugin should respond");
        assert_eq!(response.plugin_id, "cookbook");
        assert_eq!(response.results[0].id, "stew");
    }

    #[test]
    fn plugin_timeout_is_reported_without_waiting_indefinitely() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures");
        let mut timeout_manifest = manifest();
        timeout_manifest.entrypoint = "plugin_sleep.py".into();
        let error = timeout_manifest
            .execute(
                &root,
                &PluginRequest {
                    protocol_version: PLUGIN_PROTOCOL_VERSION,
                    plugin_id: "cookbook".into(),
                    operation: PluginOperation::ProvideResult,
                    payload: Value::Null,
                },
                PluginExecutionLimits {
                    timeout: Duration::from_millis(50),
                    ..PluginExecutionLimits::default()
                },
            )
            .expect_err("sleeping plugin should time out");
        assert!(matches!(error, PluginExecutionError::TimedOut(_)));
    }
}
