use serde::{Deserialize, Serialize};

// ── Key-Value ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

// ── Experiment types ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Experiment {
    pub experiment_id: String,
    pub name: String,
    #[serde(default)]
    pub lifecycle_stage: String,
    #[serde(default)]
    pub creation_time: Option<String>,
    #[serde(default)]
    pub last_update_time: Option<String>,
    #[serde(default)]
    pub tags: Vec<KeyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchExperimentsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchExperimentsResponse {
    #[serde(default)]
    pub experiments: Vec<Experiment>,
    pub next_page_token: Option<String>,
}

// ── Run types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunInfo {
    #[serde(default)]
    pub run_id: String,
    #[serde(default)]
    pub run_name: Option<String>,
    #[serde(default)]
    pub run_uuid: Option<String>,
    #[serde(default)]
    pub experiment_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub lifecycle_stage: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RunData {
    #[serde(default)]
    pub tags: Vec<KeyValue>,
    #[serde(default)]
    pub metrics: Vec<serde_json::Value>,
    #[serde(default)]
    pub params: Vec<KeyValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RunInputs {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub info: RunInfo,
    #[serde(default)]
    pub data: RunData,
    #[serde(default)]
    pub inputs: RunInputs,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchRunsRequest {
    pub experiment_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchRunsResponse {
    #[serde(default)]
    pub runs: Vec<Run>,
    pub next_page_token: Option<String>,
}
