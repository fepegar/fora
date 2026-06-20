//! Sanitised, collision-safe artifact save.
//!
//! `s` writes the previewed file bytes to `[ui].save_dir` (default `./`),
//! with the destination name derived from the artifact's leaf name. We
//! refuse anything where the leaf still contains a separator (defence
//! against an adversarial artifact tree), and append ` (1)`, ` (2)`, …
//! before the extension when the destination already exists.

use std::path::{Path, PathBuf};

use bytes::Bytes;

use crate::app::Action;
use crate::tabs::ActionSender;

/// Spawn a blocking save task and dispatch a [`Action::Notice`] /
/// [`Action::Error`] when it finishes.
pub fn spawn_save(
    run_id: String,
    artifact_path: String,
    bytes: Bytes,
    save_dir: String,
    action_tx: ActionSender,
) {
    tokio::task::spawn_blocking(move || {
        let result = save_to_dir(&run_id, &artifact_path, &bytes, &save_dir);
        match result {
            Ok(path) => {
                let _ = action_tx.send(Action::Notice(format!("Saved to {}", path.display())));
            }
            Err(e) => {
                let _ = action_tx.send(Action::Error(format!("Failed to save artifact: {:#}", e)));
            }
        }
    });
}

fn save_to_dir(
    run_id: &str,
    artifact_path: &str,
    bytes: &[u8],
    save_dir_spec: &str,
) -> anyhow::Result<PathBuf> {
    let save_dir = resolve_save_dir(save_dir_spec)?;
    let leaf = sanitised_leaf(artifact_path)?;
    // Sanitise the run id too: it is interpolated into the file name, so a
    // stray separator could otherwise redirect the write outside save_dir.
    let filename = format!("{}-{}", sanitise_component(run_id), leaf);
    std::fs::create_dir_all(&save_dir)?;
    let dest = unique_path(&save_dir.join(filename));
    std::fs::write(&dest, bytes)?;
    Ok(dest)
}

/// Replace any path separator or NUL in `s` with `_` so the result is a
/// single, safe file-name component.
fn sanitise_component(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == '\0' || c == std::path::MAIN_SEPARATOR {
                '_'
            } else {
                c
            }
        })
        .collect()
}

/// Expand `~` and turn the configured save dir into an absolute path.
pub fn resolve_save_dir(spec: &str) -> anyhow::Result<PathBuf> {
    let expanded = shellexpand::tilde(spec).into_owned();
    let p = PathBuf::from(expanded);
    if p.is_absolute() {
        Ok(p)
    } else {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(p))
    }
}

/// Return the leaf name (no separators) for the artifact path, or an
/// error if the result would be empty or still contain a path separator.
pub fn sanitised_leaf(artifact_path: &str) -> anyhow::Result<String> {
    let leaf = artifact_path
        .rsplit('/')
        .next()
        .ok_or_else(|| anyhow::anyhow!("empty artifact path"))?;
    if leaf.is_empty() || leaf == "." || leaf == ".." {
        anyhow::bail!("artifact path '{}' has no usable file name", artifact_path);
    }
    if leaf.contains(std::path::MAIN_SEPARATOR)
        || leaf.contains('/')
        || leaf.contains('\\')
        || leaf.contains('\0')
    {
        anyhow::bail!("refusing to save: file name contains path separators");
    }
    Ok(leaf.to_string())
}

/// If `path` exists, return `path (1)`/`path (2)`/… until a free name is
/// found, preserving the extension. Mirrors how browsers handle download
/// collisions.
pub fn unique_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|s| s.to_str());
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    for i in 1..u32::MAX {
        let candidate_name = match ext {
            Some(e) => format!("{} ({}).{}", stem, i, e),
            None => format!("{} ({})", stem, i),
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }
    // 4 billion collisions later — give up.
    path.to_path_buf()
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
        // After rsplit('/') a backslash-bearing leaf could still smuggle.
        assert!(sanitised_leaf(r"a/b\c").is_err());
        assert!(sanitised_leaf("a/x\0y").is_err());
    }

    #[test]
    fn unique_path_appends_counter_when_target_exists() {
        let tmp = tempdir_in_workspace();
        let target = tmp.join("hello.txt");
        std::fs::write(&target, b"a").unwrap();
        let next = unique_path(&target);
        assert_eq!(next.file_name().unwrap().to_str().unwrap(), "hello (1).txt");
        std::fs::write(&next, b"b").unwrap();
        let nextnext = unique_path(&target);
        assert_eq!(
            nextnext.file_name().unwrap().to_str().unwrap(),
            "hello (2).txt",
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn tempdir_in_workspace() -> PathBuf {
        let mut p = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!("fora-files-tests-{}-{}", pid, nanos));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
