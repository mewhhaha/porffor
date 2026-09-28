use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_calendar_script(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .unwrap_or_else(|error| panic!("calendar semantics failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn leap_month_identity_survives_field_replacement_and_arithmetic() {
    assert_calendar_script(
        r#"
        function check(value, message) { if (!value) throw new Error(message); }
        const adar = Temporal.PlainDate.from({calendar:'hebrew', year:5784, monthCode:'M06', day:15});
        check(adar.month === 7 && adar.monthsInYear === 13 && adar.inLeapYear, 'leap ordinal');
        const previous = adar.with({year:5783});
        check(previous.monthCode === 'M06' && previous.month === 6 && previous.day === 15, 'with month identity');
        const added = previous.add({years:1});
        check(added.equals(adar), 'add month identity');
        const adarOne = Temporal.PlainDate.from({calendar:'hebrew', year:5784, monthCode:'M05L', day:30});
        const constrained = adarOne.with({year:5783});
        check(constrained.monthCode === 'M06' && constrained.day === 29, 'missing leap month constrain');
        let caught;
        try { adarOne.with({year:5783}, {overflow:'reject'}); } catch (error) { caught = error; }
        check(caught instanceof RangeError, 'missing leap month reject');
        const start = Temporal.PlainDate.from({calendar:'hebrew', year:5784, month:1, day:1});
        check(start.dayOfYear === 1 && start.add({days:35}).dayOfYear === 36, 'calendar day of year');
        const dangi = new Temporal.PlainDate(2000, 1, 1, 'dangi');
        check(dangi.year === 1999, 'related ISO year');
        check(Temporal.PlainDate.from({calendar:'dangi', year:dangi.year,
              monthCode:dangi.monthCode, day:dangi.day}).equals(dangi), 'dangi roundtrip');
        true;
    "#,
    );
}

#[test]
fn calendar_era_presence_and_option_observation_follow_resolution_order() {
    assert_calendar_script(
        r#"
        function check(value, message) { if (!value) throw new Error(message); }
        const absentEra = {year:2000, month:5, day:2,
            get era() { throw new Error('ISO era accessed'); },
            get eraYear() { throw new Error('ISO eraYear accessed'); }};
        check(Temporal.PlainDate.from(absentEra).year === 2000, 'ISO ignores eras');
        for (const extra of [{era:'ce'}, {eraYear:2000}]) {
            let readOptions = false;
            let caught;
            try { Temporal.PlainDate.from({calendar:'gregory', year:2000, month:5, day:2, ...extra},
                {get overflow() { readOptions = true; return 'reject'; }}); }
            catch (error) { caught = error; }
            check(readOptions && caught instanceof TypeError, 'partial era pair');
        }
        const reiwa = Temporal.PlainDate.from({calendar:'japanese', era:'reiwa', eraYear:1, month:1, day:1});
        check(reiwa.year === 2019 && reiwa.era === 'heisei' && reiwa.eraYear === 31, 'lenient era arithmetic');
        const large = Temporal.PlainDate.from({calendar:'hebrew', year:5784, month:257, day:257});
        check(large.month === 13 && large.day === large.daysInMonth, 'constrain before wire narrowing');
        let caught;
        try { Temporal.PlainDate.from({calendar:'hebrew', year:5784, month:257, day:257}, {overflow:'reject'}); }
        catch (error) { caught = error; }
        check(caught instanceof RangeError, 'reject large fields');
        true;
    "#,
    );
}
