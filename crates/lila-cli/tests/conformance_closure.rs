use lila_cli::run_cli_capture;

#[test]
fn release_closure_cannot_select_subsets_existing_names_or_an_oracle() {
    for arguments in [
        vec!["--filter", "language/wasm/pass"],
        vec![
            "--suite-root",
            "crates/lila-test262/tests/fixtures/fake_test262/vendor/test262",
        ],
        vec!["--snapshot-name", "existing-green"],
        vec!["--resume"],
        vec!["--max-matrix-nodes", "1"],
        vec!["--matrix-node", "language"],
        vec!["--shard", "1/2"],
        vec!["--execution-backend", "spec-exec"],
        vec!["--case-harness", "none"],
        vec!["--timeout-ms", "1"],
        vec!["--threads", "2"],
        vec!["--worker-bin", "another-image"],
    ] {
        let mut command = vec!["test262", "close-release"];
        command.extend(arguments);
        let capture = run_cli_capture(command);
        assert_ne!(capture.exit_code, 0);
        assert!(String::from_utf8(capture.stderr)
            .unwrap()
            .contains("unsupported closure option"));
        assert!(
            capture.stdout.is_empty(),
            "invalid admission cannot produce a closure verdict"
        );
    }
}

#[test]
fn release_closure_rejects_missing_or_repeated_evidence_parent_before_execution() {
    for arguments in [
        vec!["--snapshot-dir"],
        vec!["--snapshot-dir", ""],
        vec!["--snapshot-dir", "--resume"],
        vec!["--snapshot-dir", "unused", "--snapshot-dir", "other"],
    ] {
        let mut command = vec!["test262", "close-release"];
        command.extend(arguments);
        let capture = run_cli_capture(command);
        assert_ne!(capture.exit_code, 0);
        assert!(String::from_utf8(capture.stderr)
            .unwrap()
            .contains("closure"));
        assert!(capture.stdout.is_empty());
    }
}
