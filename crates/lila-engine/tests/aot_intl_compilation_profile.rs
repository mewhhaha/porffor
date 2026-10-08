use lila_engine::{
    CompileOptions, CustomIntlProfile, CustomProfileId, Engine, ExecutionBackend,
    IntlCompilationProfile, ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder,
    RunOptions,
};
use lila_intl::IntlDataSelection;
use wasmparser::{Parser, Payload};

const SOURCE: &str = include_str!("fixtures/intl_compilation_profile/selected_operations.js");

#[test]
fn sparse_service_frames_preserve_selected_operations_and_reject_dependency_only_public_services() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let profile = IntlCompilationProfile::CustomProjection(CustomIntlProfile::from_manifest_json(
        r#"{"schema_version":6,"custom_id":"relative-only-emitted","services":["RelativeTimeFormat"],"locale_filters":{"relative_time":["fr"]}}"#).unwrap());
    let owner = IntlDataSelection::new(profile.clone());
    let selected = owner.selected().unwrap();
    assert_eq!(selected.component_sections().len(), 4);
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_services.js");
    for directive in ["", "\"use strict\";\n"] {
        let source = format!("{directive}{fixture}");
        let unit = engine
            .compile_script(
                &source,
                CompileOptions {
                    intl_profile: profile.clone(),
                    ..CompileOptions::default()
                },
            )
            .unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        assert_selected_sections(&artifact.bytes, &profile);
        let wire = Parser::new(0)
            .parse_all(&artifact.bytes)
            .find_map(|payload| match payload.unwrap() {
                Payload::CustomSection(section)
                    if section.name() == lila_intl::INTL_SERVICE_SELECTION_CUSTOM_SECTION =>
                {
                    Some(section.data().to_vec())
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(
            wire,
            selected.service_selection().unwrap().wire().to_le_bytes()
        );
        assert!(engine
            .run_compiled_unit(&unit, &source, wasm_run())
            .unwrap()
            .note
            .contains("number(262"));
    }
    for source in [
        "new Intl.NumberFormat('en-US').format(1);",
        "new Intl.PluralRules('en-US').select(1);",
        "new Intl.ListFormat('en-US').format(['a','b']);",
    ] {
        let unit = engine
            .compile_script(
                source,
                CompileOptions {
                    intl_profile: profile.clone(),
                    ..CompileOptions::default()
                },
            )
            .unwrap();
        let error = engine
            .run_compiled_unit(&unit, source, wasm_run())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("unavailable") && error.contains("service"),
            "{error}"
        );
    }
}

#[test]
fn sparse_supported_values_separates_retained_keys_missing_data_and_invalid_key_errors() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let profile = IntlCompilationProfile::CustomProjection(
        CustomIntlProfile::for_services(
            CustomProfileId::parse("number-only-values").unwrap(),
            &["NumberFormat"],
        )
        .unwrap(),
    );
    let source = r#"
        if (Intl.supportedValuesOf('currency').indexOf('EUR') < 0) throw 'retained Number currency catalogue';
        if (Intl.supportedValuesOf('numberingSystem').length !== 78) throw 'full retained digit authority';
        try { Intl.supportedValuesOf('timeZone'); throw 'missing data accepted'; }
        catch (error) { if (!(error instanceof TypeError) || String(error).indexOf('unavailable') < 0) throw error; }
        try { Intl.supportedValuesOf('TimeZone'); throw 'invalid key accepted'; }
        catch (error) { if (!(error instanceof RangeError)) throw error; }
        if (new Intl.NumberFormat('en-US').format(12) !== '12') throw 'selected Number operation';
        262;
    "#;
    let unit = engine
        .compile_script(
            source,
            CompileOptions {
                intl_profile: profile.clone(),
                ..CompileOptions::default()
            },
        )
        .unwrap();
    assert_selected_sections(&engine.emit_wasm(&unit).unwrap().bytes, &profile);
    assert!(engine
        .run_compiled_unit(&unit, source, wasm_run())
        .unwrap()
        .note
        .contains("number(262"));
}

#[test]
fn named_zone_projection_emits_real_transition_name_closure_and_preserves_global_identity() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_named_zones.js");
    for composed in [false, true] {
        let id = CustomProfileId::parse("emitted-named-zones").unwrap();
        let selection = if composed {
            CustomIntlProfile::new(
                id.clone(),
                None,
                None,
                None,
                None,
                None,
                Some(&["fr"]),
                None,
                None,
            )
            .unwrap()
            .with_currency_codes(&["EUR"])
            .unwrap()
            .with_date_time_calendars(&["chinese"])
            .unwrap()
            .with_numbering_systems(&["latn"])
            .unwrap()
            .with_named_time_zones(&["US/Eastern"])
            .unwrap()
        } else {
            CustomIntlProfile::for_named_time_zones(id.clone(), &["America/New_York"]).unwrap()
        };
        let profile = IntlCompilationProfile::CustomProjection(selection);
        let owner = IntlDataSelection::new(profile.clone());
        let selected = owner.selected().unwrap();
        let full_owner = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let full = full_owner.selected().unwrap();
        let frame = |selected: &lila_intl::SelectedIntlDataBundle, section: &str| {
            selected
                .component_sections()
                .into_iter()
                .find(|(name, _)| *name == section)
                .unwrap()
                .1
        };
        for section in [
            lila_intl::INTL_NAMED_TIME_ZONE_DATA_CUSTOM_SECTION,
            lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION,
            lila_intl::INTL_TIME_ZONE_NAMES_DATA_CUSTOM_SECTION,
        ] {
            assert!(
                frame(selected, section).len() < frame(full, section).len(),
                "actual selected zone closure {section}"
            );
        }
        assert_eq!(
            selected
                .supported_values(lila_intl::SupportedValuesKey::TimeZone)
                .unwrap()
                .values(),
            full.supported_values(lila_intl::SupportedValuesKey::TimeZone)
                .unwrap()
                .values()
        );
        let admitted =
            lila_intl::SelectedIntlDataBundle::from_export_bytes(&selected.export_bytes().unwrap())
                .unwrap();
        assert_eq!(admitted.identity(), selected.identity());
        for directive in ["", "\"use strict\";\n"] {
            let source = format!("{directive}{fixture}");
            let unit = engine
                .compile_script(
                    &source,
                    CompileOptions {
                        intl_profile: profile.clone(),
                        ..CompileOptions::default()
                    },
                )
                .unwrap();
            assert_selected_sections(&engine.emit_wasm(&unit).unwrap().bytes, &profile);
            assert!(engine
                .run_compiled_unit(&unit, &source, wasm_run())
                .unwrap()
                .note
                .contains("number(262"));
        }
        for source in [
            "new Intl.DateTimeFormat('en-US',{timeZone:'Europe/Paris'}).format(0);",
            "Temporal.Instant.fromEpochNanoseconds(0n).toZonedDateTimeISO('Europe/Paris').hour;",
        ] {
            let unit = engine
                .compile_script(
                    source,
                    CompileOptions {
                        intl_profile: profile.clone(),
                        ..CompileOptions::default()
                    },
                )
                .unwrap();
            let error = engine
                .run_compiled_unit(&unit, source, wasm_run())
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("unavailable")
                    && error.contains("Europe/Paris")
                    && !error.contains("unknown previously resolved"),
                "{error}"
            );
        }
    }
}

