//! Artifact metadata and content access for Azure ML runs.
//!
//! Azure ML's MLflow proxy implements `artifacts/list` (see
//! [`crate::MlflowClient::list_artifacts`]) but **not** `get-artifact`, so
//! content downloads must go through Azure ML's Run History
//! `artifact/v2.0/.../contentinfo/{origin}/{container}/{path}` endpoint instead.
//! That endpoint returns a short-lived SAS URL pointing at Azure Blob
//! Storage; the caller then talks to blob storage directly to get bytes,
//! ETag / `If-None-Match` conditional reads, and `Range`-based suffix
//! fetches needed for tailing growing log files.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use azure_core::credentials::TokenCredential;
use bytes::Bytes;
use reqwest::header::{ETAG, IF_NONE_MATCH, LAST_MODIFIED, RANGE};
use reqwest::{Client, StatusCode};

use crate::models::ArtifactContentInfo;

const ML_TOKEN_SCOPE: &str = "https://ml.azure.com/.default";

/// Client for Azure ML's artifact (run-output) service.
///
/// Wraps both the Run History `contentinfo` REST endpoint (which mints a
/// short-lived SAS URL for a single blob) and direct blob-storage GET/HEAD
/// against those SAS URLs. The two surfaces share authentication only for
/// the `contentinfo` call; blob downloads use the SAS signature and need
/// no bearer token.
#[derive(Clone)]
pub struct ArtifactsClient {
    http: Client,
    credential: Arc<dyn TokenCredential>,
    base_url: String,
}

impl ArtifactsClient {
    /// Build a client against the workspace's `artifact/v2.0` service.
    pub fn new(
        credential: Arc<dyn TokenCredential>,
        region: &str,
        subscription_id: &str,
        resource_group: &str,
        workspace_name: &str,
    ) -> Self {
        let base_url = format!(
            "https://{region}.api.azureml.ms/artifact/v2.0/subscriptions/{sub}/resourceGroups/{rg}/providers/Microsoft.MachineLearningServices/workspaces/{ws}/artifacts",
            region = region,
            sub = subscription_id,
            rg = resource_group,
            ws = workspace_name,
        );

        Self {
            http: Client::new(),
            credential,
            base_url,
        }
    }

    async fn get_token(&self) -> Result<String> {
        let token = self
            .credential
            .get_token(&[ML_TOKEN_SCOPE], None)
            .await
            .context("Failed to acquire Azure token for artifact service")?;
        Ok(token.token.secret().to_string())
    }

    /// Look up download metadata for a single artifact path within a run.
    ///
    /// Returns a `contentUri` SAS URL plus any metadata the service is
    /// willing to volunteer. The SAS URL is short-lived (~8 hours);
    /// callers may inspect the embedded `se=` parameter via
    /// [`sas_url_expiry`] to refresh proactively.
    pub async fn get_content_info(&self, run_id: &str, path: &str) -> Result<ArtifactContentInfo> {
        // This endpoint targets a single artifact file, so an empty path
        // (or just "/") is a caller error: fail fast with a clear message
        // rather than building a malformed `.../contentinfo/.../` URL.
        let path = path.trim_matches('/');
        if path.is_empty() {
            anyhow::bail!("get_content_info requires a non-empty artifact path");
        }
        // Azure ML's Run History contentinfo endpoint takes the artifact
        // path as a *URL segment*, not a query parameter:
        //
        //   /artifacts/contentinfo/{origin}/{container}/{path}
        //
        // Each segment in `path` must be percent-encoded so `/` stays a
        // separator while `+`, ` `, `?`, `#`, etc. inside a file name
        // survive routing intact.
        let encoded_path = path
            .split('/')
            .map(url_encode_segment)
            .collect::<Vec<_>>()
            .join("/");
        let url = format!(
            "{}/contentinfo/ExperimentRun/dcid.{}/{}",
            self.base_url, run_id, encoded_path,
        );

        // Log the run/path but not the constructed URL: avoid writing any
        // request URL (which can embed credentials elsewhere) to logs.
        tracing::debug!(run_id = %run_id, path = %path, "contentinfo GET");

        let token = self.get_token().await?;
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Run History contentinfo request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Run History contentinfo returned {}: {}", status, text);
        }

