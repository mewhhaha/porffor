use std::fs;
use std::path::Path;

const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/intl-date-time-format-range-locals-ownership.md"
);
const TASK: &str = include_str!("../../../tasks/23-intl402.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn rust_code(source: &str, retain_literals: bool) -> String {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if bytes[offset] == b'"' {
            let start = offset;
            offset += 1;
            let mut escaped = false;
            while offset < bytes.len() {
                let byte = bytes[offset];
                offset += 1;
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    break;
                }
            }
            if retain_literals {
                code.push_str(&source[start..offset]);
            } else {
                code.push(' ');
            }
            continue;
        }
        if bytes[offset] == b'r' {
            let start = offset;
            let mut quote = offset + 1;
            while bytes.get(quote) == Some(&b'#') {
                quote += 1;
            }
            if bytes.get(quote) == Some(&b'"') {
                let hashes = quote - start - 1;
                offset = quote + 1;
                while offset < bytes.len() {
                    if bytes[offset] == b'"'
                        && bytes
                            .get(offset + 1..offset + 1 + hashes)
                            .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
                    {
                        offset += 1 + hashes;
                        break;
                    }
                    offset += 1;
                }
                if retain_literals {
                    code.push_str(&source[start..offset]);
                } else {
                    code.push(' ');
                }
                continue;
            }
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            if !retain_literals {
                code.push(' ');
            }
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            if !retain_literals {
                code.push(' ');
            }
            offset += 2;
            let mut depth = 1;
            while offset < bytes.len() && depth != 0 {
                if bytes.get(offset..offset + 2) == Some(b"/*") {
                    depth += 1;
                    offset += 2;
                } else if bytes.get(offset..offset + 2) == Some(b"*/") {
                    depth -= 1;
                    offset += 2;
                } else {
                    offset += 1;
                }
            }
            assert_eq!(depth, 0, "unterminated block comment");
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"r#") {
            offset += 2;
            continue;
        }
        let character = source[offset..].chars().next().unwrap();
        if character.is_whitespace() {
            if !retain_literals {
                code.push(' ');
            }
        } else {
            code.push(character);
        }
        offset += character.len_utf8();
    }
    code
}

fn normalized_rust(source: &str) -> String {
    rust_code(source, false)
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + identifier.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn count_identifier_in_rust_sources(dir: &Path, identifier: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_identifier_in_rust_sources(&path, identifier);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_identifier_count(&rust_code(&source, false), identifier)
        })
        .sum()
}