#[test]
fn paired_numbering_projection_emits_sparse_service_rows_and_full_global_digit_authority() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_numbering.js");
    for composed in [false, true] {
        let id = CustomProfileId::parse("emitted-numbering").unwrap();
        let selection = if composed {
            CustomIntlProfile::new(
                id.clone(),
                None,
                None,
                None,
                None,
                Some(&["fr", "ar-EG"]),
                Some(&["fr", "ar-EG"]),
                None,
                None,
            )
            .unwrap()
            .with_currency_codes(&["EUR"])
            .unwrap()
            .with_date_time_calendars(&["chinese"])
            .unwrap()
            .with_numbering_systems(&["deva"])
            .unwrap()
        } else {
            CustomIntlProfile::for_numbering_systems(id.clone(), &["deva"]).unwrap()
        };
        let profile = IntlCompilationProfile::CustomProjection(selection);
        let owner = IntlDataSelection::new(profile.clone());
        let selected = owner.selected().unwrap();
        let full_owner = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let full = full_owner.selected().unwrap();
        let frame = |selected: &lila_intl::SelectedIntlDataBundle, section: &str| {
            selected
                .component_sections()
                .into_iter()
                .find(|(name, _)| *name == section)
                .unwrap()
                .1
        };
        for section in [
            lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION,
            lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION,
        ] {
            assert!(
                frame(selected, section).len() < frame(full, section).len(),
                "actual numbering frame {section}"
            );
        }
        for key in [
            lila_intl::SupportedValuesKey::NumberingSystem,
            lila_intl::SupportedValuesKey::Calendar,
            lila_intl::SupportedValuesKey::TimeZone,
        ] {
            assert_eq!(
                selected.supported_values(key).unwrap().values(),
                full.supported_values(key).unwrap().values()
            );
        }
        let admitted =
            lila_intl::SelectedIntlDataBundle::from_export_bytes(&selected.export_bytes().unwrap())
                .unwrap();
        assert_eq!(admitted.identity(), selected.identity());
        for directive in ["", "\"use strict\";\n"] {
            let source = format!("{directive}{fixture}");
            let unit = engine
                .compile_script(
                    &source,
                    CompileOptions {
                        intl_profile: profile.clone(),
                        ..CompileOptions::default()
                    },
                )
                .unwrap();
            assert_selected_sections(&engine.emit_wasm(&unit).unwrap().bytes, &profile);
            assert!(engine
                .run_compiled_unit(&unit, &source, wasm_run())
                .unwrap()
                .note
                .contains("number(262"));
        }
    }
}

#[test]
fn localized_calendar_projection_preserves_global_kernels_and_emits_only_real_service_rows() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_calendars.js");
    for locale_filter in [false, true] {
        let id = CustomProfileId::parse("emitted-calendars").unwrap();
        let profile = if locale_filter {
            CustomIntlProfile::new(
                id.clone(),
                None,
                None,
                None,
                None,
                None,
                Some(&["fr"]),
                None,
                None,
            )
            .unwrap()
            .with_date_time_calendars(&["chinese"])
            .unwrap()
        } else {
            CustomIntlProfile::for_date_time_calendars(id.clone(), &["chinese"]).unwrap()
        };
        let profile = IntlCompilationProfile::CustomProjection(profile);
        let selected_owner = IntlDataSelection::new(profile.clone());
        let selected = selected_owner.selected().unwrap();
        let full_owner = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let full = full_owner.selected().unwrap();
        let frame = |selected: &lila_intl::SelectedIntlDataBundle, section: &str| {
            selected
                .component_sections()
                .into_iter()
                .find(|(name, _)| *name == section)
                .unwrap()
                .1
        };
        assert!(
            frame(selected, lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION).len()
                < frame(full, lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION).len()
        );
        for key in [
            lila_intl::SupportedValuesKey::Calendar,
            lila_intl::SupportedValuesKey::NumberingSystem,
            lila_intl::SupportedValuesKey::TimeZone,
        ] {
            assert_eq!(
                selected.supported_values(key).unwrap().values(),
                full.supported_values(key).unwrap().values()
            );
        }
        for directive in ["", "\"use strict\";\n"] {
            let source = format!("{directive}{fixture}");
            let unit = engine
                .compile_script(
                    &source,
                    CompileOptions {
                        intl_profile: profile.clone(),
                        ..CompileOptions::default()
                    },
                )
                .unwrap();
            assert_selected_sections(&engine.emit_wasm(&unit).unwrap().bytes, &profile);
            assert!(engine
                .run_compiled_unit(&unit, &source, wasm_run())
                .unwrap()
                .note
                .contains("number(262"));
        }
    }
}

#[test]
fn paired_currency_projection_emits_actual_pruned_data_and_original_code_fallbacks() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_currencies.js");
    for locale_filter in [false, true] {
        let id = CustomProfileId::parse("emitted-currencies").unwrap();
        let profile = if locale_filter {
            CustomIntlProfile::new(
                id,
                None,
                None,
                Some(&["fr"]),
                None,
                Some(&["fr"]),
                None,
                None,
                None,
            )
            .unwrap()
            .with_currency_codes(&["JPY", "EUR"])
            .unwrap()
        } else {
            CustomIntlProfile::for_currency_codes(id, &["EUR", "JPY"]).unwrap()
        };
        let profile = IntlCompilationProfile::CustomProjection(profile);
        let selection = IntlDataSelection::new(profile.clone());
        let selected = selection.selected().unwrap();
        assert_eq!(
            selected
                .supported_values(lila_intl::SupportedValuesKey::Currency)
                .unwrap()
                .values()
                .iter()
                .map(|code| code.as_ref())
                .collect::<Vec<_>>(),
            ["EUR", "JPY"]
        );
        let reminted =
            lila_intl::SelectedIntlDataBundle::from_export_bytes(&selected.export_bytes().unwrap())
                .unwrap();
        assert_eq!(reminted.identity(), selected.identity());
        for directive in ["", "\"use strict\";\n"] {
            let source = format!("{directive}{fixture}");
            let options = CompileOptions {
                intl_profile: profile.clone(),
                ..CompileOptions::default()
            };
            let unit = engine.compile_script(&source, options).unwrap();
            assert_selected_sections(&engine.emit_wasm(&unit).unwrap().bytes, &profile);
            assert!(engine
                .run_compiled_unit(&unit, &source, wasm_run())
                .unwrap()
                .note
                .contains("number(262"));
        }
    }
}

