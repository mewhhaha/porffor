use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn compile_options() -> CompileOptions {
    CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        ..CompileOptions::default()
    }
}

fn run_options() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(30_000),
        ..RunOptions::default()
    }
}

fn assert_normal_zone_script(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(&script, compile_options(), run_options())
            .expect("Temporal zone controls must execute through Wasm AOT");
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observation.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{script}"
        );
        assert_eq!(
            observation.output_events,
            vec![HostOutputEvent::PrintLine("ok".into())],
            "{script}"
        );
    }
}

fn assert_named_zone_value(
    operation: &str,
    epoch_nanoseconds: i128,
    hour: u8,
    offset: &str,
    identifier: &str,
) {
    assert_normal_zone_script(&format!(
        r#"var value = {operation};
if (value.epochNanoseconds !== {epoch_nanoseconds}n || value.hour !== {hour} ||
    value.offset !== {offset:?} || value.timeZoneId !== {identifier:?} ||
    value.calendarId !== 'iso8601') throw new Error('named zone epoch/projection/identity');
print('ok');
262;
"#
    ));
}

#[test]
fn fixed_and_utc_zones_keep_normal_completion_and_exact_results() {
    assert_normal_zone_script(include_str!(
        "../fixtures/temporal_zone_authority/fixed_and_utc.js"
    ));
}

#[test]
fn unknown_slash_and_nonslash_names_are_catchable_range_errors() {
    assert_normal_zone_script(include_str!(
        "../fixtures/temporal_zone_authority/unknown_names.js"
    ));
}

#[test]
fn named_annotations_preserve_identity_and_apply_actual_offset_policies() {
    for source in [
        "1970-01-01T12:00+01:00[Europe/Paris]",
        "1970-01-01T12:00+01:00[!Europe/Paris]",
        "1970-01-01T12:00[Europe/Paris]",
    ] {
        assert_named_zone_value(
            &format!("Temporal.ZonedDateTime.from({source:?})"),
            39_600_000_000_000,
            12,
            "+01:00",
            "Europe/Paris",
        );
    }
    assert_named_zone_value(
        "Temporal.ZonedDateTime.from('1970-01-01T12:00Z[Europe/Paris]')",
        43_200_000_000_000,
        13,
        "+01:00",
        "Europe/Paris",
    );
    assert_named_zone_value(
        "Temporal.ZonedDateTime.from('1970-01-01[Europe/Paris]')",
        -3_600_000_000_000,
        0,
        "+01:00",
        "Europe/Paris",
    );
    assert_normal_zone_script(
        r#"var caught=false;
         try { Temporal.ZonedDateTime.from('1970-01-01T12:00+02:00[Europe/Paris]', {offset:'reject'}); }
         catch(error) { caught=error instanceof RangeError; }
         if(!caught) throw new Error('mismatched reject must be a catchable RangeError');
         print('ok');
         262;"#,
    );
    for (offset, epoch, hour) in [
        ("use", 36_000_000_000_000, 11),
        ("prefer", 39_600_000_000_000, 12),
        ("ignore", 39_600_000_000_000, 12),
    ] {
        assert_named_zone_value(
            &format!(
                "Temporal.ZonedDateTime.from('1970-01-01T12:00+02:00[Europe/Paris]', {{offset:{offset:?}}})"
            ),
            epoch,
            hour,
            "+01:00",
            "Europe/Paris",
        );
    }
}

#[test]
fn named_direct_constructor_projects_its_exact_epoch() {
    assert_named_zone_value(
        "new Temporal.ZonedDateTime(0n, 'Europe/Paris')",
        0,
        1,
        "+01:00",
        "Europe/Paris",
    );
}

#[test]
fn named_property_bag_resolves_its_matching_zone_offset() {
    assert_named_zone_value(
        "Temporal.ZonedDateTime.from({year:1970, month:1, day:1, hour:12, offset:'+01:00', timeZone:'Europe/Paris'})",
        39_600_000_000_000,
        12,
        "+01:00",
        "Europe/Paris",
    );
}

#[test]
fn duration_relative_to_uses_actual_named_day_spans() {
    assert_normal_zone_script(
        r#"var day=new Temporal.Duration(0,0,0,1);
         var spring=day.total({unit:'hour',relativeTo:'2020-03-29T00:00+01:00[Europe/Paris]'});
         var autumn=day.total({unit:'hour',relativeTo:'2020-10-25T00:00+02:00[Europe/Paris]'});
         if(spring!==23 || autumn!==25) throw new Error('named relative day spans');
         print('ok');
         262;"#,
    );
}

#[test]
fn named_nonslash_alias_preserves_identifier_and_primary_equality() {
    assert_normal_zone_script(
        r#"var alias=new Temporal.ZonedDateTime(0n,'CET');
         var primary=new Temporal.ZonedDateTime(0n,'Europe/Brussels');
         if(alias.epochNanoseconds!==0n || alias.hour!==1 || alias.offset!=='+01:00' ||
            alias.timeZoneId!=='CET' || !alias.equals(primary)) throw new Error('named alias identity');
         print('ok');
         262;"#,
    );
}
