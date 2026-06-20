//! Download a highlighted artifact (file or directory) to a user-chosen
//! destination.
//!
//! Triggered by the `s` hotkey in the Files sub-tab. The destination is
//! entered through a prompt (pre-filled with `<save_dir>/<item-name>`),
//! an overwrite confirmation is shown when the destination already
//! exists, and the bytes are then pulled from the artifact service. A
//! directory download recursively lists the subtree and writes every
//! file under the destination root, preserving the relative structure.

use std::path::{Path, PathBuf};

use mlflow::{ArtifactsClient, MlflowClient};
use tokio_util::sync::CancellationToken;

use super::state::leaf_name;
use crate::app::Action;
use crate::client::AzureClient;
use crate::tabs::ActionSender;

/// Chunk size used when streaming a large artifact to disk, so memory
/// stays bounded regardless of the file size.
const DOWNLOAD_CHUNK_BYTES: u64 = 8 * 1024 * 1024;

/// Spawn the asynchronous download task.
///
/// `source_is_dir` selects between a single-file download (write
/// `dest` directly) and a recursive directory download (write every
/// file under the `dest` root). The task is tied to `cancel` so that
/// switching jobs / workspaces aborts an in-flight download, and it
/// reports completion through [`Action::Notice`] / [`Action::Error`].
#[allow(clippy::too_many_arguments)]
pub fn spawn_download(
    client: AzureClient,
    run_id: String,
    artifact_path: String,
    source_is_dir: bool,
    item_name: String,
    dest: PathBuf,
    cancel: CancellationToken,
    action_tx: ActionSender,
) {
    tokio::spawn(async move {
        let _ = action_tx.send(Action::Notice(format!("Downloading {}…", item_name)));
        let result = tokio::select! {
            _ = cancel.cancelled() => {
                let _ = action_tx.send(Action::Notice(format!("Download of {} cancelled", item_name)));
                return;
            }
            r = run_download(&client, &run_id, &artifact_path, source_is_dir, &dest) => r,
        };
        match result {
            Ok(count) => {
                let msg = if source_is_dir {
                    format!("Downloaded {} file(s) to {}", count, dest.display())
                } else {
                    format!("Downloaded {} to {}", item_name, dest.display())
                };
                let _ = action_tx.send(Action::Notice(msg));
            }
            Err(e) => {
                let _ = action_tx.send(Action::Error(format!("Download failed: {:#}", e)));
            }
        }
    });
}

/// Run the download, returning the number of files written.
async fn run_download(
    client: &AzureClient,
    run_id: &str,
    artifact_path: &str,
    source_is_dir: bool,
    dest: &Path,
) -> anyhow::Result<usize> {
    if !source_is_dir {
        download_one_file(client.artifacts(), run_id, artifact_path, dest).await?;
        return Ok(1);
    }

    // We only reach here for a fresh destination or after the user
    // confirmed an overwrite, so clear any existing destination (file or
    // directory) before writing the subtree. This avoids leaving stale
    // files behind and handles the case where `dest` currently exists as
    // a plain file.
    remove_existing(dest).await?;
    // Ensure the destination exists even if the artifact subtree turns out
    // to be empty, so an empty directory download still produces the
    // folder the user asked for.
    tokio::fs::create_dir_all(dest).await?;

    let files = collect_files(client.mlflow(), run_id, artifact_path).await?;
    let targets = plan_download_targets(artifact_path, &files, dest)?;
    let count = targets.len();
    for (path, on_disk) in targets {
        download_one_file(client.artifacts(), run_id, &path, &on_disk).await?;
    }
    Ok(count)
}