#[test]
fn segmenter_only_and_eight_component_projections_emit_selected_models_and_remint_wire_owners() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_segmenter.js");
    for with_other_filters in [false, true] {
        let id = CustomProfileId::parse("segmenter-composition").unwrap();
        let profile = IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                id.clone(),
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
        let selection = IntlDataSelection::new(profile.clone());
        let selected = selection.selected().unwrap();
        let complete = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let frames = selected.component_sections();
        for ((name, bytes), (full_name, full_bytes)) in frames
            .iter()
            .zip(complete.selected().unwrap().component_sections())
        {
            assert_eq!(*name, full_name);
            if *name == lila_intl::INTL_SEGMENTER_DATA_CUSTOM_SECTION {
                assert_ne!(
                    bytes.as_ref(),
                    full_bytes.as_ref(),
                    "actual Segmenter rows and associations"
                );
            } else if with_other_filters
                && matches!(
                    *name,
                    lila_intl::INTL_LIST_DATA_CUSTOM_SECTION
                        | lila_intl::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION
                        | lila_intl::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION
                        | lila_intl::INTL_DURATION_DATA_CUSTOM_SECTION
                        | lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION
                        | lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION
                        | lila_intl::INTL_COLLATOR_DATA_CUSTOM_SECTION
                )
            {
                assert!(
                    bytes.len() < full_bytes.len(),
                    "selected physical closure {name}"
                );
            } else {
                assert_eq!(
                    bytes.as_ref(),
                    full_bytes.as_ref(),
                    "unfiltered component {name}"
                );
            }
        }
        let frame = |name: &str| {
            frames
                .iter()
                .find(|(key, _)| *key == name)
                .unwrap()
                .1
                .clone()
        };
        let locale = lila_intl::LocaleDataImage::from_bytes(frame(
            lila_intl::INTL_LOCALE_DATA_CUSTOM_SECTION,
        ))
        .unwrap();
        let segmenter = lila_intl::SegmenterDataImage::from_bytes(
            frame(lila_intl::INTL_SEGMENTER_DATA_CUSTOM_SECTION),
            &locale,
        )
        .unwrap();
        assert_eq!(
            segmenter
                .profiles()
                .available_locales()
                .map(|name| name.as_str().to_owned())
                .collect::<Vec<_>>(),
            ["el", "en-US", "sv"]
        );
        let full = lila_intl::SegmenterDataImage::for_profile(
            lila_intl::IntlDataProfile::Custom(
                CustomProfileId::parse("segmenter-composition").unwrap(),
            ),
            &locale,
        )
        .unwrap();
        let foreign = lila_intl::SegmentUtf16Request::new(
            lila_intl::CheckedSegmenterConfiguration::new(
                full.profiles()
                    .admit(lila_intl::CanonicalLocaleId::from_data("sv").unwrap())
                    .unwrap(),
                lila_intl::SegmenterGranularity::Word,
            ),
            "hello:world".encode_utf16().collect(),
        )
        .unwrap();
        let reminted = lila_intl::decode_segment_utf16_request(
            &segmenter.profiles(),
            &lila_intl::encode_segment_utf16_request(&foreign).unwrap(),
        )
        .unwrap();
        assert_eq!(reminted.configuration().locale().resolved().as_str(), "sv");
        let hidden = lila_intl::SegmentUtf16Request::new(
            lila_intl::CheckedSegmenterConfiguration::new(
                full.profiles()
                    .admit(lila_intl::CanonicalLocaleId::from_data("fi").unwrap())
                    .unwrap(),
                lila_intl::SegmenterGranularity::Word,
            ),
            vec![0x61].into_boxed_slice(),
        )
        .unwrap();
        assert!(lila_intl::decode_segment_utf16_request(
            &segmenter.profiles(),
            &lila_intl::encode_segment_utf16_request(&hidden).unwrap()
        )
        .is_err());
        let checks = if with_other_filters {
            "if (Intl.ListFormat.supportedLocalesOf(['fr']).length || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.DisplayNames('de', {type:'region'}).resolvedOptions().locale !== 'en-US' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'en-US' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.PluralRules('ar').resolvedOptions().locale !== 'en-US' || new Intl.DateTimeFormat('fr').resolvedOptions().locale !== 'en-US' || new Intl.Collator('fr').resolvedOptions().locale !== 'en-US') throw 'eight independent domains';"
        } else {
            "if (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'hi' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.DateTimeFormat('fr').resolvedOptions().locale !== 'fr' || new Intl.Collator('fr').resolvedOptions().locale !== 'fr') throw 'unfiltered domains';"
        };
        let options = CompileOptions {
            intl_profile: profile.clone(),
            ..CompileOptions::default()
        };
        for directive in ["", "\"use strict\";\n"] {
            let source = format!("{directive}{fixture}\n{checks}\n262;");
            let unit = engine.compile_script(&source, options.clone()).unwrap();
            assert_selected_sections(&engine.emit_wasm(&unit).unwrap().bytes, &profile);
            assert!(engine
                .run_compiled_unit(&unit, &source, wasm_run())
                .unwrap()
                .note
                .contains("number(262"));
            assert_eq!(
                engine
                    .observe_script(&source, options.clone(), wasm_run())
                    .unwrap()
                    .completion,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(
                    262.0
                )))
            );
        }
        assert_eq!(
            engine
                .observe_module(
                    &format!("{fixture}\n{checks}\nexport const selected = 262;"),
                    options,
                    wasm_run()
                )
                .unwrap()
                .completion,
            ObservedCompletion::Normal(ObservedJsValue::Undefined)
        );
    }
    let invalid = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
        CustomIntlProfile::new(
            CustomProfileId::parse("segmenter-unavailable").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&["qaa"]),
        )
        .unwrap(),
    ));
    assert!(invalid.selected().is_err());
}

#[test]
fn collator_only_and_seven_component_projections_emit_real_rows_and_preserve_locale_preferences() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_collator.js");
    for with_other_filters in [false, true] {
        let id = CustomProfileId::parse("collator-composition").unwrap();
        let profile = IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                id.clone(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["fr", "pl"][..]),
                with_other_filters.then_some(&["fr", "ja"][..]),
                with_other_filters.then_some(&["fr", "sr"][..]),
                with_other_filters.then_some(&["es", "pl"][..]),
                with_other_filters.then_some(&["ar-EG", "zh"][..]),
                Some(&["sv", "de-CH"]),
                None,
            )
            .unwrap(),
        );
        let selection = IntlDataSelection::new(profile.clone());
        let selected = selection.selected().unwrap();
        let complete = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let full = complete.selected().unwrap();
        let frames = selected.component_sections();
        for ((name, bytes), (full_name, full_bytes)) in frames.iter().zip(full.component_sections())
        {
            assert_eq!(*name, full_name);
            if *name == lila_intl::INTL_COLLATOR_DATA_CUSTOM_SECTION
                || (with_other_filters
                    && matches!(
                        *name,
                        lila_intl::INTL_LIST_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_DURATION_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION
                    ))
            {
                assert!(
                    bytes.len() < full_bytes.len(),
                    "selected physical closure: {name}"
                );
            } else {
                assert_eq!(
                    bytes.as_ref(),
                    full_bytes.as_ref(),
                    "unfiltered component: {name}"
                );
            }
        }
        let frame = |name: &str| {
            frames
                .iter()
                .find(|(key, _)| *key == name)
                .unwrap()
                .1
                .clone()
        };
        let locale = lila_intl::LocaleDataImage::from_bytes(frame(
            lila_intl::INTL_LOCALE_DATA_CUSTOM_SECTION,
        ))
        .unwrap();
        let collator = lila_intl::CollatorDataImage::from_bytes(
            frame(lila_intl::INTL_COLLATOR_DATA_CUSTOM_SECTION),
            &locale,
        )
        .unwrap();
        assert_eq!(
            collator
                .profiles()
                .available_locales()
                .map(|name| name.as_str().to_owned())
                .collect::<Vec<_>>(),
            ["de-CH", "en-US", "sv"]
        );
        let checks = if with_other_filters {
            "if (Intl.ListFormat.supportedLocalesOf(['fr']).length || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.DisplayNames('de', {type:'region'}).resolvedOptions().locale !== 'en-US' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'en-US' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.DateTimeFormat('fr').resolvedOptions().locale !== 'en-US') throw 'seven independent domains';"
        } else {
            "if (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.DateTimeFormat('fr').resolvedOptions().locale !== 'fr') throw 'unfiltered domains';"
        };
        let options = CompileOptions {
            intl_profile: profile.clone(),
            ..CompileOptions::default()
        };
        for directive in ["", "\"use strict\";\n"] {
            let source = format!("{directive}{fixture}\n{checks}\n262;");
            let unit = engine.compile_script(&source, options.clone()).unwrap();
            let artifact = engine.emit_wasm(&unit).unwrap();
            assert_selected_sections(&artifact.bytes, &profile);
            assert!(engine
                .run_compiled_unit(&unit, &source, wasm_run())
                .unwrap()
                .note
                .contains("number(262"));
            assert_eq!(
                engine
                    .observe_script(&source, options.clone(), wasm_run())
                    .unwrap()
                    .completion,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(
                    262.0
                )))
            );
        }
        assert_eq!(
            engine
                .observe_module(
                    &format!("{fixture}\n{checks}\nexport const selected = 262;"),
                    options.clone(),
                    wasm_run()
                )
                .unwrap()
                .completion,
            ObservedCompletion::Normal(ObservedJsValue::Undefined)
        );
        let code_only = "var names = Intl.supportedValuesOf('collation'); if (names.indexOf('phonebk') < 0 || names.indexOf('search') >= 0) throw 'selected catalogue'; 262;";
        let unit = engine.compile_script(code_only, options.clone()).unwrap();
        assert_selected_sections(&engine.emit_wasm(&unit).unwrap().bytes, &profile);
        assert_eq!(
            engine
                .observe_script(code_only, options, wasm_run())
                .unwrap()
                .completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
        );
    }
    let invalid = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
        CustomIntlProfile::new(
            CustomProfileId::parse("collator-unavailable").unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&["qaa"]),
            None,
        )
        .unwrap(),
    ));
    assert!(invalid.selected().is_err());
}

