use crate::{
    configure_compilation_jobs, CompileOptions, Engine, ExecutionBackend, HostOutputEvent,
    HostSurfacePolicy, ObservedCompletion, ObservedJsValue, RealmBuilder, RunOptions,
};

struct Case {
    pattern: Vec<u16>,
    flags: &'static str,
    input: Vec<u16>,
    expected: Option<Vec<u16>>,
}

fn units(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}
fn joined(parts: &[&[u16]]) -> Vec<u16> {
    parts.concat()
}
fn js_string(value: &[u16]) -> String {
    let mut text = String::from("\"");
    for unit in value {
        text.push_str(&format!("\\u{unit:04X}"));
    }
    text.push('"');
    text
}
fn cases() -> Vec<Case> {
    let high = vec![0xd83d];
    let low = vec![0xdc38];
    let frog = vec![0xd83d, 0xdc38];
    let fixed_high = units(r"\uD83D");
    let fixed_low = units(r"\uDC38");
    let mut result = Vec::new();
    for flags in ["", "u", "v"] {
        let mut patterns = vec![
            (frog.clone(), true),
            (joined(&[&fixed_high, &low]), false),
            (joined(&[&high, &fixed_low]), false),
            (joined(&[&fixed_high, &fixed_low]), true),
        ];
        if !flags.is_empty() {
            patterns.extend([
                (joined(&[&units(r"\u{D83D}"), &low]), false),
                (joined(&[&high, &units(r"\u{DC38}")]), false),
            ]);
        }
        for (pattern, paired_unicode) in patterns {
            let whole = flags.is_empty() || paired_unicode;
            result.push(Case {
                pattern: pattern.clone(),
                flags,
                input: frog.clone(),
                expected: whole.then(|| frog.clone()),
            });
            let optional = joined(&[&pattern, &[63]]);
            result.push(Case {
                pattern: optional.clone(),
                flags,
                input: frog.clone(),
                expected: whole.then(|| frog.clone()),
            });
            result.push(Case {
                pattern: optional.clone(),
                flags,
                input: vec![],
                expected: (!flags.is_empty() && paired_unicode).then(Vec::new),
            });
            result.push(Case {
                pattern: optional,
                flags,
                input: high.clone(),
                expected: Some(if !flags.is_empty() && paired_unicode {
                    vec![]
                } else {
                    high.clone()
                }),
            });
        }
        let mut classes = vec![
            (frog.clone(), true),
            (joined(&[&fixed_high, &low]), false),
            (joined(&[&high, &fixed_low]), false),
            (joined(&[&fixed_low, &high]), false),
        ];
        if !flags.is_empty() {
            classes.extend([
                (joined(&[&units(r"\u{D83D}"), &low]), false),
                (joined(&[&high, &units(r"\u{DC38}")]), false),
            ]);
        }
        for (middle, paired_unicode) in classes {
            let pattern = joined(&[&[91], &middle, &[93]]);
            let expected = if flags.is_empty() {
                Some(high.clone())
            } else if paired_unicode {
                Some(frog.clone())
            } else {
                None
            };
            result.push(Case {
                pattern: pattern.clone(),
                flags,
                input: frog.clone(),
                expected,
            });
            if !paired_unicode {
                result.push(Case {
                    pattern: pattern.clone(),
                    flags,
                    input: high.clone(),
                    expected: Some(high.clone()),
                });
                result.push(Case {
                    pattern,
                    flags,
                    input: low.clone(),
                    expected: Some(low.clone()),
                });
            }
        }
    }
    let marker = units("\u{F0000}D83D");
    result.push(Case {
        pattern: marker.clone(),
        flags: "u",
        input: marker.clone(),
        expected: Some(marker),
    });
    result
}

const CHECK: &str = r#"
function fromUnits(units) {
  var text=''; for (var i=0;i<units.length;i++) text+=String.fromCharCode(units[i]); return text;
}
function sameUnits(text, units) {
  if(text.length!==units.length) return false;
  for(var i=0;i<units.length;i++) if(text.charCodeAt(i)!==units[i]) return false;
  return true;
}
function check(re, input, expected, pattern, label) {
  if(!sameUnits(re.source,pattern)) throw new Error('source units '+label);
  var match=re.exec(fromUnits(input));
  if(expected===null) { if(match!==null) throw new Error('expected null '+label); return; }
  if(match===null || match.length!==1 || match.index!==0 || !sameUnits(match[0],expected)) {
    throw new Error('match units '+label);
  }
}
"#;

fn assert_native(source: &str) {
    configure_compilation_jobs(1).unwrap();
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| {
                panic!("raw UTF-16 RegExp native execution: {error}\n{source}")
            });
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true))
        );
        assert_eq!(
            outcome.output_events,
            vec![HostOutputEvent::PrintLine("raw-regexp-ok".into())]
        );
    }
}

#[test]
fn original_raw_unicode_staging_bodies_keep_every_assertion() {
    assert_native(include_str!(
        "../../tests/fixtures/regexp_raw_utf16/original_staging_bodies.js"
    ));
}

#[test]
fn literal_prepared_and_static_constructor_patterns_share_actual_utf16_units() {
    let mut source = String::from(CHECK);
    for (index, case) in cases().iter().enumerate() {
        let literal = joined(&[&[47], &case.pattern, &[47], &units(case.flags)]);
        let mut expressions = vec![
            format!("eval({})", js_string(&literal)),
            format!("new RegExp({}, {:?})", js_string(&case.pattern), case.flags),
        ];
        // A Rust source string can directly spell exactly the well-formed
        // scalar patterns. Lone source units enter the literal lexer via eval.
        if let Ok(pattern) = String::from_utf16(&case.pattern) {
            expressions.push(format!("/{pattern}/{}", case.flags));
        }
        let expected = case
            .expected
            .as_ref()
            .map_or_else(|| "null".into(), |units| format!("{units:?}"));
        for (path, expression) in expressions.iter().enumerate() {
            source.push_str(&format!(
                "check({expression}, {:?}, {expected}, {:?}, 'case {index} path {path}');\n",
                case.input, case.pattern
            ));
        }
    }
    source.push_str("print('raw-regexp-ok'); true;");
    assert_native(&source);
}

#[test]
fn runtime_only_patterns_keep_raw_escaped_quantifier_and_class_domains() {
    let mut source = String::from(CHECK);
    // Only numeric code-unit arrays spell patterns here; no matching source
    // string literal can select a precompiled finite candidate-cache row.
    for (index, case) in cases().iter().enumerate() {
        let expected = case
            .expected
            .as_ref()
            .map_or_else(|| "null".into(), |units| format!("{units:?}"));
        source.push_str(&format!("check(new RegExp(fromUnits({:?}), fromUnits({:?})), {:?}, {expected}, {:?}, 'runtime {index}');\n",case.pattern,units(case.flags),case.input,case.pattern));
    }
    source.push_str("print('raw-regexp-ok'); true;");
    assert_native(&source);
}
