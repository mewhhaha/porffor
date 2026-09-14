use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_nul_source(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("NUL source failed: {error}\n{source:?}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn literal_nuls_survive_strings_templates_and_regular_expressions() {
    assert_nul_source(
        "var text = 'a\0b'; var template = `a\0b`;\n\
         text.length === 3 && text.charCodeAt(1) === 0 && text[2] === 'b' &&\n\
         template === text && /a\0b/.test(text);",
    );
}

#[test]
fn tagged_templates_distinguish_literal_and_escaped_nuls_in_raw_strings() {
    assert_nul_source(
        "var calls = 0;\n\
         function tag(strings, raw) {\n\
           calls++;\n\
           return strings[0] === '\0' && strings[1] === '' && strings.raw[0] === raw;\n\
         }\n\
         var literal = tag`\0${'\0'}`;\n\
         var escaped = tag`\\0${'\\0'}`;\n\
         literal && escaped && calls === 2;",
    );
}

#[test]
fn comments_containing_nuls_preserve_the_following_statements() {
    assert_nul_source(
        "var value = 1; // before\0after\n\
         value += 2; /* before\0after */ value += 4; value === 7;",
    );
}