fn custom_options(id: &str) -> CompileOptions {
    CompileOptions {
        intl_profile: IntlCompilationProfile::Custom(CustomProfileId::parse(id).unwrap()),
        ..CompileOptions::default()
    }
}

fn wasm_run() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(120_000),
        ..RunOptions::default()
    }
}

fn assert_selected_sections(bytes: &[u8], profile: &IntlCompilationProfile) {
    let selection = IntlDataSelection::new(profile.clone());
    let selected = selection.selected().expect("complete pinned graph admits");
    let sections = Parser::new(0)
        .parse_all(bytes)
        .filter_map(|payload| match payload.unwrap() {
            Payload::CustomSection(section) => {
                Some((section.name().to_string(), section.data().to_vec()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    for (name, bytes) in selected.component_sections() {
        let actual = sections
            .iter()
            .filter(|(key, _)| key == name)
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 1, "exactly one selected component: {name}");
        assert_eq!(actual[0].1.as_slice(), bytes.as_ref(), "{name}");
    }
    let identity = sections
        .iter()
        .filter(|(name, _)| name == lila_intl::INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION)
        .collect::<Vec<_>>();
    assert_eq!(identity.len(), 1);
    assert_eq!(
        identity[0].1.as_slice(),
        selected.identity().artifact_identity().as_bytes()
    );
}

#[test]
fn ordinary_custom_compilation_emits_the_selected_graph_and_runs_the_same_unit() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let options = custom_options("library-image-selection");
    let engine = Engine::new(RealmBuilder::new().build());
    let unit = engine.compile_script(SOURCE, options.clone()).unwrap();
    let artifact = engine.emit_wasm(&unit).unwrap();
    assert_selected_sections(&artifact.bytes, &options.intl_profile);
    let outcome = engine.run_compiled_unit(&unit, SOURCE, wasm_run()).unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("number(262"), "{}", outcome.note);

    // A source run exercises the actual profile-framed program-cache path.
    let observed = engine
        .observe_script(SOURCE, options.clone(), wasm_run())
        .unwrap();
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
    let module = format!("{SOURCE}\nexport const selected = 262;");
    assert_eq!(
        engine
            .observe_module(&module, options, wasm_run())
            .unwrap()
            .completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
}

#[test]
fn custom_compilation_rejects_backends_that_cannot_consume_the_selection() {
    let engine = Engine::new(RealmBuilder::new().build());
    let options = custom_options("unsupported-backend");
    let unit = engine.compile_script("262;", options.clone()).unwrap();
    for error in [
        engine.emit_c(&unit).unwrap_err(),
        engine.emit_native(&unit, None).unwrap_err(),
        engine
            .run_compiled_unit(
                &unit,
                "262;",
                RunOptions {
                    backend: ExecutionBackend::SpecExec,
                    ..RunOptions::default()
                },
            )
            .unwrap_err(),
        engine
            .observe_script(
                "262;",
                options.clone(),
                RunOptions {
                    backend: ExecutionBackend::SpecExec,
                    ..RunOptions::default()
                },
            )
            .unwrap_err(),
        engine
            .run_module(
                "export {};",
                options,
                RunOptions {
                    backend: ExecutionBackend::SpecExec,
                    ..RunOptions::default()
                },
            )
            .unwrap_err(),
    ] {
        assert!(
            error
                .to_string()
                .contains("custom Intl profile 'unsupported-backend'"),
            "{error}"
        );
        assert!(error.to_string().contains("use Wasm AOT"), "{error}");
    }
}

#[test]
fn custom_code_only_catalogue_and_locale_case_consumers_execute_normally() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let options = custom_options("data-only-selection");
    for source in [
        "const a = Intl.supportedValuesOf('calendar'); const b = Intl.supportedValuesOf('calendar'); const d = Object.getOwnPropertyDescriptor(a, '0'); a !== b && a.includes('gregory') && d.writable && d.enumerable && d.configurable;",
        "'I'.toLocaleLowerCase('tr') === 'ı' && 'i'.toLocaleUpperCase('tr') === 'İ' && 'I'.toLocaleLowerCase() === 'i';",
    ] {
        let unit = engine.compile_script(source, options.clone()).unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        assert_selected_sections(&artifact.bytes, &options.intl_profile);
        let observed = engine.observe_script(source, options.clone(), wasm_run()).unwrap();
        assert_eq!(observed.completion, ObservedCompletion::Normal(ObservedJsValue::Boolean(true)), "{source}");
    }
}

#[test]
fn actual_list_projection_emits_a_smaller_selected_graph_and_keeps_duration_dependencies_private() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let profile = IntlCompilationProfile::CustomProjection(
        CustomIntlProfile::new(
            CustomProfileId::parse("library-projection").unwrap(),
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
    );
    let source = include_str!("fixtures/intl_compilation_profile/projected_list.js");
    let engine = Engine::new(RealmBuilder::new().build());
    let options = CompileOptions {
        intl_profile: profile.clone(),
        ..CompileOptions::default()
    };
    let unit = engine.compile_script(source, options.clone()).unwrap();
    let artifact = engine.emit_wasm(&unit).unwrap();
    assert_selected_sections(&artifact.bytes, &profile);
    assert!(engine
        .emit_c(&unit)
        .unwrap_err()
        .to_string()
        .contains("use Wasm AOT"));
    assert!(engine
        .run_compiled_unit(
            &unit,
            source,
            RunOptions {
                backend: ExecutionBackend::SpecExec,
                ..RunOptions::default()
            }
        )
        .unwrap_err()
        .to_string()
        .contains("custom Intl projection"));
    let projected = IntlDataSelection::new(profile);
    let full = IntlDataSelection::new(IntlCompilationProfile::Custom(
        CustomProfileId::parse("library-projection").unwrap(),
    ));
    let selected = projected.selected().unwrap();
    let unfiltered = full.selected().unwrap();
    for ((name, bytes), (full_name, full_bytes)) in selected
        .component_sections()
        .into_iter()
        .zip(unfiltered.component_sections())
    {
        assert_eq!(name, full_name);
        if name == lila_intl::INTL_LIST_DATA_CUSTOM_SECTION {
            assert!(
                bytes.len() < full_bytes.len(),
                "actual List image must shrink"
            );
        }
    }
    assert_ne!(selected.identity(), unfiltered.identity());
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observed = engine
            .observe_script(&script, options.clone(), wasm_run())
            .unwrap();
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
        );
    }
    let module = format!("{source}\nexport const projected = 262;");
    assert_eq!(
        engine
            .observe_module(&module, options, wasm_run())
            .unwrap()
            .completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
}

#[test]
fn independent_relative_and_combined_projections_emit_exact_rows_and_remint_selected_wire_owners() {
    use lila_intl::number_format::options::LocaleMatcher;
    use lila_intl::number_format::{NumberLocaleRequest, PartitionLimits};
    use lila_intl::{
        CanonicalLocaleId, FiniteRelativeNumber, ListDataImage, LocaleDataImage,
        NumberProfilesDataImage, RelativeHostOp, RelativeNumeric, RelativeRequest, RelativeStyle,
        RelativeTimeConfiguration, RelativeTimeDataImage, RelativeTimeError, RelativeUnit,
    };
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let combined =
        include_str!("fixtures/intl_compilation_profile/projected_list_relative_time.js");
    let relative_only = "Intl.RelativeTimeFormat.supportedLocalesOf(['pl', 'fr', 'hi', 'en-US']).join(',') === 'pl,fr,en-US' && new Intl.RelativeTimeFormat('hi').resolvedOptions().locale === 'en-US' && new Intl.RelativeTimeFormat('fr', { numeric: 'auto' }).format(-1, 'day') === 'hier' && new Intl.ListFormat('ar').resolvedOptions().locale === 'ar' && new Intl.NumberFormat('hi').resolvedOptions().locale === 'hi';";
    for with_list in [false, true] {
        let id = CustomProfileId::parse("composed-projection").unwrap();
        let profile = IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                id.clone(),
                with_list.then_some(&["es", "he"][..]),
                Some(&["pl", "fr"]),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap(),
        );
        let selection = IntlDataSelection::new(profile.clone());
        let selected = selection.selected().unwrap();
        let unfiltered = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let full = unfiltered.selected().unwrap();
        let frames = selected.component_sections();
        let frame = |name| {
            frames
                .iter()
                .find(|(key, _)| *key == name)
                .unwrap()
                .1
                .clone()
        };
        for ((name, bytes), (full_name, full_bytes)) in frames.iter().zip(full.component_sections())
        {
            assert_eq!(*name, full_name);
            if *name == lila_intl::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION
                || (with_list && *name == lila_intl::INTL_LIST_DATA_CUSTOM_SECTION)
            {
                assert!(
                    bytes.len() < full_bytes.len(),
                    "actual component shrinks: {name}"
                );
            } else {
                assert_eq!(
                    bytes.as_ref(),
                    full_bytes.as_ref(),
                    "unfiltered component stays exact: {name}"
                );
            }
        }
        assert_ne!(selected.identity(), full.identity());
        let locale =
            LocaleDataImage::from_bytes(frame(lila_intl::INTL_LOCALE_DATA_CUSTOM_SECTION)).unwrap();
        let lists =
            ListDataImage::from_bytes(frame(lila_intl::INTL_LIST_DATA_CUSTOM_SECTION), &locale)
                .unwrap();
        let numbers = NumberProfilesDataImage::from_bytes(
            frame(lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION),
            &locale,
            &lists,
        )
        .unwrap();
        let relative = RelativeTimeDataImage::from_bytes(
            frame(lila_intl::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION),
            &locale,
            &numbers,
        )
        .unwrap();
        let profiles = relative.profiles();
        let number_profiles = numbers.profiles();
        assert_eq!(profiles.available_locales(), ["en-US", "fr", "pl"]);
        let request = NumberLocaleRequest {
            requested: vec![CanonicalLocaleId::from_data("fr-FR").unwrap()].into_boxed_slice(),
            matcher: LocaleMatcher::Lookup,
            numbering_system: None,
        };
        let resolved = profiles
            .resolve_locale(&request, &number_profiles, &PartitionLimits::HOST_ABI)
            .unwrap();
        let configuration = RelativeTimeConfiguration::new(
            resolved,
            RelativeStyle::Long,
            RelativeNumeric::Auto,
            &number_profiles,
        )
        .unwrap();
        let value = FiniteRelativeNumber::new(-1.0).unwrap();
        let original = RelativeRequest::Parts {
            configuration: configuration.clone(),
            value,
            unit: RelativeUnit::Day,
        };
        let same_bytes_new_owner =
            RelativeTimeDataImage::from_bytes(relative.bytes(), &locale, &numbers).unwrap();
        assert_eq!(
            same_bytes_new_owner.profiles().format_parts(
                &configuration,
                value,
                RelativeUnit::Day,
                &PartitionLimits::HOST_ABI
            ),
            Err(RelativeTimeError::InvalidLocale),
        );
        let decoded = lila_intl::decode_relative_request(
            RelativeHostOp::FormatRelativeTimeParts,
            &lila_intl::encode_relative_request(&original).unwrap(),
            &same_bytes_new_owner.profiles(),
            &number_profiles,
            &PartitionLimits::HOST_ABI,
        )
        .unwrap();
        assert_ne!(
            decoded, original,
            "primitive frame remints the actual selected template owner"
        );
        let RelativeRequest::Parts {
            configuration,
            value,
            unit,
        } = decoded
        else {
            panic!("parts request")
        };
        assert_eq!(
            same_bytes_new_owner
                .profiles()
                .format_parts(&configuration, value, unit, &PartitionLimits::HOST_ABI)
                .unwrap()
                .to_text()
                .unwrap(),
            "hier"
        );
        let source = if with_list { combined } else { relative_only };
        let options = CompileOptions {
            intl_profile: profile.clone(),
            ..CompileOptions::default()
        };
        let unit = engine.compile_script(source, options.clone()).unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        assert_selected_sections(&artifact.bytes, &profile);
        assert!(engine
            .emit_c(&unit)
            .unwrap_err()
            .to_string()
            .contains("custom Intl projection"));
        assert!(engine
            .observe_script(
                source,
                options.clone(),
                RunOptions {
                    backend: ExecutionBackend::SpecExec,
                    ..RunOptions::default()
                }
            )
            .unwrap_err()
            .to_string()
            .contains("custom Intl projection"));
        for directive in ["", "\"use strict\";\n"] {
            let source = format!("{directive}{source}");
            let observed = engine
                .observe_script(&source, options.clone(), wasm_run())
                .unwrap();
            let expected = if with_list {
                ObservedJsValue::Number(ObservedNumber::from_f64(262.0))
            } else {
                ObservedJsValue::Boolean(true)
            };
            assert_eq!(observed.completion, ObservedCompletion::Normal(expected));
        }
        let module = format!("{source}\nexport const selected = 262;");
        assert_eq!(
            engine
                .observe_module(&module, options, wasm_run())
                .unwrap()
                .completion,
            ObservedCompletion::Normal(ObservedJsValue::Undefined)
        );
    }
}

