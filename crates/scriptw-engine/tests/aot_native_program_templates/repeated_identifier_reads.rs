use super::succeeds;

#[test]
fn repeated_eval_reads_preserve_each_getter_and_the_last_completion_value() {
    succeeds(
        r#"
        var reads = 0;
        Object.defineProperty(globalThis, 'repeatedRead', {
            configurable: true,
            get() { return ++reads; }
        });
        if (eval('repeatedRead;'.repeat(128)) !== 128 || reads !== 128) throw 1;
        delete globalThis.repeatedRead;
        1;
        "#,
    );
}

#[test]
fn repeated_function_reads_propagate_abrupt_completion_at_the_original_read() {
    succeeds(
        r#"
        var reads = 0, caught = 0, finalized = 0, token = {};
        Object.defineProperty(globalThis, 'repeatedRead', {
            configurable: true,
            get() { if (++reads === 37) throw token; return reads; }
        });
        var invoke = Function('repeatedRead;'.repeat(128));
        try { invoke(); }
        catch (error) { if (error !== token) throw error; caught++; }
        finally { finalized++; }
        if (reads !== 37 || caught !== 1 || finalized !== 1) throw 2;
        delete globalThis.repeatedRead;
        1;
        "#,
    );
}

#[test]
fn repeated_with_reads_resolve_the_live_unscopables_binding_on_every_iteration() {
    succeeds(
        r#"
        var reads = 0, repeatedRead = 777;
        var scope = {
            get repeatedRead() { return ++reads; },
            get [Symbol.unscopables]() { return { repeatedRead: reads >= 64 }; }
        };
        var result = eval('with (scope) {' + 'repeatedRead;'.repeat(128) + '}');
        if (reads !== 64 || result !== 777) throw 3;
        1;
        "#,
    );
}
