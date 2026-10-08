use super::*;

fn parent() -> Output {
    let output = Output::new();
    fs::create_dir(&output.0).unwrap();
    output
}
fn prelude(source: &str, mode: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version":1, "source":source, "execution_mode":mode,
        "harness_profile":"none", "merged_harness":null,
        "files":[{"name":"assert.js","contents":"/* assertion candidate */\n"},
                 {"name":"helper.js","contents":"/* declared helper */\n"}],
        "overrides":[]
    })
}
fn filesystem(operation: &str, specifier: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version":1, "operation":operation, "referrer":"none", "layout":"plain",
        "specifier":specifier, "attributes":[]
    })
}

#[test]
fn uri_data_keeps_lone_surrogates_and_escapes_every_unit_before_real_frontend_admission() {
    let data =
        serde_json::json!({"schema_version":1,"units":[0,34,92,0xd800,0xd83d,0xde00]}).to_string();
    let input = uri::UriInput::from_json(&data).unwrap();
    for (parser, name) in [
        (BuiltinParserTarget::EncodeUri, "encodeURI"),
        (
            BuiltinParserTarget::EncodeUriComponent,
            "encodeURIComponent",
        ),
        (BuiltinParserTarget::DecodeUri, "decodeURI"),
        (
            BuiltinParserTarget::DecodeUriComponent,
            "decodeURIComponent",
        ),
    ] {
        let source = input.source(parser).unwrap();
        assert_eq!(
            source,
            format!("{name}(\"\\u0000\\u0022\\u005c\\ud800\\ud83d\\ude00\");")
        );
        lila_front::parse(&source, lila_front::ParseOptions::script()).unwrap();
    }
    assert!(input.source(BuiltinParserTarget::Json).is_err());
    for invalid in [
        serde_json::json!({"schema_version":2,"units":[]}),
        serde_json::json!({"schema_version":1,"units":[65536]}),
        serde_json::json!({"schema_version":1,"units":[],"source":"injected()"}),
        serde_json::json!({"schema_version":1,"units":vec![0;4097]}),
    ] {
        assert!(uri::UriInput::from_json(&invalid.to_string()).is_err());
    }
}

#[test]
fn native_target_bytes_and_closed_stages_cannot_cross_target_or_fake_execution() {
    for name in [
        "encode-uri",
        "encode-uri-component",
        "decode-uri",
        "decode-uri-component",
        "prelude",
        "filesystem-resolver",
    ] {
        let target = RobustnessTarget::from_name(name).unwrap();
        let original =
            RobustnessInput::new("robustness/new-target", target, 1000, vec![0, 255, 128, 10])
                .unwrap();
        assert_eq!(
            RobustnessInput::from_json(&original.to_pretty_json().unwrap()).unwrap(),
            original
        );
        let plan = RobustnessCampaignPlan::new(original.clone(), 7, 3).unwrap();
        for ordinal in 0..plan.count() {
            let mutated = plan.case(ordinal).unwrap().0;
            assert_eq!(mutated.target(), target);
            assert_eq!(mutated.timeout_ms(), 1000);
        }
        let mut wire = serde_json::to_value(&original).unwrap();
        wire["target"]["foreign"] = serde_json::json!(true);
        assert!(RobustnessInput::from_json(&wire.to_string()).is_err());
        let stage = *target.stages().last().unwrap();
        let result = if matches!(target, RobustnessTarget::Builtin { .. }) {
            RobustnessResult::Executed {
                completion: RobustnessCompletionKind::Throw,
                value: "actual throw".into(),
            }
        } else {
            RobustnessResult::Accepted {
                stage,
                artifact_bytes: None,
            }
        };
        assert!(result.valid_terminal(target, target.stages()));
        assert!(!result.valid_terminal(target, &target.stages()[..target.stages().len() - 1]));
        assert!(!result.valid_terminal(
            RobustnessTarget::Snapshot {},
            RobustnessTarget::Snapshot {}.stages()
        ));
    }
}

#[test]
fn prelude_materialization_preserves_real_strict_raw_module_and_declared_include_ownership() {
    let output = parent();
    for (source, mode) in [
        (
            "/*---\nincludes: [helper.js, helper.js]\n---*/\n1;",
            "strict-script",
        ),
        ("/*---\nflags: [raw]\n---*/\n1;", "raw-script"),
        (
            "/*---\nflags: [module]\nincludes: [helper.js]\n---*/\nexport {};",
            "module",
        ),
    ] {
        let loaded = prelude::PreludeInput::from_json(&prelude(source, mode).to_string())
            .unwrap()
            .load(&output.0)
            .unwrap();
        let materialized = loaded.materialize().unwrap();
        match mode {
            "strict-script" => {
                assert!(materialized
                    .source
                    .starts_with("\"use strict\";\n/* assertion candidate */"));
                assert!(materialized.source.ends_with(source));
                assert_eq!(
                    materialized
                        .used_preludes
                        .iter()
                        .filter(|(name, _)| name == "helper.js")
                        .count(),
                    1
                );
            }
            "raw-script" => {
                assert_eq!(materialized.source, source);
                assert!(materialized.used_preludes.is_empty());
            }
            "module" => {
                assert_eq!(materialized.source, source);
                assert!(materialized
                    .module_prelude
                    .as_ref()
                    .unwrap()
                    .contains("/* declared helper */"));
            }
            _ => unreachable!(),
        }
        loaded.finish().unwrap();
        assert_eq!(fs::read_dir(&output.0).unwrap().count(), 0);
    }
    let mut missing = prelude("/*---\nincludes: [missing.js]\n---*/\n1;", "sloppy-script");
    let loaded = prelude::PreludeInput::from_json(&missing.to_string())
        .unwrap()
        .load(&output.0)
        .unwrap();
    assert!(loaded.materialize().is_err());
    loaded.finish().unwrap();
    missing["execution_mode"] = serde_json::json!("module");
    assert!(prelude::PreludeInput::from_json(&missing.to_string()).is_err());
}

