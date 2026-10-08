use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("exact RegExp bounds failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn literal_and_computed_constructors_admit_exact_nullable_counts_beyond_u64() {
    assert_wasm(
        r#"
        function computed(text) {
            var units = '';
            for (var i = 0; i < text.length; i++) units += String.fromCharCode(text.charCodeAt(i));
            return new RegExp(units, 'd');
        }
        var sources = [
            '(){18446744073709551616}',
            '(){00018446744073709551616,00018446744073709551617}',
            '(?:(){1000000000000000000000000000000}){2,3}',
            '(?<=(){18446744073709551616})a',
            '(?=(){18446744073709551616})a',
            '(?<named>){18446744073709551616,}'
        ];
        var literals = [
            /(){18446744073709551616}/d,
            /(){00018446744073709551616,00018446744073709551617}/d,
            /(?:(){1000000000000000000000000000000}){2,3}/d,
            /(?<=(){18446744073709551616})a/d,
            /(?=(){18446744073709551616})a/d,
            /(?<named>){18446744073709551616,}/d
        ];
        for (var i = 0; i < sources.length; i++) {
            var dynamic = computed(sources[i]);
            if (dynamic.source !== sources[i] || dynamic.flags !== 'd') throw new Error('dynamic admission');
            if (literals[i].source !== sources[i] || literals[i].flags !== 'd') throw new Error('literal admission');
        }
        gc();
        if (literals[5].source !== sources[5]) throw new Error('retained immutable program');
        true;
    "#,
    );
}

#[test]
fn exact_huge_bound_order_and_finite_optional_matching_preserve_transactions() {
    assert_wasm(
        r#"
        function computed(text) {
            var copy = '';
            for (var i = 0; i < text.length; i++) copy += String.fromCharCode(text.charCodeAt(i));
            return new RegExp(copy, 'd');
        }
        var finite = [
            /^(?<value>a?){0,18446744073709551616}$/d,
            computed('^(?<value>a?){0,18446744073709551616}$')
        ];
        for (var i = 0; i < finite.length; i++) {
            var match = finite[i].exec('');
            if (match === null || match[0] !== '' || match[1] !== undefined ||
                match.groups.value !== undefined || match.indices[1] !== undefined ||
                match.indices.groups.value !== undefined) throw new Error('optional empty capture');
        }
        var required = [/^(a){18446744073709551616}$/d, computed('^(a){18446744073709551616}$')];
        for (var i = 0; i < required.length; i++) {
            if (required[i].exec('a') !== null) throw new Error('required exact consuming minimum');
        }
        var reversed = [
            '(){18446744073709551617,18446744073709551616}',
            'a{00018446744073709551617,00018446744073709551616}',
            '(){999999999999999999999999999999999,100000000000000000000000000000000}'
        ];
        for (var i = 0; i < reversed.length; i++) {
            var rejected = false;
            try { computed(reversed[i]); } catch (error) { rejected = error instanceof SyntaxError; }
            if (!rejected) throw new Error('original mathematical order');
            // Failed compilation must retire all source scratch and leave later
            // named-group and repeat descriptor publication intact.
            var after = computed('^(?<ok>a){2,3}$').exec('aa');
            if (after === null || after.groups.ok !== 'a' || after.indices.groups.ok[0] !== 1)
                throw new Error('post-failure transaction');
        }
        true;
    "#,
    );
}