#[test]
fn display_names_and_three_component_projections_emit_selected_tables_and_run_real_consumers() {
    use lila_intl::{
        CanonicalLocaleId, CheckedDisplayNamesConfiguration, DisplayNameRequest,
        DisplayNamesDataImage, DisplayNamesError, DisplayNamesFallback, DisplayNamesSelection,
        DisplayNamesStyle, LocaleDataImage,
    };
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let display_source =
        include_str!("fixtures/intl_compilation_profile/projected_display_names.js");
    let combined_source =
        include_str!("fixtures/intl_compilation_profile/projected_list_relative_time.js");
    for with_other_filters in [false, true] {
        let id = CustomProfileId::parse("displaynames-composition").unwrap();
        let profile = IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                id.clone(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["pl", "fr"][..]),
                Some(&["ja", "fr"]),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap(),
        );
        let selection = IntlDataSelection::new(profile.clone());
        let selected = selection.selected().unwrap();
        let unfiltered = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let full = unfiltered.selected().unwrap();
        let frames = selected.component_sections();
        let frame = |name| {
            frames
                .iter()
                .find(|(key, _)| *key == name)
                .unwrap()
                .1
                .clone()
        };
        for ((name, bytes), (full_name, full_bytes)) in frames.iter().zip(full.component_sections())
        {
            assert_eq!(*name, full_name);
            if *name == lila_intl::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION
                || (with_other_filters
                    && matches!(
                        *name,
                        lila_intl::INTL_LIST_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION
                    ))
            {
                assert!(
                    bytes.len() < full_bytes.len(),
                    "selected physical tables shrink: {name}"
                );
            } else {
                assert_eq!(
                    bytes.as_ref(),
                    full_bytes.as_ref(),
                    "complete unfiltered domain: {name}"
                );
            }
        }
        assert_ne!(selected.identity(), full.identity());
        let locale =
            LocaleDataImage::from_bytes(frame(lila_intl::INTL_LOCALE_DATA_CUSTOM_SECTION)).unwrap();
        let display = DisplayNamesDataImage::from_bytes(
            frame(lila_intl::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION),
            &locale,
        )
        .unwrap();
        let profiles = display.profiles();
        assert_eq!(
            profiles
                .available_locales()
                .map(CanonicalLocaleId::as_str)
                .collect::<Vec<_>>(),
            ["en-US", "fr", "ja"]
        );
        assert!(profiles
            .admit(CanonicalLocaleId::from_data("de").unwrap())
            .is_err());
        let request = DisplayNameRequest::new(
            CheckedDisplayNamesConfiguration::new(
                profiles
                    .admit(CanonicalLocaleId::from_data("fr").unwrap())
                    .unwrap(),
                DisplayNamesSelection::Currency,
                DisplayNamesStyle::Short,
                DisplayNamesFallback::None,
            ),
            "USD".encode_utf16().collect::<Vec<_>>().into_boxed_slice(),
        )
        .unwrap();
        assert_eq!(
            display.display_name(&request).unwrap().name(),
            Some("dollar des États-Unis")
        );
        let second = DisplayNamesDataImage::from_bytes(display.bytes(), &locale).unwrap();
        assert_eq!(
            second.display_name(&request),
            Err(DisplayNamesError::InvalidResolvedLocale)
        );
        let decoded = lila_intl::decode_display_name_request(
            &lila_intl::encode_display_name_request(&request).unwrap(),
            &second.profiles(),
        )
        .unwrap();
        assert_eq!(
            second.display_name(&decoded).unwrap().name(),
            Some("dollar des États-Unis")
        );

        let source = if with_other_filters {
            format!("{combined_source}\n{display_source}")
        } else {
            format!("{display_source}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('hi').resolvedOptions().locale !== 'hi') throw 'unfiltered List and RelativeTime domains';\n262;")
        };
        let options = CompileOptions {
            intl_profile: profile.clone(),
            ..CompileOptions::default()
        };
        let unit = engine.compile_script(&source, options.clone()).unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        assert_selected_sections(&artifact.bytes, &profile);
        for directive in ["", "\"use strict\";\n"] {
            let source = format!("{directive}{source}");
            assert_eq!(
                engine
                    .observe_script(&source, options.clone(), wasm_run())
                    .unwrap()
                    .completion,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(
                    262.0
                )))
            );
        }
        let module = format!("{source}\nexport const selected = 262;");
        assert_eq!(
            engine
                .observe_module(&module, options, wasm_run())
                .unwrap()
                .completion,
            ObservedCompletion::Normal(ObservedJsValue::Undefined)
        );
    }
    for unavailable in ["fr-FR", "pl", "iw"] {
        let selection = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                CustomProfileId::parse("displaynames-unavailable").unwrap(),
                None,
                None,
                Some(&[unavailable]),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap(),
        ));
        assert!(
            selection.selected().is_err(),
            "projection must admit real canonical source rows: {unavailable}"
        );
    }
}

