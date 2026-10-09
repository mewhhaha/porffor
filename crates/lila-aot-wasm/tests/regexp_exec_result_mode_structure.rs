use std::fs;
use std::path::Path;
const SOURCE: &str = include_str!("../src/builtins/string/regexp_exec.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing {end}"))
        .0
}
fn normalized(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}
fn count(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .map(|path| {
            if path.is_dir() {
                return count(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(path).unwrap().matches(needle).count()
        })
        .sum()
}

#[test]
fn intrinsic_exec_has_one_complete_match_array_or_null_result_path() {
    let intrinsic = normalized(bounded(
        SOURCE,
        "fn emit_native_regexp_builtin_exec(",
        "pub(super) fn emit_native_string_result_array(",
    ));
    assert!(intrinsic.contains("object:&GcLocal<RegExpObject>"));
    assert!(intrinsic.contains("input:&GcLocal<StringValue>"));
    assert!(intrinsic.contains("result:&CompletionLocals"));
    assert!(!intrinsic.contains("result_mode"));
    assert!(!intrinsic.contains("return_boolean"));
    assert!(!intrinsic.contains("set_boolean("));
    assert_eq!(
        intrinsic
            .matches("self.emit_native_regexp_match_array(")
            .count(),
        1
    );
    assert_eq!(
        intrinsic
            .matches("last.set_scalar(ScalarValue::Null,f);result.set_normal(&last,f);")
            .count(),
        2,
        "past-end and normal miss share the complete Null publication"
    );
    let coerce = intrinsic
        .find("self.emit_to_length_i64_from_value_locals(")
        .unwrap();
    let flags = intrinsic
        .find("RegExpObjectSchema::ORIGINAL_FLAGS")
        .unwrap();
    let program = intrinsic.find("RegExpObjectSchema::PROGRAM").unwrap();
    let call = intrinsic.find("RegExpMatcherArguments::new(").unwrap();
    assert!(coerce < flags && flags < program && program < call);
    assert!(intrinsic.contains(".store(found,matched_start,matched_end,status,f)"));
    let cleanup = intrinsic.find("scratch.finish(self,f);").unwrap();
    assert!(cleanup < intrinsic.find("matchfailure.route(){").unwrap());
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count(&source_root, "emit_native_regexp_builtin_exec("),
        3,
        "one definition and exactly abstract RegExpExec/RegExp.prototype.exec callers"
    );
}

#[test]
fn abstract_exec_preserves_callable_throw_identity_and_validates_its_result() {
    let abstract_exec = normalized(bounded(
        SOURCE,
        "pub(crate) fn emit_regexp_exec_from_values(",
        "pub(crate) fn emit_regexp_prototype_exec_builtin(",
    ));
    let callable = abstract_exec
        .find("self.emit_is_callable_i32(acquired_exec,f)?;")
        .unwrap();
    let call = abstract_exec.find("self.emit_function_or_proxy_call_with_argv(acquired_exec,regexp,&arguments,result,f)?;").unwrap();
    let abrupt = abstract_exec
        .find("self.emit_native_string_abrupt_exit(result,result,exit,f);")
        .unwrap();
    let object = abstract_exec
        .find("self.emit_is_heap_object_like_tag_i32(result.value().tag(),f);")
        .unwrap();
    let null = abstract_exec
        .find("WasmRuntimeValueTag::Null.tag()")
        .unwrap();
    let invalid = abstract_exec
        .find("REGEXP_PROTOTYPE_SYMBOL_MATCH_EXEC_RESULT_IS_NOT_OBJECT_OR_NULL")
        .unwrap();
    let fallback = abstract_exec
        .find("self.emit_native_regexp_builtin_exec(")
        .unwrap();
    assert!(
        callable < call
            && call < abrupt
            && abrupt < object
            && object < null
            && null < invalid
            && invalid < fallback
    );
    assert!(
        !abstract_exec.contains("emit_native_regexp_get("),
        "the already acquired exec method is not read twice"
    );
    assert!(abstract_exec.contains("reference_type::<RegExpObject>"));
}

#[test]
fn prototype_test_projects_only_a_normal_completed_exec_result_to_boolean() {
    let test = normalized(bounded(
        SOURCE,
        "pub(crate) fn emit_regexp_prototype_test_builtin(",
        "\n}",
    ));
    let get = test
        .find("self.emit_native_regexp_get(&receiver,\"exec\",&pending,f)?;")
        .unwrap();
    let exec = test
        .find("self.emit_regexp_exec_from_values(&receiver,&input,&acquired,&pending,f)?;")
        .unwrap();
    let after = &test[exec..];
    let route = after
        .find("self.emit_native_string_abrupt_exit(&pending,&output,exit,f);")
        .unwrap();
    let project = after.find("pending.value().tag().load(f);f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null.tag()));f.instruction(&Instruction::I32Ne);truth.store(f);").unwrap();
    let publish = after
        .find("argument.set_boolean(truth,f);output.set_normal(&argument,f);")
        .unwrap();
    assert!(get < exec && route < project && project < publish);
    assert_eq!(
        test.matches("self.emit_regexp_exec_from_values(").count(),
        1
    );
    assert!(!test.contains("self.emit_native_regexp_builtin_exec("));
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count(&source_root, "emit_regexp_exec_from_values("),
        3,
        "the abstract owner, test and shared acquired-exec protocol helper"
    );
}
