//! Native source remains parsed/lowered/emitted while compiled code shares fresh data.
use lila_engine::{
    CompileOptions, Engine, ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder,
    RunOptions,
};
fn succeeds(source: &str) {
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(1.0)))
    );
}
#[test]
fn direct_eval_template_data_keeps_source_context_regex_and_fresh_object_identity() {
    succeeds(
        r#"
      var token = 'template_' + Date.now();
      function invoke(source) { let secret = 17; return eval(source); }
      for (var i = 0; i < 4; i++) {
        if (invoke('secret + 1 /* ' + token + i + ' */') !== 18) throw new Error('caller context');
        var a = invoke('/' + token + i + '/');
        var b = invoke('/' + token + i + '/');
        if (a === b || a.source !== token + i || b.source !== token + i) throw new Error('regex data/identity');
        a.lastIndex = 7;
        if (b.lastIndex !== 0) throw new Error('distinct regex state');
      }
      1;
    "#,
    );
}
#[test]
fn function_template_data_preserves_each_to_string_source_and_later_calls() {
    succeeds(
        r#"
      var token = 'template_' + Date.now();
      var a = Function('return "' + token + '_first";');
      var b = Function('return "' + token + '_other";');
      if (a() !== token + '_first' || b() !== token + '_other' || a() !== token + '_first') throw new Error('closure data');
      if (a.toString().indexOf(token + '_first') < 0 || b.toString().indexOf(token + '_other') < 0) throw new Error('source identity');
      1;
    "#,
    );
}
#[test]
fn global_script_template_data_uses_actual_realm_and_keeps_each_payload() {
    succeeds(
        r#"
      var token = 'template_' + Date.now();
      var indirect = eval;
      indirect('globalThis["' + token + '_first"] = 13;');
      indirect('globalThis["' + token + '_other"] = 29;');
      if (globalThis[token + '_first'] !== 13 || globalThis[token + '_other'] !== 29) throw new Error('global data');
      1;
    "#,
    );
}

#[path = "aot_native_program_templates/catalog_overlays.rs"]
mod catalog_overlays;