#[test]
fn duration_only_and_four_component_projections_emit_and_execute_the_selected_graph() {
    use lila_intl::{DurationDataImage, ListDataImage, LocaleDataImage, NumberProfilesDataImage};
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let duration_source = include_str!("fixtures/intl_compilation_profile/projected_duration.js");
    let combined_source =
        include_str!("fixtures/intl_compilation_profile/projected_list_relative_time.js");
    let display_source =
        include_str!("fixtures/intl_compilation_profile/projected_display_names.js");
    for with_other_filters in [false, true] {
        let id = CustomProfileId::parse("duration-composition").unwrap();
        let profile = IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                id.clone(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["pl", "fr"][..]),
                with_other_filters.then_some(&["ja", "fr"][..]),
                Some(&["sr", "fr"]),
                None,
                None,
                None,
                None,
            )
            .unwrap(),
        );
        let selection = IntlDataSelection::new(profile.clone());
        let selected = selection.selected().unwrap();
        let full_selection = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let full = full_selection.selected().unwrap();
        let frames = selected.component_sections();
        for ((name, bytes), (full_name, full_bytes)) in frames.iter().zip(full.component_sections())
        {
            assert_eq!(*name, full_name);
            if *name == lila_intl::INTL_DURATION_DATA_CUSTOM_SECTION
                || (with_other_filters
                    && matches!(
                        *name,
                        lila_intl::INTL_LIST_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION
                    ))
            {
                assert!(
                    bytes.len() < full_bytes.len(),
                    "actual selected rows shrink: {name}"
                );
            } else {
                assert_eq!(
                    bytes.as_ref(),
                    full_bytes.as_ref(),
                    "unfiltered component: {name}"
                );
            }
        }
        assert_ne!(selected.identity(), full.identity());
        let frame = |name| {
            frames
                .iter()
                .find(|(key, _)| *key == name)
                .unwrap()
                .1
                .clone()
        };
        let locale =
            LocaleDataImage::from_bytes(frame(lila_intl::INTL_LOCALE_DATA_CUSTOM_SECTION)).unwrap();
        let lists =
            ListDataImage::from_bytes(frame(lila_intl::INTL_LIST_DATA_CUSTOM_SECTION), &locale)
                .unwrap();
        let numbers = NumberProfilesDataImage::from_bytes(
            frame(lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION),
            &locale,
            &lists,
        )
        .unwrap();
        let duration = DurationDataImage::from_bytes(
            frame(lila_intl::INTL_DURATION_DATA_CUSTOM_SECTION),
            &locale,
            &numbers,
            &lists,
        )
        .unwrap();
        let duration_profiles = duration.profiles();
        assert_eq!(
            duration_profiles
                .available_locales()
                .map(|locale| locale.as_str())
                .collect::<Vec<_>>(),
            ["en-US", "fr", "sr"]
        );
        let source = if with_other_filters {
            format!("{combined_source}\n{display_source}\n{duration_source}\nif (Intl.ListFormat.supportedLocalesOf(['fr', 'sr']).length !== 0 || new Intl.ListFormat('sr').resolvedOptions().locale !== 'en-US') throw 'private Duration List rows escaped into public List support';\n262;")
        } else {
            format!("{duration_source}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('hi').resolvedOptions().locale !== 'hi' || new Intl.DisplayNames('ja', {{type: 'region'}}).resolvedOptions().locale !== 'ja') throw 'unfiltered component domains';\n262;")
        };
        let options = CompileOptions {
            intl_profile: profile.clone(),
            ..CompileOptions::default()
        };
        let unit = engine.compile_script(&source, options.clone()).unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        assert_selected_sections(&artifact.bytes, &profile);
        let outcome = engine
            .run_compiled_unit(&unit, &source, wasm_run())
            .unwrap();
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("number(262"), "{}", outcome.note);
        for directive in ["", "\"use strict\";\n"] {
            let observed = engine
                .observe_script(&format!("{directive}{source}"), options.clone(), wasm_run())
                .unwrap();
            assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
            assert_eq!(
                observed.completion,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(
                    262.0
                )))
            );
        }
        assert_eq!(
            engine
                .observe_module(
                    &format!("{source}\nexport const selected = 262;"),
                    options,
                    wasm_run()
                )
                .unwrap()
                .completion,
            ObservedCompletion::Normal(ObservedJsValue::Undefined)
        );
    }
    for unavailable in ["fr-FR", "iw", "pl"] {
        let selection = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                CustomProfileId::parse("duration-unavailable").unwrap(),
                None,
                None,
                None,
                Some(&[unavailable]),
                None,
                None,
                None,
                None,
            )
            .unwrap(),
        ));
        assert!(
            selection.selected().is_err(),
            "selected Duration needs an exact canonical pinned row: {unavailable}"
        );
    }
}

