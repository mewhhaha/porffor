//! `Intl` CLI integration tests.

use crate::*;

fn assert_dumped_runtime_matches_sdk(dump: &Path, expected: &lila_engine::Artifact) {
    let runtime = expected
        .runtime
        .as_ref()
        .expect("Intl producer must emit a linked runtime");
    let mut runtime_path = dump.as_os_str().to_os_string();
    runtime_path.push(".runtime.wasm");
    let emitted = fs::read(&runtime_path).expect("CLI runtime sidecar must be emitted");
    assert!(
        emitted.as_slice() == runtime.bytes(),
        "CLI runtime sidecar {} differs from the SDK artifact",
        Path::new(&runtime_path).display()
    );
}

#[test]
fn service_manifest_v6_cli_build_export_and_run_share_the_checked_sparse_sdk_owner() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ServiceTree(std::path::PathBuf);
    impl Drop for ServiceTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ServiceTree(std::env::temp_dir().join(format!(
            "lila-intl-services-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&tree.0).unwrap();
    let json = r#"{"schema_version":6,"custom_id":"cli-relative-only","services":["RelativeTimeFormat"],"locale_filters":{"relative_time":["fr"]}}"#;
    let manifest = tree.0.join("profile.json");
    fs::write(&manifest, json).unwrap();
    let source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_services.js"
    );
    let script = tree.0.join("services.js");
    let dump = tree.0.join("services.wasm");
    let bundle = tree.0.join("services.bundle");
    fs::write(&script, source).unwrap();
    let direct = lila_engine::CustomIntlProfile::new(
        lila_engine::CustomProfileId::parse("cli-relative-only").unwrap(),
        None,
        Some(&["fr"]),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap()
    .with_services(&["RelativeTimeFormat"])
    .unwrap();
    assert_eq!(
        direct,
        lila_engine::CustomIntlProfile::from_manifest_json(json).unwrap()
    );
    let profile = lila_engine::IntlCompilationProfile::CustomProjection(direct);
    let selection = lila_engine::IntlDataSelection::new(profile.clone());
    let selected = selection.selected().unwrap();
    assert_eq!(selected.component_sections().len(), 4);
    let expected_bundle = selected.export_bytes().unwrap();
    assert_eq!(&expected_bundle[..8], b"LILAB002");
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: profile,
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected_wasm = engine.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["build", "wasm"])
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(fs::read(&dump).unwrap(), expected_wasm.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected_wasm);
    let exported = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--intl-manifest")
        .arg(&manifest)
        .args(["intl", "export", "--output"])
        .arg(&bundle)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    assert_eq!(
        fs::read(&bundle).unwrap().as_slice(),
        expected_bundle.as_ref()
    );
    let run = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["run", "--execution-backend", "wasm"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(String::from_utf8_lossy(&run.stdout).contains("intl-services-projection:ok"));
    fs::write(&script, "new Intl.NumberFormat('en-US').format(1);").unwrap();
    let unavailable = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["run", "--execution-backend", "wasm"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(!unavailable.status.success());
    assert!(String::from_utf8_lossy(&unavailable.stderr).contains("unavailable"));
    for bad in [
        r#"{"schema_version":6,"custom_id":"bad","services":[]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["NoService"]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["ListFormat","ListFormat"]}"#,
        r#"{"schema_version":6,"custom_id":"bad","services":["ListFormat"],"numbering_systems":["deva"]}"#,
    ] {
        fs::write(&manifest, bad).unwrap();
        let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["run", "--execution-backend", "wasm"])
            .arg(tree.0.join("missing-source.js"))
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            error.contains("invalid --intl-manifest") && !error.contains("missing-source.js"),
            "{error}"
        );
        let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["intl", "export", "--output"])
            .arg(&bundle)
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        assert_eq!(
            fs::read(&bundle).unwrap().as_slice(),
            expected_bundle.as_ref()
        );
    }
}

