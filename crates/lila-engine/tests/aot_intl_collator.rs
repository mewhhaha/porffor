use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_collator_script(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| {
            panic!("Collator must compile and execute through Wasm AOT: {error}\n{source}")
        });
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
}

#[test]
fn constructor_and_bound_compare_obey_intrinsic_contract() {
    assert_collator_script(
        r#"
        function check(value) { if (!value) throw new Error('Collator intrinsic contract'); }
        const collator = Intl.Collator.call({ignored: true}, 'en');
        check(Object.getPrototypeOf(collator) === Intl.Collator.prototype);
        check(Intl.Collator.name === 'Collator' && Intl.Collator.length === 0);
        const descriptor = Object.getOwnPropertyDescriptor(Intl.Collator.prototype, 'compare');
        check(typeof descriptor.get === 'function' && descriptor.set === undefined);
        check(!descriptor.enumerable && descriptor.configurable);
        const compare = collator.compare;
        check(compare === collator.compare && compare.length === 2 && compare.name === '');
        check(compare.call(null, 'a', 'b') < 0);
        check(Object.getPrototypeOf(compare) === Function.prototype);
        for (const receiver of [{}, Intl.Collator.prototype, new Proxy(collator, {})]) {
            let threw = false;
            try { descriptor.get.call(receiver); } catch (error) { threw = error instanceof TypeError; }
            check(threw);
        }
        class Subclass extends Intl.Collator {}
        check(Object.getPrototypeOf(new Subclass('en')) === Subclass.prototype);
        print('ok');
    "#,
    );
}

#[test]
fn compare_uses_options_and_ordered_string_coercion() {
    assert_collator_script(
        r#"
        function check(value) { if (!value) throw new Error('Collator comparison'); }
        check(new Intl.Collator('en', {numeric: true}).compare('2', '10') < 0);
        check(new Intl.Collator('en', {numeric: false}).compare('2', '10') > 0);
        check(new Intl.Collator('en', {sensitivity: 'base'}).compare('a', 'á') === 0);
        check(new Intl.Collator('en', {sensitivity: 'variant'}).compare('a', 'á') !== 0);
        check(new Intl.Collator('en', {caseFirst: 'upper'}).compare('A', 'a') < 0);
        const compare = new Intl.Collator('en').compare;
        let order = '';
        compare({toString() { order += 'a'; return 'a'; }},
                {toString() { order += 'b'; return 'b'; }});
        check(order === 'ab');
        const sentinel = {};
        let caught;
        try {
            compare({toString() { throw sentinel; }},
                    {toString() { throw new Error('second operand was coerced'); }});
        } catch (error) { caught = error; }
        check(caught === sentinel);
        caught = undefined;
        try { compare(Symbol(), 'a'); } catch (error) { caught = error; }
        check(caught instanceof TypeError);
        print('ok');
    "#,
    );
}

#[test]
fn options_validation_and_nul_strings_cross_host_boundary() {
    assert_collator_script(
        r#"
        function check(value, label) { if (!value) throw new Error('Collator host boundary: ' + label); }
        for (const option of ['usage', 'sensitivity', 'caseFirst', 'localeMatcher', 'collation']) {
            let caught;
            try { new Intl.Collator('en', {[option]: 'invalid\0suffix'}); }
            catch (error) { caught = error; }
            check(caught instanceof RangeError, 'invalid option ' + option);
        }
        let early;
        try { new Intl.Collator('en', {collation:'bad\0value',
            get numeric() { throw new Error('numeric read before collation validation'); }}); }
        catch (error) { early = error; }
        check(early instanceof RangeError, 'collation validation order');
        const compare = new Intl.Collator('en').compare;
        check(compare('a\0b', 'a\0c') < 0, 'embedded NUL ordering');
        check(compare('\ud800', '\ud800') === 0, 'identical lone surrogates');
        // U+1D15E canonically decomposes to U+1D157 U+1D165, two surrogate pairs.
        check(compare('\ud834\udd5e', '\ud834\udd57\ud834\udd65') === 0, 'supplementary musical canonical equivalence');
        check(compare('\ud87e\udc2b', '北') === 0, 'supplementary CJK compatibility ideograph equivalence');
        check(compare(String.fromCodePoint(0x1d15e),
                      String.fromCharCode(0xd834, 0xdd57, 0xd834, 0xdd65)) === 0,
              'fromCodePoint and UTF-16 canonical equivalence');
        const options = new Intl.Collator('en-u-kn', {numeric: true}).resolvedOptions();
        check(options.numeric === true && options.locale.includes('-u-kn'), 'matching numeric extension retained');
        check(new Intl.Collator('en-u-kn', {numeric: false}).resolvedOptions().locale === 'en', 'overridden numeric extension removed');
        check(new Intl.Collator('en-ZZ').resolvedOptions().locale === 'en', 'unsupported region resolves to available parent');
        print('ok');
    "#,
    );
}

#[test]
fn construction_uses_new_target_realm_prototype() {
    assert_collator_script(
        r#"
        const other = __lilaCreateRealm().global;
        const Foreign = other.Intl.Collator;
        const normal = Reflect.construct(Intl.Collator, ['en'], Foreign);
        if (Object.getPrototypeOf(normal) !== Foreign.prototype) throw new Error('foreign prototype');
        function Target() {}
        Object.setPrototypeOf(Target, Foreign);
        Target.prototype = null;
        // The constructor function's own Realm, rather than its [[Prototype]],
        // determines the fallback intrinsic.
        const fallback = Reflect.construct(Intl.Collator, ['en'], Target);
        if (Object.getPrototypeOf(fallback) !== Intl.Collator.prototype) throw new Error('fallback realm');
        if (normal.compare('a', 'b') >= 0) throw new Error('foreign comparison');
        print('ok');
    "#,
    );
}
