use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_relative_script(source: &str) {
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
        .expect("RelativeTimeFormat must compile and execute through Wasm AOT");
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
fn relative_patterns_preserve_negative_zero_and_numeric_parts() {
    assert_relative_script(
        r#"
        function check(value) { if (!value) throw new Error('relative patterns'); }
        const always = new Intl.RelativeTimeFormat('en', {numeric:'always'});
        check(always.format(-0, 'day') === '0 days ago');
        check(always.format(0, 'day') === 'in 0 days');
        check(always.format(-1, 'days') === '1 day ago');
        const auto = new Intl.RelativeTimeFormat('en', {numeric:'auto'});
        check(auto.format(-1, 'day') === 'yesterday');
        check(auto.format(0, 'day') === 'today');
        check(auto.format(1, 'day') === 'tomorrow');
        const parts = always.formatToParts(12345.6, 'days');
        check(parts.map(p => p.value).join('') === always.format(12345.6, 'day'));
        check(parts.filter(p => p.type !== 'literal').every(p => p.unit === 'day'));
        check(parts.some(p => p.type === 'group') && parts.some(p => p.type === 'fraction'));
        check(parts.filter(p => p.type === 'literal').every(p => !('unit' in p)));
        const literal = auto.formatToParts(-1, 'day');
        check(literal.length === 1 && literal[0].type === 'literal' && literal[0].value === 'yesterday');
        print('ok');
    "#,
    );
}

#[test]
fn relative_constructor_and_format_follow_observable_conversion_order() {
    assert_relative_script(
        r#"
        function check(value) { if (!value) throw new Error('relative conversion'); }
        let order = [];
        new Intl.RelativeTimeFormat('en', new Proxy({}, {get(target, key) {order.push(key); return undefined;}}));
        check(order.join() === 'localeMatcher,numberingSystem,style,numeric');
        let tracked = {value: 1};
        new Intl.RelativeTimeFormat('en', {get numeric() {tracked.value = 'changed'; return 'always';}});
        check(tracked.value === 'changed');
        const rtf = new Intl.RelativeTimeFormat('en');
        order = [];
        rtf.format({valueOf() {order.push('value'); return 2;}}, {toString() {order.push('unit'); return 'day';}});
        check(order.join() === 'value,unit');
        const sentinel = {};
        let caught;
        try {rtf.format(NaN, {toString() {throw sentinel;}});} catch (error) {caught = error;}
        check(caught === sentinel);
        for (const value of [NaN, Infinity, -Infinity]) {
            caught = undefined;
            try {rtf.format(value, 'day');} catch (error) {caught = error;}
            check(caught instanceof RangeError);
        }
        for (const option of ['localeMatcher', 'numberingSystem', 'style', 'numeric']) {
            caught = undefined;
            try {new Intl.RelativeTimeFormat('en', {[option]:'bad\0suffix'});} catch (error) {caught = error;}
            check(caught instanceof RangeError);
        }
        caught = undefined;
        try {rtf.format(1, 'day\0suffix');} catch (error) {caught = error;}
        check(caught instanceof RangeError);
        print('ok');
    "#,
    );
}

#[test]
fn relative_format_enforces_brands_and_new_target_realm() {
    assert_relative_script(
        r#"
        function check(value) { if (!value) throw new Error('relative branding'); }
        let caught;
        try {Intl.RelativeTimeFormat('en');} catch (error) {caught = error;}
        check(caught instanceof TypeError);
        class Derived extends Intl.RelativeTimeFormat {}
        check(Object.getPrototypeOf(new Derived('en')) === Derived.prototype);
        const method = Intl.RelativeTimeFormat.prototype.format;
        const descriptor = Object.getOwnPropertyDescriptor(Intl.RelativeTimeFormat.prototype, 'format');
        check(descriptor.value === method && descriptor.get === undefined);
        check(method.length === 2 && method.name === 'format');
        check(Intl.RelativeTimeFormat.prototype.formatToParts.length === 2);
        for (const receiver of [{}, Intl.RelativeTimeFormat.prototype, new Proxy(new Intl.RelativeTimeFormat('en'), {})]) {
            caught = undefined;
            try {method.call(receiver, 1, 'day');} catch (error) {caught = error;}
            check(caught instanceof TypeError);
        }
        const other = __lilaCreateRealm().global;
        const foreign = other.Intl.RelativeTimeFormat;
        const instance = Reflect.construct(Intl.RelativeTimeFormat, ['en'], foreign);
        check(Object.getPrototypeOf(instance) === foreign.prototype);
        caught = undefined;
        try {foreign.prototype.format.call(instance, 1, Symbol());} catch (error) {caught = error;}
        check(caught instanceof other.TypeError && !(caught instanceof TypeError));
        caught = undefined;
        try {foreign.prototype.format.call(instance, Symbol(), 'day');} catch (error) {caught = error;}
        check(caught instanceof other.TypeError && !(caught instanceof TypeError));
        const options = new Intl.RelativeTimeFormat('en', {numberingSystem:'arab', style:'short', numeric:'auto'}).resolvedOptions();
        check(options.numberingSystem === 'arab' && options.style === 'short' && options.numeric === 'auto');
        check(Intl.RelativeTimeFormat.supportedLocalesOf(['en']).join() === 'en');
        print('ok');
    "#,
    );
}
