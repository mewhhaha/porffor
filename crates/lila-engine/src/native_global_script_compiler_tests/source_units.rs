use super::*;
#[test]
fn native_eval_over_257_real_sources_keeps_lone_units_and_only_two_chosen_factories() {
    let literals = (0..257)
        .map(|i| match i {
            0 => r#""'\uD800'.charCodeAt(0);""#.to_owned(),
            256 => r#""'\uDFFF'.charCodeAt(0);""#.to_owned(),
            _ => format!("\"'{i}';\""),
        })
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        r#"
var visits=0,selected=0,trace='';
for(const text of [{literals}]){{
 var index=visits++;
 if(index===0||index===256){{selected++;trace+=index+';';print((0,eval)(text));}}
}}
print(visits);print(selected);print(trace);
"#
    );
    let endpoint = |unit: u16| {
        [
            vec![0x27],
            vec![unit],
            "'.charCodeAt(0);".encode_utf16().collect(),
        ]
        .concat()
    };
    assert_native_trace(
        &source,
        &["55296", "57343", "257", "2", "0;256;"],
        &[
            (PreparedScriptKind::IndirectEval, endpoint(0xd800)),
            (PreparedScriptKind::IndirectEval, endpoint(0xdfff)),
        ],
        true,
        global_counts(2, 2, 0),
        0,
    );
}
