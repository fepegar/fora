//! Fixture-based parser tests for the artifact response models.

use mlflow::models::{ArtifactContentInfo, ArtifactEntry, ListArtifactsResponse};

#[test]
fn parses_mlflow_list_root_fixture() {
    let raw = include_str!("fixtures/mlflow_list_root.json");
    let parsed: ListArtifactsResponse = serde_json::from_str(raw).expect("root listing parses");

    assert!(parsed.root_uri.contains("azureml://"));
    assert_eq!(parsed.files.len(), 4);
    let paths: Vec<_> = parsed.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["logs", "outputs", "system_logs", "user_logs"]);

    for entry in &parsed.files {
        assert!(entry.is_dir, "{} should be a dir", entry.path);
        assert!(
            entry.file_size.is_none(),
            "{} reported size {:?} but Azure ML proxy returns -1 (None)",
            entry.path,
            entry.file_size,
        );
    }

    assert!(parsed.next_page_token.is_none());
}

#[test]
fn parses_mlflow_list_subdir_fixture() {
    let raw = include_str!("fixtures/mlflow_list_user_logs.json");
    let parsed: ListArtifactsResponse = serde_json::from_str(raw).expect("subdir listing parses");

    assert_eq!(parsed.files.len(), 1);
    let entry = &parsed.files[0];
    assert_eq!(entry.path, "user_logs/std_log.txt");
    assert!(!entry.is_dir);
    assert!(entry.file_size.is_none());
}

#[test]
fn parses_contentinfo_fixture() {
    let raw = include_str!("fixtures/runhistory_contentinfo.json");
    let parsed: ArtifactContentInfo = serde_json::from_str(raw).expect("contentinfo parses");

    assert!(parsed
        .content_uri
        .contains("teststorage.blob.core.windows.net"));
    assert!(parsed.content_uri.contains("sig=REDACTED"));
    assert_eq!(parsed.origin, "ExperimentRun");
    assert_eq!(parsed.container, "dcid.test_run_id");
    assert_eq!(parsed.path, "user_logs/std_log.txt");
    assert!(parsed.content_length.is_none());
}

#[test]
fn handles_numeric_file_size_strings() {
    // The Azure ML proxy returns sizes as strings; verify the deserialiser
    // handles both positive sizes and the sentinel "-1".
    let raw = r#"{
        "root_uri": "",
        "files": [
            { "path": "a.bin", "is_dir": false, "file_size": "1024" },
            { "path": "b.bin", "is_dir": false, "file_size": "-1" },
            { "path": "c.bin", "is_dir": false, "file_size": 4096 }
        ]
    }"#;
    let parsed: ListArtifactsResponse = serde_json::from_str(raw).expect("variant sizes parse");
    assert_eq!(parsed.files[0].file_size, Some(1024));
    assert_eq!(parsed.files[1].file_size, None, "-1 normalises to None");
    assert_eq!(parsed.files[2].file_size, Some(4096));
}

#[test]
fn artifact_entry_defaults_to_file() {
    // Missing is_dir should default to false (treat as a file).
    let raw = r#"{ "path": "loose" }"#;
    let parsed: ArtifactEntry = serde_json::from_str(raw).expect("partial entry parses");
    assert!(!parsed.is_dir);
    assert!(parsed.file_size.is_none());
}

#[test]
fn rejects_non_numeric_file_size_string() {
    // A non-numeric file_size is unexpected and must fail fast rather than
    // being silently treated as "no size".
    let raw = r#"{ "path": "x.bin", "is_dir": false, "file_size": "not-a-number" }"#;
    let parsed: Result<ArtifactEntry, _> = serde_json::from_str(raw);
    assert!(parsed.is_err(), "non-numeric file_size should be rejected");
}

#[test]
fn rejects_unexpected_negative_file_size() {
    // Only -1 is the documented unknown-size sentinel; other negatives must
    // fail fast (numeric and string forms).
    assert!(serde_json::from_str::<ArtifactEntry>(
        r#"{ "path": "x", "is_dir": false, "file_size": -5 }"#
    )
    .is_err());
    assert!(serde_json::from_str::<ArtifactEntry>(
        r#"{ "path": "x", "is_dir": false, "file_size": "-5" }"#
    )
    .is_err());
}