/// Remove an existing file or directory at `path`, treating a missing
/// path as success.
async fn remove_existing(path: &Path) -> anyhow::Result<()> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(meta) => {
            if meta.is_dir() {
                tokio::fs::remove_dir_all(path).await?;
            } else {
                tokio::fs::remove_file(path).await?;
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Download a single artifact file to `dest`, creating any missing parent
/// directories. Files of unknown or large size are streamed to disk in
/// bounded chunks so memory stays flat even for big checkpoints; only a
/// known, small size uses a single GET.
///
/// The bytes are written to a temporary sibling and only moved into place
/// once the transfer succeeds, so a cancel/failure mid-stream can't destroy
/// an existing file or leave a partial one at `dest`.
async fn download_one_file(
    artifacts: &ArtifactsClient,
    run_id: &str,
    artifact_path: &str,
    dest: &Path,
) -> anyhow::Result<()> {
    use tokio::io::AsyncWriteExt;

    let info = artifacts.get_content_info(run_id, artifact_path).await?;
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let tmp_path = temp_sibling(dest);
    let mut guard = TempFileGuard::new(tmp_path.clone());

    let size = artifacts
        .head_blob(&info.content_uri, None)
        .await
        .ok()
        .and_then(|h| h.content_length);

    let mut file = tokio::fs::File::create(&tmp_path).await?;
    match size {
        Some(total) if total <= DOWNLOAD_CHUNK_BYTES => {
            // Known small size: a single GET keeps it simple.
            let body = artifacts.get_blob(&info.content_uri).await?;
            file.write_all(&body.bytes).await?;
        }
        _ => {
            // Large or unknown size (HEAD failed or no Content-Length):
            // stream in bounded chunks so a big artifact can't OOM. A
            // short read (or empty chunk past EOF) ends the loop.
            let mut offset = 0u64;
            loop {
                let chunk = artifacts
                    .get_blob_chunk(&info.content_uri, offset, DOWNLOAD_CHUNK_BYTES)
                    .await?;
                let n = chunk.bytes.len() as u64;
                if n == 0 {
                    break;
                }
                file.write_all(&chunk.bytes).await?;
                offset += n;
                match size {
                    Some(total) if offset >= total => break,
                    None if n < DOWNLOAD_CHUNK_BYTES => break,
                    _ => {}
                }
            }
        }
    }
    file.flush().await?;
    drop(file);

    // Replace `dest`: unlink any existing entry first (without following
    // symlinks) so a symlink can't redirect the write, then rename the temp
    // into place. The rename is atomic on the same filesystem.
    remove_existing(dest).await?;
    tokio::fs::rename(&tmp_path, dest).await?;
    guard.commit();
    Ok(())
}

/// Build a temporary sibling path next to `dest` (same directory, so the
/// final rename stays on one filesystem and is atomic).
fn temp_sibling(dest: &Path) -> PathBuf {
    let parent = dest.parent().unwrap_or_else(|| Path::new("."));
    let name = dest
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    parent.join(format!(
        ".{}.fora-part-{}-{}",
        name,
        std::process::id(),
        nanos
    ))
}

/// Removes a temp file on drop unless committed (i.e. successfully renamed
/// into place), cleaning up partial downloads on error or cancellation.
struct TempFileGuard {
    path: Option<PathBuf>,
}

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    fn commit(&mut self) {
        self.path = None;
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Recursively gather every file artifact path under `root` (the root is
/// `""`). Uses an explicit stack to avoid boxed async recursion.
async fn collect_files(
    mlflow: &MlflowClient,
    run_id: &str,
    root: &str,
) -> anyhow::Result<Vec<String>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_string()];
    while let Some(dir) = stack.pop() {
        let entries = mlflow.list_all_artifacts(run_id, &dir).await?;
        for entry in entries {
            if entry.is_dir {
                stack.push(entry.path);
            } else {
                files.push(entry.path);
            }
        }
    }
    Ok(files)
}

/// Build the prompt's pre-filled destination string: the configured
/// `save_dir` with the artifact's leaf name appended. `~` is left
/// unexpanded so the user sees a friendly, editable path. The join
/// separator follows the configured string (backslash for Windows-style
/// `save_dir`, otherwise `/`).
pub fn default_destination(save_dir_spec: &str, item_name: &str) -> String {
    let uses_backslash = save_dir_spec.contains('\\') && !save_dir_spec.contains('/');
    let sep = if uses_backslash { '\\' } else { '/' };
    let base = save_dir_spec.trim_end_matches(['/', '\\']);
    if base.is_empty() {
        if save_dir_spec.starts_with('/') || save_dir_spec.starts_with('\\') {
            format!("{}{}", sep, item_name)
        } else {
            item_name.to_string()
        }
    } else {
        format!("{}{}{}", base, sep, item_name)
    }
}

/// Expand `~` and turn `spec` into an absolute path (relative paths
/// resolve against the current working directory).
pub fn resolve_path(spec: &str) -> anyhow::Result<PathBuf> {
    let expanded = shellexpand::tilde(spec).into_owned();
    let p = PathBuf::from(expanded);
    if p.is_absolute() {
        Ok(p)
    } else {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(p))
    }
}

/// Resolve the on-disk destination the bytes will actually be written
/// to. For a directory source the resolved path is the destination root
/// verbatim. For a file source, when the resolved path is an existing
/// directory the leaf name is appended (mirroring `cp file dir/`);
/// otherwise the resolved path is used as the file path.
pub fn final_destination(resolved: &Path, source_is_dir: bool, item_name: &str) -> PathBuf {
    if source_is_dir {
        resolved.to_path_buf()
    } else if resolved.is_dir() {
        resolved.join(item_name)
    } else {
        resolved.to_path_buf()
    }
}

/// True when something already exists at `dest` (file or directory).
pub fn destination_exists(dest: &Path) -> bool {
    dest.exists()
}

/// Map each file artifact path under `selected_dir` to an on-disk path
/// rooted at `dest_root`, preserving the relative structure. Rejects any
/// path component that would escape the destination root.
pub fn plan_download_targets(
    selected_dir: &str,
    file_paths: &[String],
    dest_root: &Path,
) -> anyhow::Result<Vec<(String, PathBuf)>> {
    let trimmed = selected_dir.trim_end_matches('/');
    let prefix = if trimmed.is_empty() {
        String::new()
    } else {
        format!("{}/", trimmed)
    };

    let mut out = Vec::with_capacity(file_paths.len());
    for fp in file_paths {
        let rel = if prefix.is_empty() {
            fp.as_str()
        } else if let Some(r) = fp.strip_prefix(&prefix) {
            r
        } else {
            anyhow::bail!("artifact '{}' is not under '{}'", fp, selected_dir);
        };

        let mut path = dest_root.to_path_buf();
        for comp in rel.split('/') {
            if comp.is_empty()
                || comp == "."
                || comp == ".."
                || comp.contains('\\')
                || comp.contains('\0')
                || comp.contains(std::path::MAIN_SEPARATOR)
                // On Windows a `:` introduces a drive/prefix (e.g. `C:`),
                // which `PathBuf::push` would treat as a new root and so
                // escape `dest_root`.
                || (cfg!(windows) && comp.contains(':'))
            {
                anyhow::bail!("refusing unsafe path component in '{}'", fp);
            }
            path.push(comp);
        }
        out.push((fp.clone(), path));
    }
    Ok(out)
}

/// Return the leaf (file/dir) name for an artifact path, or an error if
/// the result would be empty or still contain a path separator.
pub fn sanitised_leaf(artifact_path: &str) -> anyhow::Result<String> {
    let leaf = leaf_name(artifact_path);
    if leaf.is_empty() || leaf == "." || leaf == ".." {
        anyhow::bail!("artifact path '{}' has no usable file name", artifact_path);
    }
    if leaf.contains(std::path::MAIN_SEPARATOR)
        || leaf.contains('/')
        || leaf.contains('\\')
        || leaf.contains('\0')
        // `:` is a drive/prefix separator on Windows and not a valid
        // file-name character there.
        || (cfg!(windows) && leaf.contains(':'))
    {
        anyhow::bail!("refusing to download: file name contains path separators");
    }
    Ok(leaf.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitised_leaf_rejects_empty_and_dot_dot() {
        assert!(sanitised_leaf("").is_err());
        assert!(sanitised_leaf("..").is_err());
        assert!(sanitised_leaf(".").is_err());
        assert!(sanitised_leaf("a/b/..").is_err());
    }

    #[test]
    fn sanitised_leaf_accepts_normal_paths() {
        assert_eq!(sanitised_leaf("foo.txt").unwrap(), "foo.txt");
        assert_eq!(
            sanitised_leaf("user_logs/std_log.txt").unwrap(),
            "std_log.txt"
        );
    }

    #[test]
    fn sanitised_leaf_rejects_embedded_separators_in_leaf() {
        assert!(sanitised_leaf(r"a/b\c").is_err());
        assert!(sanitised_leaf("a/x\0y").is_err());
    }

    #[test]
    fn default_destination_joins_save_dir_and_name() {
        assert_eq!(default_destination("./", "model.pkl"), "./model.pkl");
        assert_eq!(default_destination(".", "model.pkl"), "./model.pkl");
        assert_eq!(default_destination("~/dl", "out"), "~/dl/out");
        assert_eq!(default_destination("~/dl/", "out"), "~/dl/out");
        assert_eq!(default_destination("/tmp", "a.txt"), "/tmp/a.txt");
        assert_eq!(default_destination("/", "a.txt"), "/a.txt");
    }

    #[test]
    fn default_destination_uses_backslash_for_windows_style_dir() {
        assert_eq!(
            default_destination(r"C:\Users\me\Downloads\", "f.txt"),
            r"C:\Users\me\Downloads\f.txt"
        );
        assert_eq!(default_destination(r"C:\tmp", "f.txt"), r"C:\tmp\f.txt");
    }

    #[test]
    fn resolve_path_makes_relative_absolute() {
        let resolved = resolve_path("some/rel/path").unwrap();
        assert!(resolved.is_absolute());
        assert!(resolved.ends_with("some/rel/path"));
    }

    #[test]
    fn resolve_path_keeps_absolute() {
        let resolved = resolve_path("/etc/hosts").unwrap();
        assert_eq!(resolved, PathBuf::from("/etc/hosts"));
    }

    #[test]
    fn plan_download_targets_strips_dir_prefix() {
        let dest = PathBuf::from("/out/dir");
        let files = vec![
            "checkpoints/model.pt".to_string(),
            "checkpoints/sub/opt.pt".to_string(),
        ];
        let targets = plan_download_targets("checkpoints", &files, &dest).unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].1, PathBuf::from("/out/dir/model.pt"));
        assert_eq!(targets[1].1, PathBuf::from("/out/dir/sub/opt.pt"));
    }

    #[test]
    fn plan_download_targets_supports_root() {
        let dest = PathBuf::from("/out");
        let files = vec!["a.txt".to_string(), "logs/b.txt".to_string()];
        let targets = plan_download_targets("", &files, &dest).unwrap();
        assert_eq!(targets[0].1, PathBuf::from("/out/a.txt"));
        assert_eq!(targets[1].1, PathBuf::from("/out/logs/b.txt"));
    }

    #[test]
    fn plan_download_targets_rejects_out_of_tree() {
        let dest = PathBuf::from("/out");
        let files = vec!["other/a.txt".to_string()];
        assert!(plan_download_targets("checkpoints", &files, &dest).is_err());
    }

    #[test]
    fn plan_download_targets_rejects_separator_and_nul_components() {
        let dest = PathBuf::from("/out");
        assert!(plan_download_targets("d", &[r"d/a\b.txt".to_string()], &dest).is_err());
        assert!(plan_download_targets("d", &["d/a\0b.txt".to_string()], &dest).is_err());
    }

    #[test]
    fn final_destination_appends_name_for_file_into_existing_dir() {
        let tmp = tempdir_in_workspace();
        let dest = final_destination(&tmp, false, "model.pkl");
        assert_eq!(dest, tmp.join("model.pkl"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn final_destination_uses_path_for_file_when_not_a_dir() {
        let target = PathBuf::from("/out/renamed.pkl");
        let dest = final_destination(&target, false, "model.pkl");
        assert_eq!(dest, target);
    }

    #[test]
    fn final_destination_uses_root_for_directory_source() {
        let tmp = tempdir_in_workspace();
        let dest = final_destination(&tmp, true, "checkpoints");
        assert_eq!(dest, tmp);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn destination_exists_detects_files_and_dirs() {
        let tmp = tempdir_in_workspace();
        assert!(destination_exists(&tmp));
        let file = tmp.join("f.txt");
        assert!(!destination_exists(&file));
        std::fs::write(&file, b"x").unwrap();
        assert!(destination_exists(&file));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn tempdir_in_workspace() -> PathBuf {
        let mut p = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!("fora-download-tests-{}-{}", pid, nanos));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