        resp.json::<ArtifactContentInfo>()
            .await
            .context("Failed to parse contentinfo response")
    }

    /// Issue a `HEAD` against a SAS URL.
    ///
    /// When `if_none_match` is `Some`, an unchanged blob will be reported
    /// via `BlobHeadResult::not_modified = true` (HTTP 304). Otherwise the
    /// returned struct carries the blob's `Content-Length`, `ETag` and
    /// `Last-Modified` headers, used for cache validation and tailing.
    pub async fn head_blob(
        &self,
        sas_url: &str,
        if_none_match: Option<&str>,
    ) -> Result<BlobHeadResult> {
        let mut req = self.http.head(sas_url);
        if let Some(etag) = if_none_match {
            req = req.header(IF_NONE_MATCH, etag);
        }

        let resp = req.send().await.context("Blob HEAD failed")?;
        let status = resp.status();

        if status == StatusCode::NOT_MODIFIED {
            return Ok(BlobHeadResult {
                not_modified: true,
                content_length: None,
                etag: None,
                last_modified: None,
            });
        }

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Blob HEAD returned {}: {}", status, text);
        }

        Ok(BlobHeadResult {
            not_modified: false,
            content_length: resp.content_length(),
            etag: header_string(resp.headers(), ETAG),
            last_modified: header_string(resp.headers(), LAST_MODIFIED),
        })
    }

    /// Download the full body of a blob via its SAS URL.
    pub async fn get_blob(&self, sas_url: &str) -> Result<BlobBytes> {
        let resp = self
            .http
            .get(sas_url)
            .send()
            .await
            .context("Blob GET failed")?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Blob GET returned {}: {}", status, text);
        }
        let etag = header_string(resp.headers(), ETAG);
        let last_modified = header_string(resp.headers(), LAST_MODIFIED);
        let bytes = resp.bytes().await.context("Reading blob body failed")?;
        Ok(BlobBytes {
            bytes,
            etag,
            last_modified,
        })
    }

    /// Download a suffix of a blob via the `Range` header.
    ///
    /// Used by the tail loop: once we have `cached_len` bytes, ask the
    /// server for `cached_len..` so we only fetch what's new. The returned
    /// `BlobBytes::bytes` contains only the suffix; the caller is
    /// responsible for stitching.
    pub async fn get_blob_range(&self, sas_url: &str, start: u64) -> Result<BlobBytes> {
        let range = format!("bytes={}-", start);
        let resp = self
            .http
            .get(sas_url)
            .header(RANGE, range)
            .send()
            .await
            .context("Blob Range GET failed")?;
        let status = resp.status();
        // 416 Range Not Satisfiable is normal when the requested range
        // starts at or past EOF (i.e. the file hasn't grown since the last
        // tail tick). Treat it as "no new bytes" rather than a hard error.
        if status == StatusCode::RANGE_NOT_SATISFIABLE {
            return Ok(BlobBytes {
                bytes: Bytes::new(),
                etag: header_string(resp.headers(), ETAG),
                last_modified: header_string(resp.headers(), LAST_MODIFIED),
            });
        }
        // A `200 OK` for a non-zero start means the server ignored the
        // `Range` header and sent the whole blob. Returning that as a
        // "suffix" would duplicate data in the stitching caller, so fail
        // fast instead.
        if start > 0 && status == StatusCode::OK {
            anyhow::bail!(
                "Blob Range GET ignored the Range header (200 OK for start={}); \
                refusing to avoid duplicating data",
                start
            );
        }
        if !(status.is_success() || status == StatusCode::PARTIAL_CONTENT) {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Blob Range GET returned {}: {}", status, text);
        }
        let etag = header_string(resp.headers(), ETAG);
        let last_modified = header_string(resp.headers(), LAST_MODIFIED);
        let bytes = resp
            .bytes()
            .await
            .context("Reading blob range body failed")?;
        Ok(BlobBytes {
            bytes,
            etag,
            last_modified,
        })
    }

    /// Download a bounded `[start, start + len)` slice of a blob.
    ///
    /// Unlike [`Self::get_blob_range`] (which is open-ended), this caps the
    /// response size, so callers can pull a large artifact to disk in
    /// fixed-size chunks without holding the whole blob in memory.
    pub async fn get_blob_chunk(&self, sas_url: &str, start: u64, len: u64) -> Result<BlobBytes> {
        // A zero-length request would build an inverted range (e.g.
        // `bytes=10-9`); there is nothing to fetch, so return early.
        if len == 0 {
            return Ok(BlobBytes {
                bytes: Bytes::new(),
                etag: None,
                last_modified: None,
            });
        }
        let end = start.saturating_add(len).saturating_sub(1);
        let range = format!("bytes={}-{}", start, end);
        let resp = self
            .http
            .get(sas_url)
            .header(RANGE, range)
            .send()
            .await
            .context("Blob chunk GET failed")?;
        let status = resp.status();
        if status == StatusCode::RANGE_NOT_SATISFIABLE {
            return Ok(BlobBytes {
                bytes: Bytes::new(),
                etag: header_string(resp.headers(), ETAG),
                last_modified: header_string(resp.headers(), LAST_MODIFIED),
            });
        }
        // A bounded read must come back as `206 Partial Content`. A
        // `200 OK` means the server ignored the `Range` header and would
        // hand us the whole blob, defeating the memory bound, so refuse it.
        if status == StatusCode::OK {
            anyhow::bail!(
                "Blob chunk GET returned 200 OK (Range ignored); \
                refusing to buffer the whole blob"
            );
        }
        if status != StatusCode::PARTIAL_CONTENT {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Blob chunk GET returned {}: {}", status, text);
        }
        let etag = header_string(resp.headers(), ETAG);
        let last_modified = header_string(resp.headers(), LAST_MODIFIED);
        let bytes = resp
            .bytes()
            .await
            .context("Reading blob chunk body failed")?;
        Ok(BlobBytes {
            bytes,
            etag,
            last_modified,
        })
    }
}

