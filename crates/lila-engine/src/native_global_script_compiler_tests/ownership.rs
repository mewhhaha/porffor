use super::*;
#[test]
fn native_global_closures_keep_owners_across_gc_old_p_reentry_and_native_function() {
    let first = "(function(){let value=17;return function(callback){gc();if(callback)callback();return [value,Function('return 17;')(),'first-owner'];};})();";
    let second = "(function(){let value=23;return function(){gc();return [value,Function('return 23;')(),'second-owner'];};})();";
    let nested = "return 42;";
    let source = format!(
        r#"
function oldPValue(){{return Function('return 17;')();}}
print(oldPValue());
var first=(0,eval)({first});
print(first().join('|'));
var second=(0,eval)({second});
print(second().join('|'));
var nested={nested};
gc();
print(first(function(){{print(oldPValue());print(Function(nested)());print(second()[0]);}}).join('|'));
print(oldPValue());
gc();
print(first()[1]);
"#,
        first = converted_text(first),
        second = converted_text(second),
        nested = converted_text(nested)
    );
    assert_native_trace(
        &source,
        &[
            "17",
            "17|17|first-owner",
            "23|23|second-owner",
            "17",
            "42",
            "23",
            "17|17|first-owner",
            "17",
            "17",
        ],
        &[
            (
                PreparedScriptKind::IndirectEval,
                first.encode_utf16().collect(),
            ),
            (
                PreparedScriptKind::IndirectEval,
                second.encode_utf16().collect(),
            ),
        ],
        false,
        global_counts(2, 2, 0),
        1,
    );
}
