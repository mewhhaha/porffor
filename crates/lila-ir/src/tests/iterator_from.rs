#[test]
fn iterator_from_nullish_symbol_iterator_retains_runtime_materialization() {
    let program = lower_script(
        "function* g() { yield 0; yield 1; yield 2; }
             let iter = (function () {
               let n = g();
               return { [Symbol.iterator]: null, next: () => n.next() };
             })();
             let array = Array.from(Iterator.from(iter));",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let array_init = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, init, .. } if name == "array" => Some(init),
            _ => None,
        })
        .expect("array lexical should be present");
    let ExprIr::CallIndirect { args, .. } = &indirect_call_body(array_init)
        .expect("Array.from must execute its iterator protocol")
        .expr
    else {
        unreachable!();
    };
    assert_eq!(args.len(), 1);
    assert!(
        indirect_call_body(&args[0]).is_some(),
        "Iterator.from must execute: {:?}",
        args[0]
    );
}

#[test]
fn iterator_from_reassigned_source_retains_runtime_materialization() {
    let program = lower_script(
        "function* g() { yield 0; yield 1; yield 2; }
             let iter = (function () {
               let n = g();
               return { [Symbol.iterator]: 0, next: () => n.next() };
             })();
             iter = (function () {
               let n = g();
               return { [Symbol.iterator]: null, next: () => n.next() };
             })();
             let array = Array.from(Iterator.from(iter));",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let array_init = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, init, .. } if name == "array" => Some(init),
            _ => None,
        })
        .expect("array lexical should be present");
    let ExprIr::CallIndirect { args, .. } = &indirect_call_body(array_init)
        .expect("Array.from must read the reassigned source")
        .expr
    else {
        unreachable!();
    };
    assert_eq!(args.len(), 1);
    assert!(
        indirect_call_body(&args[0]).is_some(),
        "Iterator.from must execute: {:?}",
        args[0]
    );
}

#[test]
fn keeps_iterator_from_wrapper_return_observable() {
    let program = lower_script(
        "const iter = {};
             const wrapper = Iterator.from(iter);
             const result = wrapper.return();",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let result_init = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, init, .. } if name == "result" => Some(init),
            _ => None,
        })
        .expect("result lexical should be present");
    assert!(indirect_call_body(result_init).is_some());
}

#[test]
fn iterator_from_keeps_an_open_object_result_before_runtime_prototype_reads() {
    let program = lower_script(
        "const iter = { next() { return { done: true, value: undefined }; } };
         const result = Iterator.from(iter);
         const wrapperPrototype = Object.getPrototypeOf(result);",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script IR");
    let result = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, init, .. } if name == "result" => Some(init),
            _ => None,
        })
        .expect("Iterator.from result");
    assert!(indirect_call_body(result).is_some(), "{result:?}");
    assert!(
        result.heap_shape.is_none(),
        "runtime @@iterator chooses the object: {result:?}"
    );
    for kind in [
        ValueKind::Object,
        ValueKind::Function,
        ValueKind::Array,
        ValueKind::Arguments,
    ] {
        assert!(result.possible_kinds.contains(kind), "{result:?}");
    }
    let prototype = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, init, .. } if name == "wrapperPrototype" => Some(init),
            _ => None,
        })
        .expect("runtime prototype read");
    assert!(indirect_call_body(prototype).is_some(), "{prototype:?}");
    assert!(prototype.heap_shape.is_none(), "{prototype:?}");
}

#[test]
fn iterator_from_callable_symbol_iterator_retains_heap_mutation_authority() {
    let program = lower_script(
        "function iteratorMethod() { return this; }
             const iter = {
               [Symbol.iterator]: iteratorMethod,
               next() { return { done: true, value: undefined }; }
             };
             const tracked = { value: {} };
             Iterator.from(iter);
             const observed = tracked.value;",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let observed = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, init, .. } if name == "observed" => Some(init),
            _ => None,
        })
        .expect("observed lexical should be present");
    assert!(
        observed.heap_shape.is_none(),
        "a callable @@iterator may mutate a previously tracked heap shape: {observed:?}"
    );
}

#[test]
fn iterator_from_existing_iterator_keeps_the_original_argument_and_open_result() {
    let program = lower_script(
        "const iteratorFrom = Iterator.from;
         const iterator = (function* () {})();
         const from = iteratorFrom(iterator);",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script IR");
    let result = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, init, .. } if name == "from" => Some(init),
            _ => None,
        })
        .expect("Iterator.from result");
    let ExprIr::CallIndirect { args, .. } = &indirect_call_body(result).expect("actual call").expr
    else {
        unreachable!();
    };
    assert_eq!(args.len(), 1);
    assert!(matches!(&args[0].expr, ExprIr::Identifier(_)), "{args:?}");
    assert!(result.heap_shape.is_none(), "{result:?}");
    for kind in [
        ValueKind::Object,
        ValueKind::Function,
        ValueKind::Array,
        ValueKind::Arguments,
    ] {
        assert!(result.possible_kinds.contains(kind), "{result:?}");
    }
}