#[test]
fn only_original_complete_embedded_host_can_materialize_host_requirements() {
    let output = parent();
    let mut wire = prelude("$262.createRealm();", "sloppy-script");
    let loaded = prelude::PreludeInput::from_json(&wire.to_string())
        .unwrap()
        .load(&output.0)
        .unwrap();
    assert!(loaded.materialize().is_err());
    loaded.finish().unwrap();
    wire["harness_profile"] = serde_json::json!("embedded_wasm_aot");
    let loaded = prelude::PreludeInput::from_json(&wire.to_string())
        .unwrap()
        .load(&output.0)
        .unwrap();
    let genuine = loaded.materialize().unwrap();
    assert!(genuine.source.ends_with("$262.createRealm();"));
    loaded.finish().unwrap();
    wire["overrides"] =
        serde_json::json!([{"name":"assert.js","contents":"/* forged assertion */"}]);
    let loaded = prelude::PreludeInput::from_json(&wire.to_string())
        .unwrap()
        .load(&output.0)
        .unwrap();
    assert!(loaded.materialize().is_err());
    loaded.finish().unwrap();
    wire["host_ownership"] = serde_json::json!(true);
    assert!(prelude::PreludeInput::from_json(&wire.to_string()).is_err());
    assert_eq!(fs::read_dir(&output.0).unwrap().count(), 0);
}

#[test]
fn prelude_fixture_names_and_materialization_budgets_reject_ambient_file_authority() {
    let base = prelude("1;", "sloppy-script");
    for name in [
        "../assert.js",
        "/etc/passwd",
        "a/../../assert.js",
        "a\\b.js",
        "C:assert.js",
        "a//b.js",
    ] {
        let mut wire = base.clone();
        wire["files"][0]["name"] = serde_json::json!(name);
        assert!(prelude::PreludeInput::from_json(&wire.to_string()).is_err());
    }
    for (field, value) in [
        ("source", serde_json::json!("x".repeat(16385))),
        ("schema_version", serde_json::json!(2)),
        ("merged_harness", serde_json::json!("///assert.js\n")),
    ] {
        let mut wire = base.clone();
        wire[field] = value;
        assert!(prelude::PreludeInput::from_json(&wire.to_string()).is_err());
    }
    let mut wire = base;
    wire["files"][0]["contents"] = serde_json::json!("x".repeat(65537));
    assert!(prelude::PreludeInput::from_json(&wire.to_string()).is_err());
}

#[test]
fn actual_filesystem_resolution_load_and_direct_load_confine_the_owned_fixture() {
    let output = parent();
    let probe = filesystem::FilesystemInput::from_json(
        &filesystem("resolve_and_load", "./nested/../dep.js").to_string(),
    )
    .unwrap()
    .setup(&output.0)
    .unwrap();
    let key = probe.resolve().unwrap();
    assert!(key.as_str().ends_with("/root/dep.js"));
    probe.load(&key).unwrap();
    probe.finish().unwrap();
    for operation in ["resolve_and_load", "load_direct"] {
        let probe = filesystem::FilesystemInput::from_json(
            &filesystem(operation, "../outside/dep.js").to_string(),
        )
        .unwrap()
        .setup(&output.0)
        .unwrap();
        let result = probe.resolve().and_then(|key| probe.load(&key));
        assert!(matches!(result, Err(filesystem::ProbeError::Rejected(_))));
        probe.finish().unwrap();
        assert_eq!(fs::read_dir(&output.0).unwrap().count(), 0);
    }
    let mut duplicate = filesystem("resolve_and_load", "./dep.js");
    duplicate["attributes"] =
        serde_json::json!([{"key":"type","value":"json"},{"key":"type","value":"json"}]);
    assert!(filesystem::FilesystemInput::from_json(&duplicate.to_string()).is_err());
}

#[cfg(unix)]
#[test]
fn surviving_symlink_escape_and_lexical_parent_identity_use_the_real_loader() {
    let output = parent();
    for (specifier, accepted) in [
        ("./inside-link/../dep.js", true),
        ("./inside-link/dep.js", true),
        ("./outside-link/dep.js", false),
    ] {
        let mut wire = filesystem("resolve_and_load", specifier);
        wire["layout"] = serde_json::json!("symlinks");
        let probe = filesystem::FilesystemInput::from_json(&wire.to_string())
            .unwrap()
            .setup(&output.0)
            .unwrap();
        let result = probe.resolve().and_then(|key| probe.load(&key));
        assert_eq!(result.is_ok(), accepted, "{specifier}");
        if !accepted {
            assert!(matches!(result, Err(filesystem::ProbeError::Rejected(_))));
        }
        probe.finish().unwrap();
    }
    let sandbox = sandbox::Sandbox::new_in(&output.0).unwrap();
    let root = sandbox.root().to_owned();
    drop(sandbox);
    assert!(!root.exists());
    assert_eq!(fs::read_dir(&output.0).unwrap().count(), 0);
}
