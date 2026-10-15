use super::*;

#[test]
fn later_native_programs_preserve_old_hooks_literals_and_closures_across_gc_and_reentry() {
    let first_body = "var value = 17; return function(callback) { gc(); if (callback) callback(); return [value, Function('return 17;')(), 'first-owner']; };";
    let second_body =
        "return function() { gc(); return [Function('return 23;')(), 'second-owner']; };";
    let nested_body = "return 42;";
    let source = format!(
        r#"
function oldPValue() {{ return Function('return 17;')(); }}
print(oldPValue());
var firstBody = {first};
var secondBody = {second};
var nestedBody = {nested};
var f1 = Function(firstBody);
var first = f1();
print(first().join('|'));
var reads = 0;
var newTarget = function() {{}}.bind(null);
Object.defineProperty(newTarget, 'prototype', {{get() {{
  reads++;
  print('prototype');
  gc();
  var nested = Function(nestedBody);
  print(nested());
  gc();
  print('reentry-done');
  return Function.prototype;
}}}});
var f2 = Reflect.construct(Function, [secondBody], newTarget);
print(reads);
print(Object.getPrototypeOf(f2) === Function.prototype);
var second = f2();
print(second().join('|'));
gc();
print(first(function() {{
  print(oldPValue());
  print(second()[0]);
}}).join('|'));
print(oldPValue());
gc();
print(first()[1]);
"#,
        first = converted_text(first_body),
        second = converted_text(second_body),
        nested = converted_text(nested_body),
    );
    for directive in ["", "'use strict';"] {
        assert_native_trace(
            &format!("{directive}{source}"),
            &[
                "17",
                "17|17|first-owner",
                "prototype",
                "42",
                "reentry-done",
                "1",
                "true",
                "23|second-owner",
                "17",
                "23",
                "17|17|first-owner",
                "17",
                "17",
            ],
            &[&[first_body], &[second_body], &[nested_body]],
            false,
            NativeFunctionCompilationSnapshot {
                requests: 3,
                completed_factories: 3,
                syntax_rejections: 0,
            },
        );
    }
}
