//! Async fetchers for the Files sub-tab.
//!
//! Three tasks are spawned, all sharing a per-`FilesView`
//! [`CancellationToken`]:
//!
//! 1. **List fetcher** — calls `mlflow::list_artifacts` once for a given
//!    `(run_id, path)` and dispatches `RunArtifactListLoaded`/`Failed`.
//! 2. **Content fetcher** — calls `get_content_info` to mint a SAS URL,
//!    then `get_blob` to pull the bytes. Dispatches
//!    `RunArtifactContentLoaded`. SAS URLs never leave this module.
//! 3. **Tail loop** — periodic `head_blob` with `If-None-Match` against
//!    the cached ETag; on growth, fetches the suffix via `get_blob_range`
//!    and dispatches an `is_suffix=true` action. Cancels on the shared
//!    token *or* the per-tail token (whichever fires first).

use std::time::Duration;

use anyhow::Context;
use mlflow::ArtifactsClient;
use tokio_util::sync::CancellationToken;

use crate::app::Action;
use crate::client::AzureClient;
use crate::tabs::ActionSender;

const MAX_PREVIEW_BYTES: u64 = 10 * 1024 * 1024;

/// Spawn a one-shot directory listing fetch.
pub fn spawn_list_fetcher(
    client: AzureClient,
    run_id: String,
    path: String,
    cancel: CancellationToken,
    action_tx: ActionSender,
) {
    tokio::spawn(async move {
        let mlflow = client.mlflow().clone();
        tokio::select! {
            _ = cancel.cancelled() => {}
            result = mlflow.list_all_artifacts(&run_id, &path) => {
                match result {
                    Ok(entries) => {
                        let _ = action_tx.send(Action::RunArtifactListLoaded {
                            run_id,
                            path,
                            entries,
                        });
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::RunArtifactListFailed {
                            run_id,
                            path,
                            error: format!("{:#}", e),
                        });
                    }
                }
            }
        }
    });
}

/// Spawn a one-shot file-content download.
pub fn spawn_content_fetcher(
    client: AzureClient,
    run_id: String,
    path: String,
    cancel: CancellationToken,
    action_tx: ActionSender,
) {
    tracing::debug!(run_id = %run_id, path = %path, "spawn_content_fetcher: scheduling");
    tokio::spawn(async move {
        let artifacts = client.artifacts().clone();
        let outcome = tokio::select! {
            _ = cancel.cancelled() => {
                tracing::debug!(run_id = %run_id, path = %path, "content fetch cancelled");
                return;
            }
            v = fetch_one(&artifacts, &run_id, &path) => v,
        };
        match outcome {
            Ok((bytes, etag, last_modified)) => {
                tracing::debug!(
                    run_id = %run_id,
                    path = %path,
                    len = bytes.len(),
                    "content fetch ok",
                );
                let _ = action_tx.send(Action::RunArtifactContentLoaded {
                    run_id,
                    path,
                    bytes,
                    etag,
                    last_modified,
                    is_suffix: false,
                });
            }
            Err(e) => {
                tracing::warn!(
                    run_id = %run_id,
                    path = %path,
                    error = %e,
                    "content fetch failed",
                );
                let _ = action_tx.send(Action::RunArtifactContentFailed {
                    run_id,
                    path,
                    error: format!("{:#}", e),
                });
            }
        }
    });
}

async fn fetch_one(
    artifacts: &ArtifactsClient,
    run_id: &str,
    path: &str,
) -> anyhow::Result<(bytes::Bytes, Option<String>, Option<String>)> {
    let info = artifacts.get_content_info(run_id, path).await?;

    // Mandatory size guard: HEAD the blob before pulling the body so an
    // oversized (or unknown-size) artifact can't be streamed into memory
    // for the preview.
    let head = artifacts
        .head_blob(&info.content_uri, None)
        .await
        .context("HEAD request for preview size check failed")?;
    match head.content_length {
        Some(len) if len > MAX_PREVIEW_BYTES => {
            anyhow::bail!(
                "File is {} bytes ({} MiB), which exceeds the {} MiB preview cap.",
                len,
                len / (1024 * 1024),
                MAX_PREVIEW_BYTES / (1024 * 1024),
            );
        }
        Some(_) => {}
        None => {
            anyhow::bail!(
                "Could not determine the file size (no Content-Length); \
                refusing to load it into the preview."
            );
        }
    }

    let body = artifacts.get_blob(&info.content_uri).await?;
    Ok((body.bytes, body.etag, body.last_modified))
}

