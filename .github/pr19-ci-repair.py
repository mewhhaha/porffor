from pathlib import Path
import hashlib

path = Path('crates/lila-aot-wasm/src/objects.rs')
source = path.read_text()
assert hashlib.sha1(b'blob ' + str(len(source.encode())).encode() + b'\0' + source.encode()).hexdigest() == '0afb0b2975ac6113a323bab7e7ad866ef7044f80'
start = source.index('    pub(crate) fn emit_object_read_ordinary_inner(')
old = '''        self.emit_array_length(current_local, payload_local, tag_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));'''
new = '''        // Arguments share Array element storage, but their configurable length
        // lives in a separate property descriptor. An absent descriptor must
        // continue the same prototype walk with the original Get receiver.
        function.instruction(&Instruction::LocalGet(current_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Commit presence before invoking an accessor. Both normal and abrupt
        // calls then leave the walk without reading any prototype or element.
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET,
            tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET,
            getter_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            current_local,
            HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET,
            getter_tag_local,
            function,
        );
        self.emit_is_callable_i32(getter_tag_local, getter_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            getter_payload_local,
            getter_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_array_length(current_local, payload_local, tag_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_local));
        function.instruction(&Instruction::End);'''
assert source[start:].count(old) == 1
path.write_text(source[:start] + source[start:].replace(old, new, 1))

path = Path('crates/lila-engine/tests/aot_array_to_locale_string_length.rs')
source = path.read_text()
assert source.count('#[test]') == 19
source += '''
#[test]
fn deleted_arguments_length_getter_keeps_receiver_and_live_mapped_values() {
    assert_wasm_true(
        r#"
function check(a, b) {
  var source = arguments, calls = 0, correctThis = false;
  delete source.length;
  Object.setPrototypeOf(source, {
    get length() { calls++; correctThis = this === source; a = 8; return 1; }
  });
  return Array.prototype.toLocaleString.call(source) === '8' && calls === 1 && correctThis;
}
check(4, 5);
"#,
    );
}

#[test]
fn arguments_length_proxy_getter_propagates_the_exact_throw_before_elements() {
    assert_wasm_true(
        r#"
function check(a) {
  var source = arguments, calls = 0, reads = 0, correctThis = false, marker = {};
  Object.defineProperty(source, '0', { get() { reads++; return 7; } });
  var getter = new Proxy(function() { throw 'target must not run'; }, {
    apply(target, receiver, args) {
      calls++; correctThis = receiver === source && args.length === 0; throw marker;
    }
  });
  Object.defineProperty(source, 'length', { get: getter });
  try { Array.prototype.toLocaleString.call(source); return false; }
  catch (e) { return e === marker && calls === 1 && reads === 0 && correctThis; }
}
check(4);
"#,
    );
}

#[test]
fn undefined_arguments_length_getter_shadows_inherited_length() {
    assert_wasm_true(
        r#"
function check(a) {
  'use strict';
  var source = arguments, calls = 0;
  Object.setPrototypeOf(source, { get length() { calls++; return 1; } });
  Object.defineProperty(source, 'length', { get: undefined });
  return Array.prototype.toLocaleString.call(source) === '' && calls === 0;
}
check(4);
"#,
    );
}

#[test]
fn inherited_arguments_length_accessor_uses_the_original_get_receiver() {
    assert_wasm_true(
        r#"
function check(a, b) {
  'use strict';
  var source = arguments, calls = 0, correctThis = false;
  var receiver = Object.create(source);
  receiver[0] = 8;
  var getter = new Proxy(function() { return 99; }, {
    apply(target, value, args) {
      calls++; correctThis = value === receiver && args.length === 0; return 1;
    }
  });
  Object.defineProperty(source, 'length', { get: getter });
  return Array.prototype.toLocaleString.call(receiver) === '8' && calls === 1 && correctThis;
}
check(4, 5);
"#,
    );
}
'''
path.write_text(source)

path = Path('crates/lila-aot-wasm/tests/typed_array_to_locale_string_witness_structure.rs')
source = path.read_text()
source += '''
#[test]
fn ordinary_get_distinguishes_arguments_length_descriptors_from_array_storage() {
    let body = bounded(
        include_str!("../src/objects.rs"),
        "pub(crate) fn emit_object_read_ordinary_inner(",
        "// Array-like exotic elements and named properties live in",
    );
    let descriptor = unique_normalized_position(
        body,
        "HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET",
        "arguments own length descriptor",
    );
    let present = unique_normalized_position(
        body,
        "HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET",
        "arguments own data value",
    );
    assert!(descriptor < present);
    for field in [
        "HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET",
        "HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET",
        "HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET",
    ] {
        assert!(body.contains(field), "missing arguments length field: {field}");
    }
    assert!(without_whitespace(body).contains(&without_whitespace(
        r#"
        self.emit_function_or_proxy_call_leave_throw_completion(
            getter_payload_local,
            getter_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        "#
    )));
    assert!(body.contains("self.emit_load_prototype_to_current_locals("));
}
'''
path.write_text(source)

path = Path('docs/rust-rewrite/aot-array-to-locale-string-length.md')
source = path.read_text().replace('contains 19\nexplicit', 'contains 23\nexplicit')
needle = 'After successful length conversion, the generic entry classifies a TypedArray\n'
assert needle in source
source = source.replace(needle, '''The shared ordinary Get owner also distinguishes the arguments length property
from Array storage. It reads the dedicated descriptor's tagged data value or
invokes its callable getter with the original Get receiver, including callable
Proxies. A deleted descriptor resumes the existing tagged prototype walk; an
undefined getter still shadows inherited length. Presence is committed before
calling a getter, so a thrown completion leaves the walk without further reads.
This correction is shared by generic Array consumers, not a toLocaleString-only
shortcut. The original 19 regressions are retained, with four additional engine
regressions for inherited getters, mapped values, Proxy getter throws, undefined
getters and inherited arguments receivers.

''' + needle)
path.write_text(source)
