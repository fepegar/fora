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
    #[serde(alias = "nextPageToken")]
    pub next_page_token: Option<String>,
}

// ── Metric types ───────────────────────────────────────────────────────

/// A single metric data point (key, value, timestamp, step).
/// Used by the `get-history` endpoint. Fields use flexible deserializers
/// because Azure ML's MLflow proxy returns numeric values as JSON strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    pub key: String,
    #[serde(default, deserialize_with = "deserialize_optional_f64")]
    pub value: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64")]
    pub timestamp: i64,
    #[serde(default, deserialize_with = "deserialize_optional_i64")]
    pub step: i64,
}

/// Metric summary as returned in `RunData` from `search_runs`.
/// Only the `key` is guaranteed; value may arrive as a string, NaN, etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunMetricSummary {
    pub key: String,
    // All other fields are intentionally ignored — serde skips unknown fields by default.
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMetricHistoryResponse {
    #[serde(default)]
    pub metrics: Vec<Metric>,
    #[serde(alias = "nextPageToken")]
    pub next_page_token: Option<String>,
}

// ── Flexible deserializers for string-or-number JSON values ────────────

fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct OptF64Visitor;
    impl<'de> de::Visitor<'de> for OptF64Visitor {
        type Value = Option<f64>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a number, numeric string, or null")
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Option<f64>, E> {
            Ok(Some(v))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<f64>, E> {
            Ok(Some(v as f64))
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<f64>, E> {
            Ok(Some(v as f64))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<f64>, E> {
            v.parse::<f64>().map(Some).map_err(de::Error::custom)
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<f64>, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<f64>, E> {
            Ok(None)
        }
    }
    deserializer.deserialize_any(OptF64Visitor)
}

fn deserialize_optional_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct OptI64Visitor;
    impl<'de> de::Visitor<'de> for OptI64Visitor {
        type Value = i64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a number, numeric string, or null")
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<i64, E> {
            Ok(v)
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<i64, E> {
            Ok(v as i64)
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<i64, E> {
            Ok(v as i64)
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<i64, E> {
            v.parse::<i64>().map_err(de::Error::custom)
        }
        fn visit_none<E: de::Error>(self) -> Result<i64, E> {
            Ok(0)
        }
        fn visit_unit<E: de::Error>(self) -> Result<i64, E> {
            Ok(0)
        }
    }
    deserializer.deserialize_any(OptI64Visitor)
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
    pub metrics: Vec<RunMetricSummary>,
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
    #[serde(alias = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetRunResponse {
    pub run: Run,
}

// ── Artifact types ─────────────────────────────────────────────────────

/// A single entry returned by `mlflow/artifacts/list`.
///
/// `file_size` is reported as a JSON string by the Azure ML MLflow proxy
/// (e.g. `"42"`, or `"-1"` when the size is unknown), so we deserialise it
/// via a flexible deserialiser and normalise to `Option<u64>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub path: String,
    #[serde(default)]
    pub is_dir: bool,
    #[serde(default, deserialize_with = "deserialize_optional_file_size")]
    pub file_size: Option<u64>,
}

/// Response of `mlflow/artifacts/list`.
///
/// Pagination is observed to be unused for typical run sizes but the field
/// is honoured if the server includes it (matching the MLflow REST spec).
#[derive(Debug, Clone, Deserialize)]
pub struct ListArtifactsResponse {
    #[serde(default)]
    pub root_uri: String,
    #[serde(default)]
    pub files: Vec<ArtifactEntry>,
    #[serde(default, alias = "nextPageToken")]
    pub next_page_token: Option<String>,
}

/// Response of `artifact/v2.0/.../contentinfo/{origin}/{container}/{path}`.
///
/// `content_uri` is a short-lived SAS URL pointing at Azure Blob Storage.
/// Callers should download the bytes (and call `HEAD` / use conditional
/// requests) directly against that URL rather than re-proxying via the
/// Run History service.
#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactContentInfo {
    #[serde(rename = "contentUri")]
    pub content_uri: String,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub container: String,
    #[serde(default)]
    pub path: String,
    #[serde(default, rename = "contentLength")]
    pub content_length: Option<u64>,
}

fn deserialize_optional_file_size<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct V;
    impl<'de> de::Visitor<'de> for V {
        type Value = Option<u64>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a file size as integer, integer string, or null")
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Option<u64>, E> {
            match v {
                -1 => Ok(None),
                n if n < 0 => Err(de::Error::custom(format!(
                    "unexpected negative file size {} (only -1 is the unknown-size sentinel)",
                    n
                ))),
                n => Ok(Some(n as u64)),
            }
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Option<u64>, E> {
            Ok(Some(v))
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Option<u64>, E> {
            match v.parse::<i64>() {
                Ok(-1) => Ok(None),
                Ok(n) if n < 0 => Err(de::Error::custom(format!(
                    "unexpected negative file size {} (only -1 is the unknown-size sentinel)",
                    n
                ))),
                Ok(n) => Ok(Some(n as u64)),
                // Sizes above i64::MAX are still valid u64 byte counts, so
                // fall back to u64 before rejecting as non-numeric.
                Err(_) => match v.parse::<u64>() {
                    Ok(n) => Ok(Some(n)),
                    Err(_) => Err(de::Error::custom(format!(
                        "invalid file size: expected an integer or integer string, got {:?}",
                        v
                    ))),
                },
            }
        }
        fn visit_none<E: de::Error>(self) -> Result<Option<u64>, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Option<u64>, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(V)
}
