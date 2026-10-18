use super::*;

#[test]
fn native_direct_empty_comments_keep_real_parsing_and_nonempty_completion() {
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

#[test]
fn native_direct_empty_blocks_preserve_update_empty_and_nested_values() {
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

#[test]
fn native_direct_empty_loop_bodies_keep_condition_and_update_effects() {
    let bodies = [
        "let n=0;while(n++<3){} n;",
        "let n=0;for(;n<3;n++){} n;",
        "let n=0;do{}while(++n<3); n;",
        "let n=0;exit:{} n=17; n;",
    ];
    let source = bodies
        .iter()
        .map(|body| format!("print(eval({}));", converted_text(body)))
        .collect::<String>();
    let exact_sources = bodies.map(units);
    assert_trace(&source, &["4", "3", "3", "17"], &exact_sources, (4, 4, 0));
}

#[test]
fn native_direct_empty_blocks_keep_lexical_shadowing_and_early_errors() {
    let valid = "let outer=17;{let outer=23;{} } outer;";
    let invalid = "{} { let repeated; let repeated; }";
    let source = format!(
        "print(eval({}));try{{eval({});}}catch(error){{print(error instanceof SyntaxError);}}",
        converted_text(valid),
        converted_text(invalid),
    );
    assert_trace(
        &source,
        &["17", "true"],
        &[units(valid), units(invalid)],
        (2, 1, 1),
    );
}

#[test]
fn native_direct_empty_try_clauses_keep_update_empty_and_catch_ownership() {
    let bodies = [
        "17;try{}finally{}",
        "17;try{}catch(error){}",
        "17;try{throw 1;}catch(error){}",
        "let n=0;try{throw 23;}catch(error){n=error;{}} n;",
        "let n=0;try{}finally{n=17;} n;",
    ];
    let source = bodies
        .iter()
        .map(|body| format!("print(eval({}));", converted_text(body)))
        .collect::<String>();
    let exact_sources = bodies.map(units);
    assert_trace(
        &source,
        &["undefined", "undefined", "undefined", "23", "17"],
        &exact_sources,
        (5, 5, 0),
    );
}

#[test]
fn native_direct_empty_statement_lists_keep_declarations_and_resource_lifetimes() {
    for prefix in ["", "'use strict';"] {
        let bodies = [
            "1;;{};",
            "1;var retained;{};",
            "var retained;{};retained;",
            "let lexical;{};lexical;",
            "1;switch(1){case 1:{};}",
            "let disposed=0;{using value={[Symbol.dispose](){disposed++;}};{};} disposed;",
        ];
        let body = bodies
            .iter()
            .map(|body| format!("print(eval({}));", converted_text(body)))
            .collect::<String>();
        let source = format!("{prefix}{body}");
        let exact_sources = bodies.map(units);
        assert_trace(
            &source,
            &["1", "1", "undefined", "undefined", "undefined", "1"],
            &exact_sources,
            (6, 6, 0),
        );
    }
}