/// Result of a HEAD request against a blob SAS URL.
#[derive(Debug, Clone)]
pub struct BlobHeadResult {
    /// True when the server returned 304 in response to `If-None-Match`.
    pub not_modified: bool,
    pub content_length: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// Body and validators returned by a blob GET / Range GET.
#[derive(Debug, Clone)]
pub struct BlobBytes {
    pub bytes: Bytes,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

fn header_string(
    headers: &reqwest::header::HeaderMap,
    name: reqwest::header::HeaderName,
) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Parse the `se=<ISO-8601>` SAS expiry parameter out of a contentinfo URL.
///
/// Returns `None` if the URL has no `se=` parameter or it can't be parsed.
/// Used by the tail loop to refresh the SAS URL a few minutes before it
/// expires, rather than waiting for a 403.
pub fn sas_url_expiry(sas_url: &str) -> Option<SystemTime> {
    let query = sas_url.split_once('?').map(|(_, q)| q)?;
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("se=") {
            let decoded = percent_decode(value);
            return parse_iso8601(&decoded);
        }
    }
    None
}

fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                out.push(((h << 4) | l) as char);
                i += 3;
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Percent-encode every byte of a single URL path segment.
///
/// Conservatively encodes anything outside the RFC 3986 "unreserved" set
/// (`A-Z a-z 0-9 - _ . ~`); in particular `+`, ` `, `?`, `#`, `&`, `:`,
/// `/`, `@`, `=` are all escaped. We only call this on individual segments
/// (the path is split on `/` first) so legitimate separators survive.
pub(crate) fn url_encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let is_unreserved =
            b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~';
        if is_unreserved {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(HEX[(b >> 4) as usize]);
            out.push(HEX[(b & 0xf) as usize]);
        }
    }
    out
}

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

