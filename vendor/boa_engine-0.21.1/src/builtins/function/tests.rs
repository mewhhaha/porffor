use crate::{
    JsNativeErrorKind, JsValue, TestAction,
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject},
    property::{Attribute, PropertyDescriptor},
    run_test_actions,
};
use boa_macros::js_str;
use indoc::indoc;

#[allow(clippy::float_cmp)]
#[test]
fn arguments_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function jason(a, b) {
                    return arguments[0];
                }
            "#}),
        TestAction::assert_eq("jason(100, 6)", 100),
    ]);
}

#[test]
fn self_mutating_function_when_calling() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function x() {
                    x.y = 3;
                }
                x();
            "#}),
        TestAction::assert_eq("x.y", 3),
    ]);
}

#[test]
fn self_mutating_function_when_constructing() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function x() {
                    x.y = 3;
                }
                new x();
            "#}),
        TestAction::assert_eq("x.y", 3),
    ]);
}

#[test]
fn function_prototype() {
    run_test_actions([
        TestAction::assert_eq("Function.prototype.name", js_string!()),
        TestAction::assert_eq("Function.prototype.length", 0),
        TestAction::assert_eq("Function.prototype()", JsValue::undefined()),
        TestAction::assert_eq(
            "Function.prototype(1, '', new String(''))",
            JsValue::undefined(),
        ),
        TestAction::assert_native_error(
            "new Function.prototype()",
            JsNativeErrorKind::Type,
            "not a constructor",
        ),
    ]);
}

#[test]
fn function_prototype_call() {
    run_test_actions([TestAction::assert_eq(
        "Object.prototype.toString.call(new Error())",
        js_str!("[object Error]"),
    )]);
}

#[test]
fn function_prototype_call_throw() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            let call = Function.prototype.call;
            call(call)
        "#},
        JsNativeErrorKind::Type,
        "undefined is not a function",
    )]);
}

#[test]
fn function_prototype_call_multiple_args() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            function f(a, b) {
                this.a = a;
                this.b = b;
            }
            let o = {a: 0, b: 0};
            f.call(o, 1, 2);
        "#}),
        TestAction::assert_eq("o.a", 1),
        TestAction::assert_eq("o.b", 2),
    ]);
}

#[test]
fn function_prototype_apply() {
    run_test_actions([
        TestAction::run("const numbers = [6, 7, 3, 4, 2]"),
        TestAction::assert_eq("Math.max.apply(null, numbers)", 7),
        TestAction::assert_eq("Math.min.apply(null, numbers)", 2),
    ]);
}

#[test]
fn function_prototype_apply_on_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function f(a, b) {
                    this.a = a;
                    this.b = b;
                }
                let o = {a: 0, b: 0};
                f.apply(o, [1, 2]);
            "#}),
        TestAction::assert_eq("o.a", 1),
        TestAction::assert_eq("o.b", 2),
    ]);
}

#[test]
fn closure_capture_clone() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let string = js_string!("Hello");
            let object = JsObject::with_object_proto(ctx.intrinsics());
            object
                .define_property_or_throw(
                    js_string!("key"),
                    PropertyDescriptor::builder()
                        .value(js_string!(" world!"))
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                    ctx,
                )
                .unwrap();

            let func = FunctionObjectBuilder::new(
                ctx.realm(),
                NativeFunction::from_copy_closure_with_captures(
                    |_, _, captures, context| {
                        let (string, object) = &captures;

                        let hw = js_string!(
                            string,
                            &object
                                .__get_own_property__(
                                    &js_string!("key").into(),
                                    &mut context.into()
                                )?
                                .and_then(|prop| prop.value().and_then(JsValue::as_string))
                                .ok_or_else(
                                    || JsNativeError::typ().with_message("invalid `key` property")
                                )?
                        );
                        Ok(hw.into())
                    },
                    (string, object),
                ),
            )
            .name(js_str!("closure"))
            .build();

            ctx.register_global_property(js_str!("closure"), func, Attribute::default())
                .unwrap();
        }),
        TestAction::assert_eq("closure()", js_str!("Hello world!")),
    ]);
}

#[test]
fn function_constructor_early_errors_super() {
    run_test_actions([
        TestAction::assert_native_error(
            "Function('super()')()",
            JsNativeErrorKind::Syntax,
            "invalid `super` call",
        ),
        TestAction::assert_native_error(
            "Function('super.a')()",
            JsNativeErrorKind::Syntax,
            "invalid `super` reference",
        ),
    ]);
}

#[test]
fn dynamic_function_parses_before_getting_new_target_prototype() {
    run_test_actions([TestAction::assert(indoc! {r#"
        let getProtoCalled = false;
        let newTarget = Object.defineProperty(function(){}.bind(), "prototype", {
            get() {
                getProtoCalled = true;
                return null;
            }
        });

        try {
            Reflect.construct(Function, ["@error"], newTarget);
            false;
        } catch (error) {
            error instanceof SyntaxError && getProtoCalled === false;
        }
    "#})]);
}

#[test]
fn legacy_function_caller_and_arguments_are_exposed_for_active_nonstrict_functions() {
    run_test_actions([TestAction::assert(indoc! {r#"
        function outer(a, b) {
            return inner();
        }

        function inner() {
            return inner.caller === outer
                && inner.arguments !== null
                && inner.arguments.length === 0;
        }

        outer(1, 2);
    "#})]);
}

#[test]
fn legacy_function_caller_skips_eval_frames() {
    run_test_actions([TestAction::assert(indoc! {r#"
        function innermost() { return arguments.callee.caller; }
        function middle() { return eval("innermost();"); }
        function outer() { return middle(); }

        outer() === middle;
    "#})]);
}