#[test]
fn range_local_carriers_are_private_non_capability_types() {
    let declarations = normalized_rust(bounded(
        DTF_SOURCE,
        "struct InitializedIntlDateTimeFormatObjectLocal(u32);",
        "fn emit_dtf_copy_components(",
    ));
    assert_eq!(
        declarations,
        concat!(
            "structDtfComponentLocals{year:u32,month:u32,day:u32,hour:u32,",
            "minute:u32,second:u32,ms:u32,weekday_index:u32,display_year:u32,}",
            "implDtfComponentLocals{fnlocals(&self)->[u32;9]{[self.year,self.month,",
            "self.day,self.hour,self.minute,self.second,self.ms,self.weekday_index,",
            "self.display_year,]}}",
            "enumDtfRangePattern{Fallback,TextMonthDifference,TextDayDifference,}",
            "implDtfRangePattern{constfncode(&self)->i64{matchself{",
            "Self::Fallback=>0,Self::TextMonthDifference=>1,",
            "Self::TextDayDifference=>2,}}}",
            "structDtfRangeLocals{second_time:u32,start:DtfComponentLocals,",
            "end:DtfComponentLocals,side:u32,side_limit:u32,practically_equal:u32,",
            "pattern:u32,}",
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "DtfComponentLocals"),
        11
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "DtfRangeLocals"),
        4
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "DtfRangePattern"),
        8
    );
    for forbidden in [
        "Clone for DtfComponentLocals",
        "Copy for DtfComponentLocals",
        "Clone for DtfRangeLocals",
        "Copy for DtfRangeLocals",
        "DtfComponentLocals::clone",
        "DtfRangeLocals::clone",
        "Clone for DtfRangePattern",
        "Copy for DtfRangePattern",
        "DtfRangePattern::clone",
    ] {
        assert!(!DTF_SOURCE.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn component_producers_and_helpers_borrow_until_release() {
    let normalized = normalized_rust(DTF_SOURCE);
    let owners = normalized_rust(bounded(
        DTF_SOURCE,
        "fn reserve_dtf_components(&mut self) -> DtfComponentLocals {",
        "fn emit_dtf_month_number(",
    ));
    assert_eq!(owners.matches("DtfComponentLocals{").count(), 2);
    assert_eq!(
        normalized.matches("self.reserve_dtf_components()").count(),
        2
    );
    assert_eq!(normalized.matches("Some(DtfRangeLocals{").count(), 1);
    assert!(normalized.contains(
        "fnemit_dtf_copy_components(from:&DtfComponentLocals,to:&DtfComponentLocals,function:&mutFunction,)"
    ));
    assert!(normalized.contains(
        "fnemit_dtf_components_from_time(&mutself,time_local:u32,offset_minutes_local:u32,comps:&DtfComponentLocals,function:&mutFunction,)"
    ));
    assert!(normalized.contains(
        "fnemit_dtf_practical_equality(&mutself,codes:[u32;9],range:&DtfRangeLocals,function:&mutFunction,)"
    ));
    let equality = normalized_rust(bounded(
        DTF_SOURCE,
        "fn emit_dtf_practical_equality(",
        "fn reserve_dtf_components(",
    ));
    assert_eq!(
        equality
            .matches("leta=&range.start;letb=&range.end;")
            .count(),
        1
    );

    let release = normalized_rust(bounded(
        DTF_SOURCE,
        "fn release_dtf_components(&mut self, comps: DtfComponentLocals) {",
        "\n    }\n}\n\nimpl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(
        release,
        "forlocalincomps.locals().into_iter().rev(){self.release_temp_local(local);}"
    );
}

#[test]
fn range_is_observed_by_shared_reference_then_consumed_for_one_reverse_release() {
    let formatter = normalized_rust(bounded(
        DTF_SOURCE,
        "pub(crate) fn emit_intl_dtf_build_format_with_kind(",
        "fn emit_dtf_month_number(",
    ));
    assert_eq!(formatter.matches("match&range{").count(), 2);
    assert_eq!(formatter.matches("ifletSome(range)=&range{").count(), 4);
    assert_eq!(formatter.matches("range.is_some()").count(), 2);
    assert_eq!(formatter.matches("ifletSome(range)=range{").count(), 1);
    assert!(!formatter.contains("range.clone()"));
    assert!(!formatter.contains("range.as_ref()"));

    for call in [
        "emit_dtf_components_from_time(times.first,applied_offset_local,&current,function,)",
        "emit_dtf_components_from_time(times.first,applied_offset_local,&range.start,function,)",
        "emit_dtf_components_from_time(range.second_time,applied_offset_local,&range.end,function,)",
        "emit_dtf_copy_components(&range.start,&current,function);",
        "emit_dtf_copy_components(&range.end,&current,function);",
    ] {
        assert_eq!(formatter.matches(call).count(), 1, "missing `{call}`");
    }

    let components = formatter.find("match&range{").unwrap();
    let capacity = formatter[components + 1..]
        .find("match&range{")
        .map(|offset| components + 1 + offset)
        .unwrap();
    let loop_start = formatter.find("ifletSome(range)=&range{").unwrap();
    let loop_end = formatter[loop_start + 1..]
        .find("ifletSome(range)=&range{")
        .map(|offset| loop_start + 1 + offset)
        .unwrap();
    let release = formatter.find("ifletSome(range)=range{").unwrap();
    assert!(components < capacity && capacity < loop_start && loop_start < loop_end);
    assert!(loop_end < release);

    let release_tail = bounded(
        &formatter,
        "ifletSome(range)=range{",
        "ifletDtfSourceAttribution::Range{source_local}=sink.source{",
    );
    assert_eq!(
        release_tail,
        concat!(
            "letDtfRangeLocals{second_time:_,start,end,side,side_limit,",
            "practically_equal,pattern,}=range;",
            "forlocalin[pattern,practically_equal,side_limit,side]{",
            "self.release_temp_local(local);}",
            "self.release_dtf_components(end);",
            "self.release_dtf_components(start);}",
        )
    );
    assert_eq!(
        formatter
            .matches("self.release_dtf_components(end)")
            .count(),
        1
    );
    assert_eq!(
        formatter
            .matches("self.release_dtf_components(start)")
            .count(),
        1
    );
    assert_eq!(release_tail.matches("second_time:_,").count(), 1);
    assert!(!release_tail.contains("release_temp_local(second_time)"));
}

#[test]
fn contract_and_task_record_the_focused_boundary() {
    for phrase in [
        "DtfRangeLocals",
        "DtfComponentLocals",
        "11/4/8 production identifier census",
        "eight shared observations",
        "second_time",
        "textual interval selection",
        "8/8",
    ] {
        assert!(CONTRACT.contains(phrase), "contract missing `{phrase}`");
    }
    assert!(TASK.contains("intl-date-time-format-range-locals-ownership.md"));
    assert!(CONTRACT.contains(
        "cargo test -p lila-aot-wasm --test intl_dtf_range_locals_ownership_structure -- --test-threads=1"
    ));
}