/// Periodically HEADs the cached blob using `If-None-Match`. On growth,
/// downloads the new suffix via `Range`. Re-mints the SAS URL on 403 or
/// near-expiry. Cancels on either the outer token or its own per-tail
/// token.
pub fn spawn_tail_loop(
    client: AzureClient,
    run_id: String,
    path: String,
    outer_cancel: CancellationToken,
    tail_cancel: CancellationToken,
    interval: Duration,
    action_tx: ActionSender,
) {
    tokio::spawn(async move {
        let artifacts = client.artifacts().clone();
        let mut etag: Option<String> = None;
        let mut cached_len: u64 = 0;
        let mut sas: Option<String> = None;
        let mut consecutive_unchanged: u32 = 0;
        let mut current_interval = interval;
        let max_interval = Duration::from_secs(30);

        // First, refresh the SAS URL so we have something to HEAD.
        loop {
            tokio::select! {
                _ = outer_cancel.cancelled() => return,
                _ = tail_cancel.cancelled() => return,
                _ = tokio::time::sleep(current_interval) => {}
            }

            if sas.is_none() {
                match artifacts.get_content_info(&run_id, &path).await {
                    Ok(info) => sas = Some(info.content_uri),
                    Err(_) => {
                        // Transient failure — try again next tick.
                        continue;
                    }
                }
            }
            let url = sas.as_deref().unwrap();

            let head = match artifacts.head_blob(url, etag.as_deref()).await {
                Ok(h) => h,
                Err(_) => {
                    // SAS might have expired — drop it and retry.
                    sas = None;
                    continue;
                }
            };

            if head.not_modified {
                let _ = action_tx.send(Action::RunArtifactNotModified {
                    run_id: run_id.clone(),
                    path: path.clone(),
                });
                consecutive_unchanged = consecutive_unchanged.saturating_add(1);
                if consecutive_unchanged >= 4 {
                    current_interval = (current_interval * 2).min(max_interval);
                }
                continue;
            }

            consecutive_unchanged = 0;
            current_interval = interval;

            // Respect the same size guard as the initial preview load: an
            // unknown size can't be bounded and an oversized file isn't
            // previewed, so in both cases skip this tick rather than risk an
            // unbounded download (the preview already shows the error).
            let new_len = match head.content_length {
                Some(len) if len <= MAX_PREVIEW_BYTES => len,
                _ => continue,
            };
            if cached_len > 0 && new_len > cached_len {
                // Append only the new suffix (bounded, since new_len <= cap).
                match artifacts.get_blob_range(url, cached_len).await {
                    Ok(body) => {
                        let len = body.bytes.len() as u64;
                        let _ = action_tx.send(Action::RunArtifactContentLoaded {
                            run_id: run_id.clone(),
                            path: path.clone(),
                            bytes: body.bytes,
                            etag: body.etag.clone(),
                            last_modified: body.last_modified,
                            is_suffix: true,
                        });
                        cached_len += len;
                        etag = body.etag.or(head.etag);
                    }
                    Err(_) => continue,
                }
            } else {
                // First fetch or a shrink: full GET (bounded by the cap).
                match artifacts.get_blob(url).await {
                    Ok(body) => {
                        cached_len = body.bytes.len() as u64;
                        etag = body.etag.clone().or(head.etag);
                        let _ = action_tx.send(Action::RunArtifactContentLoaded {
                            run_id: run_id.clone(),
                            path: path.clone(),
                            bytes: body.bytes,
                            etag: body.etag,
                            last_modified: body.last_modified,
                            is_suffix: false,
                        });
                    }
                    Err(_) => continue,
                }
            }
        }
    });
}
