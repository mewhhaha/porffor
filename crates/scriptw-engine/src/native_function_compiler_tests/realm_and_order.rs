use super::*;

#[test]
fn native_syntax_uses_constructor_realm_and_preserves_conversion_prototype_order() {
    let invalid_parameter = converted_text("a)");
    let parameter = converted_text("a");
    let body = converted_text("return realmMarker + a;");
    let source = format!(
        r#"
var other = __scriptwCreateRealm().global;
other.realmMarker = 40;
var LocalSyntax = SyntaxError, ForeignSyntax = other.SyntaxError;
SyntaxError = function() {{ throw 'poisoned local SyntaxError'; }};
other.SyntaxError = function() {{ throw 'poisoned foreign SyntaxError'; }};
var invalidParameter = {invalid_parameter};
var parameter = {parameter};
var actualBody = {body};
var trace = '', reads = 0, throwPrototype = false;
var sentinel = {{}}, chosenPrototype = {{}};
var newTarget = function() {{}}.bind(null);
Object.defineProperty(newTarget, 'prototype', {{get() {{
  reads++; trace += 'prototype;'; gc();
  if (throwPrototype) throw sentinel;
  return chosenPrototype;
}}}});
var invalid = {{toString() {{ trace += 'parameter;'; return invalidParameter; }}}};
var valid = {{toString() {{ trace += 'parameter;'; return parameter; }}}};
var bodySource = {{toString() {{ trace += 'body;'; return actualBody; }}}};
try {{ Reflect.construct(other.Function, [invalid, bodySource], newTarget); }}
catch (error) {{
  print(error.constructor === ForeignSyntax);
  print(Object.getPrototypeOf(error) === ForeignSyntax.prototype);
  print(!(error instanceof LocalSyntax));
}}
print(trace);
print(reads);
trace = '';
var created = Reflect.construct(other.Function, [valid, bodySource], newTarget);
print(created(2));
print(Object.getPrototypeOf(created) === chosenPrototype);
print(trace);
print(reads);
trace = '';
var abruptBody = {{toString() {{ trace += 'body;'; throw sentinel; }}}};
try {{ Reflect.construct(other.Function, [invalid, abruptBody], newTarget); }}
catch (error) {{ print(error === sentinel); }}
print(trace);
print(reads);
trace = ''; throwPrototype = true;
try {{ Reflect.construct(other.Function, [valid, bodySource], newTarget); }}
catch (error) {{ print(error === sentinel); }}
print(trace);
print(reads);
"#
    );
    for directive in ["", "'use strict';"] {
        assert_native_trace(
            &format!("{directive}{source}"),
            &[
                "true",
                "true",
                "true",
                "parameter;body;",
                "0",
                "42",
                "true",
                "parameter;body;prototype;",
                "1",
                "true",
                "parameter;body;",
                "1",
                "true",
                "parameter;body;prototype;",
                "2",
            ],
            &[
                &["a)", "return realmMarker + a;"],
                &["a", "return realmMarker + a;"],
            ],
            false,
            NativeFunctionCompilationSnapshot {
                requests: 3,
                completed_factories: 2,
                syntax_rejections: 1,
            },
        );
    }
}
