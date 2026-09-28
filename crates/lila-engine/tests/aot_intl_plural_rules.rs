use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_plural_script(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
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
        .expect("PluralRules must compile and execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("ok".into())]
    );
}

#[test]
fn plural_selection_uses_locale_type_and_rounded_operands() {
    assert_plural_script(
        r#"
        function check(value) { if (!value) throw new Error('plural selection'); }
        const cardinal = new Intl.PluralRules('en');
        check(cardinal.select(1) === 'one' && cardinal.select(2) === 'other');
        check(cardinal.select(NaN) === 'other' && cardinal.select(Infinity) === 'other');
        check(new Intl.PluralRules('en', {minimumFractionDigits: 1}).select(1) === 'other');
        check(new Intl.PluralRules('en', {maximumFractionDigits: 0}).select(1.2) === 'one');
        const ordinal = new Intl.PluralRules('en', {type: 'ordinal'});
        check([1,2,3,4,11,21].map(n => ordinal.select(n)).join() === 'one,two,few,other,other,one');
        const russian = new Intl.PluralRules('ru');
        check([1,2,5,21].map(n => russian.select(n)).join() === 'one,few,many,one');
        const resolved = cardinal.resolvedOptions();
        check(!('numberingSystem' in resolved));
        check(resolved.pluralCategories.join() === 'one,other');
        check(new Intl.PluralRules('en', {notation: 'scientific'}).resolvedOptions().notation === 'scientific');
        print('ok');
    "#,
    );
}

#[test]
fn plural_options_and_range_conversion_observe_required_order() {
    assert_plural_script(
        r#"
        function check(value) { if (!value) throw new Error('plural observation order'); }
        let reads = [];
        new Intl.PluralRules('en', new Proxy({}, {get(target, key) { reads.push(key); return undefined; }}));
        check(reads.join() === 'localeMatcher,type,notation,compactDisplay,minimumIntegerDigits,minimumFractionDigits,maximumFractionDigits,minimumSignificantDigits,maximumSignificantDigits,roundingIncrement,roundingMode,roundingPriority,trailingZeroDisplay');
        const rules = new Intl.PluralRules('en');
        reads = [];
        rules.selectRange({valueOf() { reads.push('start'); return 1; }},
                          {valueOf() { reads.push('end'); return 2; }});
        check(reads.join() === 'start,end');
        reads = [];
        let caught;
        try { rules.selectRange(NaN, {valueOf() { reads.push('end'); return 2; }}); }
        catch (error) { caught = error; }
        check(caught instanceof RangeError && reads.join() === 'end');
        check(rules.selectRange(1,2) === 'other');
        check(typeof rules.selectRange(2,1) === 'string');
        caught = undefined;
        try { rules.selectRange(undefined, 2); } catch (error) { caught = error; }
        check(caught instanceof TypeError);
        print('ok');
    "#,
    );
}

#[test]
fn plural_rules_construction_and_methods_enforce_branding() {
    assert_plural_script(
        r#"
        function check(value) { if (!value) throw new Error('plural branding'); }
        let caught;
        try { Intl.PluralRules('en'); } catch (error) { caught = error; }
        check(caught instanceof TypeError);
        class Derived extends Intl.PluralRules {}
        check(Object.getPrototypeOf(new Derived('en')) === Derived.prototype);
        const method = Intl.PluralRules.prototype.select;
        for (const receiver of [{}, Intl.PluralRules.prototype, new Proxy(new Intl.PluralRules('en'), {})]) {
            caught = undefined;
            try { method.call(receiver, 1); } catch (error) { caught = error; }
            check(caught instanceof TypeError);
        }
        check(new Intl.PluralRules('en').select(1n) === 'one');
        const other = __lilaCreateRealm().global;
        const foreign = other.Intl.PluralRules;
        const instance = Reflect.construct(Intl.PluralRules, ['en'], foreign);
        check(Object.getPrototypeOf(instance) === foreign.prototype);
        caught = undefined;
        try {foreign.prototype.select.call(instance, Symbol());} catch (error) {caught = error;}
        check(caught instanceof other.TypeError && !(caught instanceof TypeError));
        check(Intl.PluralRules.supportedLocalesOf(['en']).join() === 'en');
        print('ok');
    "#,
    );
}

#[test]
fn plural_selection_preserves_exact_strings_and_bigints() {
    assert_plural_script(
        r#"
        function check(value) { if (!value) throw new Error('exact plural input'); }
        const rules = new Intl.PluralRules('en', {
            minimumSignificantDigits: 1, maximumSignificantDigits: 21
        });
        check(rules.select('1.0000000000000000001') === 'other');
        check(rules.select(1n) === 'one');
        check(rules.select(Object(1n)) === 'one');
        check(rules.select(100000000000000000001n) === 'other');
        check(rules.select('100000000000000000001') === 'other');
        check(rules.select('not a number') === 'other');
        check(rules.select('-0') === 'other');
        const ordinal = new Intl.PluralRules('en', {
            type: 'ordinal', maximumSignificantDigits: 21
        });
        check(ordinal.select(100000000000000000001n) === 'one');
        check(ordinal.select('100000000000000000001') === 'one');
        for (const notation of ['scientific', 'engineering']) {
            const scientific = new Intl.PluralRules('en', {
                type: 'ordinal', notation, maximumFractionDigits: 0
            });
            check(scientific.select(1100001) === 'one');
        }
        check(rules.selectRange(1n, 2n) === 'other');
        check(rules.selectRange(1n, '1') === 'one');
        check(rules.selectRange(-1n, 1n) === 'one');
        check(new Intl.PluralRules('en', {maximumFractionDigits: 0})
              .selectRange('1.1', '1.2') === 'one');
        let reads = [];
        check(rules.select({ [Symbol.toPrimitive](hint) {
            reads.push(hint); return '1.0000000000000000001';
        }}) === 'other');
        check(reads.join() === 'number');
        reads = [];
        let caught;
        try { rules.selectRange('not a number', { [Symbol.toPrimitive](hint) {
            reads.push(hint); return 2n;
        }}); } catch (error) { caught = error; }
        check(caught instanceof RangeError && reads.join() === 'number');
        caught = undefined;
        try { rules.selectRange(1n, Symbol()); } catch (error) { caught = error; }
        check(caught instanceof TypeError);
        print('ok');
    "#,
    );
}
