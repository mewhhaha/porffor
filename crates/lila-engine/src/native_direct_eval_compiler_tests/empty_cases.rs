use super::*;

pub(super) fn native_direct_empty_comments_keep_real_parsing_and_nonempty_completion() {
    let bodies = [
        "/* first */",
        "// second",
        "'use strict';",
        "// comment\n17;",
        "/* unterminated",
    ];
    let source = format!(
        "{}try{{eval({});}}catch(error){{print(error instanceof SyntaxError);}}",
        bodies[..4]
            .iter()
            .map(|body| format!("print(eval({}));", converted_text(body)))
            .collect::<String>(),
        converted_text(bodies[4])
    );
    assert_trace(
        &source,
        &["undefined", "undefined", "use strict", "17", "true"],
        &bodies.map(units),
        (5, 4, 1),
    );
}

pub(super) fn native_direct_empty_blocks_preserve_update_empty_and_nested_values() {
    let bodies = ["{}", "17;{}{}", "'use strict';17;{}{}", "17;{{}}"];
    let source = bodies
        .iter()
        .map(|body| format!("print(eval({}));", converted_text(body)))
        .collect::<String>();
    let exact_sources = bodies.map(units);
    assert_trace(
        &source,
        &["undefined", "17", "17", "17"],
        &exact_sources,
        (4, 4, 0),
    );
}