/// Minimal ISO-8601 parser for SAS `se=YYYY-MM-DDTHH:MM:SSZ` values.
///
/// Avoids pulling in a date crate; the input shape is fully controlled by
/// Azure Storage and is always the basic UTC form when delivered via SAS.
fn parse_iso8601(s: &str) -> Option<SystemTime> {
    let s = s.trim_end_matches('Z');
    let (date, time) = s.split_once('T')?;
    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: u32 = date_parts.next()?.parse().ok()?;
    let day: u32 = date_parts.next()?.parse().ok()?;
    let mut time_parts = time.split(':');
    let hour: u32 = time_parts.next()?.parse().ok()?;
    let minute: u32 = time_parts.next()?.parse().ok()?;
    let second: u32 = time_parts.next()?.parse().ok()?;

    let secs = ymdhms_to_unix(year, month, day, hour, minute, second)?;
    Some(UNIX_EPOCH + Duration::from_secs(secs))
}

fn ymdhms_to_unix(
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<u64> {
    if !(1..=12).contains(&month) || day == 0 || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    if day > days_in_month(year, month) {
        return None;
    }
    let days = days_from_epoch(year, month, day)?;
    let total = days * 86_400 + (hour as i64) * 3_600 + (minute as i64) * 60 + second as i64;
    if total < 0 {
        return None;
    }
    Some(total as u64)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn days_from_epoch(year: i64, month: u32, day: u32) -> Option<i64> {
    // Algorithm: Howard Hinnant, "civil_from_days" inverted.
    // Converts Y-M-D (proleptic Gregorian) to days since 1970-01-01.
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let m = month as i64;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sas_expiry() {
        let url =
            "https://x.blob.core.windows.net/c/p?sv=2019-07-07&se=2026-06-01T07%3A10%3A06Z&sp=r";
        let parsed = sas_url_expiry(url).expect("expected an expiry");
        // 2026-06-01T07:10:06Z → 1_780_297_806 epoch seconds.
        let expected = UNIX_EPOCH + Duration::from_secs(1_780_297_806);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parses_sas_expiry_handles_decoded_colons() {
        // Some SAS URLs come with un-encoded colons in the timestamp.
        let url = "https://x/?se=2030-01-02T03:04:05Z";
        let parsed = sas_url_expiry(url).expect("expected an expiry");
        let epoch = parsed.duration_since(UNIX_EPOCH).unwrap().as_secs();
        // 2030-01-02T03:04:05Z → 1_893_553_445 epoch seconds.
        assert_eq!(epoch, 1_893_553_445);
    }

    #[test]
    fn rejects_impossible_calendar_dates() {
        // Feb 31 and Feb 29 on a non-leap year must not yield a bogus time.
        assert!(sas_url_expiry("https://x/?se=2026-02-31T00:00:00Z").is_none());
        assert!(sas_url_expiry("https://x/?se=2026-02-29T00:00:00Z").is_none());
        assert!(sas_url_expiry("https://x/?se=2026-04-31T00:00:00Z").is_none());
        // A valid leap day still parses.
        assert!(sas_url_expiry("https://x/?se=2028-02-29T00:00:00Z").is_some());
    }

    #[test]
    fn url_encode_segment_passes_alphanumerics() {
        assert_eq!(super::url_encode_segment("std_log.txt"), "std_log.txt");
    }

    #[test]
    fn url_encode_segment_escapes_specials() {
        assert_eq!(super::url_encode_segment("a b+c?d"), "a%20b%2Bc%3Fd");
    }

    #[test]
    fn returns_none_for_url_without_se() {
        assert!(sas_url_expiry("https://example.com/x").is_none());
    }

    #[test]
    fn deserializes_content_info_fixture() {
        let raw = include_str!("../tests/fixtures/runhistory_contentinfo.json");
        let info: ArtifactContentInfo = serde_json::from_str(raw).expect("should parse");
        assert!(info.content_uri.contains("blob.core.windows.net"));
        assert_eq!(info.origin, "ExperimentRun");
        assert_eq!(info.container, "dcid.test_run_id");
        assert_eq!(info.path, "user_logs/std_log.txt");
        assert!(info.content_length.is_none());
    }
}