#[test]
fn named_zone_manifest_v5_build_export_and_unavailable_data_follow_actual_sdk_dependencies() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct NamedZoneTree(std::path::PathBuf);
    impl Drop for NamedZoneTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = NamedZoneTree(std::env::temp_dir().join(format!(
            "lila-intl-named-zones-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&tree.0).unwrap();
    let json = r#"{"schema_version":5,"custom_id":"cli-named-zones","named_time_zones":["us/eastern"],"numbering_systems":["latn"],"date_time_calendars":["chinese"],"currency_codes":["EUR"],"locale_filters":{"date_time":["fr"]}}"#;
    let manifest = tree.0.join("profile.json");
    fs::write(&manifest, json).unwrap();
    let source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_named_zones.js"
    );
    let script = tree.0.join("named-zones.js");
    let dump = tree.0.join("named-zones.wasm");
    let output = tree.0.join("named-zones.bundle");
    fs::write(&script, source).unwrap();
    let profile = lila_engine::IntlCompilationProfile::CustomProjection(
        lila_engine::CustomIntlProfile::from_manifest_json(json).unwrap(),
    );
    let selection = lila_engine::IntlDataSelection::new(profile.clone());
    let expected_export = selection.selected().unwrap().export_bytes().unwrap();
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: profile,
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected_wasm = engine.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["build", "wasm"])
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(fs::read(&dump).unwrap(), expected_wasm.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected_wasm);
    let exported = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--intl-manifest")
        .arg(&manifest)
        .args(["intl", "export", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    assert_eq!(
        fs::read(&output).unwrap().as_slice(),
        expected_export.as_ref()
    );
    let run = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["run", "--execution-backend", "wasm"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(String::from_utf8_lossy(&run.stdout).contains("intl-named-zone-projection:ok"));
    fs::write(
        &script,
        "new Intl.DateTimeFormat('en-US',{timeZone:'Europe/Paris'}).format(0);",
    )
    .unwrap();
    let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["run", "--execution-backend", "wasm"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    let error = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        error.contains("unavailable")
            && error.contains("Europe/Paris")
            && !error.contains("unknown previously resolved"),
        "{error}"
    );
    for zones in [
        "[]",
        "null",
        "[\"Europe/Unknown\"]",
        "[\"US/Eastern\",\"America/New_York\"]",
        "[\"+05:30\"]",
    ] {
        fs::write(&manifest, format!(r#"{{"schema_version":5,"custom_id":"invalid-named-zone","named_time_zones":{zones}}}"#)).unwrap();
        let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["run", "--execution-backend", "wasm"])
            .arg(tree.0.join("missing-source.js"))
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            error.contains("invalid --intl-manifest") && !error.contains("missing-source.js"),
            "{error}"
        );
    }
}

#[test]
fn numbering_manifest_v4_build_export_and_run_consume_the_same_paired_sdk_selection() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct NumberingTree(std::path::PathBuf);
    impl Drop for NumberingTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = NumberingTree(std::env::temp_dir().join(format!(
            "lila-intl-numbering-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&tree.0).unwrap();
    let json = r#"{"schema_version":4,"custom_id":"cli-numbering","numbering_systems":["DEVA"],"date_time_calendars":["chinese"],"currency_codes":["EUR"],"locale_filters":{"number_plural":["ar-EG","fr"],"date_time":["ar-EG","fr"]}}"#;
    let manifest = tree.0.join("profile.json");
    fs::write(&manifest, json).unwrap();
    let source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_numbering.js"
    );
    let script = tree.0.join("numbering.js");
    let dump = tree.0.join("numbering.wasm");
    let output = tree.0.join("numbering.bundle");
    fs::write(&script, source).unwrap();
    let profile = lila_engine::IntlCompilationProfile::CustomProjection(
        lila_engine::CustomIntlProfile::from_manifest_json(json).unwrap(),
    );
    let selection = lila_engine::IntlDataSelection::new(profile.clone());
    let expected_export = selection.selected().unwrap().export_bytes().unwrap();
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: profile,
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected_wasm = engine.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["build", "wasm"])
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(fs::read(&dump).unwrap(), expected_wasm.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected_wasm);
    let exported = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--intl-manifest")
        .arg(&manifest)
        .args(["intl", "export", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    assert_eq!(
        fs::read(&output).unwrap().as_slice(),
        expected_export.as_ref()
    );
    let run = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["run", "--execution-backend", "wasm"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(String::from_utf8_lossy(&run.stdout).contains("intl-numbering-projection:ok"));
    for systems in ["[]", "null", "[\"zzzz\"]", "[\"deva\",\"DEVA\"]", "[\"a\"]"] {
        fs::write(&manifest, format!(r#"{{"schema_version":4,"custom_id":"invalid-numbering","numbering_systems":{systems}}}"#)).unwrap();
        let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["run", "--execution-backend", "wasm"])
            .arg(tree.0.join("missing-source.js"))
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            error.contains("invalid --intl-manifest") && !error.contains("missing-source.js"),
            "{error}"
        );
    }
}

#[test]
fn calendar_manifest_v3_build_export_and_run_share_the_sdk_projected_service_graph() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct CalendarTree(std::path::PathBuf);
    impl Drop for CalendarTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = CalendarTree(std::env::temp_dir().join(format!(
            "lila-intl-calendar-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&tree.0).unwrap();
    let json = r#"{"schema_version":3,"custom_id":"cli-calendars","date_time_calendars":["chinese"],"currency_codes":["EUR","JPY"],"locale_filters":{"date_time":["fr"]}}"#;
    let manifest = tree.0.join("profile.json");
    fs::write(&manifest, json).unwrap();
    let source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_calendars.js"
    );
    let script = tree.0.join("calendar.js");
    let dump = tree.0.join("calendar.wasm");
    let bundle = tree.0.join("calendar.bundle");
    fs::write(&script, source).unwrap();
    let profile = lila_engine::IntlCompilationProfile::CustomProjection(
        lila_engine::CustomIntlProfile::from_manifest_json(json).unwrap(),
    );
    let selection = lila_engine::IntlDataSelection::new(profile.clone());
    let expected_bundle = selection.selected().unwrap().export_bytes().unwrap();
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: profile,
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected_wasm = engine.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["build", "wasm"])
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(fs::read(&dump).unwrap(), expected_wasm.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected_wasm);
    let exported = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--intl-manifest")
        .arg(&manifest)
        .args(["intl", "export", "--output"])
        .arg(&bundle)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    assert_eq!(
        fs::read(&bundle).unwrap().as_slice(),
        expected_bundle.as_ref()
    );
    let outcome = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["run", "--execution-backend", "wasm"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        outcome.status.success(),
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert!(String::from_utf8_lossy(&outcome.stdout).contains("intl-calendar-projection:ok"));
    for calendars in ["[]", "null", "[\"unknown\"]", "[\"chinese\",\"chinese\"]"] {
        fs::write(&manifest, format!(r#"{{"schema_version":3,"custom_id":"invalid-calendars","date_time_calendars":{calendars}}}"#)).unwrap();
        let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["run", "--execution-backend", "wasm"])
            .arg(tree.0.join("missing-source.js"))
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            error.contains("invalid --intl-manifest") && !error.contains("missing-source.js"),
            "{error}"
        );
    }
}

#[test]
fn calendar_manifest_v3_cli_matches_sdk_and_preserves_global_available_values() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct CalendarTree(std::path::PathBuf);
    impl Drop for CalendarTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = CalendarTree(std::env::temp_dir().join(format!(
            "lila-intl-calendar-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&tree.0).unwrap();
    let json = r#"{"schema_version":3,"custom_id":"cli-calendars","date_time_calendars":["chinese"],"currency_codes":["EUR","JPY"],"locale_filters":{"date_time":["fr"],"number_plural":["fr"],"display_names":["fr"]}}"#;
    let manifest = tree.0.join("profile.json");
    fs::write(&manifest, json).unwrap();
    let calendars = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_calendars.js"
    );
    let currencies = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_currencies.js"
    );
    let source = format!("{calendars}\n{currencies}");
    let script = tree.0.join("calendars.js");
    let dump = tree.0.join("calendars.wasm");
    let output = tree.0.join("calendars.bundle");
    fs::write(&script, &source).unwrap();
    let profile = lila_engine::IntlCompilationProfile::CustomProjection(
        lila_engine::CustomIntlProfile::from_manifest_json(json).unwrap(),
    );
    let selection = lila_engine::IntlDataSelection::new(profile.clone());
    let expected_export = selection.selected().unwrap().export_bytes().unwrap();
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            &source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: profile,
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected = engine.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["build", "wasm"])
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(fs::read(&dump).unwrap(), expected.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected);
    let exported = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--intl-manifest")
        .arg(&manifest)
        .args(["intl", "export", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    assert_eq!(
        fs::read(&output).unwrap().as_slice(),
        expected_export.as_ref()
    );
    let run = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["run", "--execution-backend", "wasm"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(String::from_utf8_lossy(&run.stdout).contains("intl-calendar-projection:ok"));
    for calendars in ["[]", "null", "[\"unknown\"]", "[\"chinese\",\"chinese\"]"] {
        fs::write(&manifest, format!(r#"{{"schema_version":3,"custom_id":"invalid-calendar","date_time_calendars":{calendars}}}"#)).unwrap();
        let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["run", "--execution-backend", "wasm"])
            .arg(tree.0.join("missing-source.js"))
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            error.contains("invalid --intl-manifest") && !error.contains("missing-source.js"),
            "{error}"
        );
    }
}

#[test]
fn currency_manifest_v2_build_and_export_use_the_actual_sdk_data_before_loading_source() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct CurrencyTree(std::path::PathBuf);
    impl Drop for CurrencyTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = CurrencyTree(std::env::temp_dir().join(format!(
            "lila-intl-currency-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&tree.0).unwrap();
    let json = r#"{"schema_version":2,"custom_id":"cli-currencies","currency_codes":["jpy","EUR"],"locale_filters":{"number_plural":["fr"],"display_names":["fr"]}}"#;
    let manifest = tree.0.join("profile.json");
    fs::write(&manifest, json).unwrap();
    let source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_currencies.js"
    );
    let script = tree.0.join("currency.js");
    let dump = tree.0.join("currency.wasm");
    let bundle = tree.0.join("currency.bundle");
    fs::write(&script, source).unwrap();
    let profile = lila_engine::IntlCompilationProfile::CustomProjection(
        lila_engine::CustomIntlProfile::from_manifest_json(json).unwrap(),
    );
    let selection = lila_engine::IntlDataSelection::new(profile.clone());
    let expected_bundle = selection.selected().unwrap().export_bytes().unwrap();
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: profile,
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected_wasm = engine.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["build", "wasm"])
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(fs::read(&dump).unwrap(), expected_wasm.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected_wasm);
    let exported = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--intl-manifest")
        .arg(&manifest)
        .args(["intl", "export", "--output"])
        .arg(&bundle)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    assert_eq!(
        fs::read(&bundle).unwrap().as_slice(),
        expected_bundle.as_ref()
    );
    let outcome = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["run", "--execution-backend", "wasm"])
        .arg(&script)
        .output()
        .unwrap();
    assert!(
        outcome.status.success(),
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert!(String::from_utf8_lossy(&outcome.stdout).contains("intl-currency-projection:ok"));
    for codes in ["[]", "null", "[\"ZZZ\"]", "[\"EUR\",\"eur\"]"] {
        fs::write(&manifest, format!(r#"{{"schema_version":2,"custom_id":"invalid-currencies","currency_codes":{codes}}}"#)).unwrap();
        let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["run", "--execution-backend", "wasm"])
            .arg(tree.0.join("missing-source.js"))
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            error.contains("invalid --intl-manifest") && !error.contains("missing-source.js"),
            "{error}"
        );
    }
}

#[test]
fn canonical_intl_export_matches_sdk_across_processes_and_inspect_rejects_damage() {
    struct ExportTree(std::path::PathBuf);
    impl Drop for ExportTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ExportTree(std::env::temp_dir().join(
        format!("lila-intl-export-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
    ));
    fs::create_dir(&tree.0).unwrap();
    let manifest = tree.0.join("profile.json");
    let json = r#"{"schema_version":1,"custom_id":"cli-export","locale_filters":{"list":["fr"],"relative_time":["fr"],"number_plural":["es"]}}"#;
    fs::write(&manifest, json).unwrap();
    let profile = lila_engine::CustomIntlProfile::from_manifest_json(json).unwrap();
    let selection = lila_engine::IntlDataSelection::new(
        lila_engine::IntlCompilationProfile::CustomProjection(profile),
    );
    let selected = selection.selected().unwrap();
    let expected = selected.export_bytes().unwrap();
    let first = tree.0.join("first.bundle");
    let second = tree.0.join("second.bundle");
    for path in [&first, &second] {
        let output = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["intl", "export", "--output"])
            .arg(path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read(path).unwrap().as_slice(), expected.as_ref());
    }
    let inspected = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["intl", "inspect", "--input"])
        .arg(&first)
        .output()
        .unwrap();
    assert!(
        inspected.status.success(),
        "{}",
        String::from_utf8_lossy(&inspected.stderr)
    );
    assert_eq!(
        inspected.stdout.as_slice(),
        selected.identity().artifact_identity().as_bytes()
    );
    let duplicate = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--intl-manifest")
        .arg(&manifest)
        .args(["intl", "export", "--output"])
        .arg(&first)
        .output()
        .unwrap();
    assert!(!duplicate.status.success());
    assert_eq!(fs::read(&first).unwrap().as_slice(), expected.as_ref());
    let mut damaged = expected.to_vec();
    let last = damaged.len() - 1;
    damaged[last] ^= 1;
    fs::write(&second, damaged).unwrap();
    let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["intl", "inspect", "--input"])
        .arg(&second)
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    fs::write(
        &manifest,
        r#"{"schema_version":1,"custom_id":"x","locale_filters":{"calendars":["gregory"]}}"#,
    )
    .unwrap();
    let missing = tree.0.join("not-created.bundle");
    let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--intl-manifest")
        .arg(&manifest)
        .args(["intl", "export", "--output"])
        .arg(&missing)
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(!missing.exists());
}

#[test]
fn custom_manifest_cli_matches_the_actual_sdk_graph_and_rejects_before_source_loading() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ManifestTree(std::path::PathBuf);
    impl Drop for ManifestTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ManifestTree(std::env::temp_dir().join(format!(
            "lila-intl-manifest-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&tree.0).unwrap();
    let manifest = tree.0.join("profile.json");
    fs::write(
        &manifest,
        r#"{"schema_version":1,"custom_id":"cli-manifest","locale_filters":{
        "list":["he","es"],"relative_time":["pl","fr"]}}"#,
    )
    .unwrap();
    let source = include_str!("../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_list_relative_time.js");
    let script = tree.0.join("selected.js");
    let module = tree.0.join("selected.mjs");
    let dump = tree.0.join("selected.wasm");
    fs::write(&script, source).unwrap();
    fs::write(&module, format!("{source}\nexport const selected = 262;")).unwrap();
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: lila_engine::IntlCompilationProfile::CustomProjection(
                    lila_engine::CustomIntlProfile::new(
                        lila_engine::CustomProfileId::parse("cli-manifest").unwrap(),
                        Some(&["es", "he"]),
                        Some(&["fr", "pl"]),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .unwrap(),
                ),
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected = engine.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args(["--jobs", "1", "--intl-manifest"])
        .arg(&manifest)
        .args(["build", "wasm"])
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(fs::read(&dump).unwrap(), expected.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected);
    for entry in [&script, &module] {
        let run = Command::new(env!("CARGO_BIN_EXE_lila"))
            .args(["--jobs", "1", "--intl-manifest"])
            .arg(&manifest)
            .args(["run", "--execution-backend", "wasm"])
            .arg(entry)
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(String::from_utf8_lossy(&run.stdout).contains("backend_used: WasmAot"));
    }
    for json in [
        r#"{"schema_version":2,"custom_id":"bad","locale_filters":{"list":["fr"]}}"#,
        r#"{"schema_version":1,"custom_id":"bad","locale_filters":{"numbering_systems":["latn"]}}"#,
        r#"{"schema_version":1,"custom_id":"bad","locale_filters":{"list":null}}"#,
    ] {
        fs::write(&manifest, json).unwrap();
        let rejected = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--intl-manifest")
            .arg(&manifest)
            .args(["run", "--execution-backend", "wasm"])
            .arg(tree.0.join("missing-source.js"))
            .output()
            .unwrap();
        assert!(!rejected.status.success());
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(error.contains("invalid --intl-manifest"), "{error}");
        assert!(
            !error.contains("missing-source.js"),
            "manifest admission precedes source loading: {error}"
        );
    }
}

#[test]
fn projected_list_cli_build_and_module_run_use_the_actual_sdk_projection() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ProjectionTree(std::path::PathBuf);
    impl Drop for ProjectionTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ProjectionTree(std::env::temp_dir().join(format!(
        "lila-list-projection-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())));
    fs::create_dir(&tree.0).unwrap();
    let source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_list.js"
    );
    let script = tree.0.join("selected.js");
    let module = tree.0.join("selected.mjs");
    let dump = tree.0.join("selected.wasm");
    fs::write(&script, source).unwrap();
    fs::write(&module, format!("{source}\nexport const projected = 262;")).unwrap();
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: lila_engine::IntlCompilationProfile::CustomProjection(
                    lila_engine::CustomIntlProfile::new(
                        lila_engine::CustomProfileId::parse("cli-projection").unwrap(),
                        Some(&["es", "he"]),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .unwrap(),
                ),
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected = engine.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .args([
            "--jobs",
            "1",
            "--intl-list-locales",
            "he,es",
            "--intl-profile",
            "custom:cli-projection",
            "build",
            "wasm",
        ])
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(fs::read(&dump).unwrap(), expected.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected);
    for entry in [&script, &module] {
        let output = Command::new(env!("CARGO_BIN_EXE_lila"))
            .args([
                "--jobs",
                "1",
                "--intl-profile",
                "custom:cli-projection",
                "run",
                "--execution-backend",
                "wasm",
            ])
            .arg(entry)
            .args(["--intl-list-locales", "es,he"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("intl-list-projection:ok") && stdout.contains("backend_used: WasmAot"),
            "{stdout}"
        );
    }
}

#[test]
fn combined_projection_cli_build_matches_sdk_and_runs_both_selected_services() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct CombinedTree(std::path::PathBuf);
    impl Drop for CombinedTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = CombinedTree(std::env::temp_dir().join(format!(
        "lila-combined-projection-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    )));
    fs::create_dir(&tree.0).unwrap();
    let source = include_str!("../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_list_relative_time.js");
    let script = tree.0.join("selected.js");
    let module = tree.0.join("selected.mjs");
    let dump = tree.0.join("selected.wasm");
    fs::write(&script, source).unwrap();
    fs::write(&module, format!("{source}\nexport const projected = 262;")).unwrap();
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: lila_engine::IntlCompilationProfile::CustomProjection(
                    lila_engine::CustomIntlProfile::new(
                        lila_engine::CustomProfileId::parse("cli-composed").unwrap(),
                        Some(&["es", "he"]),
                        Some(&["fr", "pl"]),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .unwrap(),
                ),
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected = engine.emit_wasm(&unit).unwrap();
    for flags in [
        [
            "--intl-relative-time-locales",
            "pl,fr",
            "--intl-profile",
            "custom:cli-composed",
            "--intl-list-locales",
            "he,es",
        ],
        [
            "--intl-list-locales",
            "es,he",
            "--intl-relative-time-locales",
            "fr,pl",
            "--intl-profile",
            "custom:cli-composed",
        ],
    ] {
        let built = Command::new(env!("CARGO_BIN_EXE_lila"))
            .args(["--jobs", "1"])
            .args(flags)
            .args(["build", "wasm"])
            .arg(&script)
            .env("LILA_WASM_DUMP", &dump)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        assert_eq!(
            fs::read(&dump).unwrap(),
            expected.bytes,
            "CLI ordering keeps the exact SDK graph"
        );
        assert_dumped_runtime_matches_sdk(&dump, &expected);
    }
    for entry in [&script, &module] {
        let output = Command::new(env!("CARGO_BIN_EXE_lila"))
            .args([
                "--jobs",
                "1",
                "--intl-relative-time-locales",
                "pl,fr",
                "run",
                "--execution-backend",
                "wasm",
            ])
            .arg(entry)
            .args([
                "--intl-list-locales",
                "he,es",
                "--intl-profile",
                "custom:cli-composed",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("intl-combined-projection:ok")
                && stdout.contains("backend_used: WasmAot"),
            "{stdout}"
        );
    }
}

#[test]
fn display_names_cli_projection_matches_sdk_and_composes_all_three_filters() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ProjectionTree(std::path::PathBuf);
    impl Drop for ProjectionTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ProjectionTree(std::env::temp_dir().join(format!(
        "lila-displaynames-projection-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    )));
    fs::create_dir(&tree.0).unwrap();
    let display_source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_display_names.js"
    );
    let combined_source = include_str!("../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_list_relative_time.js");
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    for with_other_filters in [false, true] {
        let source = if with_other_filters {
            format!("{combined_source}\n{display_source}")
        } else {
            format!("{display_source}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('hi').resolvedOptions().locale !== 'hi') throw 'unfiltered List and RelativeTime domains';\n262;")
        };
        let script = tree.0.join(format!("selected-{with_other_filters}.js"));
        let module = script.with_extension("mjs");
        let dump = script.with_extension("wasm");
        fs::write(&script, &source).unwrap();
        fs::write(&module, format!("{source}\nexport const selected = 262;")).unwrap();
        let unit = engine
            .compile_script(
                &source,
                lila_engine::CompileOptions {
                    filename: Some(script.to_string_lossy().into_owned()),
                    intl_profile: lila_engine::IntlCompilationProfile::CustomProjection(
                        lila_engine::CustomIntlProfile::new(
                            lila_engine::CustomProfileId::parse("cli-displaynames").unwrap(),
                            with_other_filters.then_some(&["es", "he"][..]),
                            with_other_filters.then_some(&["fr", "pl"][..]),
                            Some(&["fr", "ja"]),
                            None,
                            None,
                            None,
                            None,
                            None,
                        )
                        .unwrap(),
                    ),
                    ..lila_engine::CompileOptions::default()
                },
            )
            .unwrap();
        let expected = engine.emit_wasm(&unit).unwrap();
        for reverse in [false, true] {
            let mut filters = vec![(
                "--intl-displaynames-locales",
                if reverse { "ja,fr" } else { "fr,ja" },
            )];
            if with_other_filters {
                filters.extend([
                    (
                        "--intl-relative-time-locales",
                        if reverse { "pl,fr" } else { "fr,pl" },
                    ),
                    (
                        "--intl-list-locales",
                        if reverse { "he,es" } else { "es,he" },
                    ),
                ]);
            }
            if reverse {
                filters.reverse();
            }
            let mut flags = filters
                .into_iter()
                .flat_map(|(flag, locales)| [flag, locales])
                .collect::<Vec<_>>();
            flags.extend(["--intl-profile", "custom:cli-displaynames"]);
            let built = Command::new(env!("CARGO_BIN_EXE_lila"))
                .args(["--jobs", "1"])
                .args(&flags)
                .args(["build", "wasm"])
                .arg(&script)
                .env("LILA_WASM_DUMP", &dump)
                .output()
                .unwrap();
            assert!(
                built.status.success(),
                "{}",
                String::from_utf8_lossy(&built.stderr)
            );
            assert_eq!(
                fs::read(&dump).unwrap(),
                expected.bytes,
                "CLI selects the same physical graph as the SDK"
            );
            assert_dumped_runtime_matches_sdk(&dump, &expected);
        }
        for entry in [&script, &module] {
            let mut run = Command::new(env!("CARGO_BIN_EXE_lila"));
            run.args([
                "--jobs",
                "1",
                "--intl-displaynames-locales",
                "ja,fr",
                "run",
                "--execution-backend",
                "wasm",
            ])
            .arg(entry)
            .args(["--intl-profile", "custom:cli-displaynames"]);
            if with_other_filters {
                run.args([
                    "--intl-list-locales",
                    "he,es",
                    "--intl-relative-time-locales",
                    "pl,fr",
                ]);
            }
            let output = run.output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("intl-displaynames-projection:ok")
                    && stdout.contains("backend_used: WasmAot"),
                "{stdout}"
            );
            if with_other_filters {
                assert!(stdout.contains("intl-combined-projection:ok"), "{stdout}");
            }
        }
    }
}

#[test]
fn duration_cli_projection_matches_sdk_and_runs_four_selected_components() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ProjectionTree(std::path::PathBuf);
    impl Drop for ProjectionTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ProjectionTree(std::env::temp_dir().join(format!(
        "lila-duration-projection-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    )));
    fs::create_dir(&tree.0).unwrap();
    let duration_source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_duration.js"
    );
    let combined_source = include_str!("../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_list_relative_time.js");
    let display_source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_display_names.js"
    );
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    for with_other_filters in [false, true] {
        let source = if with_other_filters {
            format!("{combined_source}\n{display_source}\n{duration_source}\nif (Intl.ListFormat.supportedLocalesOf(['fr', 'sr']).length !== 0) throw 'private Duration List rows';\n262;")
        } else {
            format!("{duration_source}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('hi').resolvedOptions().locale !== 'hi' || new Intl.DisplayNames('ja', {{type: 'region'}}).resolvedOptions().locale !== 'ja') throw 'unfiltered component domains';\n262;")
        };
        let profile = lila_engine::IntlCompilationProfile::CustomProjection(
            lila_engine::CustomIntlProfile::new(
                lila_engine::CustomProfileId::parse("cli-duration").unwrap(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["fr", "pl"][..]),
                with_other_filters.then_some(&["fr", "ja"][..]),
                Some(&["fr", "sr"]),
                None,
                None,
                None,
                None,
            )
            .unwrap(),
        );
        for (suffix, directive) in [("sloppy", ""), ("strict", "\"use strict\";\n")] {
            let source = format!("{directive}{source}");
            let script = tree
                .0
                .join(format!("selected-{with_other_filters}-{suffix}.js"));
            let module = script.with_extension("mjs");
            let dump = script.with_extension("wasm");
            fs::write(&script, &source).unwrap();
            fs::write(&module, format!("{source}\nexport const selected = 262;")).unwrap();
            let unit = engine
                .compile_script(
                    &source,
                    lila_engine::CompileOptions {
                        filename: Some(script.to_string_lossy().into_owned()),
                        intl_profile: profile.clone(),
                        ..lila_engine::CompileOptions::default()
                    },
                )
                .unwrap();
            let expected = engine.emit_wasm(&unit).unwrap();
            for reverse in [false, true] {
                let mut filters = vec![(
                    "--intl-duration-locales",
                    if reverse { "sr,fr" } else { "fr,sr" },
                )];
                if with_other_filters {
                    filters.extend([
                        (
                            "--intl-list-locales",
                            if reverse { "he,es" } else { "es,he" },
                        ),
                        (
                            "--intl-relative-time-locales",
                            if reverse { "pl,fr" } else { "fr,pl" },
                        ),
                        (
                            "--intl-displaynames-locales",
                            if reverse { "ja,fr" } else { "fr,ja" },
                        ),
                    ]);
                }
                if reverse {
                    filters.reverse();
                }
                let mut flags = filters
                    .into_iter()
                    .flat_map(|(flag, locales)| [flag, locales])
                    .collect::<Vec<_>>();
                if reverse {
                    flags.splice(0..0, ["--intl-profile", "custom:cli-duration"]);
                } else {
                    flags.extend(["--intl-profile", "custom:cli-duration"]);
                }
                let built = Command::new(env!("CARGO_BIN_EXE_lila"))
                    .args(["--jobs", "1"])
                    .args(&flags)
                    .args(["build", "wasm"])
                    .arg(&script)
                    .env("LILA_WASM_DUMP", &dump)
                    .output()
                    .unwrap();
                assert!(
                    built.status.success(),
                    "{}",
                    String::from_utf8_lossy(&built.stderr)
                );
                assert_eq!(
                    fs::read(&dump).unwrap(),
                    expected.bytes,
                    "CLI and SDK retain exactly the same selected graph"
                );
                assert_dumped_runtime_matches_sdk(&dump, &expected);
            }
            for entry in [&script, &module] {
                let mut run = Command::new(env!("CARGO_BIN_EXE_lila"));
                run.args([
                    "--jobs",
                    "1",
                    "--intl-duration-locales",
                    "sr,fr",
                    "run",
                    "--execution-backend",
                    "wasm",
                ])
                .arg(entry)
                .args(["--intl-profile", "custom:cli-duration"]);
                if with_other_filters {
                    run.args([
                        "--intl-list-locales",
                        "he,es",
                        "--intl-relative-time-locales",
                        "pl,fr",
                        "--intl-displaynames-locales",
                        "ja,fr",
                    ]);
                }
                let output = run.output().unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(
                    stdout.contains("intl-duration-projection:ok")
                        && stdout.contains("backend_used: WasmAot"),
                    "{stdout}"
                );
                if with_other_filters {
                    assert!(
                        stdout.contains("intl-combined-projection:ok")
                            && stdout.contains("intl-displaynames-projection:ok"),
                        "{stdout}"
                    );
                }
            }
        }
    }
}

#[test]
fn number_cli_projection_matches_sdk_and_runs_the_coupled_public_services() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ProjectionTree(std::path::PathBuf);
    impl Drop for ProjectionTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ProjectionTree(std::env::temp_dir().join(format!(
        "lila-number-projection-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    )));
    fs::create_dir(&tree.0).unwrap();
    let fixture = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_number.js"
    );
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    for with_other_filters in [false, true] {
        let source = if with_other_filters {
            format!("{fixture}\nif (Intl.ListFormat.supportedLocalesOf(['fr', 'sr']).length !== 0 || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.DisplayNames('de', {{type: 'region'}}).resolvedOptions().locale !== 'en-US' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'en-US') throw 'independent selected component domains';\n262;")
        } else {
            format!("{fixture}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.DisplayNames('ja', {{type: 'region'}}).resolvedOptions().locale !== 'ja' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'hi') throw 'unfiltered component domains';\n262;")
        };
        let profile = lila_engine::IntlCompilationProfile::CustomProjection(
            lila_engine::CustomIntlProfile::new(
                lila_engine::CustomProfileId::parse("cli-number").unwrap(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["fr", "pl"][..]),
                with_other_filters.then_some(&["fr", "ja"][..]),
                with_other_filters.then_some(&["fr", "sr"][..]),
                Some(&["es", "pl"]),
                None,
                None,
                None,
            )
            .unwrap(),
        );
        for (suffix, directive) in [("sloppy", ""), ("strict", "\"use strict\";\n")] {
            let source = format!("{directive}{source}");
            let script = tree
                .0
                .join(format!("selected-{with_other_filters}-{suffix}.js"));
            let module = script.with_extension("mjs");
            let dump = script.with_extension("wasm");
            fs::write(&script, &source).unwrap();
            fs::write(&module, format!("{source}\nexport const selected = 262;")).unwrap();
            let unit = engine
                .compile_script(
                    &source,
                    lila_engine::CompileOptions {
                        filename: Some(script.to_string_lossy().into_owned()),
                        intl_profile: profile.clone(),
                        ..lila_engine::CompileOptions::default()
                    },
                )
                .unwrap();
            let expected = engine.emit_wasm(&unit).unwrap();
            for reverse in [false, true] {
                let mut filters = vec![(
                    "--intl-number-locales",
                    if reverse { "pl,es" } else { "es,pl" },
                )];
                if with_other_filters {
                    filters.extend([
                        (
                            "--intl-list-locales",
                            if reverse { "he,es" } else { "es,he" },
                        ),
                        (
                            "--intl-relative-time-locales",
                            if reverse { "pl,fr" } else { "fr,pl" },
                        ),
                        (
                            "--intl-displaynames-locales",
                            if reverse { "ja,fr" } else { "fr,ja" },
                        ),
                        (
                            "--intl-duration-locales",
                            if reverse { "sr,fr" } else { "fr,sr" },
                        ),
                    ]);
                }
                if reverse {
                    filters.reverse();
                }
                let mut flags = filters
                    .into_iter()
                    .flat_map(|(flag, locales)| [flag, locales])
                    .collect::<Vec<_>>();
                if reverse {
                    flags.splice(0..0, ["--intl-profile", "custom:cli-number"]);
                } else {
                    flags.extend(["--intl-profile", "custom:cli-number"]);
                }
                let built = Command::new(env!("CARGO_BIN_EXE_lila"))
                    .args(["--jobs", "1"])
                    .args(&flags)
                    .args(["build", "wasm"])
                    .arg(&script)
                    .env("LILA_WASM_DUMP", &dump)
                    .output()
                    .unwrap();
                assert!(
                    built.status.success(),
                    "{}",
                    String::from_utf8_lossy(&built.stderr)
                );
                assert_eq!(
                    fs::read(&dump).unwrap(),
                    expected.bytes,
                    "CLI and SDK retain the same real Number-dependent image graph"
                );
                assert_dumped_runtime_matches_sdk(&dump, &expected);
            }
            for entry in [&script, &module] {
                let mut run = Command::new(env!("CARGO_BIN_EXE_lila"));
                run.args([
                    "--jobs",
                    "1",
                    "--intl-number-locales",
                    "pl,es",
                    "run",
                    "--execution-backend",
                    "wasm",
                ])
                .arg(entry)
                .args(["--intl-profile", "custom:cli-number"]);
                if with_other_filters {
                    run.args([
                        "--intl-list-locales",
                        "he,es",
                        "--intl-relative-time-locales",
                        "pl,fr",
                        "--intl-displaynames-locales",
                        "ja,fr",
                        "--intl-duration-locales",
                        "sr,fr",
                    ]);
                }
                let output = run.output().unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(
                    stdout.contains("intl-number-projection:ok")
                        && stdout.contains("backend_used: WasmAot"),
                    "{stdout}"
                );
            }
        }
    }
}

#[test]
fn datetime_cli_projection_matches_sdk_and_runs_six_selected_components() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ProjectionTree(std::path::PathBuf);
    impl Drop for ProjectionTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ProjectionTree(std::env::temp_dir().join(format!(
        "lila-datetime-projection-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    )));
    fs::create_dir(&tree.0).unwrap();
    let fixture = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_datetime.js"
    );
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    for with_other_filters in [false, true] {
        let source = if with_other_filters {
            format!("{fixture}\nif (Intl.ListFormat.supportedLocalesOf(['fr', 'sr']).length !== 0 || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.DisplayNames('de', {{type: 'region'}}).resolvedOptions().locale !== 'en-US' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'en-US' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.PluralRules('ar').resolvedOptions().locale !== 'en-US') throw 'six independent selected component domains';\n262;")
        } else {
            format!("{fixture}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.DisplayNames('ja', {{type: 'region'}}).resolvedOptions().locale !== 'ja' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'hi' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.PluralRules('ar').resolvedOptions().locale !== 'ar') throw 'unfiltered component domains';\n262;")
        };
        let profile = lila_engine::IntlCompilationProfile::CustomProjection(
            lila_engine::CustomIntlProfile::new(
                lila_engine::CustomProfileId::parse("cli-datetime").unwrap(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["fr", "pl"][..]),
                with_other_filters.then_some(&["fr", "ja"][..]),
                with_other_filters.then_some(&["fr", "sr"][..]),
                with_other_filters.then_some(&["es", "pl"][..]),
                Some(&["ar-EG", "zh"]),
                None,
                None,
            )
            .unwrap(),
        );
        for (suffix, directive) in [("sloppy", ""), ("strict", "\"use strict\";\n")] {
            let source = format!("{directive}{source}");
            let script = tree
                .0
                .join(format!("selected-{with_other_filters}-{suffix}.js"));
            let module = script.with_extension("mjs");
            let dump = script.with_extension("wasm");
            fs::write(&script, &source).unwrap();
            fs::write(&module, format!("{source}\nexport const selected = 262;")).unwrap();
            let unit = engine
                .compile_script(
                    &source,
                    lila_engine::CompileOptions {
                        filename: Some(script.to_string_lossy().into_owned()),
                        intl_profile: profile.clone(),
                        ..lila_engine::CompileOptions::default()
                    },
                )
                .unwrap();
            let expected = engine.emit_wasm(&unit).unwrap();
            for reverse in [false, true] {
                let mut filters = vec![(
                    "--intl-datetime-locales",
                    if reverse { "zh,ar-EG" } else { "ar-EG,zh" },
                )];
                if with_other_filters {
                    filters.extend([
                        (
                            "--intl-list-locales",
                            if reverse { "he,es" } else { "es,he" },
                        ),
                        (
                            "--intl-relative-time-locales",
                            if reverse { "pl,fr" } else { "fr,pl" },
                        ),
                        (
                            "--intl-displaynames-locales",
                            if reverse { "ja,fr" } else { "fr,ja" },
                        ),
                        (
                            "--intl-duration-locales",
                            if reverse { "sr,fr" } else { "fr,sr" },
                        ),
                        (
                            "--intl-number-locales",
                            if reverse { "pl,es" } else { "es,pl" },
                        ),
                    ]);
                }
                if reverse {
                    filters.reverse();
                }
                let mut flags = filters
                    .into_iter()
                    .flat_map(|(flag, locales)| [flag, locales])
                    .collect::<Vec<_>>();
                if reverse {
                    flags.splice(0..0, ["--intl-profile", "custom:cli-datetime"]);
                } else {
                    flags.extend(["--intl-profile", "custom:cli-datetime"]);
                }
                let built = Command::new(env!("CARGO_BIN_EXE_lila"))
                    .args(["--jobs", "1"])
                    .args(&flags)
                    .args(["build", "wasm"])
                    .arg(&script)
                    .env("LILA_WASM_DUMP", &dump)
                    .output()
                    .unwrap();
                assert!(
                    built.status.success(),
                    "{}",
                    String::from_utf8_lossy(&built.stderr)
                );
                assert_eq!(
                    fs::read(&dump).unwrap(),
                    expected.bytes,
                    "CLI and SDK retain the same real DateTime-dependent graph"
                );
                assert_dumped_runtime_matches_sdk(&dump, &expected);
            }
            for entry in [&script, &module] {
                let mut run = Command::new(env!("CARGO_BIN_EXE_lila"));
                run.args([
                    "--jobs",
                    "1",
                    "--intl-datetime-locales",
                    "zh,ar-EG",
                    "run",
                    "--execution-backend",
                    "wasm",
                ])
                .arg(entry)
                .args(["--intl-profile", "custom:cli-datetime"]);
                if with_other_filters {
                    run.args([
                        "--intl-list-locales",
                        "he,es",
                        "--intl-relative-time-locales",
                        "pl,fr",
                        "--intl-displaynames-locales",
                        "ja,fr",
                        "--intl-duration-locales",
                        "sr,fr",
                        "--intl-number-locales",
                        "pl,es",
                    ]);
                }
                let output = run.output().unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(
                    stdout.contains("intl-datetime-projection:ok")
                        && stdout.contains("backend_used: WasmAot"),
                    "{stdout}"
                );
            }
        }
    }
}

#[test]
fn segmenter_cli_projection_matches_sdk_and_runs_eight_selected_components() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ProjectionTree(std::path::PathBuf);
    impl Drop for ProjectionTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ProjectionTree(std::env::temp_dir().join(format!(
            "lila-segmenter-projection-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir(&tree.0).unwrap();
    let fixture = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_segmenter.js"
    );
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    for with_other_filters in [false, true] {
        let profile = lila_engine::IntlCompilationProfile::CustomProjection(
            lila_engine::CustomIntlProfile::new(
                lila_engine::CustomProfileId::parse("cli-segmenter").unwrap(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["fr", "pl"][..]),
                with_other_filters.then_some(&["fr", "ja"][..]),
                with_other_filters.then_some(&["fr", "sr"][..]),
                with_other_filters.then_some(&["es", "pl"][..]),
                with_other_filters.then_some(&["ar-EG", "zh"][..]),
                with_other_filters.then_some(&["de-CH", "sv"][..]),
                Some(&["sv", "el"]),
            )
            .unwrap(),
        );
        for (suffix, directive) in [("sloppy", ""), ("strict", "\"use strict\";\n")] {
            let source = format!("{directive}{fixture}\n262;");
            let script = tree
                .0
                .join(format!("selected-{with_other_filters}-{suffix}.js"));
            let module = script.with_extension("mjs");
            let dump = script.with_extension("wasm");
            fs::write(&script, &source).unwrap();
            fs::write(&module, format!("{source}\nexport const selected = 262;")).unwrap();
            let unit = engine
                .compile_script(
                    &source,
                    lila_engine::CompileOptions {
                        filename: Some(script.to_string_lossy().into_owned()),
                        intl_profile: profile.clone(),
                        ..lila_engine::CompileOptions::default()
                    },
                )
                .unwrap();
            let expected = engine.emit_wasm(&unit).unwrap();
            for reverse in [false, true] {
                let mut filters = vec![(
                    "--intl-segmenter-locales",
                    if reverse { "sv,el" } else { "el,sv" },
                )];
                if with_other_filters {
                    filters.extend([
                        ("--intl-list-locales", "he,es"),
                        ("--intl-relative-time-locales", "pl,fr"),
                        ("--intl-displaynames-locales", "ja,fr"),
                        ("--intl-duration-locales", "sr,fr"),
                        ("--intl-number-locales", "pl,es"),
                        ("--intl-datetime-locales", "zh,ar-EG"),
                        ("--intl-collator-locales", "sv,de-CH"),
                    ]);
                }
                if reverse {
                    filters.reverse();
                }
                let mut flags = filters
                    .into_iter()
                    .flat_map(|(flag, value)| [flag, value])
                    .collect::<Vec<_>>();
                if reverse {
                    flags.splice(0..0, ["--intl-profile", "custom:cli-segmenter"]);
                } else {
                    flags.extend(["--intl-profile", "custom:cli-segmenter"]);
                }
                let output = Command::new(env!("CARGO_BIN_EXE_lila"))
                    .args(["--jobs", "1"])
                    .args(&flags)
                    .args(["build", "wasm"])
                    .arg(&script)
                    .env("LILA_WASM_DUMP", &dump)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(
                    fs::read(&dump).unwrap(),
                    expected.bytes,
                    "same actual selected SDK/CLI graph"
                );
                assert_dumped_runtime_matches_sdk(&dump, &expected);
                for entry in [&script, &module] {
                    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
                        .args(["--jobs", "1"])
                        .args(&flags)
                        .args(["run", "--execution-backend", "wasm"])
                        .arg(entry)
                        .output()
                        .unwrap();
                    assert!(
                        output.status.success(),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    assert!(
                        stdout.contains("intl-segmenter-projection:ok")
                            && stdout.contains("backend_used: WasmAot"),
                        "{stdout}"
                    );
                }
            }
        }
    }
}

#[test]
fn collator_cli_projection_matches_sdk_and_runs_seven_selected_components() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ProjectionTree(std::path::PathBuf);
    impl Drop for ProjectionTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ProjectionTree(std::env::temp_dir().join(format!(
        "lila-collator-projection-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    )));
    fs::create_dir(&tree.0).unwrap();
    let fixture = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/projected_collator.js"
    );
    let engine = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    for with_other_filters in [false, true] {
        let source = if with_other_filters {
            format!("{fixture}\nif (Intl.ListFormat.supportedLocalesOf(['fr', 'sr']).length !== 0 || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.DisplayNames('de', {{type: 'region'}}).resolvedOptions().locale !== 'en-US' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'en-US' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.PluralRules('ar').resolvedOptions().locale !== 'en-US' || new Intl.DateTimeFormat('fr').resolvedOptions().locale !== 'en-US') throw 'seven independent selected component domains';\n262;")
        } else {
            format!("{fixture}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.DisplayNames('ja', {{type: 'region'}}).resolvedOptions().locale !== 'ja' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'hi' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.PluralRules('ar').resolvedOptions().locale !== 'ar' || new Intl.DateTimeFormat('fr').resolvedOptions().locale !== 'fr') throw 'unfiltered component domains';\n262;")
        };
        let profile = lila_engine::IntlCompilationProfile::CustomProjection(
            lila_engine::CustomIntlProfile::new(
                lila_engine::CustomProfileId::parse("cli-collator").unwrap(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["fr", "pl"][..]),
                with_other_filters.then_some(&["fr", "ja"][..]),
                with_other_filters.then_some(&["fr", "sr"][..]),
                with_other_filters.then_some(&["es", "pl"][..]),
                with_other_filters.then_some(&["ar-EG", "zh"][..]),
                Some(&["de-CH", "sv"]),
                None,
            )
            .unwrap(),
        );
        for (suffix, directive) in [("sloppy", ""), ("strict", "\"use strict\";\n")] {
            let source = format!("{directive}{source}");
            let script = tree
                .0
                .join(format!("selected-{with_other_filters}-{suffix}.js"));
            let module = script.with_extension("mjs");
            let dump = script.with_extension("wasm");
            fs::write(&script, &source).unwrap();
            fs::write(&module, format!("{source}\nexport const selected = 262;")).unwrap();
            let unit = engine
                .compile_script(
                    &source,
                    lila_engine::CompileOptions {
                        filename: Some(script.to_string_lossy().into_owned()),
                        intl_profile: profile.clone(),
                        ..lila_engine::CompileOptions::default()
                    },
                )
                .unwrap();
            let expected = engine.emit_wasm(&unit).unwrap();
            for reverse in [false, true] {
                let mut filters = vec![(
                    "--intl-collator-locales",
                    if reverse { "sv,de-CH" } else { "de-CH,sv" },
                )];
                if with_other_filters {
                    filters.extend([
                        (
                            "--intl-list-locales",
                            if reverse { "he,es" } else { "es,he" },
                        ),
                        (
                            "--intl-relative-time-locales",
                            if reverse { "pl,fr" } else { "fr,pl" },
                        ),
                        (
                            "--intl-displaynames-locales",
                            if reverse { "ja,fr" } else { "fr,ja" },
                        ),
                        (
                            "--intl-duration-locales",
                            if reverse { "sr,fr" } else { "fr,sr" },
                        ),
                        (
                            "--intl-number-locales",
                            if reverse { "pl,es" } else { "es,pl" },
                        ),
                        (
                            "--intl-datetime-locales",
                            if reverse { "zh,ar-EG" } else { "ar-EG,zh" },
                        ),
                    ]);
                }
                if reverse {
                    filters.reverse();
                }
                let mut flags = filters
                    .into_iter()
                    .flat_map(|(flag, value)| [flag, value])
                    .collect::<Vec<_>>();
                if reverse {
                    flags.splice(0..0, ["--intl-profile", "custom:cli-collator"]);
                } else {
                    flags.extend(["--intl-profile", "custom:cli-collator"]);
                }
                let built = Command::new(env!("CARGO_BIN_EXE_lila"))
                    .args(["--jobs", "1"])
                    .args(&flags)
                    .args(["build", "wasm"])
                    .arg(&script)
                    .env("LILA_WASM_DUMP", &dump)
                    .output()
                    .unwrap();
                assert!(
                    built.status.success(),
                    "{}",
                    String::from_utf8_lossy(&built.stderr)
                );
                assert_eq!(
                    fs::read(&dump).unwrap(),
                    expected.bytes,
                    "CLI and SDK retain the same raw-row Collator graph"
                );
                assert_dumped_runtime_matches_sdk(&dump, &expected);
            }
            for entry in [&script, &module] {
                let mut run = Command::new(env!("CARGO_BIN_EXE_lila"));
                run.args([
                    "--jobs",
                    "1",
                    "--intl-collator-locales",
                    "sv,de-CH",
                    "run",
                    "--execution-backend",
                    "wasm",
                ])
                .arg(entry)
                .args(["--intl-profile", "custom:cli-collator"]);
                if with_other_filters {
                    run.args([
                        "--intl-list-locales",
                        "he,es",
                        "--intl-relative-time-locales",
                        "pl,fr",
                        "--intl-displaynames-locales",
                        "ja,fr",
                        "--intl-duration-locales",
                        "sr,fr",
                        "--intl-number-locales",
                        "pl,es",
                        "--intl-datetime-locales",
                        "zh,ar-EG",
                    ]);
                }
                let output = run.output().unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(
                    stdout.contains("intl-collator-projection:ok")
                        && stdout.contains("backend_used: WasmAot"),
                    "{stdout}"
                );
            }
        }
    }
}

#[test]
fn custom_intl_profile_reaches_ordinary_cli_build_script_and_module_run() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    struct ProfileTree(std::path::PathBuf);
    impl Drop for ProfileTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let tree = ProfileTree(std::env::temp_dir().join(format!(
        "lila-intl-profile-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    )));
    fs::create_dir(&tree.0).unwrap();
    let source = include_str!(
        "../../../lila-engine/tests/fixtures/intl_compilation_profile/selected_operations.js"
    );
    let script = tree.0.join("selected.js");
    let module = tree.0.join("selected.mjs");
    let dump = tree.0.join("selected.wasm");
    fs::write(&script, source).unwrap();
    fs::write(&module, format!("{source}\nexport const selected = 262;")).unwrap();
    let expected = lila_engine::Engine::new(lila_engine::RealmBuilder::new().build());
    let unit = expected
        .compile_script(
            source,
            lila_engine::CompileOptions {
                filename: Some(script.to_string_lossy().into_owned()),
                intl_profile: lila_engine::IntlCompilationProfile::Custom(
                    lila_engine::CustomProfileId::parse("cli-image-selection").unwrap(),
                ),
                ..lila_engine::CompileOptions::default()
            },
        )
        .unwrap();
    let expected = expected.emit_wasm(&unit).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("--jobs")
        .arg("1")
        .arg("--host-surface")
        .arg("product")
        .arg("--intl-profile")
        .arg("custom:cli-image-selection")
        .arg("build")
        .arg("wasm")
        .arg(&script)
        .env("LILA_WASM_DUMP", &dump)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    // The real CLI producer must emit the same program and selected runtime
    // data as ordinary library compilation; no sections are rewritten.
    assert_eq!(fs::read(&dump).unwrap(), expected.bytes);
    assert_dumped_runtime_matches_sdk(&dump, &expected);
    for entry in [&script, &module] {
        let output = Command::new(env!("CARGO_BIN_EXE_lila"))
            .arg("--host-surface")
            .arg("product")
            .arg("run")
            .arg("--execution-backend")
            .arg("wasm")
            .arg(entry)
            .arg("--intl-profile")
            .arg("custom:cli-image-selection")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("intl-profile:ok") && stdout.contains("backend_used: WasmAot"),
            "{stdout}"
        );
    }
}

/// Pins ECMA-402 `Intl.DateTimeFormat` construction order: the tagged result
/// is reserved through NewTarget.prototype before locale/options observation,
/// then published only after its record and brand are complete.
#[test]
fn run_wasm_intl_date_time_format_construction_order_fixture_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_intl_date_time_format_construction_order.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("number(262"), "{stdout}");
}

/// Pins ECMA-402 `Intl.Locale` construction order: NewTarget prototype
/// resolution reserves an unreachable object before tag/options observation,
/// and only the fully initialized Locale state is published.
#[test]
fn run_wasm_intl_locale_construction_order_fixture_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_intl_locale_construction_order.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("number(262"), "{stdout}");
}

/// Pins the distinct canonical tag/component result roles consumed by
/// `Intl.Locale`, `Intl.getCanonicalLocales`, and `Intl.DateTimeFormat`.
#[test]
fn run_wasm_intl_canonical_locale_tag_roles_fixture_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_intl_canonical_locale_tag_roles.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("number(262"), "{stdout}");
}