#[test]
fn coupled_number_only_and_five_component_projections_reach_real_library_artifacts_and_hosts() {
    use lila_intl::{
        CustomListProfile, DurationDataImage, ListDataImage, LocaleDataImage, LocaleId,
        NumberProfilesDataImage, RelativeTimeDataImage,
    };
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_number.js");
    for with_other_filters in [false, true] {
        let id = CustomProfileId::parse("number-composition").unwrap();
        let profile = IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                id.clone(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["pl", "fr"][..]),
                with_other_filters.then_some(&["ja", "fr"][..]),
                with_other_filters.then_some(&["sr", "fr"][..]),
                Some(&["pl", "es"]),
                None,
                None,
                None,
            )
            .unwrap(),
        );
        let selection = IntlDataSelection::new(profile.clone());
        let selected = selection.selected().unwrap();
        let full_selection = IntlDataSelection::new(IntlCompilationProfile::Custom(id.clone()));
        let full = full_selection.selected().unwrap();
        let frames = selected.component_sections();
        for ((name, bytes), (full_name, full_bytes)) in frames.iter().zip(full.component_sections())
        {
            assert_eq!(*name, full_name);
            if *name == lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION
                || (with_other_filters
                    && matches!(
                        *name,
                        lila_intl::INTL_LIST_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_DURATION_DATA_CUSTOM_SECTION
                    ))
            {
                assert!(
                    bytes.len() < full_bytes.len(),
                    "real selected component is smaller: {name}"
                );
            } else {
                assert_eq!(
                    bytes.as_ref(),
                    full_bytes.as_ref(),
                    "unfiltered component remains exact: {name}"
                );
            }
        }
        assert_ne!(selected.identity(), full.identity());
        let frame = |name| {
            frames
                .iter()
                .find(|(key, _)| *key == name)
                .unwrap()
                .1
                .clone()
        };
        let locale =
            LocaleDataImage::from_bytes(frame(lila_intl::INTL_LOCALE_DATA_CUSTOM_SECTION)).unwrap();
        let lists =
            ListDataImage::from_bytes(frame(lila_intl::INTL_LIST_DATA_CUSTOM_SECTION), &locale)
                .unwrap();
        let numbers = NumberProfilesDataImage::from_bytes(
            frame(lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION),
            &locale,
            &lists,
        )
        .unwrap();
        assert_eq!(
            numbers
                .profiles()
                .available_locales()
                .iter()
                .map(|locale| locale.as_ref())
                .collect::<Vec<_>>(),
            ["en-US", "es", "pl"]
        );
        let foreign_lists = ListDataImage::for_custom_projection(
            &CustomListProfile::new(id.clone(), &["ja"]).unwrap(),
            &locale,
        )
        .unwrap();
        assert!(
            NumberProfilesDataImage::from_bytes(
                frame(lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION),
                &locale,
                &foreign_lists
            )
            .is_err(),
            "actual Number frame is bound to its chosen List association source"
        );
        let foreign_relative = RelativeTimeDataImage::for_custom_projection(
            &id,
            &[LocaleId::parse("fr").unwrap()],
            &locale,
            &numbers,
        )
        .unwrap_err();
        assert!(
            foreign_relative.to_string().contains("catalogue differs"),
            "{foreign_relative}"
        );
        let foreign_duration = DurationDataImage::for_custom_projection(
            &id,
            &[LocaleId::parse("fr").unwrap()],
            &locale,
            &numbers,
            &lists,
        )
        .unwrap_err();
        assert!(
            foreign_duration.to_string().contains("catalogue differs"),
            "{foreign_duration}"
        );
        let source = if with_other_filters {
            format!("{fixture}\nif (Intl.ListFormat.supportedLocalesOf(['fr', 'sr']).length !== 0 || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.DisplayNames('de', {{type: 'region'}}).resolvedOptions().locale !== 'en-US' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'en-US') throw 'independent selected component domains';\n262;")
        } else {
            format!("{fixture}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.DisplayNames('ja', {{type: 'region'}}).resolvedOptions().locale !== 'ja' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'hi') throw 'unfiltered component domains';\n262;")
        };
        let options = CompileOptions {
            intl_profile: profile.clone(),
            ..CompileOptions::default()
        };
        let unit = engine.compile_script(&source, options.clone()).unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        assert_selected_sections(&artifact.bytes, &profile);
        let outcome = engine
            .run_compiled_unit(&unit, &source, wasm_run())
            .unwrap();
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("number(262"), "{}", outcome.note);
        for directive in ["", "\"use strict\";\n"] {
            let outcome = engine
                .observe_script(&format!("{directive}{source}"), options.clone(), wasm_run())
                .unwrap();
            assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
            assert_eq!(
                outcome.completion,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(
                    262.0
                )))
            );
        }
        let module = format!("{source}\nexport const selected = 262;");
        let outcome = engine.observe_module(&module, options, wasm_run()).unwrap();
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Undefined)
        );
    }
    for unavailable in ["zz", "es-XX"] {
        let selection = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                CustomProfileId::parse("number-unavailable").unwrap(),
                None,
                None,
                None,
                None,
                Some(&[unavailable]),
                None,
                None,
                None,
            )
            .unwrap(),
        ));
        assert!(
            selection.selected().is_err(),
            "public Number projection requires a real canonical pinned row: {unavailable}"
        );
    }
}

