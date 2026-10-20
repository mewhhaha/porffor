use super::*;
#[test]
fn native_realm_script_uses_captured_realm_and_intrinsic_syntax_after_poisoning() {
    let valid = "var realmMarker=40; realmMarker+2;";
    let invalid = "let = ;";
    let source = format!(
        r#"
var realm=__scriptwCreateRealm(), other=realm.global, run=realm.evalScript;
var LocalSyntax=SyntaxError, ForeignSyntax=other.SyntaxError;
SyntaxError=function(){{throw 'poisoned local SyntaxError';}};
other.SyntaxError=function(){{throw 'poisoned foreign SyntaxError';}};
realm.evalScript=function(){{throw 'replacement callable';}};
var valid={valid}, invalid={invalid}, trace='';
var text={{toString(){{trace+='coerce;';gc();return valid;}}}};
print(run.call(null,text));
print(other.realmMarker);
print(typeof realmMarker);
try{{run(invalid);}}catch(error){{
 print(error.constructor===ForeignSyntax);
 print(Object.getPrototypeOf(error)===ForeignSyntax.prototype);
 print(!(error instanceof LocalSyntax));
}}
print(trace);
var sentinel={{}};
try{{run({{toString(){{trace+='throw;';throw sentinel;}}}});}}catch(error){{print(error===sentinel);}}
print(trace);
print(Object.getPrototypeOf({{}})===Object.prototype);
"#,
        valid = converted_text(valid),
        invalid = converted_text(invalid)
    );
    assert_native_trace(
        &source,
        &[
            "42",
            "40",
            "undefined",
            "true",
            "true",
            "true",
            "coerce;",
            "true",
            "coerce;throw;",
            "true",
        ],
        &[
            (
                PreparedScriptKind::RealmScript,
                valid.encode_utf16().collect(),
            ),
            (
                PreparedScriptKind::RealmScript,
                invalid.encode_utf16().collect(),
            ),
        ],
        false,
        global_counts(2, 1, 1),
        0,
    );
}

#[test]
fn replaced_eval_script_method_keeps_the_original_argument_receiver_and_no_compiler_call() {
    let source = r#"
var realm=__scriptwCreateRealm(), calls=0, coercions=0;
var token={toString(){coercions++;throw 'must not coerce';}};
realm.evalScript=function(value){calls++;return this===realm&&value===token;};
print(realm.evalScript(token));print(calls);print(coercions);
"#;
    assert_native_trace(
        source,
        &["true", "1", "0"],
        &[],
        false,
        global_counts(0, 0, 0),
        0,
    );
}
