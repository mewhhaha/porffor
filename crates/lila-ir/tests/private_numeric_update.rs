use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, FunctionFlavor, FunctionIr, KindSet, NumericUpdateOp, NumericUpdateValueKind,
    ScriptIr, StatementIr, TypedExpr, UpdateReturnMode, ValueKind,
};

fn lower_private_updates(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::script()).expect("private update fixture parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}

fn method<'a>(script: &'a ScriptIr, name: &str) -> &'a FunctionIr {
    script
        .functions
        .iter()
        .find(|function| function.name.rsplit('.').next() == Some(name))
        .unwrap_or_else(|| panic!("missing method {name}"))
}

fn returned_expression(function: &FunctionIr) -> &TypedExpr {
    function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(value) => Some(value),
            _ => None,
        })
        .expect("method returns its update")
}

#[test]
fn private_updates_capture_even_identifier_bases_and_keep_the_numeric_result() {
    let script = lower_private_updates(
        r#"
class Counter {
  #value;
  postIncrement(base) { return base.#value++; }
  preIncrement(base) { return ++base.#value; }
  postDecrement(base) { return base.#value--; }
  preDecrement(base) { return --base.#value; }
}
"#,
    );
    for (name, expected_op, expected_mode) in [
        (
            "postIncrement",
            NumericUpdateOp::Increment,
            UpdateReturnMode::Postfix,
        ),
        (
            "preIncrement",
            NumericUpdateOp::Increment,
            UpdateReturnMode::Prefix,
        ),
        (
            "postDecrement",
            NumericUpdateOp::Decrement,
            UpdateReturnMode::Postfix,
        ),
        (
            "preDecrement",
            NumericUpdateOp::Decrement,
            UpdateReturnMode::Prefix,
        ),
    ] {
        let expression = returned_expression(method(&script, name));
        assert_eq!(
            expression.possible_kinds,
            KindSet::from_kind(ValueKind::Number).union(KindSet::from_kind(ValueKind::BigInt)),
        );
        let ExprIr::MaterializeBinding {
            name: target_name,
            value: target,
            body,
        } = &expression.expr
        else {
            panic!("base must be captured: {expression:?}");
        };
        assert!(matches!(&target.expr, ExprIr::Identifier(_)));
        let ExprIr::MaterializeBinding {
            name: old_name,
            value: read,
            body,
        } = &body.expr
        else {
            panic!("one PrivateGet precedes ToNumeric");
        };
        let ExprIr::PrivateRead {
            target,
            private_name_id,
        } = &read.expr
        else {
            panic!("update reads a private reference");
        };
        assert!(matches!(&target.expr, ExprIr::Identifier(name) if name == target_name));
        let ExprIr::MaterializeBinding {
            name: result_name,
            value: update,
            body,
        } = &body.expr
        else {
            panic!("numeric result must survive PrivateSet");
        };
        assert!(matches!(&update.expr, ExprIr::UpdateIdentifier {
            name, op, return_mode, value_kind: NumericUpdateValueKind::Dynamic,
        } if name == old_name && *op == expected_op && *return_mode == expected_mode));
        let ExprIr::Comma {
            lhs: write,
            rhs: result,
        } = &body.expr
        else {
            panic!("PrivateSet precedes returning the numeric result");
        };
        let ExprIr::PrivateWrite {
            target,
            private_name_id: written_name,
            value,
        } = &write.expr
        else {
            panic!("update writes a private reference");
        };
        assert_eq!(private_name_id, written_name);
        assert!(matches!(&target.expr, ExprIr::Identifier(name) if name == target_name));
        assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == old_name));
        assert!(matches!(&result.expr, ExprIr::Identifier(name) if name == result_name));
        assert_ne!(target_name, old_name);
        assert_ne!(old_name, result_name);
    }
}

#[test]
fn private_accessor_exceptions_widen_otherwise_numeric_catch_bindings() {
    let script = lower_private_updates(
        r#"
class Counter {
  get #value() { throw 'private'; }
  update(flag) {
    try { if (flag) throw 1; this.#value++; }
    catch (caught) { return caught + 1; }
  }
}
"#,
    );
    let update = method(&script, "update");
    let catch = update
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::TryCatch { catch_block, .. } => Some(catch_block),
            _ => None,
        })
        .expect("private update is inside a try block");
    let result = catch
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(value) => Some(value),
            _ => None,
        })
        .expect("catch returns its thrown value plus one");
    assert!(
        matches!(result.expr, ExprIr::CoerciveAdd { .. }),
        "{result:?}"
    );
}

#[test]
fn private_numeric_hooks_invalidate_following_captured_binding_facts() {
    let script = lower_private_updates(
        r#"
function outer() {
  let label = 1;
  class Counter {
    #value = { valueOf() { label = 'changed'; return 1; } };
    update() { return this.#value++; }
  }
  const counter = new Counter();
  counter.update();
  return label + 1;
}

"#,
    );
    let result = returned_expression(method(&script, "outer"));
    assert!(
        matches!(result.expr, ExprIr::CoerciveAdd { .. }),
        "{result:?}"
    );
}

#[test]
fn private_update_arrows_capture_the_lexical_receiver() {
    let script = lower_private_updates(
        "class Counter { #value = 0; closure() { return () => this.#value++; } }",
    );
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .expect("private update arrow");
    assert!(arrow.captures_lexical_this);
    assert!(arrow.captures_private_environment);
}

#[test]
fn property_updates_capture_both_the_base_and_computed_key() {
    let script = lower_private_updates(
        "function make() { const base = { value: 1 }; const key = 'value'; return () => base[key]++; }",
    );
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .expect("computed property update arrow");
    for name in ["base", "key"] {
        assert!(
            arrow
                .captured_bindings
                .iter()
                .any(|binding| binding.source_name == name),
            "missing {name}: {:?}",
            arrow.captured_bindings
        );
    }
}