#[test]
fn datetime_only_and_six_component_projections_bind_real_pools_plans_and_library_artifacts() {
    use lila_intl::{
        CanonicalLocaleId, DateTimeComponents, DateTimeDataImage, DateTimeDefaults,
        DateTimeExactInput, DateTimeFormatError, DateTimeFormatMatcher, DateTimeFormatRequest,
        DateTimeHourCyclePreference, DateTimeInput, DateTimeLocaleMatcher, DateTimeLocaleRequest,
        DateTimeNumericWidth, DateTimePlanRequest, DateTimeRequired, DateTimeStyleSelection,
        DateTimeSupportedLocalesRequest, DateTimeValueKind, LocaleDataImage,
        NamedTimeZoneDataImage, TimeZoneId, TimeZoneSelection,
    };
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let fixture = include_str!("fixtures/intl_compilation_profile/projected_datetime.js");
    for with_other_filters in [false, true] {
        let id = CustomProfileId::parse("date-composition").unwrap();
        let profile = IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                id.clone(),
                with_other_filters.then_some(&["es", "he"][..]),
                with_other_filters.then_some(&["fr", "pl"][..]),
                with_other_filters.then_some(&["fr", "ja"][..]),
                with_other_filters.then_some(&["fr", "sr"][..]),
                with_other_filters.then_some(&["es", "pl"][..]),
                Some(&["zh", "ar-EG"]),
                None,
                None,
            )
            .unwrap(),
        );
        let selection = IntlDataSelection::new(profile.clone());
        let selected = selection.selected().unwrap();
        let complete = IntlDataSelection::new(IntlCompilationProfile::Custom(id));
        let full = complete.selected().unwrap();
        let frames = selected.component_sections();
        let full_frames = full.component_sections();
        for ((name, bytes), (full_name, full_bytes)) in frames.iter().zip(&full_frames) {
            assert_eq!(name, full_name);
            if *name == lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION
                || (with_other_filters
                    && matches!(
                        *name,
                        lila_intl::INTL_LIST_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_RELATIVE_TIME_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_DISPLAY_NAMES_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_DURATION_DATA_CUSTOM_SECTION
                            | lila_intl::INTL_NUMBER_DATA_CUSTOM_SECTION
                    ))
            {
                assert!(
                    bytes.len() < full_bytes.len(),
                    "actual selected pool closure shrinks: {name}"
                );
            } else {
                assert_eq!(
                    bytes.as_ref(),
                    full_bytes.as_ref(),
                    "unfiltered component bytes: {name}"
                );
            }
        }
        assert_ne!(selected.identity(), full.identity());
        let frame = |name| {
            frames
                .iter()
                .find(|(key, _)| *key == name)
                .unwrap()
                .1
                .clone()
        };
        let locale =
            LocaleDataImage::from_bytes(frame(lila_intl::INTL_LOCALE_DATA_CUSTOM_SECTION)).unwrap();
        let named = NamedTimeZoneDataImage::from_bytes(frame(
            lila_intl::INTL_NAMED_TIME_ZONE_DATA_CUSTOM_SECTION,
        ))
        .unwrap();
        let date = DateTimeDataImage::from_bytes(
            frame(lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION),
            &locale,
            &named,
        )
        .unwrap();
        let requested = ["ar-EG", "zh-Hans-CN", "en-US", "fr"]
            .into_iter()
            .map(|name| CanonicalLocaleId::from_data(name).unwrap())
            .collect();
        assert_eq!(
            date.supported_locales(DateTimeSupportedLocalesRequest {
                requested,
                matcher: DateTimeLocaleMatcher::Lookup,
            })
            .unwrap()
            .locales
            .iter()
            .map(CanonicalLocaleId::as_str)
            .collect::<Vec<_>>(),
            ["ar-EG", "zh-Hans-CN", "en-US"]
        );
        let resolved = date
            .resolve_locale(DateTimeLocaleRequest {
                requested: Vec::new(),
                matcher: DateTimeLocaleMatcher::Lookup,
                calendar: None,
                numbering_system: None,
                hour_cycle: DateTimeHourCyclePreference::Default,
            })
            .unwrap();
        let request = DateTimePlanRequest {
            locale: resolved.clone(),
            time_zone: TimeZoneSelection::Named(TimeZoneId::parse("UTC").unwrap()),
            selection: DateTimeStyleSelection::Components(DateTimeComponents {
                year: Some(DateTimeNumericWidth::Numeric),
                ..Default::default()
            }),
            matcher: DateTimeFormatMatcher::Basic,
            required: DateTimeRequired::Any,
            defaults: DateTimeDefaults::Date,
        };
        let plan = date.select_plan(request.clone()).unwrap().plan;
        let input =
            DateTimeInput::Exact(DateTimeExactInput::new(DateTimeValueKind::Legacy, 0, 0).unwrap());
        let portable = DateTimeDataImage::from_bytes(date.bytes(), &locale, &named).unwrap();
        assert_eq!(
            portable
                .format_parts(DateTimeFormatRequest {
                    plan: plan.clone(),
                    input
                })
                .unwrap()
                .to_formatted_string(),
            "1970"
        );
        let full_date = DateTimeDataImage::from_bytes(
            full_frames
                .iter()
                .find(|(name, _)| *name == lila_intl::INTL_DATETIME_DATA_CUSTOM_SECTION)
                .unwrap()
                .1
                .clone(),
            &locale,
            &named,
        )
        .unwrap();
        assert!(
            matches!(
                full_date.format_parts(DateTimeFormatRequest { plan, input }),
                Err(DateTimeFormatError::InvalidPlan(_))
            ),
            "selected opaque plan cannot use the complete foreign image"
        );
        let mut absent = request;
        absent.locale.data_locale = CanonicalLocaleId::from_data("fr").unwrap();
        assert!(
            date.select_plan(absent).is_err(),
            "excluded rows cannot mint a selected primitive plan"
        );
        let source = if with_other_filters {
            format!("{fixture}\nif (Intl.ListFormat.supportedLocalesOf(['fr', 'sr']).length !== 0 || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.DisplayNames('de', {{type: 'region'}}).resolvedOptions().locale !== 'en-US' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'en-US' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'en-US' || new Intl.PluralRules('ar').resolvedOptions().locale !== 'en-US') throw 'six independent selected component domains';\n262;")
        } else {
            format!("{fixture}\nif (new Intl.ListFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.DisplayNames('ja', {{type: 'region'}}).resolvedOptions().locale !== 'ja' || new Intl.DurationFormat('hi').resolvedOptions().locale !== 'hi' || new Intl.NumberFormat('ar').resolvedOptions().locale !== 'ar' || new Intl.PluralRules('ar').resolvedOptions().locale !== 'ar') throw 'unfiltered component domains';\n262;")
        };
        let options = CompileOptions {
            intl_profile: profile.clone(),
            ..CompileOptions::default()
        };
        let unit = engine.compile_script(&source, options.clone()).unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        assert_selected_sections(&artifact.bytes, &profile);
        let outcome = engine
            .run_compiled_unit(&unit, &source, wasm_run())
            .unwrap();
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("number(262"), "{}", outcome.note);
        for directive in ["", "\"use strict\";\n"] {
            let outcome = engine
                .observe_script(&format!("{directive}{source}"), options.clone(), wasm_run())
                .unwrap();
            assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
            assert_eq!(
                outcome.completion,
                ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(
                    262.0
                )))
            );
        }
        let module = format!("{source}\nexport const selected = 262;");
        let outcome = engine.observe_module(&module, options, wasm_run()).unwrap();
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Undefined)
        );
    }
    for unavailable in ["zz", "fr-FR"] {
        let selection = IntlDataSelection::new(IntlCompilationProfile::CustomProjection(
            CustomIntlProfile::new(
                CustomProfileId::parse("date-unavailable").unwrap(),
                None,
                None,
                None,
                None,
                None,
                Some(&[unavailable]),
                None,
                None,
            )
            .unwrap(),
        ));
        assert!(
            selection.selected().is_err(),
            "DateTime filter requires an actual canonical pinned row: {unavailable}"
        );
    }
}
