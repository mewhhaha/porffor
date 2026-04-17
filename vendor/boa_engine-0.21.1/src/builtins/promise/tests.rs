use crate::{TestAction, run_test_actions};
use indoc::indoc;

#[test]
fn promise() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                    let count = 0;
                    const promise = new Promise((resolve, reject) => {
                        count += 1;
                        resolve(undefined);
                    }).then((_) => (count += 1));
                    count += 1;
                "#}),
        TestAction::assert_eq("count", 2),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("count", 3),
    ]);
}

#[test]
fn promise_all_keyed_resolves_empty_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let result;
            Promise.allKeyed({}).then(value => { result = value; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert("Object.getPrototypeOf(result) === Object.prototype"),
        TestAction::assert_eq("Reflect.ownKeys(result).length", 0),
    ]);
}

#[test]
fn promise_all_keyed_preserves_key_order() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let result;
            Promise.allKeyed({
              b: Promise.resolve(2),
              a: 1,
              [Symbol.for("skip")]: 3,
            }).then(value => { result = value; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert(indoc! {r#"
            const keys = Reflect.ownKeys(result);
            keys.length === 3 &&
            keys[0] === "b" &&
            keys[1] === "a" &&
            keys[2] === Symbol.for("skip")
        "#}),
        TestAction::assert_eq("result.b", 2),
        TestAction::assert_eq("result.a", 1),
        TestAction::assert_eq("result[Symbol.for('skip')]", 3),
    ]);
}

#[test]
fn promise_all_keyed_rejects_on_abrupt_getter() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let rejected;
            const input = Object.defineProperty({}, "boom", {
              enumerable: true,
              get() { throw new Error("boom"); }
            });
            Promise.allKeyed(input).then(
              () => { rejected = "fulfilled"; },
              err => { rejected = err.message; }
            );
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("rejected", js_str!("boom")),
    ]);
}

#[test]
fn promise_all_settled_keyed_matches_all_settled_shape() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let result;
            Promise.allSettledKeyed({
              ok: Promise.resolve(1),
              bad: Promise.reject("nope"),
            }).then(value => { result = value; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("result.ok.status", js_str!("fulfilled")),
        TestAction::assert_eq("result.ok.value", 1),
        TestAction::assert_eq("result.bad.status", js_str!("rejected")),
        TestAction::assert_eq("result.bad.reason", js_str!("nope")),
    ]);
}
